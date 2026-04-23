```
tufted-build/
├── Cargo.toml          # 依赖：clap / walkdir / regex / chrono / anyhow
└── src/
    ├── main.rs         # CLI 入口 (clap 子命令解析 + 项目根目录定位)
    ├── config.rs       # 路径常量 (content/ / _site/ / assets/ / config.typ)
    ├── stats.rs        # BuildStats 统计结构体
    ├── deps.rs         # 增量编译核心：依赖解析 + needs_rebuild 判断
    ├── typst.rs        # Typst CLI 封装：文件发现 + 参数构建 + 执行
    ├── html_meta.rs    # HTML 元数据解析 (lang/title/description/link/date)
    ├── builder.rs      # 构建命令：build/html/pdf/assets/clean
    ├── feed.rs         # 站点元文件生成：RSS 2.0 / sitemap.xml / robots.txt
    └── preview.rs      # 本地预览服务器 (python3 http.server + 自动开浏览器)
```

```bash
cargo build --release
.\target\release\stuPETER12138_github_io.exe build         # 增量构建
.\target\release\stuPETER12138_github_io.exe build --force # 强制全量构建
.\target\release\stuPETER12138_github_io.exe preview -p 3000
```
