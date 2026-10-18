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
