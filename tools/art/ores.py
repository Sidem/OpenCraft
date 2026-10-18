"""Ore blocks: the plain `natural.stone` body with a deposit painted on, one shape language per ore so they differ
without colour: coal is dark angular shards (darkest tile), iron is fat round nodules, copper is green malachite crusts
along cracks, uranium black pitchblende masses with yellow flakes. Ore colours follow the real minerals.
`alt` (0..3) moves the deposit but keeps the body, so seams between alternates stay quiet."""
import numpy as np

from lib import N, coords, edges, fbm, grow, hash2, paint, ramp, relief_light, rgba, roll, smooth, worley
from natural import leaf_shape, stone
from stains import COAL, RUST as IRON

# Mineral colours from specimens: malachite banded deep to pale green, azurite deep blue, chalcopyrite brass.
MALACHITE = ramp(["#072219", "#0e3a29", "#175a3c", "#227a50", "#3a9a66", "#6cbd8a", "#a8dcb6"])
AZURITE = ramp(["#0c1630", "#18295a", "#25407f", "#3a5c9e", "#7088c0"])
CHALCOPYRITE = ramp(["#2e230c", "#6a5520", "#a68e3c", "#d4c068", "#f0e4a0"])


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
PITCH = ramp(["#050506", "#111113", "#1f1d1c", "#33302c", "#5a544c", "#8a8278"])
AUTUNITE = ramp(["#5a5208", "#9a9214", "#d2cc30", "#ece86c", "#faf8bc"])


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
    """Pitchblende as it is found: black, greasy, kidney-shaped masses with a soft sheen, and the lemon-yellow flakes of
    autunite (its weathering crust) on and around them; the flakes glow faintly, as autunite fluoresces."""
    X, Y = coords()
    seed = 170 + alt * 37
    field = fbm(X, Y, 3, 4, seed, 0.5, 0.3)
    mass = field > 0.5
    g1, _, _ = worley(X, Y, 14, seed + 1, 1.0)  # botryoidal: the mass is a crowd of small domes
    dome = np.clip((field - 0.5) / 0.1, 0, 1) ** 0.5 * 2 + (1 - np.clip(g1, 0, 0.7) / 0.7) ** 0.5
    light = relief_light(dome * 4, 1.0)
    sheen = np.clip((light - 0.6) * 4, 0, 1) ** 2  # a greasy lustre catches the light on each dome
    black = smooth(PITCH, 0.2 + (light - 0.5) * 0.5 + sheen * 0.5 + (fbm(X, Y, 30, 2, seed + 2) - 0.5) * 0.1)
    body = _body(alt).astype(float)
    out = np.where(mass[..., None], black, body)
    rim = (field > 0.44) & (field < 0.53)  # the yellow crust forms where the pitchblende meets air and water
    crust = rim & (fbm(X, Y, 20, 3, seed + 4) > 0.55)
    yellow = smooth(AUTUNITE, 0.3 + (light - 0.5) * 0.8 + (fbm(X, Y, 40, 1, seed + 5) - 0.5) * 0.5)
    out = np.where(crust[..., None], yellow, out)
    glow = np.where(crust, 0.3, 0.0)
    return rgba(np.clip(out, 0, 255).astype(np.uint8), glow=glow)


def copper(alt=0):
    """Copper ore as it shows at the surface: green malachite crusts spreading from a crack network into the rock,
    banded parallel to the crack, a green stain fading into the stone around them, a few blue azurite spots and brassy
    chalcopyrite grains in the cracks. Continuous tone, no outlines (the realistic look, ART_STYLE.md section 12)."""
    X, Y = coords()
    seed = 90 + alt * 17
    line = np.abs(fbm(X, Y, 2, 3, seed, 0.5, 0.5) - 0.5)  # 0 on a crack
    patch = fbm(X, Y, 3, 3, seed + 3)
    rough = fbm(X, Y, 12, 4, seed + 5)
    width = 0.006 + 0.12 * np.clip((patch - 0.38) / 0.25, 0, 1) ** 1.3  # the crust gathers in a few broad patches
    d = line / width + (rough - 0.5) * 1.1  # < 1 inside the crust, ragged at its edge
    crust = d < 1
    band = 0.5 + 0.5 * np.sin(np.clip(d, 0, 1.5) * 12 + rough * 5)
    height = np.clip(1 - d, 0, 1) ** 0.5 * 3 + rough * 0.8
    light = relief_light(height, 1.2)
    green = smooth(MALACHITE, 0.36 + 0.25 * np.clip(1 - d, 0, 1) + (band - 0.5) * 0.3 + (light - 0.5) * 0.6
                   + (fbm(X, Y, 32, 2, seed + 6) - 0.5) * 0.15)
    body = _body(alt).astype(float)
    halo = np.clip((3.0 - d) / 2.0, 0, 1) ** 1.3 * (0.25 + 0.45 * fbm(X, Y, 10, 3, seed + 8))
    out = body * (1 - halo[..., None]) + MALACHITE[3].astype(float) * halo[..., None]
    out = np.where(crust[..., None], green, out)
    azure = crust & (fbm(X, Y, 8, 3, seed + 7) > 0.71) & (d < 0.7)
    out = np.where(azure[..., None], smooth(AZURITE, 0.25 + (light - 0.5) * 0.7 + (band - 0.5) * 0.2), out)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    grain = (d < 0.3) & (hash2(ix // 2, iy // 2, seed + 9) > 0.93)
    out = np.where(grain[..., None], smooth(CHALCOPYRITE, 0.35 + (light - 0.5) * 1.2), out)
    out = np.where((line < 0.0035)[..., None], out * 0.45, out)  # the crack itself, a shadow not an outline
    return rgba(np.clip(out, 0, 255).astype(np.uint8))
