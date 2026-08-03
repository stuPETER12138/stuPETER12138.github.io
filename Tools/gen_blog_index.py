#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# ///

"""
Tools/gen_blog_index.py — 自动生成 content/Blog/index.typ 的文章列表

遍历 content/Blog/ 下符合 YYYY-MM-DD-slug 命名规范的文件夹，
提取日期（来自文件夹名）与标题（来自文章 index.typ 的 template.with(title: ...)），
按年份分组、日期倒序，重新生成 index.typ 中 "#tufted.blog-entry(...)" 列表部分。

文件头部（#import / #show: template.with(...)）与列表前的一级标题会被原样保留；
一级标题之后的内容视为自动生成区，会被整体重写。

用法:
    uv run Tools/gen_blog_index.py         # 生成并写入 content/Blog/index.typ
    uv run Tools/gen_blog_index.py -n      # 仅预览变更，不写入
    python Tools/gen_blog_index.py --help  # 查看全部参数
"""

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

# 文件夹命名规范：YYYY-MM-DD-slug（如 2026-04-25-ros2-dds-qos）
POST_DIR_RE = re.compile(r"^(\d{4})-(\d{2})-(\d{2})-(.+)$")
# 文章标题：template.with(title: "...") 中的字符串字面量（支持 \" 转义）
TITLE_RE = re.compile(r'title:\s*"((?:[^"\\]|\\.)*)"', re.DOTALL)
# 一级标题行（自动生成区起点）
HEADING_RE = re.compile(r"^=\s+")
# 列表主体中的一级标题，默认值
DEFAULT_HEADING = "= 博客"


@dataclass(frozen=True)
class Post:
    """一篇博客文章的元数据，全部来自文件夹名 + 文章内 title。"""

    year: int
    month: int
    day: int
    slug: str
    title: str
    folder: str

    @property
    def path(self) -> str:
        """blog-entry 的 path 参数：文件夹名 + 斜杠。"""
        return f"{self.folder}/"

    @property
    def date_key(self) -> tuple[int, int, int]:
        return (self.year, self.month, self.day)


def get_repo_root() -> Path:
    """脚本位于 Tools/ 下，仓库根目录即其父目录。"""
    return Path(__file__).resolve().parent.parent


def extract_title(post_typ: Path) -> str | None:
    """从文章 index.typ 中提取 template.with(title: "...") 的标题。"""
    try:
        text = post_typ.read_text(encoding="utf-8")
    except OSError as e:
        print(f"    ⚠ 无法读取 {post_typ}: {e}")
        return None
    match = TITLE_RE.search(text)
    return match.group(1) if match else None


def list_posts(blog_dir: Path) -> list[Post]:
    """扫描博客目录，解析所有符合命名规范的子文件夹。"""
    posts: list[Post] = []
    skipped: list[str] = []

    for folder in sorted(blog_dir.iterdir()):
        if not folder.is_dir():
            continue
        match = POST_DIR_RE.match(folder.name)
        if not match:
            skipped.append(folder.name)
            continue

        year, month, day, slug = match.groups()
        post_typ = folder / "index.typ"

        title = extract_title(post_typ) if post_typ.is_file() else None
        if title is None:
            print(f"    ⚠ {folder.name}: 未找到 title，使用文件夹名作为标题")
            title = slug

        posts.append(
            Post(
                year=int(year),
                month=int(month),
                day=int(day),
                slug=slug,
                title=title,
                folder=folder.name,
            )
        )

    if skipped:
        print(f"  ℹ 以下文件夹不符合 YYYY-MM-DD-slug 命名规范，已跳过: {', '.join(skipped)}")

    # 按日期 + 文件夹名倒序排序，保证结果稳定可复现
    posts.sort(key=lambda p: (p.date_key, p.folder), reverse=True)
    return posts


def render_entry(post: Post) -> str:
    """渲染单条 #tufted.blog-entry(...)。"""
    return (
        "#tufted.blog-entry(\n"
        f"  date: datetime(year: {post.year}, month: {post.month}, day: {post.day}),\n"
        f'  path: "{post.path}",\n'
        f'  title: "{post.title}",\n'
        ")"
    )


def render_body(posts: list[Post], heading: str) -> str:
    """渲染列表主体：一级标题 + 按年份分组（年份倒序、组内日期倒序）的条目。"""
    groups: dict[int, list[Post]] = {}
    for post in posts:
        groups.setdefault(post.year, []).append(post)

    lines = [heading]
    for year in sorted(groups, reverse=True):
        lines.append("")
        lines.append(f"== {year}")
        for post in groups[year]:
            lines.append("")
            lines.append(render_entry(post))
    return "\n".join(lines) + "\n"


def build_new_content(original: str, body: str) -> tuple[str, str | None]:
    """
    组装新文件内容：保留一级标题之前的部分，其余替换为自动生成的列表。

    返回 (新内容, 复用的标题行)。找不到一级标题时整个文件视为头部，
    使用默认标题并在文件末尾追加列表。
    """
    lines = original.split("\n")
    for idx, line in enumerate(lines):
        if HEADING_RE.match(line):
            heading = line
            header_lines = lines[:idx]
            # 去掉头部末尾的空行，统一由下面补充的 "\n\n" 分隔
            while header_lines and header_lines[-1] == "":
                header_lines.pop()
            header = "\n".join(header_lines)
            return header + "\n\n" + body, heading
    # 文件里还没有列表（可能是空模板），原样保留并追加
    return original.rstrip("\n") + "\n\n" + body, DEFAULT_HEADING


def update_index(
    index_path: Path,
    posts: list[Post],
    dry_run: bool = False,
) -> bool:
    """重写 index.typ 的列表部分，返回文件是否发生变化。"""
    if not posts:
        print("  ⚠ 未找到任何符合规范的博客文章，跳过写入。")
        return False

    original = index_path.read_text(encoding="utf-8")
    # 复用已有标题行（如 "= 博客"），保留用户自定义
    _, old_heading = build_new_content(original, "")
    new_content, heading = build_new_content(original, render_body(posts, old_heading or DEFAULT_HEADING))

    # 统计新增 / 移除的条目（通过 path 参数对比）
    old_paths = set(re.findall(r'path:\s*"([^"]+)"', original))
    new_paths = {post.path for post in posts}
    added = sorted(new_paths - old_paths)
    removed = sorted(old_paths - new_paths)

    print(f"  📄 共找到 {len(posts)} 篇文章，标题行: {heading}")
    if added:
        for path in added:
            print(f"  ➕ 新增: {path}")
    if removed:
        for path in removed:
            print(f"  ➖ 移除: {path}")
    if not added and not removed and new_content == original:
        print("  ✅ 列表已是最新，无需更新。")
        return False

    if dry_run:
        print("  🔎 干跑模式（-n），未写入文件。")
        return True

    # 保留文件原有的换行风格，避免产生无谓的整文件 diff
    newline = "\r\n" if "\r\n" in original else "\n"
    with index_path.open("w", encoding="utf-8", newline="") as f:
        f.write(new_content.replace("\n", newline))
    print(f"  ✅ 已写入 {index_path.relative_to(get_repo_root()).as_posix()}")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="gen_blog_index.py",
        description="遍历 content/Blog/ 下的文章文件夹，自动生成 index.typ 中的博客列表",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""示例:
    uv run Tools/gen_blog_index.py          # 生成并写入
    uv run Tools/gen_blog_index.py -n       # 仅预览变更
""",
    )
    parser.add_argument("-n", "--dry-run", action="store_true", help="只预览变更，不写入文件")
    args = parser.parse_args()

    root = get_repo_root()
    blog_dir = root / "content" / "Blog"
    index_path = blog_dir / "index.typ"

    if not blog_dir.is_dir():
        print(f"❌ 未找到博客目录: {blog_dir}")
        return 1
    if not index_path.is_file():
        print(f"❌ 未找到列表文件: {index_path}")
        return 1

    print(f"🔎 扫描 {blog_dir.relative_to(root).as_posix()}/ …")
    posts = list_posts(blog_dir)
    update_index(index_path, posts, dry_run=args.dry_run)
    return 0


if __name__ == "__main__":
    sys.exit(main())
