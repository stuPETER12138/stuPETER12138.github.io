#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# ///

"""
Tools/watch.py — 监控源文件并自动增量构建

基于 build.py 的构建逻辑：轮询监控 content/、assets/ 与 config.typ 的变化，
文件变动稳定后自动执行 `build.py build`（增量构建），配合
`build.py preview`（uvx livereload 自动刷新浏览器）即可实现"保存即刷新"。

用法:
    uv run Tools/watch.py                     # 仅监控 + 自动构建
    uv run Tools/watch.py --serve             # 同时启动预览服务器（自动开浏览器）
    uv run Tools/watch.py -s -p 3000          # 自定义预览端口
    uv run Tools/watch.py --no-initial-build  # 跳过启动时的首次构建
    python Tools/watch.py --help              # 查看全部参数

提示:
    - 运行 watch.py 时无需再手动执行 build，文件变更后会自动增量构建；
    - 若你已在另一个终端运行 `build.py preview`，直接用 watch.py 即可联动刷新；
    - 持续写入的文件会延迟到写入稳定（防抖窗口）后才触发构建，避免半成品编译。
"""

import argparse
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

# 递归监控的目录（对应 build.py 的 CONTENT_DIR / ASSETS_DIR）
WATCH_DIRS = ("content", "assets")
# 根目录下的单文件依赖（config.typ 改动会触发全站重建）
WATCH_FILES = ("config.typ",)

DEFAULT_POLL = 0.5  # 轮询间隔（秒）
DEFAULT_DEBOUNCE = 0.8  # 防抖窗口（秒），文件稳定这么久后才开始构建
MAX_CHANGED_SHOWN = 10  # 变更列表最多打印多少条


def now() -> str:
    """当前时间戳，用于日志前缀。"""
    return datetime.now().strftime("[%H:%M:%S]")


def get_repo_root() -> Path:
    """脚本位于 Tools/ 下，仓库根目录即其父目录。"""
    return Path(__file__).resolve().parent.parent


def get_watch_paths(root: Path) -> list[Path]:
    """返回待监控路径（目录 + 单文件）。路径暂不存在也会保留，出现后自动纳入。"""
    return [root / name for name in WATCH_DIRS] + [root / name for name in WATCH_FILES]


def take_snapshot(watch_paths: list[Path]) -> dict[str, int]:
    """
    记录所有被监控文件的最新修改时间（mtime_ns）。

    返回 {绝对路径字符串: mtime_ns}。目录会递归展开，新增/删除文件天然会被捕获。
    """
    snap: dict[str, int] = {}
    for path in watch_paths:
        if path.is_dir():
            for item in path.rglob("*"):
                if item.is_file():
                    try:
                        snap[str(item)] = item.stat().st_mtime_ns
                    except OSError:
                        pass  # 文件可能正在被写入/删除，跳过本次轮询
        elif path.is_file():
            try:
                snap[str(path)] = path.stat().st_mtime_ns
            except OSError:
                pass
    return snap


def diff_snapshots(old: dict[str, int], new: dict[str, int]) -> tuple[list[str], list[str], list[str]]:
    """对比两次快照，返回 (新增, 删除, 修改) 的文件列表。"""
    added = [p for p in new if p not in old]
    removed = [p for p in old if p not in new]
    modified = [p for p in old if p in new and old[p] != new[p]]
    return added, removed, modified


def print_changes(root: Path, debounce: float, added: list[str], removed: list[str], modified: list[str]) -> None:
    """打印变更摘要（相对仓库根目录的路径）。"""
    entries: list[tuple[str, str]] = []
    entries += [("➕", p) for p in added]
    entries += [("➖", p) for p in removed]
    entries += [("✏️ ", p) for p in modified]

    print(f"{now()} 📁 检测到文件变更：")
    shown = entries[:MAX_CHANGED_SHOWN]
    for icon, path in shown:
        rel = Path(path).relative_to(root)
        print(f"    {icon} {rel.as_posix()}")
    if len(entries) > len(shown):
        print(f"    … 共 {len(entries)} 处变更")
    print(f"    等待 {debounce}s 无新变更后开始构建…")


def run_build(root: Path) -> bool:
    """调用 build.py 执行增量构建（与手动执行 `build.py build` 等价）。"""
    cmd = [sys.executable, str(root / "build.py"), "build"]
    print(f"{now()} 🔨 开始增量构建…")
    start = time.monotonic()
    proc = subprocess.run(cmd, cwd=root)
    elapsed = time.monotonic() - start
    ok = proc.returncode == 0
    print(f"{now()} {'✅ 构建成功' if ok else '⚠ 构建失败'}（耗时 {elapsed:.1f}s）")
    return ok


def start_preview_server(root: Path, port: int) -> subprocess.Popen | None:
    """启动 build.py preview 作为子进程（优先 uvx livereload，失败回退 http.server）。"""
    cmd = [sys.executable, str(root / "build.py"), "preview", "-p", str(port)]
    try:
        proc = subprocess.Popen(cmd, cwd=root)
    except OSError as e:
        print(f"{now()} ❌ 启动预览服务器失败: {e}")
        return None
    print(f"{now()} 🚀 预览服务器已启动：http://localhost:{port}（浏览器将自动打开）")
    return proc


def watch(
    root: Path,
    serve: bool = False,
    port: int = 8000,
    poll: float = DEFAULT_POLL,
    debounce: float = DEFAULT_DEBOUNCE,
    initial: bool = True,
) -> None:
    watch_paths = get_watch_paths(root)

    missing = [p for p in watch_paths if not p.exists()]
    if missing:
        names = ", ".join(p.name for p in missing)
        print(f"{now()} ⚠ 以下路径暂不存在，将在其出现后自动纳入监控: {names}")

    if initial:
        run_build(root)

    server = start_preview_server(root, port) if serve else None

    last = take_snapshot(watch_paths)
    stable_since: float | None = None
    print(f"{now()} 👀 开始监控（轮询 {poll}s，防抖 {debounce}s），按 Ctrl+C 停止…")
    print()

    try:
        while True:
            time.sleep(poll)

            current = take_snapshot(watch_paths)

            if current != last:
                if stable_since is None:
                    added, removed, modified = diff_snapshots(last, current)
                    print_changes(root, debounce, added, removed, modified)
                last = current
                stable_since = time.monotonic()  # 有新变更则重置防抖计时
            elif stable_since is not None and time.monotonic() - stable_since >= debounce:
                stable_since = None
                run_build(root)
                print()
                # 构建期间若有新变更，下一次轮询会再次进入防抖流程，无需额外处理

            if server is not None and server.poll() is not None:
                print(f"{now()} ⚠ 预览服务器已退出（退出码 {server.returncode}），仅保留监控与自动构建。")
                server = None
    except KeyboardInterrupt:
        print(f"\n{now()} 👋 监控已停止。")
    finally:
        if server is not None:
            server.terminate()


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="watch.py",
        description="监控 content/ 等源文件，变更后自动运行 build.py build（增量构建）",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""示例:
    uv run Tools/watch.py               # 仅监控 + 自动构建
    uv run Tools/watch.py --serve       # 同时启动预览服务器
    uv run Tools/watch.py -s -p 3000    # 自定义预览端口
""",
    )
    parser.add_argument("-s", "--serve", action="store_true", help="同时启动预览服务器（build.py preview）")
    parser.add_argument("-p", "--port", type=int, default=8000, help="预览服务器端口号（默认: 8000）")
    parser.add_argument(
        "--poll", type=float, default=DEFAULT_POLL, help=f"轮询间隔秒数（默认: {DEFAULT_POLL}）"
    )
    parser.add_argument(
        "--debounce", type=float, default=DEFAULT_DEBOUNCE,
        help=f"变更防抖秒数，文件稳定这么久后才构建（默认: {DEFAULT_DEBOUNCE}）",
    )
    parser.add_argument("--no-initial-build", action="store_true", help="跳过启动时的首次构建")
    args = parser.parse_args()

    if sys.version_info < (3, 10):
        print("❌ 需要 Python 3.10+（build.py 使用了 match 语法）。")
        return 1
    if args.poll <= 0 or args.debounce <= 0:
        print("❌ --poll 与 --debounce 必须大于 0。")
        return 1

    root = get_repo_root()
    os.chdir(root)  # 与 build.py 保持一致，相对路径以仓库根目录为基准

    watch(
        root,
        serve=args.serve,
        port=args.port,
        poll=args.poll,
        debounce=args.debounce,
        initial=not args.no_initial_build,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
