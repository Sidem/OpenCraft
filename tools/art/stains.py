"""Surface hints: the tinted ground over each ore (docs/ART_STYLE.md). A stain recolours part of a plain tile (dirt, grass,
sand) into the ore's ramp, keeping the tile's own light and shade, and each ore has its own SHAPE so the four read apart
without colour (the same language as the ores in `ores.py`): rust is fat round blotches, coal angular shards, copper thin
teal veins, chalk fine pale chips. `apply` is the hook `natural.dirt`, `grass_top`, `grass_side` and `sand` take as `post`.
Add a stain: a ramp, a mask function and a row in `KINDS`.
"""
import numpy as np

from lib import N, coords, fbm, grow, hash2, ramp, worley
from natural import leaf_shape

RUST = ramp(["#241012", "#4a1f1a", "#74341f", "#a0502a", "#c47a3e", "#e3a566"])
COAL = ramp(["#05060b", "#0c0f19", "#171b2a", "#272d42", "#3f4a68", "#6f86ad"])
VERDIGRIS = ramp(["#0d2a31", "#1c5456", "#33887a", "#6fc4a8", "#b4ecd0"])
CHALK = ramp(["#2b2640", "#5c5578", "#8d86a6", "#c5c0d8", "#f1eefc"])


def _rust():
    X, Y = coords()
    f1, _, pid = worley(X, Y, 6, 103, 0.95)
    r = (0.2 + pid * 0.25) * (0.75 + 0.5 * fbm(X, Y, 14, 2, 104))
    return (pid > 0.4) & (f1 < r)


def _coal():
    rng = np.random.default_rng(105)
    ys, xs = np.mgrid[-14:15, -14:15]
    mask = np.zeros((N, N), bool)
    for _ in range(26):
        cx, cy = rng.integers(0, N, 2)
        ang, la = rng.uniform(0, np.pi), rng.uniform(5, 11)
        inside, _, _ = leaf_shape(xs, ys, ang, la, la * rng.uniform(0.3, 0.5))
        mask[((cy + ys) % N)[inside], ((cx + xs) % N)[inside]] = True
    return mask


def _verdigris():
    X, Y = coords()
    line = np.abs(fbm(X, Y, 4, 3, 107, 0.5, 0.45) - 0.5)
    return (line < 0.02 + 0.04 * np.clip(fbm(X, Y, 7, 2, 108) - 0.4, 0, 1)) | (grow(line < 0.012, 1) & (fbm(X, Y, 12, 2, 109) > 0.6))


def _chalk():
    X, Y = coords()
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    return (hash2(ix // 3, iy // 3, 110) > 0.68) & (hash2(ix, iy, 111) > 0.25) & (fbm(X, Y, 5, 2, 112) > 0.38)


KINDS = {"rusty": (_rust, RUST), "dark": (_coal, COAL), "green": (_verdigris, VERDIGRIS), "pale": (_chalk, CHALK)}


def apply(img, base, kind):
    """`img` (n, n, 3) painted from the ramp `base`, with the stain `kind` recoloured into its ramp where its shape falls.
    The stain always differs in lightness from the ground, not just in hue (so colour-blind players see it): coal is dark,
    and the others go lighter on dark ground and darker on light ground (sand)."""
    mask_fn, tones = KINDS[kind]
    d = ((img[..., None, :].astype(int) - base[None, None].astype(int)) ** 2).sum(-1)
    idx = d.argmin(-1) / (len(base) - 1) * (len(tones) - 1)
    if kind == "dark":
        idx = idx * 0.4
    elif img.mean() > 135:
        idx = idx * 0.5
    else:
        idx = idx + (3 if kind == "rusty" else 2)
    return np.where(mask_fn()[..., None], tones[np.clip(np.round(idx).astype(int), 0, len(tones) - 1)], img).astype(np.uint8)