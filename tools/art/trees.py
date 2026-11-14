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
    """Scots pine: furrowed plates, warm orange-brown where the outer bark has flaked."""
    return shift(wood.bark(seed=11), L=0.03, a=0.022, b=0.02)


def acacia_bark():
    """Acacia: dark grey, deeply and finely furrowed."""
    return shift(wood.bark(seed=21), L=-0.07, chroma=0.55)


def palm_bark():
    """A palm's stem: pale grey-brown fibres up it, crossed by the ring scars of old fronds; tiles."""
    X, Y = coords()
    fibre = wood.streaks(X, Y, 32, 4, 3, 31)
    ring = np.zeros_like(X)
    for k, y0 in enumerate((0.12, 0.45, 0.78)):  # leaf scars: a ridge with a dark groove under it, uneven
        wob = (fbm(X, Y, 4, 2, 32 + k) - 0.5) * 0.06
        d = (Y - y0 + wob + 0.5) % 1.0 - 0.5
        ring += (_soft(0.01 - np.abs(d), 0.012) - 0.8 * _soft(0.008 - np.abs(d - 0.02), 0.01)) * (0.6 + 0.4 * fibre)
    h = fibre * 0.5 + ring * 0.4 + wood.streaks(X, Y, 6, 3, 2, 34) * 0.3
    field = relief_light(blur(h, 0.7) * 5, 1.0) * 0.3 + h * 0.6 + fbm(X, Y, 8, 3, 33) * 0.15
    return shift(rgba(colour("dead_wood", field, 1.2, seed=31)), L=-0.04, a=0.006, b=0.025)


def cherry_bark():
    """Cherry: smooth, glossy dark red-brown, banded across by pale lenticels; tiles."""
    X, Y = coords()
    rng = np.random.default_rng(41)
    sheen = wood.streaks(X, Y, 3, 24, 3, 42)  # fibres run round the trunk
    lent = np.zeros((N, N))
    for _ in range(70):
        x, y, w = rng.uniform(0, 1), rng.uniform(0, 1), rng.uniform(0.04, 0.16)
        d = np.hypot(((X - x + 0.5) % 1.0 - 0.5) / w, ((Y - y + 0.5) % 1.0 - 0.5) / rng.uniform(0.005, 0.01))
        lent = np.maximum(lent, _soft(1 - d, 0.4) * rng.uniform(0.4, 1.0))
    field = 0.45 + sheen * 0.3 + lent * 0.5 + fbm(X, Y, 6, 3, 43) * 0.15
    return shift(rgba(colour("bark", field, 1.0, seed=41)), L=-0.06, a=0.03, b=-0.005)


def baobab_bark():
    """Baobab: smooth grey with a pink-copper cast, folded in soft horizontal wrinkles; tiles."""
    X, Y = coords()
    fold = wood.streaks(X, Y, 3, 14, 3, 51)
    h = fold + fbm(X, Y, 4, 3, 52) * 0.4
    field = relief_light(blur(h, 1.5) * 4, 1.0) * 0.5 + h * 0.4
    return shift(rgba(colour("dead_wood", field, 0.9, seed=51)), L=0.02, a=0.014, b=0.01)


def redwood_bark():
    """Redwood: thick, soft red-brown bark in long ropy fibres and deep vertical furrows; tiles."""
    X, Y = coords()
    Xw = (X + (wood.streaks(X, Y, 2, 4, 2, 61) - 0.5) * 0.08) % 1.0
    rope = wood.streaks(Xw, Y, 20, 2, 3, 62)
    furrow = _soft(0.3 - wood.streaks(Xw, Y, 7, 1, 2, 63), 0.15)
    h = rope * 0.7 - furrow * 0.8
    field = relief_light(blur(h, 0.7) * 5, 1.0) * 0.3 + h * 0.6
    return shift(rgba(colour("bark", field, 1.2, seed=61)), L=0.04, a=0.05, b=0.03)


def eucalyptus_bark():
    """Eucalyptus: smooth, shedding in patches: cream, grey-green and ochre, the edges of each peel a little darker."""
    X, Y = coords()
    patch = wood.streaks(X, Y, 3, 5, 4, 71, 0.6)  # long patches, as the bark sheds in strips down the trunk
    edge = sum(_soft(0.012 - np.abs(patch - t), 0.012) for t in (0.42, 0.58))
    field = 0.6 + (fbm(X, Y, 16, 3, 72) - 0.5) * 0.25 - blur(edge, 1.0) * 0.1
    pale = rgba(colour("birch", field, 0.8, seed=71)).astype(float)
    out = pale * 0
    for t0, (L, a, b) in ((0.0, (0.0, -0.005, 0.01)), (0.42, (-0.1, -0.02, 0.02)), (0.58, (-0.06, 0.012, 0.06))):
        w = _soft(patch - t0, 0.04)[..., None]  # each later layer shows over the one before, softly
        out = out * (1 - w) + shift(pale.astype(np.uint8), L=L, a=a, b=b).astype(float) * w
    return out.clip(0, 255).astype(np.uint8)


def bamboo_culm():
    """Bamboo: a smooth green culm, faintly streaked lengthwise, a pale node ring across it with a dark line above; tiles."""
    X, Y = coords()
    streak = wood.streaks(X, Y, 24, 2, 2, 81)
    d = (Y - 0.5 + 0.5) % 1.0 - 0.5
    node = _soft(0.02 - np.abs(d), 0.015)
    line = _soft(0.006 - np.abs(d + 0.03), 0.006)
    field = 0.62 + streak * 0.12 + node * 0.25 - line * 0.5 + fbm(X, Y, 4, 2, 82) * 0.1
    green = rgba(colour("leaves", field, 0.7, seed=81))
    return shift(green, L=0.12, a=-0.005, b=0.02, chroma=0.85)


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
