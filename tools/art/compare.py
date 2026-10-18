"""Before / after sheet: python tools/art/compare.py <current.bin> [out.png]

Each pack layer as the engine draws it today (64 x 64, shown doubled with no smoothing) beside the new 128 x 128 tile.
Repeating layers are shown 2 x 2 so the seams and the repeat can be judged. Writes artifacts/art2/before_after.png.
"""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

import dump
import registry

ROOT = Path(__file__).resolve().parents[2]
MAT = np.array([92.0, 96.0, 104.0])


def flat(rgba):
    a = rgba[..., 3:4] / 255.0
    return (rgba[..., :3] * a + MAT * (1 - a)).astype(np.uint8)


def main():
    cur = dump.load(sys.argv[1])
    out = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "artifacts" / "art2" / "before_after.png"
    cells = []
    for e in registry.LAYERS:
        old = flat(np.repeat(np.repeat(cur[e.layer], 2, 0), 2, 1))
        new = flat(e.make()[..., :4])
        if e.tiles:
            old, new = np.tile(old, (2, 2, 1)), np.tile(new, (2, 2, 1))
        cells.append((e.name, e.layer, old, new))
    size = max(c[2].shape[0] for c in cells)
    per_row, pad = 2, 10
    cw = 2 * size + 3 * pad
    rows = (len(cells) + per_row - 1) // per_row
    sheet = Image.new("RGB", (per_row * cw, rows * (size + 28) + pad), (40, 42, 50))
    draw = ImageDraw.Draw(sheet)
    for i, (name, layer, old, new) in enumerate(cells):
        x, y = (i % per_row) * cw + pad, (i // per_row) * (size + 28) + pad
        sheet.paste(Image.fromarray(old), (x, y))
        sheet.paste(Image.fromarray(new), (x + size + pad, y))
        draw.text((x, y + size + 6), f"{layer} {name}: now (left)  /  new (right)", fill=(255, 255, 255))
    sheet.save(out)
    print(out, sheet.size)


if __name__ == "__main__":
    main()
