# AGENTS.md — 魔法窝瓜 (Typst Blog)

## Build & Dev

```bash
uv run build.py build        # incremental build (HTML + PDF + assets + RSS + sitemap)
uv run build.py build -f     # full rebuild (cleans `_site/` first)
uv run build.py preview      # livereload server on :8000 (uses `uvx livereload`, falls back to `python -m http.server`)
uv run build.py html         # HTML only
uv run build.py assets       # copy static assets only (from `assets/` → `_site/assets/`)
uv run build.py clean        # delete `_site/`
```

All commands auto-detect project root. Prerequisites: `typst` CLI, `uv`, Python ≥3.10.

## Architecture

- **Typst → HTML static site** using [Tufted-Blog-Template](https://github.com/Yousa-Mirage/Tufted-Blog-Template) — NOT JavaScript/Node.
- `content/` — source `.typ` files. Each subdirectory with an `index.typ` becomes a page route.
- `_site/` — build output (gitignored). Two CI deploy targets: **Cloudflare Pages** (`cloudflare-pages.yml`) and **GitHub Pages** (`deploy.yml`).
- `config.typ` — site-wide template config (nav links, title, RSS feed dirs, footer). Imported by every page.

## Content conventions

- **Blog posts**: `content/Blog/YYYY_MM_DD-slug-name/index.typ`. Import template via `#import "../index.typ": template, tufted`. Set `#show: template.with(title: ..., date: ..., description: ...)`.
- **PDF pages**: if filename contains `pdf` (e.g., `CV-pdf.typ`), it's compiled as PDF instead of HTML.
- **`content/_*` directories** (underscore-prefixed) hold shared Typst dependencies — not standalone pages.
- Non-`.typ` files in `content/` (images, `.md`, `.bib`) are copied to `_site/` as assets by the build script.
- Upstream template sync via manual `update.yml` workflow (pushes a PR from `Yousa-Mirage/Tufted-Blog-Template`).

## Key quirks

- Template uses Typst's **experimental HTML feature** (`--format html --features html`).
- `--font-path assets/` is passed to `typst compile` — fonts live in `assets/`.
- Build script uses `uv run` (PEP 723 inline script deps). Works with plain `python build.py` too.
- No formatter, linter, or typechecker. No tests. No package.json.