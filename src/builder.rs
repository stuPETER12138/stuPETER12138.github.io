//! 构建命令实现

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::config::{assets_dir, content_dir, site_dir};
use crate::deps::{find_common_dependencies, get_file_mtime, needs_rebuild};
use crate::stats::BuildStats;
use crate::typst::{
    find_typ_files, get_output_path, html_compile_args, pdf_compile_args, run_typst_command,
};

fn compile_files<F>(
    files: &[PathBuf],
    force: bool,
    common_deps: &[PathBuf],
    get_output: impl Fn(&PathBuf) -> PathBuf,
    build_args: F,
) -> Result<BuildStats>
where
    F: Fn(&PathBuf, &PathBuf) -> Vec<String>,
{
    let mut stats = BuildStats::default();
    for typ_file in files {
        let output_path = get_output(typ_file);
        if !force && !needs_rebuild(typ_file, &output_path, common_deps) {
            stats.skipped += 1;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let args = build_args(typ_file, &output_path);
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        if run_typst_command(&arg_refs)? {
            stats.success += 1;
        } else {
            println!("  ❌ {} 编译失败", typ_file.display());
            stats.failed += 1;
        }
    }
    Ok(stats)
}

pub fn build_html(force: bool) -> Result<bool> {
    fs::create_dir_all(site_dir())?;
    let typ_files = find_typ_files();
    let html_files: Vec<PathBuf> = typ_files
        .into_iter()
        .filter(|f| {
            !f.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase()
                .contains("pdf")
        })
        .collect();
    if html_files.is_empty() {
        println!("  ⚠️ 未找到任何 HTML 文件。");
        return Ok(true);
    }
    println!("正在构建 HTML 文件...");
    let common_deps = find_common_dependencies();
    let stats = compile_files(
        &html_files,
        force,
        &common_deps,
        |f| get_output_path(f, "html"),
        |f, o| html_compile_args(f, o),
    )?;
    println!("✅ HTML 构建完成。{}", stats.format_summary());
    Ok(!stats.has_failures())
}

pub fn build_pdf(force: bool) -> Result<bool> {
    fs::create_dir_all(site_dir())?;
    let typ_files = find_typ_files();
    let pdf_files: Vec<PathBuf> = typ_files
        .into_iter()
        .filter(|f| {
            f.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase()
                .contains("pdf")
        })
        .collect();
    if pdf_files.is_empty() {
        return Ok(true);
    }
    println!("正在构建 PDF 文件...");
    let common_deps = find_common_dependencies();
    let stats = compile_files(
        &pdf_files,
        force,
        &common_deps,
        |f| get_output_path(f, "pdf"),
        |f, o| pdf_compile_args(f, o),
    )?;
    println!("✅ PDF 构建完成。{}", stats.format_summary());
    Ok(!stats.has_failures())
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn copy_assets() -> Result<bool> {
    let assets = assets_dir();
    if !assets.exists() {
        println!("  ⚠ 静态资源目录 {} 不存在。", assets.display());
        return Ok(true);
    }
    fs::create_dir_all(site_dir())?;
    let target = site_dir().join("assets");
    if target.exists() {
        fs::remove_dir_all(&target)?;
    }
    copy_dir_all(&assets, &target)?;
    Ok(true)
}

pub fn copy_content_assets(force: bool) -> Result<bool> {
    let content = content_dir();
    let site = site_dir();
    fs::create_dir_all(&site)?;
    if !content.exists() {
        println!("  ⚠ 内容目录 {} 不存在，跳过。", content.display());
        return Ok(true);
    }
    let walker = walkdir::WalkDir::new(&content)
        .into_iter()
        .filter_map(|e| e.ok());
    for entry in walker {
        let path = entry.path();
        if path.is_dir() || path.extension().and_then(|e| e.to_str()) == Some("typ") {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(&content) {
            let has_underscore = rel
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('_'));
            if has_underscore {
                continue;
            }
            let target_path = site.join(rel);
            if !force
                && target_path.exists()
                && get_file_mtime(path) <= get_file_mtime(&target_path)
            {
                continue;
            }
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target_path)?;
        }
    }
    Ok(true)
}

pub fn clean() -> Result<bool> {
    let site = site_dir();
    println!("正在清理生成的文件...");
    if !site.exists() {
        println!("  输出目录 {} 不存在，无需清理。", site.display());
        return Ok(true);
    }
    for entry in fs::read_dir(&site)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    println!("  ✅ 已清理 {}/ 目录。", site.display());
    Ok(true)
}

pub fn build(force: bool) -> Result<bool> {
    println!("{}", "-".repeat(60));
    if force {
        clean()?;
        println!("🛠️  开始完整构建...");
    } else {
        println!("🚀 开始增量构建...");
    }
    println!("{}", "-".repeat(60));
    fs::create_dir_all(site_dir())?;
    let mut results = Vec::new();
    println!();
    results.push(build_html(force)?);
    results.push(build_pdf(force)?);
    println!();
    results.push(copy_assets()?);
    results.push(copy_content_assets(force)?);
    if let Some(site_url) = crate::feed::get_site_url() {
        results.push(crate::feed::generate_sitemap(&site_url)?);
        results.push(crate::feed::generate_robots_txt(&site_url)?);
        results.push(crate::feed::generate_rss(&site_url)?);
    }
    println!("{}", "-".repeat(60));
    let all_ok = results.iter().all(|&r| r);
    if all_ok {
        println!("✅ 所有构建任务完成！");
        let abs = site_dir().canonicalize().unwrap_or(site_dir());
        println!("  📂 输出目录: {}", abs.display());
    } else {
        println!("⚠  构建完成，但有部分任务失败。");
    }
    println!("{}", "-".repeat(60));
    Ok(all_ok)
}
