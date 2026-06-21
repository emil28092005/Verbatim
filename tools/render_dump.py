#!/usr/bin/env python3
"""Render a headless_dump.txt frame to PNG for visual analysis."""

import sys
from PIL import Image, ImageDraw, ImageFont

TERM_COLORS = {
    " ": (15, 15, 20, 10, 10, 15),
    "?": (80, 80, 80, 10, 10, 15),
    ".": (194, 178, 128, 30, 25, 15),
    "~": (80, 140, 200, 20, 30, 40),
    "#": (120, 120, 120, 50, 50, 55),
    "T": (139, 90, 43, 40, 25, 15),
    "%": (200, 80, 80, 60, 20, 20),
    "`": (230, 230, 220, 80, 80, 75),
    "^": (255, 100, 20, 40, 10, 0),
    "*": (80, 80, 80, 30, 30, 30),
    '"': (60, 180, 60, 15, 40, 15),
    ":": (120, 100, 70, 30, 25, 15),
    "@": (200, 180, 255, 20, 20, 30),
    "g": (160, 240, 120, 20, 30, 15),
    "s": (120, 240, 160, 15, 30, 20),
}


def render_frame(text, out_path, font_size=14):
    lines = text.splitlines()
    if not lines:
        return
    h = len(lines)
    w = max(len(line) for line in lines)
    cell_w = font_size
    cell_h = int(font_size * 1.2)
    img = Image.new("RGB", (w * cell_w, h * cell_h), (10, 10, 15))
    draw = ImageDraw.Draw(img)
    try:
        font = ImageFont.truetype(
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", font_size
        )
    except Exception:
        font = ImageFont.load_default()
    for y, line in enumerate(lines):
        for x, ch in enumerate(line):
            colors = TERM_COLORS.get(ch, (200, 200, 200, 10, 10, 15))
            fg = (colors[0], colors[1], colors[2])
            bg = (colors[3], colors[4], colors[5])
            draw.rectangle(
                [x * cell_w, y * cell_h, (x + 1) * cell_w, (y + 1) * cell_h], fill=bg
            )
            draw.text((x * cell_w, y * cell_h), ch, fill=fg, font=font)
    img.save(out_path)


def main():
    dump_path = sys.argv[1] if len(sys.argv) > 1 else "headless_dump.txt"
    out_path = sys.argv[2] if len(sys.argv) > 2 else "headless_dump.png"
    with open(dump_path, "r", encoding="utf-8") as f:
        content = f.read()
    sections = content.split("=== ")
    if len(sections) > 1:
        first = sections[1].split("\n", 1)[1]
        frame = first.split("\n\n")[0]
    else:
        frame = content
    render_frame(frame, out_path)
    print(f"Rendered {out_path}")


if __name__ == "__main__":
    main()
