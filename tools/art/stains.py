"""Surface hints: the tinted ground over each ore (docs/ART_STYLE.md). A stain recolours part of a plain tile (dirt, grass,
sand) into the ore's ramp, keeping the tile's own light and shade, and each ore has its own SHAPE so the four read apart
without colour (the same language as the ores in `ores.py`): rust is fat round blotches, coal angular shards, copper thin
teal veins, chalk fine pale chips. `apply` is the hook `natural.dirt`, `grass_top`, `grass_side` and `sand` take as `post`.
Add a stain: a ramp, a mask function and a row in `KINDS`.
"""
import numpy as np

from lib import N, coords, fbm, hash2, ramp
from natural import leaf_shape

RUST = ramp(["#241012", "#4a1f1a", "#6e2c20", "#8f3d28", "#b05a38", "#cc7e52"])
COAL = ramp(["#05060b", "#0c0f19", "#171b2a", "#272d42", "#3f4a68", "#6f86ad"])
VERDIGRIS = ramp(["#1a2a2a", "#38504a", "#5d7f6c", "#8aa890", "#b4c8ae"])
CHALK = ramp(["#2b2640", "#5c5578", "#8d86a6", "#c5c0d8", "#f1eefc"])


def _rust():
    """Iron staining: big ragged patches of oxide, a dithered halo where it thins out, never round or leaf-shaped.
    Returns 0 (clean), 0.5 (halo) or 1 (core)."""
    X, Y = coords()
    n = fbm(X, Y, 5, 4, 103, 0.55, 0.35)
    grain = hash2((X * N).astype(int), (Y * N).astype(int), 104)
    core = n > 0.63
    halo = (n > 0.56) & ~core & (grain > 0.5)
    return np.where(core, 1.0, np.where(halo, 0.5, 0.0))


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
    """Copper salts weathering out of the soil: a pale sage crust in small broken patches, speckled where it is thin."""
    X, Y = coords()
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    n = fbm(X, Y, 7, 4, 107, 0.5, 0.3)
    grain = hash2(ix, iy, 108)
    crust = (n > 0.62) & (grain > 0.12)
    thin = (n > 0.56) & ~crust & (grain > 0.66)
    return crust | thin


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
    strength = mask_fn().astype(float)  # 0..1: a stain may fade out (rust) or be all or nothing
    if kind == "dark":
        idx = idx * 0.4
    elif img.mean() > 135:
        idx = idx * 0.5
    else:
        idx = idx + (4 * strength if kind == "rusty" else 2)
    return np.where((strength > 0)[..., None], tones[np.clip(np.round(idx).astype(int), 0, len(tones) - 1)], img).astype(np.uint8)