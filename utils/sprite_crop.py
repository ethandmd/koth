#!/usr/bin/env python3
"""Crop sprites to content, remove background, and pad to a uniform canvas.

Usage:
  python utils/sprite_crop.py --pad 24 --tol 10 retro-*.png

Outputs:
  For each input, writes <name>-sprite.png in the same directory.
"""

from __future__ import annotations

import argparse
from pathlib import Path

import numpy as np
from PIL import Image


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="+", help="Input PNG files or globs")
    parser.add_argument("--pad", type=int, default=24, help="Padding in pixels")
    parser.add_argument("--tol", type=int, default=10, help="Background color tolerance")
    parser.add_argument(
        "--out-dir",
        type=str,
        default="",
        help="Optional output directory (defaults to input file directory)",
    )
    parser.add_argument(
        "--suffix",
        type=str,
        default="-sprite",
        help="Suffix appended to the output filename stem",
    )
    return parser.parse_args()


def expand_files(patterns: list[str]) -> list[Path]:
    files: list[Path] = []
    for pat in patterns:
        if any(ch in pat for ch in "*?["):
            files.extend(sorted(Path().glob(pat)))
        else:
            files.append(Path(pat))
    return [p for p in files if p.exists()]


def background_color(arr: np.ndarray) -> tuple[int, int, int]:
    corners = [
        tuple(arr[0, 0, :3]),
        tuple(arr[0, -1, :3]),
        tuple(arr[-1, 0, :3]),
        tuple(arr[-1, -1, :3]),
    ]
    return max(set(corners), key=corners.count)


def corners_transparent(arr: np.ndarray) -> bool:
    alphas = [
        arr[0, 0, 3],
        arr[0, -1, 3],
        arr[-1, 0, 3],
        arr[-1, -1, 3],
    ]
    return sum(1 for a in alphas if a == 0) >= 3


def remove_background(im: Image.Image, tol: int) -> Image.Image:
    arr = np.array(im)
    if corners_transparent(arr):
        return im
    bg = background_color(arr)
    diff = np.abs(arr[:, :, :3].astype(int) - np.array(bg, dtype=int))
    mask = (diff <= tol).all(axis=2)
    arr[mask, 3] = 0
    return Image.fromarray(arr, "RGBA")


def main() -> None:
    args = parse_args()
    paths = expand_files(args.files)
    if not paths:
        raise SystemExit("No input files found.")

    processed: list[tuple[Path, Image.Image]] = []
    max_w = 0
    max_h = 0

    for p in paths:
        im = Image.open(p).convert("RGBA")
        out = remove_background(im, args.tol)
        bbox = out.getbbox() or (0, 0, out.width, out.height)
        out = out.crop(bbox)

        w, h = out.size
        w2, h2 = w + args.pad * 2, h + args.pad * 2
        padded = Image.new("RGBA", (w2, h2), (0, 0, 0, 0))
        padded.paste(out, (args.pad, args.pad))

        processed.append((p, padded))
        max_w = max(max_w, w2)
        max_h = max(max_h, h2)

    for p, im in processed:
        canvas = Image.new("RGBA", (max_w, max_h), (0, 0, 0, 0))
        x = (max_w - im.width) // 2
        y = (max_h - im.height) // 2
        canvas.paste(im, (x, y))

        out_dir = Path(args.out_dir) if args.out_dir else p.parent
        out_path = out_dir / f"{p.stem}{args.suffix}.png"
        canvas.save(out_path)
        print(f"{p.name} -> {out_path.name} ({canvas.size[0]}x{canvas.size[1]})")

    print(f"Final canvas size: {max_w}x{max_h}")


if __name__ == "__main__":
    main()
