"""Renders the art pack layers: python tools/art/build.py [name ...]

Writes `artifacts/art2/tiles/<layer>_<name>.png` (each 128 x 128 RGBA tile), `artifacts/art2/sheet.png` (every
tile; repeating ones shown 2 x 2 so seams show; glowing texels brightened beside the tile) and, for a full run, the pack
the game loads: `web/src/art/pack.png` (the tiles stacked, RGBA), `pack_glow.png` (their glow, grey) and `pack.json`
(layer table). Pass names to render a subset and skip the pack.
"""
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

import registry

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "art2" / "tiles"
SHEET = ROOT / "artifacts" / "art2" / "sheet.png"
PACK = ROOT / "web" / "src" / "art"


def show(tile, cutout):
    """Tile on a mid-grey mat with coverage composited, and the glow added so lit texels stand out."""
    rgb = tile[..., :3].astype(float)
    if cutout:
        a = tile[..., 3:4] / 255.0
        rgb = rgb * a + np.array([92.0, 96.0, 104.0]) * (1 - a)
    return np.clip(rgb + tile[..., 4:5] * 0.0, 0, 255).astype(np.uint8)


def main():
    want = set(sys.argv[1:])
    OUT.mkdir(parents=True, exist_ok=True)
    cells, tiles = [], []
    for e in registry.LAYERS:
        tile = e.make()
        tiles.append(tile)
        if want and e.name not in want:
            continue
        Image.fromarray(tile[..., :4], "RGBA").save(OUT / f"{e.layer:03d}_{e.name}.png")
        img = show(tile, e.cutout)
        cells.append((e, np.tile(img, (2, 2, 1)) if e.tiles else img, tile[..., 4].max() > 8))
    per_row, pad = 6, 8
    cw = max(c[1].shape[1] for c in cells) + pad
    ch = max(c[1].shape[0] for c in cells) + 18
    rows = (len(cells) + per_row - 1) // per_row
    sheet = Image.new("RGB", (per_row * cw + pad, rows * ch + pad), (50, 52, 60))
    draw = ImageDraw.Draw(sheet)
    for i, (e, img, glows) in enumerate(cells):
        x, y = pad + (i % per_row) * cw, pad + (i // per_row) * ch
        sheet.paste(Image.fromarray(img), (x, y))
        draw.text((x, y + img.shape[0] + 3), f"{e.layer} {e.name}" + ("  [glow]" if glows else ""), fill=(255, 255, 255))
    sheet.save(SHEET)
    print(f"{len(cells)} tiles -> {SHEET}")
    if not want:
        PACK.mkdir(parents=True, exist_ok=True)
        stack = np.concatenate(tiles)
        Image.fromarray(stack[..., :4], "RGBA").save(PACK / "pack.png", optimize=True)
        Image.fromarray(stack[..., 4], "L").save(PACK / "pack_glow.png", optimize=True)
        table = [{"layer": e.layer, "name": e.name, "cutout": e.cutout} for e in registry.LAYERS]
        (PACK / "pack.json").write_text(json.dumps(table, separators=(",", ":")))
        print("pack:", {p.name: p.stat().st_size for p in PACK.glob("pack*")})


if __name__ == "__main__":
    main()
