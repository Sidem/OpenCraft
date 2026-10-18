"""The other rocks in the realistic look (docs/ART_STYLE.md section 12): granite, sandstone, basalt, limestone, bedrock,
spent rock and quartz ore. Structure is procedural, colour comes from photos through `real.colour`; each rock keeps its
own structure so they read apart without colour: granite a crystal mosaic, sandstone soft cross-bedded strata, basalt
dark rock with column joints and gas vesicles, limestone pale beds split by joints, bedrock massive black plates with
deep fractures, spent rock bleached and pitted stone with rust seams, quartz ore a milky vein with a pocket of crystal\npoints (specimen colours, MILKY and CLEAR). All wrap at the edge. The ramps stay for the cut
stone in `decor.py`. Add a rock: photos in `realism.MATERIALS`, a recipe here, an entry with `real=True`.
"""
import numpy as np

from lib import blur, cavity, coords, fbm, hash2, ramp, relief_light, rgba, smooth, worley
import real
from real import colour
from stains import OXIDE

GRANITE = ramp(["#141824", "#2f2a38", "#574649", "#836359", "#ab8672", "#d0ab93", "#ecd3bc"])
SANDSTONE = ramp(["#1c1624", "#44283a", "#74423f", "#a3604a", "#cc8858", "#e9b27c", "#f9d9a2"])
BASALT = ramp(["#070a12", "#0d1320", "#16202f", "#223045", "#344760", "#4d6482", "#7690ab"])
MILKY = ramp(["#56585c", "#86888b", "#b2b3b3", "#d2d1cc", "#e6e3dc", "#f4f2ec"])  # vein quartz, milky with fluid inclusions
CLEAR = ramp(["#33393f", "#5d666c", "#959da2", "#c9cfd1", "#eef1f1", "#ffffff"])  # rock crystal points, grey glass


def _soft(x, width):
    return np.clip(x / width + 0.5, 0, 1)


def _crystals(X, Y, cells, seed):
    """Scattered points on a wrapping grid: distance to the nearest and second nearest (cell units), the offset from
    the nearest point to the texel, and a 0..1 id per point."""
    gx, gy = X * cells, Y * cells
    ix, iy = np.floor(gx).astype(np.int64), np.floor(gy).astype(np.int64)
    f1, f2 = np.full(gx.shape, 9.0), np.full(gx.shape, 9.0)
    ox_, oy_, ident = np.zeros(gx.shape), np.zeros(gx.shape), np.zeros(gx.shape)
    for oy in (-1, 0, 1):
        for ox in (-1, 0, 1):
            kx, ky = (ix + ox) % cells, (iy + oy) % cells
            px = ix + ox + 0.5 + (hash2(kx, ky, seed) - 0.5) * 0.8
            py = iy + oy + 0.5 + (hash2(kx, ky, seed + 5) - 0.5) * 0.8
            d = np.hypot(gx - px, gy - py)
            closer = d < f1
            f2 = np.where(closer, f1, np.minimum(f2, d))
            ox_, oy_ = np.where(closer, gx - px, ox_), np.where(closer, gy - py, oy_)
            ident = np.where(closer, hash2(kx, ky, seed + 9), ident)
            f1 = np.where(closer, d, f1)
    return f1, f2, ox_, oy_, ident


def granite(seed=0):
    """Coarse-grained plutonic rock: interlocking crystals, each one mineral (pale feldspar, grey glassy quartz, black
    mica) with a little cloudiness; a sawn-flat face, so no relief per grain and no outlines, grains meet edge to edge."""
    X, Y = coords()
    Xw = (X + (fbm(X, Y, 16, 2, seed + 4) - 0.5) * 0.03) % 1.0  # irregular grains, not Voronoi polygons
    Yw = (Y + (fbm(X, Y, 16, 2, seed + 5) - 0.5) * 0.03) % 1.0
    _, _, pid = worley(Xw, Yw, 30, seed + 1, 1.0)
    _, _, fine = worley(Xw, Yw, 60, seed + 6, 1.0)
    mineral = np.where(pid < 0.12, 0.04, np.where(pid < 0.42, 0.4, 0.72))  # mica, quartz, feldspar
    mineral = np.where(fine < 0.08, 0.06, mineral)  # small mica flakes between the big grains
    field = (mineral + (pid * 7 % 1 - 0.5) * 0.14 + (fbm(X, Y, 64, 1, seed + 2) - 0.5) * 0.12
             + (fbm(X, Y, 4, 3, seed + 3) - 0.5) * 0.1)
    return rgba(colour("granite", blur(field, 0.4), 1.0, seed=seed))


def sandstone(seed=0):
    """Red-brown sandstone: soft strata of slightly different tone, faint cross-bedding inside them, sandy grain, and
    shallow weathered hollows."""
    X, Y = coords()
    pos = Y * 6 + (fbm(X, Y, 3, 3, seed + 1) - 0.5) * 1.2
    layer, local = np.floor(pos).astype(int) % 6, pos % 1.0
    tone = hash2(layer, 0, seed + 2)
    cross = np.sin((X * 4 + local * 0.8 + layer * 0.37) * 2 * np.pi * 3) * 0.5 + 0.5
    seam = blur(_soft(0.06 - local, 0.05), 0.8)
    hollow = fbm(X, Y, 5, 4, seed + 3, 0.5, 0.3)
    light = relief_light(blur(hollow, 1.0) * 6 - seam * 0.5, 1.0)
    grain = fbm(X, Y, 64, 1, seed + 4)
    field = tone * 0.3 + cross * 0.06 + light * 0.35 + grain * 0.22 + hollow * 0.15 - seam * 0.15
    return rgba(colour("sandstone", field, 1.2, seed=seed))


def basalt(seed=0):
    """Dark, fine-grained lava rock: a rough, finely hackly surface, a long cooling joint here and there, and scattered
    small gas vesicles; no plates, so it reads apart from bedrock."""
    X, Y = coords()
    rough = fbm(X, Y, 8, 5, seed + 2, 0.55)
    hackly = 1 - np.abs(fbm(X, Y, 16, 3, seed + 5) - 0.5) * 2
    light = relief_light(rough * 5 + hackly ** 2 * 1.2, 1.0)
    joint = np.abs(fbm(X, Y, 2, 3, seed + 6, 0.4, 0.15) - 0.5)  # a long, nearly straight cooling joint
    joint = _soft(0.006 - joint, 0.006) * _soft(fbm(X, Y, 3, 2, seed + 9) - 0.45, 0.1)
    g1, _, gid = worley(X, Y, 22, seed + 3, 1.0)
    vesicle = (gid > 0.85) * np.clip(1 - g1 / 0.3, 0, 1) ** 0.7
    field = (light * 0.5 + rough * 0.25 + fbm(X, Y, 64, 1, seed + 4) * 0.15 - blur(joint, 0.6) * 0.5
             - blur(vesicle, 0.5) * 0.45)
    return rgba(colour("basalt", field, 1.2, seed=seed))


def limestone(seed=0):
    """Pale bedded limestone: beds of slightly different tone with darker seams that open and close, a few long
    fractures across the beds, and a weathered, finely pitted skin."""
    X, Y = coords()
    pos = Y * 4 + (fbm(X, Y, 3, 3, seed + 1) - 0.5) * 0.8
    layer, local = np.floor(pos).astype(int) % 4, pos % 1.0
    tone = hash2(layer, 0, seed + 2)
    seam = _soft(0.03 - local, 0.035) * _soft(fbm(X, Y, 3, 3, seed + 3) - 0.45, 0.15)
    crack = np.abs(fbm(X, Y, 2, 4, seed + 4, 0.5, 0.5) - 0.5)
    crack = _soft(0.008 - crack, 0.008) * _soft(fbm(X, Y, 4, 2, seed + 5) - 0.5, 0.1)
    skin = fbm(X, Y, 6, 4, seed + 7)
    pit = _soft(fbm(X, Y, 40, 2, seed + 8) - 0.74, 0.06) * _soft(fbm(X, Y, 4, 2, seed + 9) - 0.45, 0.2)
    light = relief_light(skin * 4 - blur(pit, 0.5) * 0.4, 1.0)
    field = (tone * 0.15 + light * 0.4 + skin * 0.2 + fbm(X, Y, 48, 1, seed + 10) * 0.12
             - blur(seam + crack, 0.6) * 0.4 - blur(pit, 0.5) * 0.15)
    return rgba(colour("limestone", field, 1.2, seed=seed))


def bedrock(seed=5):
    """The floor of the world: massive near-black rock under great pressure, crushed into a jumble of big angular shards,
    darker in the gaps; nothing a player can dig. The basalt photos' colours, a shade darker."""
    X, Y = coords()
    Xw = (X + (fbm(X, Y, 4, 3, seed + 11) - 0.5) * 0.15) % 1.0
    Yw = (Y + (fbm(X, Y, 4, 3, seed + 12) - 0.5) * 0.15) % 1.0
    h = np.zeros_like(X)
    for cells, amp in ((3, 0.6), (6, 0.3)):  # shards of two sizes, each a tilted flat facet
        f1, f2, pid = worley(Xw, Yw, cells, seed + cells, 1.0)
        tilt = ((hash2((pid * 997).astype(int), 0, seed + 1) - 0.5) * Xw + (pid - 0.5) * Yw) * 0.6
        h = h + amp * (tilt * cells * 0.6 - _soft(0.04 - (f2 - f1), 0.04) * 0.15)
    light = relief_light(blur(h, 1.2) * 3, 1.0)  # blurred, so a step between shards is a soft edge, not a lit line
    gap = cavity(h, 2.0)
    field = light * 0.55 + fbm(X, Y, 4, 3, seed + 2) * 0.25 + fbm(X, Y, 48, 2, seed + 3) * 0.1 - gap * 1.5
    return rgba(colour("basalt", field, 1.2, seed=seed, lift=-0.01))


def spent_rock(seed=9):
    """What a worked-out deposit leaves: the stone bleached pale and dusty, pitted where the ore was taken, with rust
    stains along old seams."""
    X, Y = coords()
    rock = real.stone(seed, lift=0.13, chroma=0.5)[..., :3].astype(float)
    pit = _soft(fbm(X, Y, 24, 2, seed + 1) - 0.7, 0.05) * _soft(fbm(X, Y, 4, 2, seed + 7) - 0.4, 0.1)
    rock = rock * (1 - 0.4 * blur(pit, 0.6))[..., None]
    seam = np.abs(fbm(X, Y, 3, 4, seed + 2, 0.5, 0.4) - 0.5)
    rust = _soft(0.03 - seam, 0.05) * (0.2 + 0.3 * fbm(X, Y, 12, 3, seed + 3))
    rel = rock.mean(-1, keepdims=True) / rock.mean()
    rock = rock * (1 - rust[..., None]) + OXIDE[1].astype(float) * rel * rust[..., None]
    return rgba(rock.clip(0, 255).astype(np.uint8))


def quartz_ore(alt=0):
    """A milky quartz vein through the stone: sugary white quartz with grey translucent patches and hairline fractures,
    a pocket (druse) of small glassy crystal tips, each a six-faceted point lit from the top left, and a faint iron
    stain along the contact. Alternates turn the vein (across, down, the two diagonals; all wrap)."""
    X, Y = coords()
    seed = 130 + alt * 29
    body = real.stone(0)[..., :3].astype(float)
    lum = body.mean(-1)
    rel = (lum / lum.mean())[..., None]
    run = (Y, X, (X + Y) % 1.0, (X - Y) % 1.0)[alt]
    along = (X, Y, (X - Y) % 1.0, (X + Y) % 1.0)[alt]
    centre = run + (fbm(X, Y, 2, 3, seed) - 0.5) * 0.25
    dist = np.abs(centre % 1.0 - 0.5)  # 0 on the vein, which runs through the middle of the block
    pinch = _soft(fbm(along, np.full_like(X, 0.3), 2, 2, seed + 10) - 0.32, 0.3)  # the vein swells and pinches out
    thick = (0.03 + 0.1 * fbm(X, Y, 3, 2, seed + 1)) * pinch - 0.005
    rough = fbm(X, Y, 20, 3, seed + 2) - 0.5
    vein = _soft(thick - dist + rough * 0.025, 0.012)
    sugar = fbm(X, Y, 48, 2, seed + 3)
    cloud = fbm(X, Y, 6, 3, seed + 4)
    crack = np.abs(fbm(X, Y, 5, 3, seed + 5, 0.5, 0.3) - 0.5)
    crack = _soft(0.006 - crack, 0.006) * _soft(fbm(X, Y, 4, 2, seed + 6) - 0.45, 0.1)
    milky = smooth(MILKY, 0.62 + (sugar - 0.5) * 0.35 - _soft(cloud - 0.6, 0.15) * 0.3 - blur(crack, 0.5) * 0.35)
    out = body * (1 - vein[..., None]) + milky * rel ** 0.3 * vein[..., None]
    # the druse: crystal points crowd a pocket in the middle of the vein
    pocket = _soft(fbm(along, np.full_like(X, 0.5), 3, 2, seed + 7) - 0.58, 0.1) * _soft(thick * 0.8 - dist, 0.02)
    f1, f2, dx, dy, pid = _crystals(X, Y, 13, seed + 8)
    turn = pid * 2 * np.pi  # each point is a six-sided pyramid seen from above, turned at random
    facet = np.floor((np.arctan2(dy, dx) - turn) / (np.pi / 3))
    normal = (facet + 0.5) * np.pi / 3 + turn
    lit = 0.5 + 0.5 * np.cos(normal + 3 * np.pi / 4)  # light from the top left: facing (-1, -1) is lit
    glass = smooth(CLEAR, np.clip(0.1 + lit * 0.8 - _soft(0.1 - (f2 - f1), 0.1) * 0.3, 0, 1))
    cryst = pocket * _soft(0.6 - f1, 0.15)
    out = out * (1 - cryst[..., None]) + glass * cryst[..., None]
    fade = np.clip(1 - (dist - thick) / 0.12, 0, 1) ** 2  # iron stain bleeds from the contact in a few patches
    stain = blur(fade * (1 - vein) * _soft(fbm(X, Y, 6, 3, seed + 9) - 0.55, 0.2), 1.5) * 0.3
    out = out * (1 - stain[..., None]) + OXIDE[1].astype(float) * rel * stain[..., None]
    return rgba(out.clip(0, 255).astype(np.uint8))
