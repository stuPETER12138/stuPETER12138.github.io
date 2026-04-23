//! 增量编译依赖分析

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use regex::Regex;

use crate::config::{config_file, content_dir};

pub fn get_file_mtime(path: &Path) -> u64 {
    path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn is_dep_file(path: &Path) -> bool {
    let Ok(resolved) = path.canonicalize() else {
        return true;
    };
    if let Ok(cfg) = config_file().canonicalize() {
        if resolved == cfg {
            return true;
        }
    }
    if let Ok(content) = content_dir().canonicalize() {
        if let Ok(rel) = resolved.strip_prefix(&content) {
            if let Some(first) = rel.components().next() {
                return first.as_os_str().to_string_lossy().starts_with('_');
            }
            return false;
        }
    }
    true
}

pub fn find_typ_dependencies(typ_file: &Path) -> HashSet<PathBuf> {
    let mut deps = HashSet::new();
    let content = match std::fs::read_to_string(typ_file) {
        Ok(c) => c,
        Err(_) => return deps,
    };
    let base_dir = typ_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let patterns = [
        r#"#import\s+"([^"]+)""#,
        r#"#import\s+'([^']+)'"#,
        r#"#include\s+"([^"]+)""#,
        r#"#include\s+'([^']+)'"#,
    ];
    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        for cap in re.captures_iter(&content) {
            let dep_str = &cap[1];
            if dep_str.starts_with('@') {
                continue;
            }
            let dep_path = if dep_str.starts_with('/') {
                PathBuf::from(dep_str.trim_start_matches('/'))
            } else {
                base_dir.join(dep_str)
            };
            if let Ok(resolved) = dep_path.canonicalize() {
                if resolved.exists()
                    && resolved.extension().and_then(|e| e.to_str()) == Some("typ")
                    && is_dep_file(&resolved)
                {
                    deps.insert(resolved);
                }
            }
        }
    }
    deps
}

pub fn get_all_dependencies(typ_file: &Path, visited: &mut HashSet<PathBuf>) -> HashSet<PathBuf> {
    let abs = match typ_file.canonicalize() {
        Ok(p) => p,
        Err(_) => return HashSet::new(),
    };
    if visited.contains(&abs) {
        return HashSet::new();
    }
    visited.insert(abs);
    let mut all_deps = HashSet::new();
    for dep in find_typ_dependencies(typ_file) {
        all_deps.insert(dep.clone());
        if dep.extension().and_then(|e| e.to_str()) == Some("typ") {
            all_deps.extend(get_all_dependencies(&dep, visited));
        }
    }
    all_deps
}

pub fn needs_rebuild(source: &Path, target: &Path, extra_deps: &[PathBuf]) -> bool {
    if !target.exists() {
        return true;
    }
    let target_mtime = get_file_mtime(target);
    if get_file_mtime(source) > target_mtime {
        return true;
    }
    for dep in extra_deps {
        if dep.exists() && get_file_mtime(dep) > target_mtime {
            return true;
        }
    }
    let mut visited = HashSet::new();
    for dep in get_all_dependencies(source, &mut visited) {
        if get_file_mtime(&dep) > target_mtime {
            return true;
        }
    }
    if let Some(src_dir) = source.parent() {
        if let Ok(entries) = std::fs::read_dir(src_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|e| e.to_str()) != Some("typ") {
                    if get_file_mtime(&p) > target_mtime {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn find_common_dependencies() -> Vec<PathBuf> {
    let mut deps = Vec::new();
    let cfg = config_file();
    if cfg.exists() {
        deps.push(cfg);
    }
    let content = content_dir();
    if content.exists() {
        if let Ok(entries) = std::fs::read_dir(&content) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if name.starts_with('_') {
                        let walker = walkdir::WalkDir::new(&path)
                            .into_iter()
                            .filter_map(|e| e.ok());
                        for e in walker {
                            if e.path().is_file()
                                && e.path().extension().and_then(|x| x.to_str()) == Some("typ")
                            {
                                deps.push(e.path().to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }
    deps
}
