"""Generator 12's species in the realistic look (docs/ART_STYLE.md section 12): a bark and a leaf tile each for pine,
palm, cherry, baobab, acacia, redwood, eucalyptus and bamboo (`block/trees.rs`). Leaf tiles are cutouts drawn on the
mesher's leaf cards (`mesher/foliage.rs`), so each shows the leaf's own shape: a palm's ranks of leaflets, a cherry's
five-petalled blossom, a eucalyptus' long sickle leaves. Structure is procedural; colour comes from the photo palettes
(`real.colour`: bark, leaves, needles, birch, dead wood), moved in Oklab toward each species' colour (`shift`), since
no photos of these species are in `realism.MATERIALS` yet. Add a species: a recipe here and its rows in `registry.py`.
"""
import numpy as np

import foliage
import wood
from lib import N, blur, coords, fbm, relief_light, rgba
from real import colour
from style import oklab_to_srgb, srgb_to_oklab


def shift(tile, L=0.0, a=0.0, b=0.0, chroma=1.0):
    """Move a tile's colour in Oklab: lightness by `L`, chroma scaled by `chroma`, then nudged by (`a`, `b`)."""
    lab = srgb_to_oklab(tile[..., :3].reshape(-1, 3).astype(float))
    lab[:, 0] += L
    lab[:, 1] = lab[:, 1] * chroma + a
    lab[:, 2] = lab[:, 2] * chroma + b
    return rgba(oklab_to_srgb(lab).reshape(N, N, 3), alpha=tile[..., 3])


def _soft(x, width):
    return np.clip(x / width + 0.5, 0, 1)


# ---------------------------------------------------------------------------------------------------- bark

def pine_bark():
    """Scots pine: broad scaly reddish-orange outer plates flaking over deep dark grey-brown fissures; tiles."""
    X, Y = coords()
    Xw = (X + (wood.streaks(X, Y, 2, 4, 2, 11) - 0.5) * 0.12) % 1.0
    gap, pid = wood.cells(Xw, Y, 5, 3, 12)
    fissure = _soft(0.11 - gap, 0.1)
    scales = wood.streaks(X, Y, 20, 8, 3, 13)
    flake = _soft(pid - 0.45, 0.25) * (1 - fissure)  # warmer orange-rust where outer bark has flaked
    h = (1 - fissure) * (0.65 + 0.25 * pid) + scales * 0.25
    field = relief_light(blur(h, 0.8) * 5, 1.0) * 0.25 + h * 0.65 - fissure * 0.25
    dark = shift(rgba(colour("bark", field, 1.3, seed=11)), L=-0.05, a=0.012, b=0.015).astype(float)
    warm = shift(rgba(colour("bark", field + 0.12, 1.1, seed=14)), L=0.06, a=0.042, b=0.048).astype(float)
    out = dark * (1 - flake[..., None]) + warm * flake[..., None]
    return out.clip(0, 255).astype(np.uint8)


def acacia_bark():
    """Acacia: deep charcoal-umber bark, tightly and roughly fissured in narrow vertical ridges; tiles."""
    X, Y = coords()
    Xw = (X + (wood.streaks(X, Y, 3, 6, 3, 21) - 0.5) * 0.14) % 1.0
    gap, pid = wood.cells(Xw, Y, 11, 3, 22)
    fissure = _soft(0.1 - gap, 0.09)
    grit = wood.streaks(X, Y, 36, 10, 3, 23)
    h = (1 - fissure) * (0.65 + 0.2 * pid) + grit * 0.3
    field = relief_light(blur(h, 0.7) * 5, 1.0) * 0.25 + (1 - fissure) * 0.55 + grit * 0.2
    return shift(rgba(colour("bark", field, 1.3, seed=21)), L=-0.10, a=0.004, b=-0.008, chroma=0.45)


def palm_bark():
    """A palm's stem: pale ash-brown fibrous trunk crossed by prominent weathered horizontal frond-scar rings; tiles."""
    X, Y = coords()
    fibre = wood.streaks(X, Y, 36, 4, 3, 31)
    ring = np.zeros_like(X)
    for k, y0 in enumerate((0.1, 0.35, 0.6, 0.85)):  # close leaf-base scars: bright ridge over shadowed groove
        wob = (fbm(X, Y, 4, 2, 32 + k) - 0.5) * 0.045
        d = (Y - y0 + wob + 0.5) % 1.0 - 0.5
        ring += (_soft(0.014 - np.abs(d), 0.012) - 0.9 * _soft(0.01 - np.abs(d - 0.022), 0.01)) * (0.65 + 0.35 * fibre)
    h = fibre * 0.45 + ring * 0.45 + wood.streaks(X, Y, 8, 3, 2, 34) * 0.25
    field = relief_light(blur(h, 0.7) * 5, 1.0) * 0.32 + h * 0.58 + fbm(X, Y, 8, 3, 33) * 0.14
    return shift(rgba(colour("dead_wood", field, 1.2, seed=31)), L=-0.02, a=0.004, b=0.018, chroma=0.8)


def cherry_bark():
    """Cherry: smooth, satiny dark mahogany-red bark banded horizontally by crisp pale ochre lenticels; tiles."""
    X, Y = coords()
    rng = np.random.default_rng(41)
    sheen = wood.streaks(X, Y, 3, 28, 3, 42)
    lent = np.zeros((N, N))
    for _ in range(85):
        x, y, w = rng.uniform(0, 1), rng.uniform(0, 1), rng.uniform(0.04, 0.18)
        d = np.hypot(((X - x + 0.5) % 1.0 - 0.5) / w, ((Y - y + 0.5) % 1.0 - 0.5) / rng.uniform(0.004, 0.009))
        lent = np.maximum(lent, _soft(1 - d, 0.35) * rng.uniform(0.5, 1.0))
    field = 0.42 + sheen * 0.28 + fbm(X, Y, 6, 3, 43) * 0.14
    base = shift(rgba(colour("bark", field, 0.95, seed=41)), L=-0.07, a=0.042, b=0.004).astype(float)
    mark = shift(rgba(colour("dead_wood", np.clip(field + 0.35, 0, 1), 0.8, seed=44)), L=0.06, a=0.025, b=0.035).astype(float)
    w = (lent * 0.82)[..., None]
    return (base * (1 - w) + mark * w).clip(0, 255).astype(np.uint8)


def baobab_bark():
    """Baobab: smooth silvery-grey elephantskin bark with a warm copper-rose sheen and soft horizontal creases; tiles."""
    X, Y = coords()
    fold = wood.streaks(X, Y, 3, 14, 3, 51)
    h = fold * 0.85 + fbm(X, Y, 5, 3, 52) * 0.45
    field = relief_light(blur(h, 1.5) * 4, 1.0) * 0.45 + h * 0.45
    return shift(rgba(colour("dead_wood", field, 0.85, seed=51)), L=0.04, a=0.014, b=0.006, chroma=0.72)


def redwood_bark():
    """Redwood: thick fibrous cinnamon-rust bark in long ropy strands and deep vertical furrows; tiles."""
    X, Y = coords()
    Xw = (X + (wood.streaks(X, Y, 2, 4, 2, 61) - 0.5) * 0.07) % 1.0
    rope = wood.streaks(Xw, Y, 22, 2, 3, 62)
    furrow = _soft(0.32 - wood.streaks(Xw, Y, 7, 1, 2, 63), 0.14)
    h = rope * 0.72 - furrow * 0.85
    field = relief_light(blur(h, 0.7) * 5, 1.0) * 0.32 + h * 0.62
    return shift(rgba(colour("bark", field, 1.25, seed=61)), L=0.02, a=0.062, b=0.036)


def eucalyptus_bark():
    """Eucalyptus: smooth mottled ribbon bark shedding in tall cream, sage-grey, ochre and pale-tan patches; tiles."""
    X, Y = coords()
    patch = wood.streaks(X, Y, 3, 5, 4, 71, 0.6)
    edge = sum(_soft(0.012 - np.abs(patch - t), 0.012) for t in (0.38, 0.56, 0.72))
    field = 0.64 + (fbm(X, Y, 16, 3, 72) - 0.5) * 0.22 - blur(edge, 1.0) * 0.12
    pale = rgba(colour("birch", field, 0.75, seed=71)).astype(float)
    out = pale * 0
    for t0, (L, a, b) in (
        (0.0, (0.04, -0.004, 0.012)),   # creamy inner bark
        (0.38, (-0.08, -0.022, 0.018)), # muted sage grey-green
        (0.56, (-0.04, 0.018, 0.055)),  # warm ochre strip
        (0.72, (-0.11, 0.028, 0.028)),  # weathered cinnamon-tan peel
    ):
        w = _soft(patch - t0, 0.04)[..., None]
        out = out * (1 - w) + shift(pale.astype(np.uint8), L=L, a=a, b=b).astype(float) * w
    return out.clip(0, 255).astype(np.uint8)


def bamboo_culm():
    """Bamboo: glossy fresh olive-emerald culm with fine vertical fibres and crisp pale nodal rings; tiles."""
    X, Y = coords()
    streak = wood.streaks(X, Y, 28, 2, 2, 81)
    d = (Y - 0.5 + 0.5) % 1.0 - 0.5
    node = _soft(0.022 - np.abs(d), 0.014)
    line = _soft(0.007 - np.abs(d + 0.032), 0.006)
    sheen = 0.08 * np.sin(X * 2 * np.pi)
    field = 0.64 + streak * 0.12 + node * 0.28 - line * 0.55 + sheen + fbm(X, Y, 4, 2, 82) * 0.08
    green = rgba(colour("leaves", field, 0.7, seed=81))
    return shift(green, L=0.09, a=-0.012, b=0.032, chroma=0.92)


# ---------------------------------------------------------------------------------------------------- leaves

def _fronds(seed, length, width, count, cover_at, mat, lift, spacing=None, spread=(0.45, 0.8)):
    """Leaflets in ranks: straight midribs crossing the tile, each set on both sides with long narrow leaflets that
    sweep forward (palm, bamboo, acacia's tiny ones); three depths, back to front."""
    rng = np.random.default_rng(seed)
    spacing = spacing or width * 2.4
    field, depth, cover = np.zeros((N, N)), np.zeros((N, N)), np.zeros((N, N), bool)
    r = int(length * 1.3) + 2
    ys, xs = np.mgrid[-r:r + 1, -r:r + 1]
    for layer, target in enumerate(cover_at):
        while cover.mean() < target:
            x, y, ang = rng.uniform(0, N), rng.uniform(0, N), rng.uniform(0, 2 * np.pi)
            tilt = rng.uniform(-0.2, 0.2)
            for s in range(count):
                tx, ty = x + np.cos(ang) * s * spacing, y + np.sin(ang) * s * spacing
                for side in (-1, 1):
                    a = ang + side * rng.uniform(*spread)
                    la = length * rng.uniform(0.75, 1.15) * (1 - 0.4 * s / count)
                    u = (xs * np.cos(a) + ys * np.sin(a)) / la
                    w = (-xs * np.sin(a) + ys * np.cos(a)) / width
                    inside = (u > 0) & (u < 1) & (np.abs(w) < np.sin(np.clip(u, 0, 1) * np.pi) ** 0.6)
                    v = 0.3 + layer * 0.25 + tilt + 0.1 * u - 0.05 * np.abs(w) + 0.05 * (np.abs(w) < 0.2)
                    px, py = (int(tx) + xs) % N, (int(ty) + ys) % N
                    field[py[inside], px[inside]] = v[inside]
                    depth[py[inside], px[inside]] = (layer + 1) / 3
                    cover[py[inside], px[inside]] = True
    return foliage._finish(field, depth, cover, mat, lift, seed)


def palm_fronds():
    """Coconut palm: long, glossy, yellow-green leaflets in ranks down each frond."""
    return shift(_fronds(91, 26, 1.5, 9, (0.34, 0.5, 0.62), "leaves", -0.1, 4.0, (0.9, 1.3)), L=-0.03, a=-0.004, b=0.02)


def bamboo_leaves():
    """Bamboo: long lance-shaped leaves fanned from thin twigs, fresh light green."""
    return shift(_fronds(92, 24, 2.2, 3, (0.32, 0.48, 0.6), "leaves", -0.14, 7.0, (0.3, 0.7)), L=-0.04, b=0.01)


def acacia_leaves():
    """Acacia: tiny paired leaflets in feathery sprays, dull olive green, with light through them."""
    return shift(_fronds(93, 4.5, 1.3, 9, (0.3, 0.45, 0.57), "leaves", -0.15), a=0.004, b=0.012, chroma=0.85)


def baobab_leaves():
    """Baobab: a sparse crown of hand-shaped leaves, mid green."""
    return foliage.leaves(seed=94, size=6.5, width=0.42, cover_at=(0.3, 0.44, 0.56), lift=-0.16)


def eucalyptus_leaves():
    """Eucalyptus: long, narrow, sickle-shaped grey-blue leaves hanging edge-on, so the crown lets light through."""
    t = foliage.leaves(seed=95, size=11, width=0.22, cover_at=(0.3, 0.44, 0.56), lift=-0.08)
    return shift(t, L=-0.06, a=-0.02, b=-0.035, chroma=0.75)


def redwood_needles():
    """Redwood: flat sprays of short, dark green needles."""
    return shift(foliage.pine_leaves(seed=96), L=-0.03, a=-0.006, b=-0.012)


def cherry_blossom():
    """Cherry in flower: clusters of five-petalled pale pink blossoms with deeper pink hearts, a few young leaves
    showing between them; three depths."""
    rng = np.random.default_rng(97)
    X, Y = coords()
    field, depth, cover = np.zeros((N, N)), np.zeros((N, N)), np.zeros((N, N), bool)
    leaf = foliage.leaves(seed=98, size=5, width=0.45, cover_at=(0.18,), lift=-0.2)
    r = 9
    ys, xs = np.mgrid[-r:r + 1, -r:r + 1]
    for layer, target in enumerate((0.32, 0.48, 0.6)):
        while (cover | (leaf[..., 3] > 0)).mean() < target:
            cx, cy, size = rng.integers(0, N), rng.integers(0, N), rng.uniform(4.5, 7.5)
            spin = rng.uniform(0, 2 * np.pi)
            ang, dist = np.arctan2(ys, xs) - spin, np.hypot(xs, ys) / size
            petal = 0.75 + 0.25 * np.cos(5 * ang)  # five rounded petals, notched between
            inside = dist < petal
            v = 0.55 + layer * 0.15 + 0.25 * dist - 0.35 * (dist < 0.25)
            px, py = (cx + xs) % N, (cy + ys) % N
            field[py[inside], px[inside]] = v[inside]
            depth[py[inside], px[inside]] = (layer + 1) / 3
            cover[py[inside], px[inside]] = True
    pink = np.zeros((N, N, 3))
    lab = np.stack([0.72 + (field - 0.6) * 0.35, 0.075 - (field - 0.6) * 0.06, 0.005 + 0 * field], -1)
    lab[..., 0] -= foliage._shadowed(depth, cover) * 0.12
    lab[..., 1] += (fbm(X, Y, 6, 3, 99) - 0.5) * 0.03
    pink = oklab_to_srgb(lab.reshape(-1, 3)).reshape(N, N, 3)
    out = leaf.copy()
    out[cover, :3] = pink[cover]
    out[cover, 3] = 255
    return out
