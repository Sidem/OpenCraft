"""Realistic natural tiles (ART_STYLE.md section 12): structure is procedural, colour comes from photos.

`colour(mat, field)` maps any structure field onto a material's photo palette (`reference/materials.json`, built by
`realism.py reference` from the fit photos): the field's ranks become the photos' lightness quantiles (spread by
`contrast`, since a block face needs a little more relief than a photo to read), and each lightness gets the photos'
mean a, b there plus patchy colour variation of the photos' size. Continuous tone, no outlines, no posterising.
Recipes return `rgba` tiles and wrap. Add a material: give it photos in `realism.MATERIALS`, then a recipe here.
Rolled out: the ground (stone, so every ore body, dirt, grass top and side, sand; `hints.py` stains them).
"""
import numpy as np

import realism
from lib import N, blur, coords, fbm, hash2, relief_light, rgba, worley
from style import oklab_to_srgb

_REF = None


def colour(mat, field, contrast=1.5, hue_var=1.0, seed=0):
    """Paint a 0..1-ish structure field in the photos' colours of `mat`. Returns uint8 (N, N, 3)."""
    global _REF
    _REF = _REF or realism.load()
    prof = _REF[mat]["profile"]
    q = np.array(prof["quantiles"])
    bins = np.array(prof["bins"])  # per lightness bin: L, a, b, std a, std b
    rank = np.argsort(np.argsort(field.ravel())).reshape(field.shape) / (field.size - 1)
    L = np.interp(rank, np.linspace(0, 1, len(q)), q)
    L = np.median(q) + (L - np.median(q)) * contrast
    a = np.interp(L, bins[:, 0], bins[:, 1])
    b = np.interp(L, bins[:, 0], bins[:, 2])
    X, Y = coords()
    for ch, k in ((1, 3), (2, 4)):  # colour wanders in patches, with a little texel grain
        n = (fbm(X, Y, 3, 3, seed + 40 + ch) - 0.5) * 2.4 + (hash2((X * N).astype(int), (Y * N).astype(int), seed + 50 + ch) - 0.5) * 0.8
        std = np.interp(L, bins[:, 0], bins[:, k])
        if ch == 1:
            a = a + n * std * hue_var
        else:
            b = b + n * std * hue_var
    lab = np.stack([L, a, b], -1).reshape(-1, 3)
    return oklab_to_srgb(lab).reshape(N, N, 3)


def stone(seed=0):
    """Rock face: broad weathered facets and ridges, a few hairline fractures that fade, mineral grain."""
    X, Y = coords()
    big = fbm(X, Y, 3, 5, seed + 1, 0.5, 0.25)
    mid = fbm(X, Y, 6, 4, seed + 6)
    ridged = 1 - np.abs(fbm(X, Y, 6, 4, seed + 2) - 0.5) * 2
    h = big * 0.45 + mid * 0.45 + ridged ** 3 * 0.35  # little of the largest scale, so a wall shows no grid
    light = relief_light(h * 9, 1.0)
    crack = np.abs(fbm(X, Y, 3, 4, seed + 3, 0.5, 0.4) - 0.5)
    crack = np.clip(1 - crack / 0.012, 0, 1) * (fbm(X, Y, 4, 2, seed + 4) > 0.5)  # only some stretches open up
    grain = fbm(X, Y, 48, 2, seed + 5)
    field = light * 0.55 + h * 0.3 + grain * 0.15 - blur(crack, 0.7) * 0.35
    return rgba(colour("stone", field, 1.6, seed=seed))


def dirt(seed=0):
    """Loose soil: crumbly clods with soft shadows between them, fine grit, a few small half-buried stones."""
    X, Y = coords()
    clods = fbm(X, Y, 8, 4, seed + 1, 0.5)
    f1, _, pid = worley(X, Y, 16, seed + 2, 1.0)
    crumb = 1 - np.clip(f1 / 0.8, 0, 1)
    stones = (pid > 0.95) & (f1 < 0.4)
    h = clods * 0.8 + crumb * 0.12 + np.where(stones, 0.4 * (1 - f1 / 0.4), 0)
    light = relief_light(h * 5, 1.0)
    grit = hash2((X * N).astype(int), (Y * N).astype(int), seed + 3)
    field = light * 0.45 + clods * 0.35 + crumb * 0.08 + grit * 0.04 + stones * 0.15
    return rgba(colour("dirt", field, 1.0, seed=seed))


def grass_top(seed=0):
    """A lawn seen from above: dense fine blades in soft focus, light and dark patches, a little yellowed thatch."""
    X, Y = coords()
    rng = np.random.default_rng(seed + 11)
    blades = np.zeros((N, N))
    for _ in range(5000):  # each blade adds a short soft stroke, brighter at its tip
        x, y, ang = rng.uniform(0, N), rng.uniform(0, N), rng.uniform(0, 2 * np.pi)
        length, tip = rng.integers(3, 8), rng.uniform(0.4, 1.0)
        for s in range(length):
            px, py = int(x + np.cos(ang) * s) % N, int(y + np.sin(ang) * s) % N
            blades[py, px] = max(blades[py, px], tip * (s + 1) / length)
    blades = blur(blades, 0.6)
    patches = fbm(X, Y, 6, 3, seed + 2)
    field = blades * 0.75 + patches * 0.25
    return rgba(colour("grass", field, 1.3, hue_var=1.2, seed=seed))


def grass_side(seed=0):
    """Turf hanging over the soil: a ragged lip of grass (single blades poke lower), a soft shadow under it; tiles
    sideways. Built from this module's dirt and grass, so it matches both."""
    soil = dirt(seed + 4)[..., :3].astype(float)
    top = grass_top(seed + 2)[..., :3].astype(float)
    col = np.arange(N)
    X, _ = coords()
    depth = 22 + 10 * fbm(X[:1], np.zeros((1, N)), 6, 3, seed + 3)[0] + 7 * (hash2(col, 0, seed + 8) > 0.8) * hash2(col, 1, seed + 9)
    yy = np.arange(N)[:, None]
    lip = yy < depth[None, :]
    shade = 1 - 0.45 * np.clip(1 - (yy - depth[None, :]) / 12.0, 0, 1) * ~lip  # soft, no dither
    return rgba(np.where(lip[..., None], top, soil * shade[..., None]).clip(0, 255).astype(np.uint8))


def sand(seed=0):
    """Wind-blown sand: low asymmetric ripples lit from the side, fine grain, a few darker heavy-mineral streaks."""
    X, Y = coords()
    warp = fbm(X, Y, 3, 3, seed + 1)
    phase = (Y * 9 + warp * 1.6 + X * 0.6) % 1.0
    ripple = np.where(phase < 0.7, phase / 0.7, (1 - phase) / 0.3)  # gentle windward slope, steep lee
    light = relief_light(blur(ripple, 1.0) * 2.5, 1.0)
    grain = fbm(X, Y, 64, 1, seed + 2)
    streak = fbm(X, Y, 4, 3, seed + 3, 0.5, 0.3)
    field = blur(light, 1.0) * 0.3 + grain * 0.45 + streak * 0.25
    return rgba(colour("sand", field, 1.2, seed=seed))
