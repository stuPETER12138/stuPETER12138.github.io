//! 站点元文件生成：RSS、Sitemap、robots.txt

use anyhow::Result;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use regex::Regex;
use std::path::Path;

use crate::config::{config_file, site_dir};
use crate::html_meta::parse_html_metadata;

// ── 站点 URL ──────────────────────────────────────────────────────────────────

pub fn get_site_url() -> Option<String> {
    let index = site_dir().join("index.html");
    let meta = parse_html_metadata(&index).ok()?;
    meta.get("link")
        .map(|s| s.trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
}

// ── Feed 目录配置 ─────────────────────────────────────────────────────────────

pub fn get_feed_dirs() -> Vec<String> {
    let cfg = config_file();
    if !cfg.exists() {
        return Vec::new();
    }
    let content = match std::fs::read_to_string(&cfg) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let content = Regex::new(r"//[^\n]*")
        .unwrap()
        .replace_all(&content, "")
        .to_string();
    let content = Regex::new(r"/\*[\s\S]*?\*/")
        .unwrap()
        .replace_all(&content, "")
        .to_string();
    let re_feed = Regex::new(r"feed-dir\s*:\s*\(([\s\S]*?)\)").unwrap();
    if let Some(cap) = re_feed.captures(&content) {
        return Regex::new(r#""([^"]*)""#)
            .unwrap()
            .captures_iter(&cap[1])
            .map(|c| c[1].trim_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    Vec::new()
}

// ── 文章元数据 ────────────────────────────────────────────────────────────────

pub struct Post {
    pub title: String,
    pub description: String,
    pub dir: String,
    pub link: String,
    pub date: DateTime<Utc>,
}

pub fn collect_posts(dirs: &[String], _site_url: &str) -> Vec<Post> {
    let site = site_dir();
    let mut posts = Vec::new();
    for d in dirs {
        let dir_path = site.join(d);
        if !dir_path.exists() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir_path) else {
            continue;
        };
        for entry in entries.flatten() {
            let item = entry.path();
            if !item.is_dir() {
                continue;
            }
            let index_html = item.join("index.html");
            if !index_html.exists() {
                continue;
            }
            let (title, description, link, date_opt) = extract_post_metadata(&index_html, &item);
            let Some(date) = date_opt else {
                println!(
                    "⚠️ 无法确定文章 '{}' 的日期，已跳过。",
                    item.file_name().unwrap_or_default().to_string_lossy()
                );
                continue;
            };
            posts.push(Post {
                title,
                description,
                dir: d.clone(),
                link,
                date,
            });
        }
    }
    posts
}

fn extract_post_metadata(
    index_html: &Path,
    item_dir: &Path,
) -> (String, String, String, Option<DateTime<Utc>>) {
    let Ok(meta) = parse_html_metadata(index_html) else {
        return (String::new(), String::new(), String::new(), None);
    };
    let title = meta
        .get("title")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let description = meta
        .get("description")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let link = meta.get("link").cloned().unwrap_or_default();
    let date_opt = meta
        .get("date")
        .and_then(|d| {
            let s = d.split('T').next().unwrap_or(d);
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .map(|nd| Utc.from_utc_datetime(&nd.and_hms_opt(0, 0, 0).unwrap()))
        })
        .or_else(|| {
            let folder = item_dir.file_name()?.to_string_lossy().to_string();
            Regex::new(r"(\d{4}-\d{2}-\d{2})")
                .unwrap()
                .captures(&folder)
                .and_then(|cap| {
                    NaiveDate::parse_from_str(&cap[1], "%Y-%m-%d")
                        .ok()
                        .map(|nd| Utc.from_utc_datetime(&nd.and_hms_opt(0, 0, 0).unwrap()))
                })
        });
    (title, description, link, date_opt)
}

// ── RSS ───────────────────────────────────────────────────────────────────────

pub fn generate_rss(site_url: &str) -> Result<bool> {
    let site = site_dir();
    let rss_file = site.join("feed.xml");
    let dirs = get_feed_dirs();
    if dirs.is_empty() {
        println!("⚠️ 跳过 RSS 订阅源生成: 未配置任何目录。");
        return Ok(true);
    }
    let existing: Vec<String> = dirs
        .iter()
        .filter(|d| site.join(d).exists())
        .cloned()
        .collect();
    for d in &dirs {
        if !site.join(d).exists() {
            println!("⚠️ 警告: 配置的目录 '{d}' 不存在。");
        }
    }
    if existing.is_empty() {
        println!("⚠️ 跳过 RSS 订阅源生成: 配置的目录都不存在。");
        return Ok(true);
    }
    let mut posts = collect_posts(&existing, site_url);
    if posts.is_empty() {
        println!("⚠️ 未找到任何文章，RSS 订阅源为空。");
        return Ok(true);
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    let index_meta = parse_html_metadata(&site.join("index.html")).unwrap_or_default();
    let lang = index_meta.get("lang").cloned().unwrap_or_default();
    let site_title = index_meta
        .get("title")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let site_desc = index_meta
        .get("description")
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    let xml = build_rss_xml(&posts, site_url, &site_title, &site_desc, &lang);
    std::fs::write(&rss_file, xml)?;
    println!(
        "✅ RSS 订阅源生成成功: {} ({} 篇文章)",
        rss_file.display(),
        posts.len()
    );
    Ok(true)
}

fn build_rss_xml(
    posts: &[Post],
    site_url: &str,
    site_title: &str,
    site_desc: &str,
    lang: &str,
) -> String {
    let now = Utc::now().format("%a, %d %b %Y %H:%M:%S +0000").to_string();
    let mut items = String::new();
    for post in posts {
        let pub_date = post.date.format("%a, %d %b %Y %H:%M:%S +0000").to_string();
        let desc_tag = if !post.description.is_empty() {
            format!(
                "      <description>{}</description>\n",
                xml_escape(&post.description)
            )
        } else {
            String::new()
        };
        items.push_str(&format!(
            "    <item>\n      <title>{}</title>\n      <link>{}</link>\n{}      <guid isPermaLink=\"true\">{}</guid>\n      <pubDate>{}</pubDate>\n      <category>{}</category>\n    </item>\n",
            xml_escape(&post.title), xml_escape(&post.link), desc_tag,
            xml_escape(&post.link), pub_date, xml_escape(&post.dir)
        ));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n  <channel>\n\
         <title>{site_title}</title>\n<link>{site_url}</link>\n\
         <description>{site_desc}</description>\n<language>{lang}</language>\n\
         <lastBuildDate>{now}</lastBuildDate>\n\
         <atom:link href=\"{site_url}/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>\n\
         {items}  </channel>\n</rss>\n",
        site_title = xml_escape(site_title),
        site_url = site_url,
        site_desc = xml_escape(site_desc),
        lang = lang,
        now = now,
        items = items
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ── Sitemap ───────────────────────────────────────────────────────────────────

pub fn generate_sitemap(site_url: &str) -> Result<bool> {
    let site = site_dir();
    let mut urls = Vec::new();
    let walker = walkdir::WalkDir::new(&site)
        .into_iter()
        .filter_map(|e| e.ok());
    for entry in walker {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|x| x.to_str()) != Some("html") {
            continue;
        }
        let rel = path.strip_prefix(&site).unwrap_or(path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let url_path = if rel_str == "index.html" {
            String::new()
        } else if rel_str.ends_with("/index.html") {
            rel_str.strip_suffix("index.html").unwrap_or("").to_string()
        } else if rel_str.ends_with(".html") {
            format!("{}/", rel_str.strip_suffix(".html").unwrap_or(&rel_str))
        } else {
            rel_str.clone()
        };
        let full_url = format!("{}/{}", site_url, url_path);
        let lastmod = path
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
            .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
            .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "1970-01-01".to_string());
        urls.push((full_url, lastmod));
    }
    urls.sort();
    let mut url_elems = String::new();
    for (url, lastmod) in &urls {
        url_elems.push_str(&format!(
            "  <url>\n    <loc>{}</loc>\n    <lastmod>{}</lastmod>\n  </url>\n",
            xml_escape(url),
            lastmod
        ));
    }
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{url_elems}</urlset>\n"
    );
    std::fs::write(site.join("sitemap.xml"), xml)?;
    println!("✅ Sitemap 构建完成: 包含 {} 个页面", urls.len());
    Ok(true)
}

// ── robots.txt ────────────────────────────────────────────────────────────────

pub fn generate_robots_txt(site_url: &str) -> Result<bool> {
    std::fs::write(
        site_dir().join("robots.txt"),
        format!("User-agent: *\nAllow: /\n\nSitemap: {site_url}/sitemap.xml\n"),
    )?;
    Ok(true)
}
