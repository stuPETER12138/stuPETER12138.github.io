#!/usr/bin/env -S uv run

"""
图片转 WebP 工具（content/ 下所有 imgs 目录）

遍历 content/ 下所有名为 imgs 的目录（含子目录）中的 JPG/PNG 图片，
转换为 WebP 并保存在原目录，方便博客图片优化。

说明: 转换后只需保留 .webp 文件，原始 JPG/PNG 已通过 .gitignore 忽略，
      不纳入版本管理（重复转换不受影响，已是最新的 .webp 会自动跳过）。

用法:
    uv run img2webp.py               # 转换 content/ 下所有 imgs 中的图片
    uv run img2webp.py -q 90         # 指定质量（默认 80）
    uv run img2webp.py --lossless    # 无损模式（适合 PNG 截图）
    uv run img2webp.py -f            # 强制重新转换（覆盖已存在的 .webp）
    uv run img2webp.py -d            # 转换成功后删除原 JPG/PNG
    uv run img2webp.py --help        # 查看全部选项
"""

import argparse
import sys
from pathlib import Path

from PIL import Image, features

# 脚本位于 Tools/ 下，项目根目录为其上一级
PROJECT_ROOT = Path(__file__).resolve().parent.parent
CONTENT_DIR = PROJECT_ROOT / "content"

SUPPORTED_EXTENSIONS = {".jpg", ".jpeg", ".png"}
TARGET_EXTENSION = ".webp"


def convert_image(src: Path, dst: Path, quality: int, lossless: bool) -> None:
    """将单张图片转换为 WebP。"""
    with Image.open(src) as img:
        # 处理颜色模式: RGBA/LA/P 保留透明度，其余统一转 RGB
        if img.mode not in ("RGBA", "LA", "P"):
            img = img.convert("RGB")

        save_kwargs: dict[str, int | bool] = {"lossless": lossless}
        if not lossless:
            save_kwargs["quality"] = quality

        # 尽量保留 EXIF 元数据
        exif = img.info.get("exif")
        if exif:
            save_kwargs["exif"] = exif

        img.save(dst, "WEBP", **save_kwargs)


def collect_images(content_dir: Path) -> list[Path]:
    """递归收集 content/ 下所有 imgs 目录中的 JPG/PNG 图片。"""
    return sorted(
        p
        for p in content_dir.rglob("*")
        if p.is_file()
        and p.suffix.lower() in SUPPORTED_EXTENSIONS
        and "imgs" in p.parts
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="img2webp.py",
        description="遍历 content/ 下所有 imgs 目录，将 JPG/PNG 图片转换为 WebP",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("-q", "--quality", type=int, default=80, help="有损质量 0-100（默认 80）")
    parser.add_argument("-l", "--lossless", action="store_true", help="无损模式（忽略 --quality）")
    parser.add_argument("-d", "--delete", action="store_true", help="转换成功后删除原 JPG/PNG")
    parser.add_argument("-f", "--force", action="store_true", help="强制重新转换已存在的 .webp")
    args = parser.parse_args()

    if not 0 <= args.quality <= 100:
        print("错误: 质量参数必须在 0-100 之间")
        return 1

    if not CONTENT_DIR.exists():
        print(f"错误: 目录不存在: {CONTENT_DIR}")
        return 1

    if not features.check("webp"):
        print("错误: 当前 Pillow 不支持 WebP，请重新安装 Pillow（需要 libwebp 支持）")
        return 1

    files = collect_images(CONTENT_DIR)
    if not files:
        print(f"content/ 下没有找到 JPG/PNG 图片（需位于 imgs 目录中）: {CONTENT_DIR}")
        return 0

    converted = skipped = failed = 0
    saved_bytes = 0
    print(f"共找到 {len(files)} 张图片，开始转换...\n")

    for src in files:
        try:
            dst = src.with_suffix(TARGET_EXTENSION)
            if dst.exists() and not args.force and dst.stat().st_mtime >= src.stat().st_mtime:
                skipped += 1
                print(f"  跳过（webp 已是最新）: {dst.name}")
                continue

            convert_image(src, dst, args.quality, args.lossless)
            old_size = src.stat().st_size
            new_size = dst.stat().st_size
            saved_bytes += max(0, old_size - new_size)
            converted += 1
            print(f"  完成: {src.name} -> {dst.name} ({new_size / old_size * 100:.0f}%)")

            if args.delete:
                src.unlink()
        except Exception as e:  # noqa: BLE001 - 单个文件失败不影响其他文件
            failed += 1
            print(f"  失败: {src.name} ({e})")

    print(
        f"\n转换结束: 成功 {converted} 张, 跳过 {skipped} 张, 失败 {failed} 张"
        f"{f', 共节省 {saved_bytes / 1024:.1f} KB' if converted else ''}"
    )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
