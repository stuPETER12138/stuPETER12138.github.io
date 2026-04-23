//! 本地预览服务器（SSE LiveReload，纯标准库）

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime};

use anyhow::Result;

use crate::builder;
use crate::config::{assets_dir, config_file, content_dir, site_dir};

// ── SSE 广播器 ────────────────────────────────────────────────────────────────

/// 每个 SSE 客户端对应一个 Sender<()>。
/// 广播时逐一发送，send 失败说明客户端已断开，自动移除。
type Clients = Arc<Mutex<Vec<mpsc::Sender<()>>>>;

fn broadcast(clients: &Clients) {
    let mut list = clients.lock().unwrap();
    let before = list.len();
    list.retain(|tx| tx.send(()).is_ok());
    let alive = list.len();
    if alive > 0 {
        println!(
            "  🔄 已通知 {alive} 个浏览器标签页刷新（{} 个已断开）",
            before - alive
        );
    }
}

// ── 入口 ──────────────────────────────────────────────────────────────────────

pub fn preview(port: u16, open_browser: bool) -> Result<bool> {
    // 启动前检查：若 _site/ 不存在则完整构建，否则增量构建
    println!("🔍 检查是否需要重新构建...");
    if !builder::build(false)? {
        println!("  ⚠ 构建失败，预览可能显示旧内容。");
    }

    let site = site_dir();
    if !site.exists() {
        println!(
            "  ⚠ 输出目录 {} 不存在，请先运行 build 命令。",
            site.display()
        );
        return Ok(false);
    }

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .map_err(|e| anyhow::anyhow!("无法绑定端口 {port}: {e}"))?;

    let root = Arc::new(site.canonicalize().unwrap_or(site));
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    println!("🌐 预览服务器已启动（LiveReload 已开启）");
    println!("   地址:   http://localhost:{port}");
    println!("   根目录: {}", root.display());
    println!("   按 Ctrl+C 停止\n");

    // 文件监听线程
    {
        let c = Arc::clone(&clients);
        thread::spawn(move || watch_and_rebuild(c));
    }

    // 打开浏览器
    if open_browser {
        let url = format!("http://localhost:{port}");
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(800));
            println!("  🚀 正在打开浏览器: {url}");
            open_in_browser(&url);
        });
    }

    for stream in listener.incoming().flatten() {
        let root = Arc::clone(&root);
        let clients = Arc::clone(&clients);
        thread::spawn(move || {
            if let Err(e) = handle(stream, &root, clients) {
                let s = e.to_string();
                if !s.contains("broken pipe") && !s.contains("connection reset") {
                    eprintln!("  ⚠ {e}");
                }
            }
        });
    }
    Ok(true)
}

// ── 请求处理 ──────────────────────────────────────────────────────────────────

fn handle(stream: TcpStream, root: &Path, clients: Clients) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    // 请求行
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.trim().splitn(3, ' ');
    let method = parts.next().unwrap_or("GET").to_string();
    let path = percent_decode(parts.next().unwrap_or("/").split('?').next().unwrap_or("/"));

    // 消费请求头（SSE 不需要解析，HTTP 文件服务需要 If-None-Match）
    let mut headers = HashMap::new();
    loop {
        let mut h = String::new();
        reader.read_line(&mut h)?;
        if h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.trim().split_once(": ") {
            headers.insert(k.to_lowercase(), v.to_string());
        }
    }

    let mut stream = reader.into_inner();

    if path == "/_livereload" {
        serve_sse(stream, clients)
    } else {
        serve_file(&mut stream, root, &method, &path, &headers)
    }
}

// ── SSE 端点 ──────────────────────────────────────────────────────────────────

fn serve_sse(mut stream: TcpStream, clients: Clients) -> Result<()> {
    // SSE 响应头
    stream.write_all(
        b"HTTP/1.1 200 OK\r\n\
          Content-Type: text/event-stream\r\n\
          Cache-Control: no-cache\r\n\
          Connection: keep-alive\r\n\
          Access-Control-Allow-Origin: *\r\n\
          \r\n",
    )?;
    // 首条注释心跳，让浏览器确认连接已建立
    stream.write_all(b": connected\n\n")?;

    // 注册到广播列表
    let (tx, rx) = mpsc::channel::<()>();
    {
        let mut list = clients.lock().unwrap();
        list.push(tx);
        println!("  🔌 浏览器已连接 LiveReload（共 {} 个标签页）", list.len());
    }

    // 关闭读超时，等待广播
    stream.set_read_timeout(None)?;

    // 保持连接：收到广播就推送 SSE 事件，发送失败说明客户端断开
    for () in rx {
        if stream.write_all(b"data: reload\n\n").is_err() {
            break;
        }
    }

    Ok(())
}

// ── 静态文件服务 ──────────────────────────────────────────────────────────────

fn serve_file(
    stream: &mut TcpStream,
    root: &Path,
    method: &str,
    url_path: &str,
    headers: &HashMap<String, String>,
) -> Result<()> {
    if method != "GET" && method != "HEAD" {
        return respond(stream, 405, "Method Not Allowed", &[], b"");
    }

    // 路径安全检查
    let canonical = root.join(url_path.trim_start_matches('/')).canonicalize();
    let canonical = match canonical {
        Ok(p) if p.starts_with(root) => p,
        _ => {
            log(method, url_path, 404);
            return respond(stream, 404, "Not Found", &[], b"<h1>404</h1>");
        }
    };

    // 目录 → index.html 或目录列表
    let serve = if canonical.is_dir() {
        let idx = canonical.join("index.html");
        if idx.exists() {
            idx
        } else {
            let body = inject(&dir_listing(&canonical, root, url_path));
            log(method, url_path, 200);
            return respond(
                stream,
                200,
                "OK",
                &[("Content-Type", "text/html; charset=utf-8")],
                body.as_bytes(),
            );
        }
    } else {
        canonical
    };

    if !serve.exists() {
        log(method, url_path, 404);
        return respond(stream, 404, "Not Found", &[], b"<h1>404</h1>");
    }

    let data = fs::read(&serve)?;
    let etag = etag(&serve);

    // 304 缓存
    if headers
        .get("if-none-match")
        .map(|e| e.trim_matches('"') == etag)
        .unwrap_or(false)
    {
        log(method, url_path, 304);
        return respond(stream, 304, "Not Modified", &[], b"");
    }

    let ct = mime(&serve);
    let etag_hdr = format!("\"{etag}\"");
    let hdrs = [
        ("Content-Type", ct),
        ("ETag", &etag_hdr),
        ("Cache-Control", "no-cache"),
    ];

    // HTML 注入 LiveReload 脚本
    let body: Vec<u8> = if ct.starts_with("text/html") {
        inject(&String::from_utf8_lossy(&data)).into_bytes()
    } else {
        data
    };

    log(method, url_path, 200);
    respond(
        stream,
        200,
        "OK",
        &hdrs,
        if method == "HEAD" { b"" } else { &body },
    )
}

// ── LiveReload 脚本注入 ───────────────────────────────────────────────────────

fn inject(html: &str) -> String {
    const SCRIPT: &str = "\
<script>\
(function(){\
  var es=new EventSource('/_livereload');\
  es.onmessage=function(e){if(e.data==='reload')location.reload();};\
  es.onerror=function(){setTimeout(function(){location.reload();},1000);};\
}());\
</script>";
    match html.to_lowercase().rfind("</body>") {
        Some(i) => format!("{}{}{}", &html[..i], SCRIPT, &html[i..]),
        None => format!("{html}{SCRIPT}"),
    }
}

// ── 文件监听 + 自动重建 ───────────────────────────────────────────────────────

fn watch_and_rebuild(clients: Clients) {
    let mut snap = snapshot();
    thread::sleep(Duration::from_millis(500)); // 跳过首次

    loop {
        thread::sleep(Duration::from_millis(300));
        let cur = snapshot();
        if cur == snap {
            continue;
        }

        let changed: Vec<_> = cur
            .keys()
            .filter(|k| cur[*k] != snap.get(*k).copied().unwrap_or(0))
            .collect();
        println!(
            "\n  📝 检测到 {} 个文件变动，开始增量重建...",
            changed.len()
        );
        for p in &changed {
            println!(
                "     {}",
                p.strip_prefix(std::env::current_dir().unwrap_or_default())
                    .unwrap_or(p)
                    .display()
            );
        }

        if builder::build(false).unwrap_or(false) {
            println!("  ✅ 重建完成");
            broadcast(&clients);
        } else {
            println!("  ❌ 重建失败");
        }

        snap = snapshot();
        println!();
    }
}

fn snapshot() -> HashMap<PathBuf, u64> {
    let mut m = HashMap::new();
    for dir in [content_dir(), assets_dir()] {
        scan(&dir, &mut m);
    }
    let cfg = config_file();
    if cfg.exists() {
        m.insert(cfg.clone(), mtime(&cfg));
    }
    m
}

fn scan(dir: &Path, m: &mut HashMap<PathBuf, u64>) {
    if !dir.exists() {
        return;
    }
    for e in walkdir::WalkDir::new(dir).into_iter().flatten() {
        if e.path().is_file() {
            m.insert(e.path().to_path_buf(), mtime(e.path()));
        }
    }
}

fn mtime(p: &Path) -> u64 {
    p.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ── 辅助函数 ──────────────────────────────────────────────────────────────────

fn respond(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Result<()> {
    let mut r = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (k, v) in headers {
        r.push_str(&format!("{k}: {v}\r\n"));
    }
    r.push_str("\r\n");
    stream.write_all(r.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

fn dir_listing(dir: &Path, root: &Path, url_path: &str) -> String {
    let mut entries: Vec<(String, bool)> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| {
            (
                e.file_name().to_string_lossy().to_string(),
                e.path().is_dir(),
            )
        })
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let title = format!(
        "/{}",
        dir.strip_prefix(root).unwrap_or(dir).to_string_lossy()
    );
    let mut html = format!(
        "<!DOCTYPE html><html><head><meta charset=utf-8><title>Index of {title}</title>\
        <style>body{{font-family:monospace;padding:2em}}a{{display:block;margin:.3em 0}}</style></head>\
        <body><h2>Index of {title}</h2><hr>"
    );
    if url_path != "/" {
        let parent = PathBuf::from(url_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".into());
        let _ = write!(html, "<a href=\"{parent}/\">../</a>");
    }
    for (name, is_dir) in &entries {
        let suf = if *is_dir { "/" } else { "" };
        let _ = write!(
            html,
            "<a href=\"{}/{name}{suf}\">{name}{suf}</a>",
            url_path.trim_end_matches('/')
        );
    }
    html + "<hr></body></html>"
}

fn etag(path: &Path) -> String {
    path.metadata()
        .map(|m| {
            let t = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("{:x}-{t:x}", m.len())
        })
        .unwrap_or_else(|_| "0".into())
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(h) = std::str::from_utf8(&b[i + 1..i + 3]) {
                if let Ok(c) = u8::from_str_radix(h, 16) {
                    out.push(c as char);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

fn log(method: &str, path: &str, status: u16) {
    let c = match status {
        200..=299 => "\x1b[32m",
        300..=399 => "\x1b[36m",
        400..=499 => "\x1b[33m",
        _ => "\x1b[31m",
    };
    println!("  {c}{status}\x1b[0m  {method} {path}");
}

fn open_in_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd").args(["/C", "start", url]).spawn();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = Command::new("xdg-open").arg(url).spawn();
}
