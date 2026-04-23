//! HTML 元数据解析器
//!
//! 从 HTML 文件中提取 lang、title、description、link (canonical)、date。

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub fn parse_html_metadata(html_path: &Path) -> Result<HashMap<String, String>> {
    let content = std::fs::read_to_string(html_path)?;
    Ok(extract_metadata(&content))
}

pub fn extract_metadata(html: &str) -> HashMap<String, String> {
    let mut meta = HashMap::new();
    meta.insert("title".to_string(), String::new());
    if let Some(v) = extract_attr(html, "html", "lang") {
        meta.insert("lang".into(), v);
    }
    if let Some(v) = extract_tag_content(html, "title") {
        meta.insert("title".into(), v);
    }
    for name in &["description", "date"] {
        if let Some(v) = extract_meta_content(html, name) {
            meta.insert(name.to_string(), v);
        }
    }
    if let Some(v) = extract_link_canonical(html) {
        meta.insert("link".into(), v);
    }
    meta
}

fn extract_attr(html: &str, tag: &str, attr: &str) -> Option<String> {
    let tag_start = format!("<{}", tag);
    let pos = html.find(&tag_start)?;
    let rest = &html[pos..];
    let end = rest.find('>')?;
    extract_attr_from_str(&rest[..end], attr)
}

fn extract_attr_from_str(attrs: &str, attr: &str) -> Option<String> {
    let dq = format!("{}=\"", attr);
    let sq = format!("{}='", attr);
    if let Some(pos) = attrs.find(&dq) {
        let rest = &attrs[pos + dq.len()..];
        return rest.find('"').map(|e| rest[..e].to_string());
    }
    if let Some(pos) = attrs.find(&sq) {
        let rest = &attrs[pos + sq.len()..];
        return rest.find('\'').map(|e| rest[..e].to_string());
    }
    None
}

fn extract_tag_content(html: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = html.find(&open)? + open.len();
    let end = html[start..].find(&close)?;
    Some(html[start..start + end].to_string())
}

fn extract_meta_content(html: &str, name: &str) -> Option<String> {
    let mut search = html;
    loop {
        let pos = search.find("<meta")?;
        let rest = &search[pos..];
        let end = rest.find('>')?;
        let tag = &rest[..=end];
        let has_name = extract_attr_from_str(tag, "name")
            .map(|n| n == name)
            .unwrap_or(false);
        if has_name {
            return extract_attr_from_str(tag, "content");
        }
        search = &search[pos + 5..];
    }
}

fn extract_link_canonical(html: &str) -> Option<String> {
    let mut search = html;
    loop {
        let pos = search.find("<link")?;
        let rest = &search[pos..];
        let end = rest.find('>')?;
        let tag = &rest[..=end];
        let is_canon = extract_attr_from_str(tag, "rel")
            .map(|r| r == "canonical")
            .unwrap_or(false);
        if is_canon {
            return extract_attr_from_str(tag, "href");
        }
        search = &search[pos + 5..];
    }
}
