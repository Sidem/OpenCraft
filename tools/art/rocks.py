"""The other rocks (docs/ART_STYLE.md): granite, sandstone, basalt, limestone, quartz ore, bedrock and spent rock. Each is its
own texture language so they read apart at a glance: granite is a coarse speckled crystal mosaic, sandstone hard red strata,
basalt dark polygonal columns, limestone pale layers with pits and shells. All wrap at the edge. Ores are `natural.stone`
bodies with a deposit, as in `ores.py`.
"""
import numpy as np

from lib import N, bayer, cavity, coords, edges, fbm, grow, hash2, paint, ramp, relief_light, rgba, roll, worley
from natural import RAMPS, leaf_shape, stone

GRANITE = ramp(["#141824", "#2f2a38", "#574649", "#836359", "#ab8672", "#d0ab93", "#ecd3bc"])
SANDSTONE = ramp(["#1c1624", "#44283a", "#74423f", "#a3604a", "#cc8858", "#e9b27c", "#f9d9a2"])
BASALT = ramp(["#070a12", "#0d1320", "#16202f", "#223045", "#344760", "#4d6482", "#7690ab"])
LIMESTONE = ramp(["#1a2130", "#3a4150", "#686a72", "#958f8d", "#bcb2a7", "#dcd0bf", "#f2e9d6"])
BEDROCK = ramp(["#030409", "#07090f", "#0e1220", "#171d30", "#232b42", "#323d5a", "#475577"])
ASH = ramp(["#1b1c26", "#383844", "#5a5863", "#827e83", "#a8a29e", "#cdc4b8", "#e9e0d0"])
QUARTZ = ramp(["#0b1c28", "#1f4458", "#4a8697", "#93c9d0", "#d4f1f2", "#ffffff"])


def _wrap_dist(a, b):
    d = np.abs(a - b)
    return np.minimum(d, 1 - d)


def granite(seed=0):
    """Coarse crystal mosaic: grains of feldspar pink and quartz grey with dark mica and bright glint specks."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, 14, seed + 1, 1.0)
    edge = f2 - f1
    h = pid * 0.5 + np.minimum(edge / 0.25, 1) * 0.3
    light = relief_light(h * 4, 0.9)
    big = fbm(X, Y, 3, 3, seed + 2)
    v = 0.3 + pid * 0.36 + (light - 0.5) * 0.45 + (big - 0.5) * 0.25 + (fbm(X, Y, 30, 2, seed + 3) - 0.5) * 0.1
    v = np.where(edge < 0.05, v - 0.2, v)
    img = paint(GRANITE, v, 0.25)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    mica = hash2(ix // 2, iy // 2, seed + 4) > 0.9
    glint = hash2(ix // 2, iy // 2, seed + 5) > 0.985
    img[mica] = GRANITE[0]
    img[glint] = GRANITE[6]
    return rgba(img)


def sandstone(seed=0):
    """Hard red strata: each band its own tone, a dark seam under it, crossbed flecks and grain."""
    X, Y = coords()
    warp = fbm(X, Y, 3, 3, seed + 1) - 0.5
    pos = Y * 8 + warp * 1.6
    band = np.floor(pos).astype(int) % 8
    local = pos % 1.0
    tone = hash2(band, 0, seed + 2)
    cross = np.sin((X * 5 + local * 0.7 + band * 0.37) * 2 * np.pi) * 0.5 + 0.5
    grain = fbm(X, Y, 40, 1, seed + 3)
    v = 0.28 + tone * 0.34 + (local - 0.5) * 0.18 + (cross - 0.5) * 0.08 + (grain - 0.5) * 0.2
    v = np.where(local < 0.07, v - 0.22, v)
    img = paint(SANDSTONE, v, 0.3)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    img[(hash2(ix // 2, iy // 2, seed + 4) > 0.975) & (local > 0.2)] = SANDSTONE[6]
    return rgba(img)


def basalt(seed=0):
    """Dark polygonal columns: near-regular cells split by deep cracks, each facet lit from the top left."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, 5, seed + 1, 0.4)
    edge = f2 - f1
    h = np.minimum(edge / 0.4, 1) ** 0.5 * 0.7 + pid * 0.2
    light = relief_light(h * 6, 1.1)
    v = 0.3 + pid * 0.22 + (light - 0.5) * 0.7 + (fbm(X, Y, 22, 3, seed + 2) - 0.5) * 0.2
    v = v - cavity(h * 3, 2.5)
    v = np.where(edge < 0.035, 0.02 + 0.06 * fbm(X, Y, 20, 2, seed + 3), v)
    img = paint(BASALT, v, 0.3)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    img[hash2(ix, iy, seed + 4) > 0.993] = BASALT[6]
    return rgba(img)


def _shell(X, Y, cx, cy, r):
    """An ammonite: a mask of its whorls (a spiral of ridges) and its centre, on a wrapped tile."""
    dx, dy = ((X - cx + 0.5) % 1) - 0.5, ((Y - cy + 0.5) % 1) - 0.5
    d = np.hypot(dx, dy) * N
    ang = np.arctan2(dy, dx)
    whorl = ((d / r * 2.0 - ang / (2 * np.pi)) % 1.0) < 0.3
    return (d < r) & whorl, d < r


def limestone(seed=0):
    """Pale layers with hard seams, small pits, and a few fossil shells."""
    X, Y = coords()
    warp = fbm(X, Y, 3, 3, seed + 1) - 0.5
    pos = Y * 7 + warp * 1.4
    local = pos % 1.0
    tone = hash2(np.floor(pos).astype(int) % 7, 0, seed + 2)
    grain = fbm(X, Y, 36, 2, seed + 3)
    v = 0.5 + tone * 0.16 + (local - 0.5) * 0.1 + (grain - 0.5) * 0.2 + (fbm(X, Y, 5, 3, seed + 4) - 0.5) * 0.2
    v = np.where(local < 0.05, v - 0.2, v)
    pits = (fbm(X, Y, 30, 2, seed + 5) > 0.72) & (fbm(X, Y, 4, 2, seed + 8) > 0.4)
    v = np.where(pits, v - 0.2, np.where(grow(pits, 1) & roll(pits, -1, -1), v + 0.1, v))
    img = paint(LIMESTONE, v, 0.3)
    rng = np.random.default_rng(seed + 6)
    for _ in range(1 + seed % 2):
        cx, cy, r = rng.uniform(0, 1), rng.uniform(0, 1), rng.uniform(12, 17)
        whorl, disc = _shell(X, Y, cx, cy, r)
        img[disc] = LIMESTONE[5]
        img[whorl] = LIMESTONE[3]
        img[edges(disc) & disc] = LIMESTONE[2]
    return rgba(img)


def quartz_ore(alt=0):
    """Clusters of long glassy crystals fanning out of the stone, each with a lit and a shaded facet and a pointed tip."""
    rng = np.random.default_rng(130 + alt * 29)
    img = stone(0)[..., :3].copy()
    ys, xs = np.mgrid[-36:37, -36:37]
    for _ in range(3):
        bx, by = rng.integers(26, N - 26, 2)
        fan = rng.uniform(0, 2 * np.pi)
        for _ in range(int(rng.integers(4, 7))):
            ang = fan + rng.uniform(-0.8, 0.8)
            la, lb = rng.uniform(15, 24), rng.uniform(4.5, 7.0)
            u = xs * np.cos(ang) + ys * np.sin(ang) - la * 0.7
            w = -xs * np.sin(ang) + ys * np.cos(ang)
            tip = np.clip((la - u) / (la * 0.28), 0, 1)
            inside = (np.abs(w) < lb * np.minimum(1, tip + 0.15)) & (u > -la) & (u < la)
            facet = np.clip(0.55 + 0.9 * (-w / lb) * 0.5 + 0.25 * (u / la), 0, 0.999)
            idx = (facet * len(QUARTZ)).astype(int)
            px, py = (bx + xs) % N, (by + ys) % N
            img[py[inside], px[inside]] = QUARTZ[idx[inside]]
            rim = edges(inside) & inside
            img[py[rim], px[rim]] = QUARTZ[0]
    return rgba(img)


def bedrock(seed=5):
    """The floor of the world: almost black blue-violet, deep cracks, nothing a player can dig."""
    return stone(seed, BEDROCK, 0.0)


def spent_rock(seed=9):
    """What a worked-out deposit leaves: bleached, pitted ash-grey rock with rust-stained seams."""
    X, Y = coords()
    base = stone(seed, ASH, 0.1)[..., :3].copy()
    pits = (fbm(X, Y, 28, 2, seed + 1) > 0.7) & (fbm(X, Y, 4, 2, seed + 7) > 0.4)
    rim = grow(pits, 1) & ~pits & ~roll(pits, 1, 1)
    base[pits] = ASH[1]
    base[rim] = ASH[6]
    rust = (fbm(X, Y, 4, 3, seed + 2, 0.5, 0.4) > 0.64) & ~pits
    base[rust & (hash2((X * N).astype(int) // 2, (Y * N).astype(int) // 2, seed + 3) > 0.45)] = RAMPS["dirt"][4]
    return rgba(base)
