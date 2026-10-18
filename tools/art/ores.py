"""Ore blocks: the plain `natural.stone` body with a deposit painted on, one shape language per ore so they differ
without colour: coal is dark angular shards (darkest tile), iron is fat round nodules, copper is thin branching veins
with a teal crust. `alt` (0..3) moves the deposit but keeps the body, so seams between alternates stay quiet."""
import numpy as np

from lib import N, coords, edges, fbm, grow, hash2, paint, ramp, relief_light, rgba, roll, worley
from natural import leaf_shape, stone
from stains import COAL, RUST as IRON, VERDIGRIS

COPPER = ramp(["#33160f", "#70361f", "#b9632e", "#e8915a", "#ffc58a"])


def _body(alt):
    return stone(0)[..., :3]


def iron(alt=0):
    """Fat rust nodules, domed and outlined, with a bright glint on each."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, 5, 40 + alt * 13, 0.95)
    r = (0.2 + pid * 0.2) * (0.75 + 0.5 * fbm(X, Y, 14, 2, 44 + alt))
    node = (pid > 0.5) & (f1 < r)
    cx = (X * 5) % 1.0
    dome = np.where(node, 1 - f1 / r, 0)
    h = dome
    light = relief_light(h * 5, 1.4)
    v = 0.25 + dome * 0.4 + (light - 0.5) * 1.0 + (fbm(X, Y, 22, 2, 41 + alt) - 0.5) * 0.18
    img = paint(IRON, v, 0.3)
    ring = node & (f1 > r - 0.045)
    img = np.where(ring[..., None], IRON[0], img)
    body = _body(alt)
    shadow = grow(node, 3) & ~node & roll(node, -2, -2)
    body = np.where(shadow[..., None], (body * 0.55).astype(np.uint8), body)
    return rgba(np.where(node[..., None], img, body))


def coal(alt=0):
    """Clusters of dark glassy shards with a cold blue gleam on their lit edges."""
    rng = np.random.default_rng(70 + alt * 31)
    body = _body(alt).copy()
    ys, xs = np.mgrid[-24:25, -24:25]
    img = body
    for cx0, cy0 in rng.integers(0, N, (3, 2)):
        for _ in range(7):
            cx, cy = (cx0 + rng.integers(-14, 15)) % N, (cy0 + rng.integers(-14, 15)) % N
            ang, la = rng.uniform(0, np.pi), rng.uniform(11, 20)
            inside, u, w = leaf_shape(xs, ys, ang, la, la * rng.uniform(0.35, 0.55))
            gleam = np.clip(0.25 + 0.5 * np.clip((-(xs + ys) / (2 * la)), 0, 1) + 0.2 * (np.abs(w) < 0.2), 0, 1)
            idx = (gleam * len(COAL)).astype(int).clip(0, len(COAL) - 1)
            px, py = (cx + xs) % N, (cy + ys) % N
            img[py[inside], px[inside]] = COAL[idx[inside]]
            rim = edges(inside)
            img[py[rim & inside], px[rim & inside]] = COAL[0]
    return rgba(img)


BAUXITE = ramp(["#2a0f0a", "#5a2418", "#8c3e26", "#bf6a40", "#e8a070"])
TAR = ramp(["#050403", "#120d08", "#241a10", "#3c2c18", "#6a5230"])
URANIUM = ramp(["#0a2a06", "#2a7a10", "#70d020", "#c8ff60", "#f0ffb0"])


def bauxite(alt=0):
    """Clay-red pebbles (pisoliths) packed into the stone: many small round domes, each with a pale glint."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, 9, 150 + alt, 0.9)
    r = 0.3 + pid * 0.16
    node = (pid > 0.12) & (f1 < r)
    dome = np.where(node, 1 - f1 / r, 0)
    light = relief_light(dome * 5, 1.3)
    v = 0.3 + dome * 0.35 + (light - 0.5) * 0.9 + (fbm(X, Y, 24, 2, 151) - 0.5) * 0.15
    img = np.where(node[..., None], paint(BAUXITE, v, 0.3), _body(alt))
    ring = node & (f1 > r - 0.05)
    img[ring] = BAUXITE[0]
    return rgba(img)


def oil_sand(alt=0):
    """Stone soaked with tar: black glossy pools that creep along the grain, a few amber flecks."""
    X, Y = coords()
    pools = fbm(X, Y, 5, 4, 160 + alt, 0.55, 0.4) > 0.5
    light = relief_light(fbm(X, Y, 16, 2, 161) * 3, 1.2)
    img = np.where(pools[..., None], paint(TAR, 0.2 + (light - 0.5) * 0.9, 0.3), _body(alt))
    img[pools & (light > 0.7)] = TAR[4]
    amber = (hash2((X * N).astype(int) // 2, (Y * N).astype(int) // 2, 162) > 0.96) & ~pools
    img[amber] = ramp(["#a86010", "#f0a020"])[1]
    return rgba(img)


def uranium(alt=0):
    """Angular green crystals that light themselves, clustered in the stone."""
    rng = np.random.default_rng(170 + alt * 37)
    img = _body(alt).copy()
    glow = np.zeros((N, N))
    ys, xs = np.mgrid[-22:23, -22:23]
    for cx0, cy0 in rng.integers(0, N, (4, 2)):
        for _ in range(5):
            cx, cy = (cx0 + rng.integers(-12, 13)) % N, (cy0 + rng.integers(-12, 13)) % N
            ang, la = rng.uniform(0, np.pi), rng.uniform(8, 15)
            # A prism: straight sides, one square end and one pointed tip, split into a lit and a shaded facet.
            u = xs * np.cos(ang) + ys * np.sin(ang)
            w = -xs * np.sin(ang) + ys * np.cos(ang)
            half = la * 0.32 * np.clip((la - u) / (la * 0.45), 0.0, 1.0)
            inside = (u > -la) & (u < la) & (np.abs(w) < np.maximum(half, 0.5 * (u < la - 1)))
            shade = np.clip(0.45 + 0.45 * np.clip(-w / (la * 0.32), -1, 1) + 0.1 * (u / la), 0, 1)
            idx = (shade * len(URANIUM)).astype(int).clip(0, len(URANIUM) - 1)
            px, py = (cx + xs) % N, (cy + ys) % N
            img[py[inside], px[inside]] = URANIUM[idx[inside]]
            glow[py[inside], px[inside]] = 0.45 + 0.4 * shade[inside]
            rim = edges(inside) & inside
            img[py[rim], px[rim]] = URANIUM[0]
            glow[py[rim], px[rim]] = 0.1
    return rgba(img, glow=glow)


def copper(alt=0):
    """Thin branching veins of bright metal, crusted with teal verdigris where they thicken."""
    X, Y = coords()
    seed = 90 + alt * 17
    line = np.abs(fbm(X, Y, 4, 3, seed, 0.5, 0.45) - 0.5)
    blob = fbm(X, Y, 7, 2, seed + 3)
    vein = line < 0.022 + 0.05 * np.clip(blob - 0.4, 0, 1)
    crust = grow(vein, 2) & ~vein & (fbm(X, Y, 12, 2, seed + 5) > 0.42)
    body = _body(alt)
    light = relief_light(fbm(X, Y, 18, 2, seed + 6) * 3, 1.0)
    metal = paint(COPPER, 0.35 + (light - 0.5) * 0.8 + blob * 0.35, 0.3)
    green = paint(VERDIGRIS, 0.3 + (light - 0.5) * 0.7 + blob * 0.3, 0.3)
    out = np.where(crust[..., None], green, body)
    out = np.where(vein[..., None], metal, out)
    ink = edges(vein) & ~roll(vein, 1, 1)
    out[ink] = COPPER[0]
    return rgba(out)
