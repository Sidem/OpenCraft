"""Wood in the realistic look (docs/ART_STYLE.md section 12): the log's bark and cut end, planks, birch bark, dead
wood, and the planed wood of tool handles and the stick. Structure is procedural, colour comes from CC0 photos
through `real.colour` (materials `bark`, `end_grain`, `planks`, `birch`, `dead_wood`, `planed` in `realism.py`).
Wood is fibrous, so most fields here are stretched along the grain (`streaks`, which wraps with its own period on
each axis). Repeating tiles (bark, planks, birch, dead wood) wrap; the log end is a face. Add a wood: photos in
`realism.MATERIALS`, a recipe here, an entry with `real=True` in `registry.py`.
"""
import numpy as np

from lib import N, blur, cavity, coords, fbm, hash2, relief_light, rgba
from real import colour


def _soft(x, width):
    return np.clip(x / width + 0.5, 0, 1)


def streaks(X, Y, cx, cy, octaves=3, seed=0, gain=0.5):
    """Fractal value noise 0..1 with `cx` cells across and `cy` down (each doubling per octave), wrapping on both axes:
    stretched along the grain when the counts differ."""
    total, amp, norm = 0.0, 1.0, 0.0
    for o in range(octaves):
        px, py = cx * 2 ** o, cy * 2 ** o
        x, y = X * px, Y * py
        x0, y0 = np.floor(x), np.floor(y)
        tx, ty = x - x0, y - y0
        tx, ty = tx * tx * (3 - 2 * tx), ty * ty * (3 - 2 * ty)
        ix0, iy0 = x0.astype(np.int64) % px, y0.astype(np.int64) % py
        ix1, iy1 = (ix0 + 1) % px, (iy0 + 1) % py
        s = seed + o * 7
        a = hash2(ix0, iy0, s) * (1 - tx) + hash2(ix1, iy0, s) * tx
        b = hash2(ix0, iy1, s) * (1 - tx) + hash2(ix1, iy1, s) * tx
        total = total + amp * (a * (1 - ty) + b * ty)
        norm += amp
        amp *= gain
    return total / norm


def cells(X, Y, cx, cy, seed=0):
    """Cellular noise on a `cx` x `cy` grid, wrapping: (gap to the nearest cell border, a 0..1 id per cell), in cell
    units. With `cx` > `cy` the cells are tall, like bark plates."""
    gx, gy = X * cx, Y * cy
    ix, iy = np.floor(gx).astype(np.int64), np.floor(gy).astype(np.int64)
    f1, f2, ident = np.full(gx.shape, 9.0), np.full(gx.shape, 9.0), np.zeros(gx.shape)
    for oy in (-1, 0, 1):
        for ox in (-1, 0, 1):
            kx, ky = (ix + ox) % cx, (iy + oy) % cy
            px = ix + ox + 0.5 + (hash2(kx, ky, seed) - 0.5) * 0.9
            py = iy + oy + 0.5 + (hash2(kx, ky, seed + 5) - 0.5) * 0.9
            d = np.hypot(px - gx, py - gy)
            closer = d < f1
            f2 = np.where(closer, f1, np.minimum(f2, d))
            ident = np.where(closer, hash2(kx, ky, seed + 9), ident)
            f1 = np.where(closer, d, f1)
    return f2 - f1, ident


def bark(seed=0):
    """Rough furrowed bark (oak and pine logs): long corky plates split by deep fissures that join and part again,
    flat-topped and scaly, each plate a little different in tone; tiles."""
    X, Y = coords()
    Xw = (X + (streaks(X, Y, 2, 5, 3, seed + 1) - 0.5) * 0.2) % 1.0  # furrows meander as they run down
    Yw = (Y + (fbm(X, Y, 3, 3, seed + 2) - 0.5) * 0.3) % 1.0
    gap, pid = cells(Xw, Yw, 7, 2, seed + 3)
    gap2, _ = cells(Xw, Yw, 5, 4, seed + 7)  # a second, shorter net breaks the lattice
    gap = np.minimum(gap, gap2 + 0.12 + 0.3 * streaks(X, Y, 4, 4, 2, seed + 8))
    gap = gap + (streaks(X, Y, 24, 6, 3, seed + 4) - 0.5) * 0.35  # ragged fissure walls, some fissures closing
    fissure = _soft(0.08 - gap, 0.12)
    scales = streaks(X, Y, 32, 10, 3, seed + 5)
    h = (1 - fissure) * (0.7 + 0.2 * pid) + scales * 0.3
    light = relief_light(blur(h, 0.8) * 5, 1.0)
    field = (light * 0.2 + (1 - fissure) * 0.6 + pid * 0.05 + scales * 0.25 + fbm(X, Y, 64, 1, seed + 6) * 0.05
             - cavity(h, 2.0) * 1.2)
    return rgba(colour("bark", field, 1.4, seed=seed))


def log_end(seed=0):
    """The sawn end of a log: wobbly growth rings round an off-centre pith (darker heartwood, paler sapwood), a few
    radial drying checks, faint saw marks, and the bark's rim at the edge (a face, not a repeating tile)."""
    X, Y = coords()
    dx, dy = X - 0.47, Y - 0.53
    ang = np.arctan2(dy, dx)
    wob = (fbm(X, Y, 3, 3, seed + 1) - 0.5) * 0.08 + 0.015 * np.sin(ang * 3 + 1.0)
    r = np.hypot(dx, dy) * 2 + wob
    rings = r * 11 + (fbm(X, Y, 8, 2, seed + 2) - 0.5) * 0.3
    late = np.clip(1 - np.abs((rings % 1.0) - 0.85) / 0.15, 0, 1) ** 2  # a soft darker band of latewood per ring
    heart = np.clip(1 - r / 0.8, 0, 1) ** 1.5  # heartwood darkens toward the pith, no sharp boundary
    check = np.zeros_like(X)
    for a0, length in ((0.7, 0.75), (2.9, 0.5)):  # two radial drying checks, widest at the rim
        da = np.abs((ang - a0 + (fbm(X, Y, 6, 2, seed + 3) - 0.5) * 0.25 + np.pi) % (2 * np.pi) - np.pi)
        check = np.maximum(check, _soft(0.005 * (0.4 + r) - da * r, 0.006) * _soft(r - (1 - length), 0.1))
    saw = np.sin((X * 0.6 + Y) * 2 * np.pi * 22 + fbm(X, Y, 6, 2, seed + 4) * 4) * 0.5 + 0.5
    grain = blur(fbm(X, Y, 64, 1, seed + 5), 0.7)
    stain = fbm(X, Y, 3, 3, seed + 8)  # broad patches of colour, so fine detail moves the tone only a little
    field = (0.6 - late * 0.1 - heart * 0.3 + stain * 0.25 + saw * 0.015 + grain * 0.02
             - blur(check, 0.5) * 0.4)
    wood = colour("end_grain", blur(field, 0.5), 0.7, seed=seed).astype(float)  # the fit logs differ in tone
    edge = np.maximum(np.abs(X - 0.5), np.abs(Y - 0.5)) * 2 + (fbm(X, Y, 12, 2, seed + 6) - 0.5) * 0.03
    rim = _soft(edge - 0.955, 0.03)
    cambium = _soft(edge - 0.93, 0.03) * (1 - rim) * 0.25  # a darker line where the bark meets the wood
    bark_rgb = colour("bark", fbm(X, Y, 16, 3, seed + 7), 0.5, seed=seed + 1).astype(float)
    out = wood * (1 - cambium)[..., None]
    out = out * (1 - rim[..., None]) + bark_rgb * rim[..., None]
    return rgba(out.clip(0, 255).astype(np.uint8))


def planks(seed=0):
    """Four sawn boards: long continuous grain with the odd knot, a different tone per board, staggered butt joints,
    soft dark gaps between the boards and a nail head near each joint; tiles."""
    X, Y = coords()
    board = np.floor(Y * 4).astype(int) % 4
    local = (Y * 4) % 1.0
    tone = hash2(board, 0, seed + 1)
    shift = hash2(board, 2, seed + 7)
    grain = streaks((X + shift) % 1.0, Y, 2, 48, 3, seed + 2)
    fine = streaks((X + shift) % 1.0, Y, 4, 128, 2, seed + 3)
    knot_x, knot_y = (hash2(board, 3, seed + 8) * 0.8 + 0.1), (board + 0.5) / 4
    kd = np.hypot((X - knot_x + 0.5) % 1.0 - 0.5, (Y - knot_y) * 2.2)
    knot = (hash2(board, 4, seed + 9) > 0.45) * _soft(0.035 - kd, 0.02)
    flow = np.sin(kd * 260) * _soft(0.09 - kd, 0.06) * 0.08  # grain swirls round a knot
    joint_x = hash2(board, 1, seed + 4)
    jd = np.abs(((X - joint_x + 0.5) % 1.0) - 0.5)
    gap = np.maximum(_soft(0.03 - local, 0.04) + _soft(local - 0.985, 0.03), _soft(0.006 - jd, 0.008))
    nail = np.zeros_like(X)
    for off in (-0.045, 0.045):
        nd = np.hypot((X - joint_x - off + 0.5) % 1.0 - 0.5, (local - 0.5) / 4)
        nail = np.maximum(nail, _soft(0.007 - nd, 0.005))
    field = (tone * 0.25 + grain * 0.4 + fine * 0.15 + flow - knot * 0.5 - blur(gap, 0.5) * 0.6 - nail * 0.5
             + fbm(X, Y, 64, 1, seed + 5) * 0.05)
    return rgba(colour("planks", field, 1.1, seed=seed))


def birch_bark(seed=0):
    """Papery white birch bark: fine horizontal fibres with pale grey peeling bands, thin dark lenticels running across
    the trunk, and a few rough black fissure patches, wide and lens-shaped; tiles."""
    X, Y = coords()
    fibre = streaks(X, Y, 4, 48, 3, seed + 1)
    mottle = streaks(X, Y, 3, 10, 4, seed + 2, 0.6)  # grey patches where the outer paper has peeled
    rng = np.random.default_rng(seed + 3)
    rag = fbm(X, Y, 24, 3, seed + 4) - 0.5
    lent = np.zeros((N, N))
    for _ in range(90):  # lenticels: thin dark dashes across the trunk, some long, most short and faint
        x, y, w = rng.uniform(0, 1), rng.uniform(0, 1), rng.uniform(0.01, 0.07) * rng.choice([1, 1, 2])
        d = np.hypot(((X - x + 0.5) % 1.0 - 0.5) / w, ((Y - y + 0.5) % 1.0 - 0.5) / rng.uniform(0.003, 0.007))
        lent = np.maximum(lent, _soft(1 - d + rag * 0.6, 0.5) * rng.uniform(0.2, 1.0))
    scar = np.zeros((N, N))
    for _ in range(6):  # black fissure patches: rough, wider than tall, each its own ragged shape
        x, y = rng.uniform(0, 1), rng.uniform(0, 1)
        w, hgt, tilt = rng.uniform(0.06, 0.2), rng.uniform(0.015, 0.045), rng.uniform(-0.15, 0.15)
        ux, uy = (X - x + 0.5) % 1.0 - 0.5, (Y - y + 0.5) % 1.0 - 0.5
        d = np.hypot(ux / w, (uy - ux * tilt) / hgt) ** rng.uniform(0.7, 1.3)
        scar = np.maximum(scar, _soft(1 - d + rag * 1.4, 0.35))
    rough = relief_light(fbm(X, Y, 24, 3, seed + 5) * 6, 1.0)
    field = (0.7 + fibre * 0.1 - _soft(mottle - 0.58, 0.25) * 0.12 - lent * 0.5
             - scar * (0.8 + (rough - 0.5) * 0.6))
    return rgba(colour("birch", field, 1.0, seed=seed))


def dead_wood(seed=0):
    """A dead trunk with the bark gone: sun-bleached grey wood, long fibres, drying checks that open along the grain
    and taper out, and a weathered knot; tiles."""
    X, Y = coords()
    warp = (streaks(X, Y, 2, 2, 2, seed + 1) - 0.5) * 0.04
    Xw = (X + warp) % 1.0
    fibre = streaks(Xw, Y, 48, 3, 3, seed + 2)
    tone = streaks(Xw, Y, 6, 2, 2, seed + 3)
    check = np.zeros_like(X)
    rng = np.random.default_rng(seed + 4)
    for _ in range(7):  # drying checks: nearly straight, open in the middle, tapering to nothing at both ends
        x, y, length = rng.uniform(0, 1), rng.uniform(0, 1), rng.uniform(0.15, 0.6)
        uy = (Y - y + 0.5) % 1.0 - 0.5
        along = np.clip(1 - np.abs(uy) / (length / 2), 0, 1)
        dx = np.abs((Xw - x + 0.5) % 1.0 - 0.5 + (fbm(X, Y, 32, 2, seed + 5) - 0.5) * 0.006)
        check = np.maximum(check, _soft(rng.uniform(0.003, 0.008) * along ** 0.5 - dx, 0.004))
    h = fibre * 0.5 + tone * 0.3 - check * 0.6
    light = relief_light(blur(h, 0.6) * 5, 1.0)
    field = light * 0.35 + fibre * 0.35 + tone * 0.25 - blur(check, 0.5) * 0.7
    return rgba(colour("dead_wood", field, 1.1, seed=seed))


def _planed(seed, across):
    """Planed wood for models: straight, fine grain with soft growth lines; `across` turns the grain horizontal."""
    X, Y = coords()
    if across:
        X, Y = Y, X
    phase = X * 24 + (streaks(X, Y, 3, 2, 2, seed) - 0.5) * 1.2 + streaks(X, Y, 12, 1, 2, seed + 3) * 1.5
    lines = np.clip(1 - np.abs(phase % 1.0 - 0.5) / 0.12, 0, 1)  # thin latewood lines at uneven spacing
    fibre = streaks(X, Y, 64, 4, 2, seed + 1)
    field = -lines * 0.35 + fibre * 0.4 + streaks(X, Y, 3, 2, 2, seed + 2) * 0.25
    return rgba(colour("planed", field, 1.0, seed=seed))


def handle(seed=103):
    """A tool handle: planed hardwood, the grain running along the shaft (down the tile)."""
    return _planed(seed, False)


def stick(seed=112):
    """The stick: the same planed wood, the grain running across the tile, as the old stick did."""
    return _planed(seed, True)
