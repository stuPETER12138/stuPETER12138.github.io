# Personal Academic Homepage

基于 **Eleventy 3 + Nunjucks** 的轻量个人学术主页。灰白底色、墨绿色点缀、衬线标题与留白布局，支持响应式设计、深浅主题、键盘导航和打印。无外部字体、前端框架或 CDN 依赖，关闭 JavaScript 后仍可阅读全部内容。

> 当前姓名、机构和研究方向为明确的占位文案；论文来自你提供的 `papers.bib`；动态和经历为空列表，未虚构任何学术记录。发布前请替换个人资料。

## 本地开发

安装 Node.js 22 或更新版本，然后运行：

```sh
npm ci
npm run dev
```

打开终端提示的地址（通常是 `http://localhost:8080`）。修改源码后 Eleventy 会自动重建。

```sh
npm run build  # 生成 _site/
npm test       # 构建并检查章节、资源、内部链接、元数据与站点地图
```

`_site/` 与 `node_modules/` 不提交到 Git。`package-lock.json` 必须提交，GitHub Actions 使用 `npm ci` 安装锁定版本。

## 修改个人信息

日常维护主要编辑 **`src/_data/site.json`**，无需修改 HTML：

| 字段 | 用途 |
| --- | --- |
| `title`, `description`, `url` | 搜索摘要、规范链接、站点地图中的网站地址 |
| `name`, `englishName`, `initials` | 中文姓名、英文姓名、头像/标识缩写 |
| `role`, `department`, `institution`, `location` | 职位、院系、机构、所在地 |
| `intro`, `about`, `tagline` | 简介、详细介绍段落、个人寄语 |
| `email` | 邮箱；为空时隐藏邮件按钮 |
| `portrait` | 头像路径；为空时显示缩写头像 |
| `cv` | 简历 PDF 路径；为空时隐藏简历链接 |
| `social` | GitHub、ORCID 等链接 |
| `research` | 研究方向卡片 |
| `news`, `experience` | 动态、学习与研究经历（论文单独维护于 `papers.bib`） |

上传头像到 `src/assets/images/portrait.jpg` 后，设置 `"portrait": "/assets/images/portrait.jpg"`。上传简历到 `src/assets/files/cv.pdf` 后，设置 `"cv": "/assets/files/cv.pdf"`。按需创建目录。

JSON 字符串内容按纯文本展示，不需要写 HTML；使用有效 JSON，不可加入注释或末尾多余逗号。链接请使用可信的 `https://` 地址或站内路径。

### 添加最新动态

将条目添加到 `news` 数组，按希望显示的顺序排列（建议最新在前）：

```json
{
  "date": "2026-01-15",
  "tag": "动态",
  "text": "在这里填写真实的研究进展或学术活动。",
  "url": "https://example.org/"
}
```

`date` 使用 `YYYY-MM-DD` 格式，`tag` 和 `url` 可省略。

### 用 BibTeX 维护论文

根目录 **`papers.bib` 是论文列表的唯一数据源**。不使用 Google Scholar，不需要在 `site.json` 中重复填写论文。使用 Citation.js 解析标准 BibTeX，包括多行作者、引号/花括号、LaTeX 重音和嵌套花括号。

```bibtex
@article{uniqueKey,
  title = {Your Paper Title},
  author = {Family, Given and Another, Author},
  journal = {Journal Name},
  year = {2026},
  doi = {10.1234/example},
  abstract = {Optional abstract.}
}
```

- 支持 `title`、`author`、`year`、`journal` / `booktitle`、`publisher`、`doi`、`url`、`abstract` 和 `note`。
- arXiv 条目可使用 `eprint` 与 `archivePrefix={arXiv}`，自动生成预印本标签和链接。
- 按年份降序排列，同年保留文件中的顺序；年份缺失的条目排在最后。不根据 arXiv 编号修改你填写的年份。
- 标题链接优先使用有效 HTTP(S) `url`，其次是 `doi`，最后是 arXiv 地址；无链接则显示普通标题。不会把 citation key 自动认定为 DOI。
- 摘要使用折叠展示。缺失的可选字段不会虚构补全。重复引用键、缺少标题或解析失败会阻止构建，避免静默丢失论文。
- 当前 SentiMM 条目的引用键看起来是 DOI，但未填写 `doi` / `url` 字段，因此暂不生成链接。核实后可自行添加 `doi = {10.1007/978-981-95-5679-3_15}`。

本地运行 `npm run dev` 后，保存 `papers.bib` 会触发重建。将更新后的文件提交并推送到部署分支，GitHub Actions 会重新构建并发布。这个流程不访问外部学术服务，也不会自动发现新论文。

修改解析代码后建议重启开发服务器。

#### 统一格式约定

- 条目类型和字段名小写；双空格缩进；字段值统一使用花括号；每个字段独立一行，末尾保留逗号。
- 常用字段顺序为 `title` → `author` → `year` → 发表载体及出版信息 → 标识符/链接 → `abstract`；作者和编辑使用单行 `and` 分隔，保留原姓名表示法和作者顺序。
- 不修改引用键、年份及事实信息，不从引用键推断 DOI，也不删除额外元数据。
- 页面采用统一信息层级：类型与年份 → 标题 → 全部作者 → 发表载体与页码 → 资源链接与折叠摘要。它是主页展示样式，不宣称符合 APA / IEEE 等正式参考文献规范。
- 页码在 BibTeX 中使用 `--`，页面中统一显示为 `–`。期刊／会议条目即使有 arXiv 链接，也优先标记为期刊／会议论文。
- 论文区域提供“下载 BibTeX”按钮，发布时原始 `papers.bib` 会复制到网站根目录公开下载。不要在该文件中存放私人注释或未公开资料。

样式文件 `src/assets/css/style.css` 按设计变量、导航、个人资料、栏目、论文、响应式和打印样式分区组织。移动端完整保留作者名单和论文标题，不截断长内容。

### 添加教育或研究经历

加入 `experience` 数组：

```json
{
  "period": "起始年份 — 至今",
  "title": "学位 / 研究职位",
  "organization": "真实学校或机构名称",
  "description": "可选：研究主题、导师或职责。"
}
```

## GitHub Pages 部署

本仓库远程地址已指向 `stuPETER12138/stuPETER12138.github.io`，默认站点地址为：

**https://stupeter12138.github.io/**

1. 在 GitHub 仓库打开 **Settings → Pages**。
2. 在 **Build and deployment → Source** 中选择 **GitHub Actions**（不是 Deploy from a branch）。
3. 将代码和 `package-lock.json` 提交并推送到 `main` 或 `master` 分支。
4. 打开仓库 **Actions**，查看 **Deploy Eleventy to GitHub Pages** 工作流。
5. 工作流成功后，在部署环境或 Settings → Pages 中打开站点。首次发布可能需要几分钟。

也可在 Actions 中使用 **Run workflow** 手动触发。若实际发布分支不是 `main` 或 `master`，请修改 `.github/workflows/deploy.yml` 中的分支列表。如果对 `github-pages` 环境设置了分支限制，请确保放行你的发布分支。

流程为：检出代码 → Node.js 22 → `npm ci` → `npm test` → 上传 `_site` → GitHub Pages 官方部署。部署 job 使用 `pages: write` 与 `id-token: write`，无需个人访问令牌，也不需要生成 `gh-pages` 分支。

**本地创建工作流不等于已经上线**：仍需要由仓库所有者启用 Pages 并推送代码。

### 域名与路径

当前配置面向此仓库的用户主页（域名根路径），不是 `/repository/` 子路径网站。如果迁移至项目级 Pages，需配置 Eleventy 的 `pathPrefix` 并同步调整站点 URL 与构建检查中的路径处理。

使用自定义域名时：

- 在 GitHub Pages 设置中填写域名并按 GitHub 文档配置 DNS；
- 更新 `site.json` 中的 `url`（保留末尾 `/`）；
- 更新 `src/robots.txt` 中的 Sitemap 地址；
- 如需通过文件保留域名设置，可添加 `src/CNAME`，并在 `eleventy.config.js` 中加入对应 passthrough copy。

## 项目结构

```text
.github/workflows/deploy.yml   GitHub Pages 自动部署
papers.bib                    唯一论文数据源
scripts/publications.mjs       BibTeX 解析和论文数据映射
scripts/publications.test.mjs  解析器测试
scripts/check-build.mjs        构建产物检查
src/
  _data/publications.js        构建时加载 BibTeX
  _data/site.json              个人信息及非论文栏目数据
  _includes/base.njk          公共布局与 SEO 元数据
  assets/css/style.css        响应式样式、主题与打印样式
  assets/js/main.js           主题切换与本地记忆
  assets/favicon.svg         网站图标
  index.njk                  学术主页
  404.njk                    自定义 404
  sitemap.njk                站点地图
  robots.txt                 搜索引擎抓取规则
eleventy.config.js           Eleventy 配置
package.json                 命令与依赖
package-lock.json            锁定依赖版本
```

## 发布前检查

- [ ] 替换姓名、英文名、机构、研究方向及默认 SEO 描述。
- [ ] 移除简介中的方括号提示文字。
- [ ] 上传头像、简历并填写邮箱（可选，公开邮箱可能收到垃圾邮件）。
- [ ] 添加真实成果，核对作者、链接与发表信息。
- [ ] 执行 `npm test`。
- [ ] 在浏览器检查桌面、手机、深浅主题及键盘操作。
- [ ] GitHub Pages Source 选择 GitHub Actions，并确认部署运行成功。

仓库原有 `LICENSE` 保持不变。
