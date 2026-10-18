"""Ore blocks in the realistic look (docs/ART_STYLE.md section 12): continuous tone, no outlines, real mineral colours.
Most are the `real.stone` body with a deposit grown into it: soft-edged masks, and the rock's own relief shading carried
through the deposit so it sits in the surface instead of on it. Bauxite and oil sand are the whole block. Each ore keeps
its own shape language so they differ without colour: coal a black seam along the bedding (bands), iron ragged rusty
masses, copper malachite crusts along cracks, uranium black pitchblende with yellow flakes, bauxite small pisoliths in
pale earthy rock, oil sand dark granular sand with glossy pools.
`alt` (0..3) moves the deposit but keeps the body, so seams between alternates stay quiet."""
import numpy as np

from lib import N, blur, coords, fbm, hash2, ramp, relief_light, rgba, smooth, worley
from real import stone

# Mineral colours from specimens: malachite banded deep to pale green, azurite deep blue, chalcopyrite brass.
MALACHITE = ramp(["#072219", "#0e3a29", "#175a3c", "#227a50", "#3a9a66", "#6cbd8a", "#a8dcb6"])
AZURITE = ramp(["#0c1630", "#18295a", "#25407f", "#3a5c9e", "#7088c0"])
CHALCOPYRITE = ramp(["#2e230c", "#6a5520", "#a68e3c", "#d4c068", "#f0e4a0"])
COAL_SEAM = ramp(["#060607", "#0e0e10", "#18181b", "#25262a", "#3a3c42", "#62666f"])
LIMONITE = ramp(["#2e140e", "#4a2016", "#6a2e1e", "#88422a", "#a45a38", "#bc7448"])  # hematite red-brown
BAUXITE = ramp(["#3a1208", "#62200e", "#8c3a1c", "#b05a30", "#cc8050"])  # crushed and washed bauxite use it too
PISOLITH = ramp(["#5e3022", "#7c442e", "#985a40", "#ae7256", "#c48e72"])  # bauxite's pisoliths in the rock
LATERITE = ramp(["#6e4a3a", "#906a56", "#ac8670", "#c4a28a", "#d8bca4"])  # bauxite's earthy matrix
OILSAND = ramp(["#140d05", "#24170a", "#382410", "#4e3418", "#684822", "#8a6630"])
PITCH = ramp(["#050506", "#111113", "#1f1d1c", "#33302c", "#5a544c", "#8a8278"])
AUTUNITE = ramp(["#5a5208", "#9a9214", "#d2cc30", "#ece86c", "#faf8bc"])


def _body(alt):
    return stone(0)[..., :3].astype(float)


def _relief(body):
    """The body's lightness as a multiplier around 1, so a deposit keeps the rock's bumps and cracks."""
    lum = body.mean(-1)
    return (lum / lum.mean())[..., None]


def _blend(body, colour, amount):
    """`body` moved toward `colour` (one rgb or an image) by `amount` (0..1 per texel), as floats."""
    a = np.clip(np.asarray(amount, float), 0, 1)[..., None]
    return body * (1 - a) + np.asarray(colour, float) * a


def _soft(x, width):
    """0 below -width/2, 1 above +width/2, linear between: a mask edge about `width` field units wide."""
    return np.clip(x / width + 0.5, 0, 1)


def iron(alt=0):
    """Iron ore weathering in the rock: ragged red-brown limonite and hematite masses with a darker core, rust stain
    bleeding into the stone around them, and rare steel-grey hematite specks."""
    X, Y = coords()
    seed = 40 + alt * 13
    body = _body(alt)
    rel = _relief(body)
    field = fbm(X, Y, 3, 5, seed, 0.55, 0.6)
    top = np.quantile(field, 0.72)  # the masses cover about a quarter of every alternate
    mass = _soft(field - top, 0.035)
    stain = _soft(field - top + 0.07, 0.08) * (0.25 + 0.35 * fbm(X, Y, 10, 3, seed + 3)) * (1 - mass)
    core = np.clip((field - top) / 0.12, 0, 1)
    rough = fbm(X, Y, 24, 3, seed + 2)
    v = 0.5 - core * 0.35 + (rough - 0.5) * 0.35 + (fbm(X, Y, 8, 3, seed + 4) - 0.5) * 0.25
    out = _blend(body, LIMONITE[3] * rel, stain)
    out = _blend(out, smooth(LIMONITE, v) * rel ** 1.2, mass)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    speck = (hash2(ix, iy, seed + 5) > 0.995) & (core > 0.3)
    out = np.where(speck[..., None], np.array([100.0, 96, 100]) * rel, out)
    return rgba(out.clip(0, 255).astype(np.uint8))


def coal(alt=0):
    """A coal seam in the rock: a dull black band along the bedding (and a thin second one) with ragged edges, faint
    brighter laminae, cleat cracks, and a soot stain fading into the stone either side."""
    X, Y = coords()
    seed = 70 + alt * 31
    body = _body(alt)
    rel = _relief(body)
    flat = np.full_like(X, 0.37)
    centre = (Y + (fbm(X, flat, 2, 3, seed) - 0.5) * 0.3 + alt * 0.23) % 1.0
    thick = 0.07 + 0.09 * fbm(X, flat, 3, 3, seed + 1)
    d = np.minimum(np.abs(centre - 0.5) / thick, np.abs(centre - 0.12) / (thick * 0.35))  # < 1 inside a band
    d = d + (fbm(X, Y, 16, 3, seed + 6) - 0.5) * 0.6  # ragged edges
    band = _soft(1 - d, 0.25)
    lamina = np.clip((np.sin(centre * 90 + fbm(X, Y, 8, 2, seed + 2) * 6) - 0.6) / 0.4, 0, 1)
    v = 0.1 + lamina * 0.12 + (fbm(X, Y, 40, 2, seed + 4) - 0.5) * 0.2 + (fbm(X, Y, 6, 3, seed + 3) - 0.5) * 0.25
    cleat = np.abs(((X * 9 + fbm(X, Y, 6, 2, seed + 5) * 1.5) % 1.0) - 0.5)
    v = v * (1 - 0.6 * _soft(cleat - 0.475, 0.03))
    soot = np.clip(1 - (d - 1) / 1.2, 0, 1) * (1 - band) * 0.5
    out = _blend(body, COAL_SEAM[1] * rel, blur(soot, 1.0))
    out = _blend(out, smooth(COAL_SEAM, v) * rel, band)
    return rgba(out.clip(0, 255).astype(np.uint8))


def bauxite(alt=0):
    """Bauxite as it is mined, the whole block ore: a pale, earthy pink-tan rock stained red in patches, crowded with
    rounded red-brown pisoliths of many sizes grown into it, darker at the core and fading into the matrix."""
    X, Y = coords()
    seed = 150 + alt
    wobble = 0.85 + 0.3 * fbm(X, Y, 20, 2, seed + 4)
    peb, dome = np.zeros_like(X), np.zeros_like(X)
    for cells, keep, k in ((9, 0.45, 0), (17, 0.3, 1), (31, 0.4, 2)):
        f1, _, pid = worley(X, Y, cells, seed + 10 * k, 1.0)
        r = (0.2 + pid * 0.25) * wobble
        m = _soft(r - f1, 0.14) * (pid > keep)
        dome = np.maximum(dome, m * np.clip(1 - f1 / r, 0, 1))
        peb = np.maximum(peb, m)
    earth = fbm(X, Y, 6, 4, seed + 1)
    height = blur(dome, 0.8) * 2.5 + earth * 1.5 + fbm(X, Y, 30, 2, seed + 2) * 0.5
    light = relief_light(height, 1.0)
    grain = fbm(X, Y, 48, 1, seed + 3) - 0.5
    matrix = smooth(LATERITE, 0.45 + (earth - 0.5) * 0.5 + (light - 0.5) * 0.5 + grain * 0.2)
    matrix = _blend(matrix, PISOLITH[3], _soft(fbm(X, Y, 4, 3, seed + 5) - 0.55, 0.2) * 0.45)
    pebble = smooth(PISOLITH, 0.6 - 0.35 * dome + (light - 0.5) * 0.6 + grain * 0.15)
    return rgba(_blend(matrix, pebble, peb).clip(0, 255).astype(np.uint8))


def oil_sand(alt=0):
    """Oil sand, the whole block ore: quartz sand bound in bitumen, dark brown and granular, smooth and glossy where
    the tar has pooled, a few cleaner grains showing."""
    X, Y = coords()
    seed = 160 + alt
    grain = fbm(X, Y, 64, 1, seed)
    clump = fbm(X, Y, 8, 3, seed + 1)
    pool = _soft(fbm(X, Y, 4, 3, seed + 2, 0.5, 0.3) - 0.58, 0.12)
    light = relief_light(blur(clump, 1.5) * 4 + blur(grain, 0.6) * (1 - pool), 1.0)
    sheen = np.clip((light - 0.55) * 5, 0, 1) ** 2 * pool
    v = 0.4 + (grain - 0.5) * 0.7 * (1 - pool) + (clump - 0.5) * 0.3 + (light - 0.5) * 0.4 - pool * 0.3 + sheen * 0.5
    out = smooth(OILSAND, v)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    clean = (hash2(ix, iy, seed + 3) > 0.88) * (1 - pool) * (0.2 + 0.35 * hash2(ix, iy, seed + 4))
    out = _blend(out, np.array([150.0, 130, 98]), clean)
    return rgba(out.clip(0, 255).astype(np.uint8))


def uranium(alt=0):
    """Pitchblende as it is found: black, greasy, kidney-shaped masses with a soft sheen, and the lemon-yellow flakes of
    autunite (its weathering crust) on and around them; the flakes glow faintly, as autunite fluoresces."""
    X, Y = coords()
    seed = 170 + alt * 37
    field = fbm(X, Y, 3, 4, seed, 0.5, 0.3)
    mass = _soft(field - 0.5, 0.06)
    g1, _, _ = worley(X, Y, 14, seed + 1, 1.0)  # botryoidal: the mass is a crowd of small domes
    dome = np.clip((field - 0.5) / 0.1, 0, 1) ** 0.5 * 2 + (1 - np.clip(g1, 0, 0.7) / 0.7) ** 0.5
    light = relief_light(dome * 4, 1.0)
    sheen = np.clip((light - 0.6) * 4, 0, 1) ** 2  # a greasy lustre catches the light on each dome
    black = smooth(PITCH, 0.2 + (light - 0.5) * 0.5 + sheen * 0.5 + (fbm(X, Y, 30, 2, seed + 2) - 0.5) * 0.1)
    body = _body(alt).astype(float)
    out = _blend(body, black * _relief(body) ** 0.6, mass)
    rim = _soft(field - 0.44, 0.02) * _soft(0.53 - field, 0.02)  # the yellow crust forms where the pitchblende meets air and water
    crust = rim * _soft(fbm(X, Y, 20, 3, seed + 4) - 0.56, 0.04)
    yellow = smooth(AUTUNITE, 0.3 + (light - 0.5) * 0.8 + (fbm(X, Y, 40, 1, seed + 5) - 0.5) * 0.5)
    out = _blend(out, yellow, crust)
    glow = crust * 0.3
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
    edge = _soft(1 - d, 0.7)
    band = 0.5 + 0.5 * np.sin(np.clip(d, 0, 1.5) * 12 + rough * 5)
    height = np.clip(1 - d, 0, 1) ** 0.5 * 3 + rough * 0.8
    light = relief_light(height, 1.2)
    green = smooth(MALACHITE, 0.36 + 0.25 * np.clip(1 - d, 0, 1) + (band - 0.5) * 0.3 + (light - 0.5) * 0.6
                   + (fbm(X, Y, 32, 2, seed + 6) - 0.5) * 0.15)
    body = _body(alt).astype(float)
    halo = np.clip((3.0 - d) / 2.0, 0, 1) ** 1.3 * (0.25 + 0.45 * fbm(X, Y, 10, 3, seed + 8))
    out = body * (1 - halo[..., None]) + MALACHITE[3].astype(float) * halo[..., None]
    out = _blend(out, green, edge)
    azure = _soft(fbm(X, Y, 8, 3, seed + 7, 0.5, 0.4) - 0.71, 0.05) * _soft(0.7 - d, 0.4)
    out = _blend(out, smooth(AZURITE, 0.25 + (light - 0.5) * 0.7 + (band - 0.5) * 0.2), azure)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    grain = (d < 0.3) & (hash2(ix // 2, iy // 2, seed + 9) > 0.93)
    out = np.where(grain[..., None], smooth(CHALCOPYRITE, 0.35 + (light - 0.5) * 1.2), out)
    out = np.where((line < 0.0035)[..., None], out * 0.45, out)  # the crack itself, a shadow not an outline
    return rgba(np.clip(out, 0, 255).astype(np.uint8))
