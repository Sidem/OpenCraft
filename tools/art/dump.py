"""Looks at the engine's current layers: python tools/art/dump.py <current.bin> <out_sheet.png> [first] [last] [scale]

`current.bin` comes from `OC_DUMP=<file> cargo test -q --release dump_layers -- --ignored` (see textures/tests.rs).
Writes a contact sheet of layers first..last, each labelled with its index, on a mid-grey mat (so cutouts show).
"""
import struct
import sys

import numpy as np
from PIL import Image, ImageDraw


def load(path):
    raw = open(path, "rb").read()
    size, count = struct.unpack("<II", raw[:8])
    return np.frombuffer(raw[8:], np.uint8).reshape(count, size, size, 4)


def main():
    layers = load(sys.argv[1])
    first = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    last = int(sys.argv[4]) if len(sys.argv) > 4 else len(layers) - 1
    scale = int(sys.argv[5]) if len(sys.argv) > 5 else 1
    size = layers.shape[1] * scale
    per_row = max(1, 1600 // (size + 6))
    rows = (last - first) // per_row + 1
    sheet = Image.new("RGB", (per_row * (size + 6) + 6, rows * (size + 18) + 6), (96, 96, 104))
    draw = ImageDraw.Draw(sheet)
    for i, layer in enumerate(range(first, last + 1)):
        img = Image.fromarray(layers[layer], "RGBA")
        if scale > 1:
            img = img.resize((size, size), Image.NEAREST)
        x, y = 6 + (i % per_row) * (size + 6), 6 + (i // per_row) * (size + 18)
        sheet.paste(img, (x, y), img)
        draw.text((x, y + size + 3), str(layer), fill=(255, 255, 255))
    sheet.save(sys.argv[2])


if __name__ == "__main__":
    main()
