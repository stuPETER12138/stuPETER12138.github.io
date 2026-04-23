//! 本地预览服务器（纯 Rust 实现，无外部依赖）
//!
//! 功能：
//! - 静态文件服务（从 _site/ 目录）
//! - 自动 Content-Type 推断
//! - 目录请求自动回退到 index.html
//! - ETag / If-None-Match 304 缓存
//! - 简单目录列表（无 index.html 时）
//! - 可选自动打开浏览器

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::Result;

use crate::config::site_dir;

// ── 公共入口 ──────────────────────────────────────────────────────────────────

pub fn preview(port: u16, open_browser: bool) -> Result<bool> {
    let site = site_dir();
    if !site.exists() {
        println!(
            "  ⚠ 输出目录 {} 不存在，请先运行 build 命令。",
            site.display()
        );
        return Ok(false);
    }

    let addr = format!("0.0.0.0:{port}");
    let listener =
        TcpListener::bind(&addr).map_err(|e| anyhow::anyhow!("无法绑定端口 {port}: {e}"))?;

    println!("🌐 预览服务器已启动");
    println!("   地址: http://localhost:{port}");
    println!(
        "   根目录: {}",
        site.canonicalize().unwrap_or(site.clone()).display()
    );
    println!("   按 Ctrl+C 停止");
    println!();

    // 延迟打开浏览器
    if open_browser {
        let url = format!("http://localhost:{port}");
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(800));
            println!("  🚀 正在打开浏览器: {url}");
            open_in_browser(&url);
        });
    }

    // 将 site 路径包装为 Arc，跨线程共享
    let root = Arc::new(site.canonicalize().unwrap_or(site));

    // 主接受循环
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let root = Arc::clone(&root);
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, &root) {
                        eprintln!("  ⚠ 连接处理出错: {e}");
                    }
                });
            }
            Err(e) => eprintln!("  ⚠ 接受连接失败: {e}"),
        }
    }

    Ok(true)
}

// ── HTTP 请求处理 ─────────────────────────────────────────────────────────────

fn handle_connection(mut stream: TcpStream, root: &Path) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let mut reader = BufReader::new(stream.try_clone()?);

    // 读取请求行
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let request_line = request_line.trim();

    // 解析 "METHOD /path HTTP/1.x"
    let mut parts = request_line.splitn(3, ' ');
    let method = parts.next().unwrap_or("GET");
    let raw_path = parts.next().unwrap_or("/");

    // 读取剩余请求头
    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(": ") {
            headers.insert(k.to_lowercase(), v.to_string());
        }
    }

    // 只处理 GET / HEAD
    if method != "GET" && method != "HEAD" {
        send_response(&mut stream, 405, "Method Not Allowed", &[], b"")?;
        return Ok(());
    }

    // URL 解码路径，去除查询字符串
    let url_path = raw_path.split('?').next().unwrap_or("/");
    let url_path = percent_decode(url_path);

    // 安全：防止路径穿越
    let rel = url_path.trim_start_matches('/');
    let file_path = root.join(rel);
    let canonical = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            send_not_found(&mut stream, method == "HEAD")?;
            log_request(method, &url_path, 404);
            return Ok(());
        }
    };
    if !canonical.starts_with(root) {
        send_response(&mut stream, 403, "Forbidden", &[], b"")?;
        log_request(method, &url_path, 403);
        return Ok(());
    }

    // 目录 -> 尝试 index.html，否则生成目录列表
    let serve_path = if canonical.is_dir() {
        let idx = canonical.join("index.html");
        if idx.exists() {
            idx
        } else {
            let body = dir_listing(&canonical, root, &url_path);
            let headers = [("Content-Type", "text/html; charset=utf-8")];
            log_request(method, &url_path, 200);
            if method == "HEAD" {
                send_response(&mut stream, 200, "OK", &headers, b"")?;
            } else {
                send_response(&mut stream, 200, "OK", &headers, body.as_bytes())?;
            }
            return Ok(());
        }
    } else {
        canonical.clone()
    };

    if !serve_path.exists() {
        send_not_found(&mut stream, method == "HEAD")?;
        log_request(method, &url_path, 404);
        return Ok(());
    }

    // 读取文件
    let data = fs::read(&serve_path)?;

    // ETag = 文件大小 + 修改时间（简单哈希）
    let etag = compute_etag(&serve_path);

    // 304 缓存检查
    if let Some(client_etag) = headers.get("if-none-match") {
        if client_etag.trim_matches('"') == etag {
            send_response(&mut stream, 304, "Not Modified", &[], b"")?;
            log_request(method, &url_path, 304);
            return Ok(());
        }
    }

    let content_type = mime_type(&serve_path);
    let etag_value = format!("\"{}\"", etag);
    let resp_headers = [
        ("Content-Type", content_type),
        ("ETag", etag_value.as_str()),
        ("Cache-Control", "no-cache"),
    ];

    log_request(method, &url_path, 200);
    if method == "HEAD" {
        send_response(&mut stream, 200, "OK", &resp_headers, b"")?;
    } else {
        send_response(&mut stream, 200, "OK", &resp_headers, &data)?;
    }

    Ok(())
}

// ── 响应发送 ──────────────────────────────────────────────────────────────────

fn send_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Result<()> {
    let mut response = format!("HTTP/1.1 {status} {reason}\r\n");
    response.push_str(&format!("Content-Length: {}\r\n", body.len()));
    response.push_str("Connection: close\r\n");
    for (k, v) in headers {
        response.push_str(&format!("{k}: {v}\r\n"));
    }
    response.push_str("\r\n");
    stream.write_all(response.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

fn send_not_found(stream: &mut TcpStream, head_only: bool) -> Result<()> {
    let body = b"<html><body><h1>404 Not Found</h1></body></html>";
    let headers = [("Content-Type", "text/html; charset=utf-8")];
    send_response(
        stream,
        404,
        "Not Found",
        &headers,
        if head_only { b"" } else { body },
    )
}

// ── 目录列表 ──────────────────────────────────────────────────────────────────

fn dir_listing(dir: &Path, root: &Path, url_path: &str) -> String {
    let mut entries: Vec<(String, bool)> = Vec::new();

    if let Ok(read) = fs::read_dir(dir) {
        for entry in read.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.path().is_dir();
            entries.push((name, is_dir));
        }
    }
    entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let rel = dir.strip_prefix(root).unwrap_or(dir).to_string_lossy();
    let title = format!("/{rel}");
    let mut html = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\">\
         <title>Index of {title}</title>\
         <style>body{{font-family:monospace;padding:2em}}a{{display:block;margin:.3em 0}}</style>\
         </head><body><h2>Index of {title}</h2><hr>"
    );

    // 上级目录链接
    if url_path != "/" {
        let parent = PathBuf::from(url_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        let _ = write!(html, "<a href=\"{parent}/\">../</a>");
    }

    for (name, is_dir) in &entries {
        let suffix = if *is_dir { "/" } else { "" };
        let base = url_path.trim_end_matches('/');
        let _ = write!(html, "<a href=\"{base}/{name}{suffix}\">{name}{suffix}</a>");
    }

    html.push_str("<hr></body></html>");
    html
}

// ── 辅助函数 ──────────────────────────────────────────────────────────────────

/// 根据文件扩展名返回 MIME 类型。
fn mime_type(path: &Path) -> &'static str {
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
        "md" => "text/markdown; charset=utf-8",
        "atom" | "rss" => "application/rss+xml",
        _ => "application/octet-stream",
    }
}

/// 简单 ETag：文件大小 + 修改时间（十六进制）。
fn compute_etag(path: &Path) -> String {
    path.metadata()
        .map(|m| {
            let size = m.len();
            let mtime = m
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs())
                })
                .unwrap_or(0);
            format!("{size:x}-{mtime:x}")
        })
        .unwrap_or_else(|_| "0".to_string())
}

/// 简单百分比解码（处理 %XX）。
fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte as char);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// 打印请求日志。
fn log_request(method: &str, path: &str, status: u16) {
    let color = match status {
        200..=299 => "\x1b[32m", // 绿
        300..=399 => "\x1b[36m", // 青
        400..=499 => "\x1b[33m", // 黄
        _ => "\x1b[31m",         // 红
    };
    println!("  {color}{status}\x1b[0m  {method} {path}");
}

/// 在系统默认浏览器中打开 URL。
fn open_in_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd").args(["/C", "start", url]).spawn();

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = Command::new("xdg-open").arg(url).spawn();
}
