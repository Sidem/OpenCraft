"""The sunny grade (ART_STYLE.md, "The key-art look"): the greens of grass, leaves and plants moved toward the key art.

A tile is painted first (photo colours for `real` tiles, judged by `realism.py` as painted); the pack then ships it
graded. Only green texels move: the weight is a smooth band on the Oklab hue around green times a ramp on chroma, so
soil under a grass lip, stains, bark, petals and stalks keep their colour. Within the band lightness and chroma are
scaled and the hue turns a little toward yellow. The targets were measured on the key art (that ART_STYLE.md section
has the numbers). To grade another layer: add its name to `GRADED` (or a prefix to `PREFIXES`).
"""
import numpy as np

from style import oklab_to_srgb, srgb_to_oklab

# Lightness and chroma factors and a hue turn in degrees (negative is toward yellow), at full weight.
LIGHT = 1.25
CHROMA = 1.4
TURN = -3.0
# Canopies are seen mostly from the side, where the face shading darkens them, so they lift further.
CANOPY_LIGHT = 1.4
# The green band, in Oklab hue degrees: full weight between the inner edges, none outside the outer ones.
BAND = (85.0, 105.0, 165.0, 195.0)
# Chroma below the first value is grey and stays; full weight above the second.
GREY = (0.015, 0.04)

GRADED = {"grass_top", "grass_side", "tuft", "daisy", "poppy", "bluebell", "fern", "reed", "sapling",
          "birch_sapling", "pine_sapling", "rust_bloom", "copper_bell", "pale_star", "birch_leaves", "pine_leaves",
          "palm_fronds", "cherry_blossom", "baobab_leaves", "acacia_leaves", "redwood_needles", "eucalyptus_leaves",
          "bamboo_leaves"}
PREFIXES = ("grass_top_", "grass_side_", "leaves")


def wants(name):
    return name in GRADED or name.startswith(PREFIXES)


def _ramp(x, lo, hi):
    return np.clip((x - lo) / (hi - lo), 0, 1)


def sunny(tile, light=LIGHT):
    """The tile (N, N, 4 or 5 uint8) with its green texels graded; alpha and glow untouched."""
    out = tile.copy()
    rgb = tile[..., :3].reshape(-1, 3).astype(float)
    lab = srgb_to_oklab(rgb)
    L, a, b = lab[:, 0], lab[:, 1], lab[:, 2]
    C = np.hypot(a, b)
    h = np.degrees(np.arctan2(b, a)) % 360
    w = _ramp(h, BAND[0], BAND[1]) * (1 - _ramp(h, BAND[2], BAND[3])) * _ramp(C, *GREY)
    L2 = L * (1 + (light - 1) * w)
    C2 = C * (1 + (CHROMA - 1) * w)
    h2 = np.radians(h + TURN * w)
    graded = oklab_to_srgb(np.stack([L2, C2 * np.cos(h2), C2 * np.sin(h2)], -1))
    moved = (w > 0)[:, None]  # texels outside the band keep their exact bytes
    out[..., :3] = np.where(moved, graded, tile[..., :3].reshape(-1, 3)).reshape(tile.shape[:2] + (3,))
    return out


def finish(entry, tile):
    """What the pack ships for this entry: the painted tile, graded if the entry is vegetation."""
    if not wants(entry.name):
        return tile
    return sunny(tile, CANOPY_LIGHT if "leaves" in entry.name else LIGHT)
