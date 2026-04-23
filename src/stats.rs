//! 构建统计信息

#[derive(Debug, Default)]
pub struct BuildStats {
    pub success: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl BuildStats {
    pub fn format_summary(&self) -> String {
        let mut parts = Vec::new();
        if self.success > 0 {
            parts.push(format!("编译: {}", self.success));
        }
        if self.skipped > 0 {
            parts.push(format!("跳过: {}", self.skipped));
        }
        if self.failed > 0 {
            parts.push(format!("失败: {}", self.failed));
        }
        if parts.is_empty() {
            "无文件需要处理".to_string()
        } else {
            parts.join(", ")
        }
    }
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}
