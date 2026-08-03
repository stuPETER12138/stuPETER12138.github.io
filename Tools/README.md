# Tools

## gen_blog_index.py — 自动生成博客列表

遍历 `content/Blog/` 下符合 `YYYY-MM-DD-slug` 命名规范的文件夹，提取日期（来自文件夹名）
与标题（来自文章 `index.typ` 的 `template.with(title: ...)`），按年份分组、日期倒序，
自动重写 `content/Blog/index.typ` 中 `#tufted.blog-entry(...)` 列表部分。

在项目根目录执行：

```sh
# 生成并写入（新增文章后跑一次即可）
uv run gen_blog_index.py

# 仅预览变更，不写入
uv run gen_blog_index.py -n
```

行为细节：

- **保留头部**：`#import` / `#show: template.with(...)` 以及列表前的一级标题（如 `= 博客`）原样保留；
- **新增文章**：新建 `content/Blog/YYYY-MM-DD-slug/` 文件夹并写好文章后运行即可，无需手动维护列表；
- **移除文章**：删除文件夹后运行，对应条目会自动消失；
- **幂等**：重复运行不会产生多余改动；
- **容错**：标题提取失败时回退用文件夹名，不符合命名规范的文件夹会跳过并提示。

## watch.py — 文件监控 + 自动增量构建

基于 `build.py` 的构建逻辑，监控 `content/`、`assets/` 与 `config.typ` 的变化，
文件变动稳定后自动执行 `build.py build`（增量构建）。配合 `build.py preview`
（uvx livereload）即可实现「保存即刷新」的本地预览，无需手动反复跑构建。

在项目根目录执行：

```sh
# 仅监控 + 自动构建（配合另一个终端里的 build.py preview 使用）
uv run watch.py

# 一步到位：自动构建 + 启动预览服务器（浏览器自动打开）
uv run watch.py --serve

# 自定义预览端口
uv run watch.py -s -p 3000
```

参数说明：

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-s, --serve` | 同时启动预览服务器（`build.py preview`） | 关闭 |
| `-p, --port N` | 预览服务器端口号（需配合 `-s`） | `8000` |
| `--poll N` | 轮询间隔秒数 | `0.5` |
| `--debounce N` | 变更防抖秒数，文件稳定这么久后才构建 | `0.8` |
| `--no-initial-build` | 跳过启动时的首次构建 | 关闭 |
| `-h, --help` | 查看帮助信息 | - |

行为细节：

- **首次构建**：启动时自动执行一次增量构建，保证 `_site/` 与源码同步；
- **防抖**：持续写入的文件会延迟到写入稳定后才触发构建，避免编译到半成品；
- **覆盖范围**：`content/`（含 `_*` 模板目录）、`assets/`、`config.typ`，与 `build.py` 的依赖模型一致；
- **容错**：构建失败不影响监控，修改文件后会自动重试；
- **停止**：按 `Ctrl+C` 退出，若启动了预览服务器会一并关闭。

## img2webp.py — 图片批量转 WebP

将 `content/` 下所有 `imgs` 目录中的 JPG/PNG 图片批量转换为 WebP 格式，
转换结果保存在原目录，用于博客图片优化。

## 基本用法

在项目根目录执行：

```sh
cd .\Tools\
uv run img2webp.py
# 转换 content/imgs/ 下所有图片
```

脚本会递归遍历 `content/` 下所有名为 `imgs` 的目录（含子目录），
找到所有 `.jpg` / `.jpeg` / `.png` 文件，转换为同名 `.webp` 文件。

## 参数说明

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-q, --quality N` | 有损压缩质量（0-100） | `80` |
| `-l, --lossless` | 无损模式（忽略 `-q`） | 关闭 |
| `-f, --force` | 强制重新转换，覆盖已存在的 `.webp` | 关闭 |
| `-d, --delete` | 转换成功后删除原 JPG/PNG | 关闭 |
| `-h, --help` | 查看帮助信息 | - |

## 示例

```sh
# 默认质量转换
uv run img2webp.py

# 高质量转换（适合照片）
uv run img2webp.py -q 90

# 无损转换（适合截图、带文字的图片）
uv run img2webp.py --lossless

# 原图已不需要时，转换后自动删除 JPG/PNG
uv run img2webp.py -d

# 更新了原图后强制重新转换
uv run img2webp.py -f
```

## 工作流程

日常更新图片的流程：

1. 将新图片（jpg/png）放入 `content/` 下任意 `imgs` 目录；
2. 运行 `uv run img2webp.py`；
3. 提交 `.webp` 文件即可，原图不会进入版本库。

## 行为细节

- **增量转换**：已存在且比原图新的 `.webp` 会自动跳过；替换了原图后重跑脚本即可更新。
- **透明度**：PNG 的 RGBA 透明度会保留。
- **EXIF**：照片的拍摄信息等元数据会尽量保留。
- **容错**：单张图片转换失败不影响其他图片，最后会汇总统计。

## 常见问题

**质量怎么选**

- 照片：`-q 80~90`，体积与画质平衡较好；
- 截图 / 文字 / 图标：用 `--lossless` 无损模式，避免文字边缘发糊。

**图片在非 imgs 目录**

脚本只处理路径中带有 `imgs` 目录的图片（与 `.gitignore` 的忽略范围一致），
放在其他目录的图片不会被转换。若需要调整匹配规则，修改脚本中
`collect_images` 函数的判断条件即可。
