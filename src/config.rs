//! 项目配置常量
use std::path::PathBuf;

pub fn content_dir() -> PathBuf {
    PathBuf::from("content")
}
pub fn site_dir() -> PathBuf {
    PathBuf::from("_site")
}
pub fn assets_dir() -> PathBuf {
    PathBuf::from("assets")
}
pub fn config_file() -> PathBuf {
    PathBuf::from("config.typ")
}
