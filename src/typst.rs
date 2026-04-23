//! Typst 编译器集成

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

use crate::config::{assets_dir, content_dir, site_dir};

pub fn find_typ_files() -> Vec<PathBuf> {
    let content = content_dir();
    let mut files = Vec::new();
    let walker = walkdir::WalkDir::new(&content)
        .into_iter()
        .filter_map(|e| e.ok());
    for entry in walker {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("typ") {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(&content) {
            let has_underscore = rel
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('_'));
            if !has_underscore {
                files.push(path.to_path_buf());
            }
        }
    }
    files
}

pub fn get_output_path(typ_file: &Path, ext: &str) -> PathBuf {
    let rel = typ_file.strip_prefix(content_dir()).unwrap_or(typ_file);
    site_dir().join(rel).with_extension(ext)
}

pub fn run_typst_command(args: &[&str]) -> Result<bool> {
    match Command::new("typst").args(args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("  ❌ Typst 错误: {}", stderr.trim());
                Ok(false)
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("  ❌ 错误: 未找到 typst 命令。请确保已安装 Typst 并添加到 PATH。");
            println!("  📝 安装说明: https://typst.app/open-source/#download");
            Ok(false)
        }
        Err(e) => {
            println!("  ❌ 执行 typst 命令时出错: {e}");
            Ok(false)
        }
    }
}

pub fn html_compile_args(typ_file: &Path, output_path: &Path) -> Vec<String> {
    let content = content_dir();
    let page_path = if let Ok(rel) = typ_file.strip_prefix(&content) {
        if rel.file_name().and_then(|n| n.to_str()) == Some("index.typ") {
            let parent = rel.parent().unwrap_or(Path::new(""));
            let s = parent.to_string_lossy().replace('\\', "/");
            if s == "." { String::new() } else { s }
        } else {
            rel.with_extension("").to_string_lossy().replace('\\', "/")
        }
    } else {
        String::new()
    };

    vec![
        "compile".into(),
        "--root".into(),
        ".".into(),
        "--font-path".into(),
        assets_dir().to_string_lossy().into_owned(),
        "--features".into(),
        "html".into(),
        "--format".into(),
        "html".into(),
        "--input".into(),
        format!("page-path={page_path}"),
        typ_file.to_string_lossy().into_owned(),
        output_path.to_string_lossy().into_owned(),
    ]
}

pub fn pdf_compile_args(typ_file: &Path, output_path: &Path) -> Vec<String> {
    vec![
        "compile".into(),
        "--root".into(),
        ".".into(),
        "--font-path".into(),
        assets_dir().to_string_lossy().into_owned(),
        typ_file.to_string_lossy().into_owned(),
        output_path.to_string_lossy().into_owned(),
    ]
}
