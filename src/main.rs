//! Tufted Blog Template 构建工具 (Rust 版)

mod builder;
mod config;
mod deps;
mod feed;
mod html_meta;
mod preview;
mod stats;
mod typst;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "build",
    about = "Tufted Blog Template 构建脚本 - 将 content/ 中的 Typst 文件编译为 HTML 和 PDF",
    after_help = "默认只重新编译修改过的文件，可使用 -f/--force 强制完整重建。"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 完整构建 (HTML + PDF + 资源 + RSS + Sitemap)
    Build {
        /// 强制完整重建（忽略增量检查）
        #[arg(short, long)]
        force: bool,
    },
    /// 仅构建 HTML 文件
    Html {
        #[arg(short, long)]
        force: bool,
    },
    /// 仅构建 PDF 文件
    Pdf {
        #[arg(short, long)]
        force: bool,
    },
    /// 仅复制静态资源
    Assets,
    /// 清理 _site/ 目录
    Clean,
    /// 启动本地预览服务器
    Preview {
        /// 端口号（默认 8000）
        #[arg(short, long, default_value_t = 8000)]
        port: u16,
        /// 不自动打开浏览器
        #[arg(long)]
        no_open: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // 切换到项目根目录（包含 content/ 或 config.typ 的目录）
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if let Some(root) = find_project_root(dir) {
                let _ = std::env::set_current_dir(&root);
            }
        }
    }

    let result = match cli.command {
        Commands::Build { force } => builder::build(force),
        Commands::Html { force } => builder::build_html(force),
        Commands::Pdf { force } => builder::build_pdf(force),
        Commands::Assets => builder::copy_assets(),
        Commands::Clean => builder::clean(),
        Commands::Preview { port, no_open } => preview::preview(port, !no_open),
    };

    match result {
        Ok(true) => process::exit(0),
        Ok(false) => process::exit(1),
        Err(e) => {
            eprintln!("❌ 错误: {e}");
            process::exit(1);
        }
    }
}

fn find_project_root(start: &std::path::Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("content").exists() || dir.join("config.typ").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}
