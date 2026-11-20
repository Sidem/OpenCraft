"""Ground cover, indicator plants, saplings and the torch (docs/ART_STYLE.md).

Drawn with `draw.Pen` in plant space (x 0..1 left to right, y 0..1 bottom to top). Ground cover and indicator plants
are painted in continuous tone (`smooth=True`) with no dark rims to match the realistic terrain and trees, and are
meshed as 3D botanical shapes in `crates/engine/src/mesher/cover/shape.rs`:
  - Split 2x2 tiles (`daisy`, `poppy`, `rust_bloom`, `pale_star`, `coal_shrub`): top-left and top-right are two
    upward-facing blossom/foliage variants (centred at (0.25, 0.75) and (0.75, 0.75)), bottom-left is the stem rooted
    at (0.25, 0.0) to (0.25, 0.46), bottom-right is the basal leaf rosette (or second woody branch) rooted at (0.75, 0.0).
  - Full-tile plants: `tuft`, `fern` (one symmetrical frond bent into a 3D rosette), `bluebell` and `copper_bell`
    (arching stem rooted at x = 0.22..0.24 fanning outward at 120°), `reed`, `dry_brush`, `lichen` (top-down rock patch).
Saplings and the torch stay on crossed quads (`Render::Plant`). Greens ship brighter through `grade.py`.
"""
import numpy as np

from draw import Pen, curve
from lib import ramp
from natural import RAMPS
from stains import COAL, RUST, VERDIGRIS

GREEN = RAMPS["grass"]
LEAF = ramp(["#0e1f15", "#1f3e27", "#32622e", "#4e8436", "#74a640", "#a6c75d"])
FERN = ramp(["#0c1e18", "#183c2c", "#2a6038", "#448242", "#6aa24f", "#9ec46a"])
PINE = ramp(["#07141a", "#0e2a2a", "#17463a", "#25664a", "#3a865a", "#62a878"])
BIRCH_LEAF = ramp(["#10201a", "#26452a", "#43703a", "#6e9c48", "#a4c46a", "#d4e098"])
BARK = RAMPS["bark"]
BIRCH = ramp(["#1a1a24", "#4a4a5a", "#8a8a98", "#c8c6c8", "#f0eee8"])
DRY = ramp(["#2a1e16", "#58422d", "#846845", "#af9265", "#d4be92"])
CREAM = ramp(["#565c76", "#8c92aa", "#c2c6d4", "#e0dfd8", "#f0efe8"])
AMBER = ramp(["#68320c", "#ad5e16", "#df9422", "#f5c23e", "#ffe27a"])
CRIMSON = ramp(["#3a0810", "#7d121c", "#be2224", "#e43e2e", "#f76b48"])
POPPY_CORE = ramp(["#140d16", "#2a1e2e", "#4b3b4e", "#766956"])
BLUE = ramp(["#1b2152", "#314394", "#5472cf", "#829ff0", "#b8cbff"])
SLATE = ramp(["#10151e", "#1d2634", "#2c3a4c", "#3f5166", "#576b80", "#768a9e"])
BERRY = ramp(["#080a10", "#141826", "#262e46", "#546284", "#98a8c6"])
LICHEN = ramp(["#364632", "#586d4d", "#7f966a", "#a7ba8a", "#ced9ae"])
FLAME = ramp(["#5a0f10", "#b32030", "#f0662a", "#ffb43c", "#fff2b0"])
BRONZE = ramp(["#1e0f10", "#4a2a1c", "#8a5a2a", "#c8923c", "#f0c86a"])
RUST_PETAL = ramp(["#5c1c0c", "#983614", "#cb541e", "#ea7a32", "#f8a656"])
TEAL_BELL = ramp(["#123a3a", "#206460", "#38948a", "#5ec0b0", "#9ee6d8"])
PALE_PETAL = ramp(["#505674", "#848aa8", "#b4bad2", "#d4d8e8", "#e8eaf4"])
SAGE = ramp(["#1c2e24", "#304a38", "#4a6a52", "#6a8e6e", "#94b492"])
OAK = RAMPS["oak"]


def _quad_stem(pen, leaf_ramp, seed, bow=0.025):
    """Bottom-left quadrant: a crisp stem rooted at (0.25, 0.0) rising to (0.25, 0.47) with two small leaves."""
    rng = np.random.default_rng(seed)
    pts = curve((0.25, 0.0), (0.25, 0.475), bow, n=48)
    pen.stroke(pts, np.linspace(0.024, 0.015, len(pts)), leaf_ramp, 0.48)
    for y, side, ln in ((0.11, -1, 0.14), (0.24, 1, 0.13)):
        x = pts[int(y / 0.475 * (len(pts) - 1)), 0]
        tip = (x + side * ln, y + 0.09 + rng.uniform(-0.01, 0.02))
        pen.leaf((x, y), tip, 0.032, leaf_ramp, 0.56 + 0.05 * side)


def _quad_basal(pen, leaf_ramp, seed, count=6, wide=0.026):
    """Bottom-right quadrant: a low rosette of basal leaves rooted around (0.75, 0.0)."""
    rng = np.random.default_rng(seed)
    for i in range(count):
        f = (i / max(count - 1, 1)) * 2 - 1
        x0 = 0.75 + f * 0.04
        tip = (0.75 + f * 0.19 + rng.uniform(-0.02, 0.02), 0.18 + (1 - abs(f) * 0.45) * 0.18 + rng.uniform(-0.02, 0.03))
        pen.leaf((x0, 0.0), tip, wide * rng.uniform(0.85, 1.15), leaf_ramp, rng.uniform(0.42, 0.66))


def tuft(seed=1):
    rng = np.random.default_rng(seed)
    pen = Pen(smooth=True, seed=101)
    blades = []
    for i in range(16):
        f = i / 15
        x0 = 0.12 + 0.76 * f + rng.uniform(-0.03, 0.03)
        h = 0.36 + 0.54 * rng.uniform(0.35, 1.0) * (1.0 - abs(f - 0.5) * 0.65)
        lean = (f - 0.5) * 0.38 + rng.uniform(-0.10, 0.10)
        bow = -lean * 0.55 + rng.uniform(-0.08, 0.08)
        blades.append((rng.uniform(0.24, 0.70), x0, h, lean, bow))
    for light, x0, h, lean, bow in sorted(blades):
        w = 0.013 + 0.006 * light
        pen.blade((x0, 0.0), (np.clip(x0 + lean, 0.04, 0.96), h), bow, w, GREEN, light, taper=0.92)
    return pen.rgba()


def daisy():
    rng = np.random.default_rng(2)
    pen = Pen(smooth=True, seed=102)
    _quad_stem(pen, LEAF, 20, bow=0.025)
    _quad_basal(pen, LEAF, 21, count=7, wide=0.024)
    # Two top-down daisy heads in the upper quadrants (15 and 13 delicate ray petals around a warm golden button).
    for cx, count, length, width, c_rad in ((0.25, 15, 0.195, 0.024, 0.056), (0.75, 13, 0.175, 0.025, 0.050)):
        c = (cx, 0.75)
        start = rng.uniform(0.0, 0.4)
        for i in range(count):
            a = start + 2 * np.pi * i / count + rng.uniform(-0.05, 0.05)
            ln = length * rng.uniform(0.88, 1.06)
            pen.petal(c, a, ln, width, CREAM, 0.66 + 0.06 * np.sin(a), shape=0.82, vein=0.06)
        pen.ellipse(c, c_rad, c_rad, 0.0, AMBER, 0.62)
        pen.ellipse((cx - 0.006, 0.75 + 0.006), c_rad * 0.62, c_rad * 0.62, 0.0, AMBER, 0.76)
        for _ in range(9):
            a, r = rng.uniform(0, 2 * np.pi), rng.uniform(0.25, 0.85) * c_rad
            pen.ellipse((cx + np.cos(a) * r, 0.75 + np.sin(a) * r), 0.008, 0.008, 0.0, AMBER, 0.28)
    return pen.rgba()


def poppy():
    rng = np.random.default_rng(3)
    pen = Pen(smooth=True, seed=103)
    _quad_stem(pen, LEAF, 30, bow=-0.03)
    _quad_basal(pen, LEAF, 31, count=6, wide=0.022)
    # Two top-down silky crimson poppy cups: broad overlapping ruffled petals round a deep velvety dark centre.
    for cx, n_petals, length, width in ((0.25, 5, 0.195, 0.098), (0.75, 4, 0.185, 0.105)):
        c = (cx, 0.75)
        start = rng.uniform(0.1, 0.9)
        for i in range(n_petals):
            a = start + 2 * np.pi * i / n_petals + rng.uniform(-0.08, 0.08)
            ln = length * rng.uniform(0.92, 1.05)
            pen.petal(c, a, ln, width, CRIMSON, 0.64 + 0.08 * np.sin(a + 0.5), shape=0.62, ruffle=0.09, vein=0.11)
        pen.ellipse(c, 0.044, 0.044, 0.0, POPPY_CORE, 0.18)
        for i in range(8):
            a = i * np.pi / 4 + rng.uniform(-0.15, 0.15)
            pen.ellipse((cx + np.cos(a) * 0.034, 0.75 + np.sin(a) * 0.034), 0.009, 0.009, 0.0, POPPY_CORE, 0.55)
        pen.ellipse(c, 0.018, 0.018, 0.0, LEAF, 0.42)
    return pen.rgba()


def bluebell():
    pen = Pen(smooth=True, seed=104)
    # Rooted at (0.22, 0.0), arching gracefully outward so 3 stems at 120° form a 3D fountain of nodding bells.
    for tip, bow, w, lt in (
        ((0.05, 0.48), 0.18, 0.021, 0.52), ((0.48, 0.36), -0.20, 0.023, 0.46),
        ((0.12, 0.36), 0.12, 0.018, 0.58), ((0.36, 0.44), -0.12, 0.017, 0.54),
    ):
        pen.blade((0.22, 0.0), tip, bow, w, LEAF, lt, taper=0.88)
    pts = curve((0.22, 0.0), (0.86, 0.76), 0.38)
    pen.stroke(pts, np.linspace(0.013, 0.007, len(pts)), LEAF, 0.48)
    for i, t in enumerate((0.44, 0.57, 0.70, 0.82, 0.93)):
        x, y = pts[int(t * (len(pts) - 1))]
        ped = (x + 0.028, y - 0.032)
        pen.blade((x, y), ped, 0.18, 0.006, LEAF, 0.45, taper=0.4)
        scale = 1.04 - 0.08 * i
        pen.bell(ped, 0.145 * scale, 0.060 * scale, -0.18 - 0.04 * i, BLUE, 0.54 + 0.03 * i)
    return pen.rgba()


def fern():
    rng = np.random.default_rng(5)
    pen = Pen(smooth=True, seed=105)
    # One lush, symmetrical woodland fern frond rooted at (0.5, 0.0); meshed as 5 bent 2-segment arching fronds.
    pts = curve((0.5, 0.0), (0.5, 0.95), 0.02, n=80)
    pen.stroke(pts, np.linspace(0.017, 0.005, len(pts)), FERN, 0.42)
    pairs = 15
    for k in range(pairs):
        t = 0.08 + 0.86 * (k / (pairs - 1))
        cx, y = pts[int(t * (len(pts) - 1))]
        span = (0.42 * np.sin(np.pi * t ** 0.72) ** 0.78 + 0.03) * rng.uniform(0.94, 1.05)
        w = 0.038 * (1.0 - 0.52 * t)
        for s in (-1, 1):
            ang = np.pi / 2 - s * (1.24 - 0.40 * t + rng.uniform(-0.04, 0.04))
            tip = (cx + np.cos(ang) * span, y + np.sin(ang) * span * 0.82)
            pen.leaf((cx, y), tip, w, FERN, 0.44 + 0.26 * t + 0.06 * s + rng.uniform(-0.03, 0.03))
    pen.leaf((0.5, 0.90), (0.5, 0.98), 0.024, FERN, 0.72)
    return pen.rgba()


def reed():
    rng = np.random.default_rng(6)
    pen = Pen(smooth=True, seed=106)
    # Slender riverside sedge blades and two clean cattail stalks (seed heads centred directly on their stalks).
    for x0, lean, h, w in (
        (0.24, -0.16, 0.58, 0.017), (0.36, 0.16, 0.68, 0.018), (0.50, -0.10, 0.74, 0.019),
        (0.64, 0.18, 0.64, 0.017), (0.76, -0.14, 0.52, 0.016), (0.44, 0.06, 0.62, 0.015),
    ):
        pen.blade((x0, 0.0), (x0 + lean, h), -lean * 0.35, w, LEAF, rng.uniform(0.38, 0.64), taper=0.92)
    for x, top, lean in ((0.38, 0.90, -0.02), (0.62, 0.78, 0.02)):
        pts = curve((x, 0.0), (x + lean, top + 0.06), 0.02, n=60)
        pen.stroke(pts, np.linspace(0.012, 0.006, len(pts)), LEAF, 0.48)
        hx, hy = pts[int(0.84 * (len(pts) - 1))]
        pen.ellipse((hx, hy), 0.031, 0.095, -lean * 0.8, DRY, 0.26)
        pen.ellipse((hx - 0.006, hy + 0.01), 0.016, 0.070, -lean * 0.8, DRY, 0.40)
    return pen.rgba()


def dry_brush():
    rng = np.random.default_rng(7)
    pen = Pen(smooth=True, seed=107)

    def branch(p, ang, length, w, depth):
        end = (np.clip(p[0] + np.cos(ang) * length, 0.06, 0.94), np.clip(p[1] + np.sin(ang) * length, 0.02, 0.92))
        pen.blade(p, end, rng.uniform(-0.14, 0.14), w, DRY, rng.uniform(0.36, 0.68), taper=0.55)
        if depth:
            for f in (0.44, 0.80):
                q = (p[0] + (end[0] - p[0]) * f, p[1] + (end[1] - p[1]) * f)
                branch(q, ang + rng.choice((-1, 1)) * rng.uniform(0.32, 0.72), length * 0.64, w * 0.64, depth - 1)
        elif rng.uniform() < 0.45:
            pen.ellipse(end, 0.017, 0.011, ang, SAGE, rng.uniform(0.35, 0.62))

    for a in (np.pi / 2 - 0.46, np.pi / 2 + 0.42, np.pi / 2 - 0.03):
        branch((0.5, 0.0), a, 0.42, 0.016, 3)
    return pen.rgba()


def lichen():
    rng = np.random.default_rng(8)
    pen = Pen(smooth=True, seed=108)
    # Top-down organic foliose rock lichen patch lying flat on stone/basalt, with tiny golden fruiting cups.
    for i, (cx, cy, rx, ry, lt) in enumerate((
        (0.48, 0.52, 0.30, 0.24, 0.46), (0.34, 0.42, 0.22, 0.18, 0.56), (0.64, 0.58, 0.22, 0.17, 0.52),
        (0.56, 0.36, 0.18, 0.15, 0.60), (0.38, 0.66, 0.17, 0.14, 0.64), (0.72, 0.42, 0.13, 0.11, 0.58),
    )):
        pen.blob((cx, cy), rx, ry, 80 + i, LICHEN, lt, wobble=0.28)
    for _ in range(14):
        a, r = rng.uniform(0, 2 * np.pi), rng.uniform(0.05, 0.28)
        cx, cy = 0.50 + np.cos(a) * r, 0.50 + np.sin(a) * r * 0.85
        pen.ellipse((cx, cy), 0.019, 0.017, 0.0, AMBER, rng.uniform(0.45, 0.72))
        pen.ellipse((cx, cy), 0.009, 0.008, 0.0, AMBER, 0.25)
    return pen.rgba()


def _sapling(trunk, leaf, seed, count, size, spread, top=0.78):
    rng = np.random.default_rng(seed)
    pen = Pen()
    pen.blade((0.5, 0.0), (0.5 + rng.uniform(-0.04, 0.04), top), 0.04, 0.026, trunk, 0.45, 0.55)
    for i in range(count):
        t = rng.uniform(0.25, 1.0)
        side = rng.choice((-1, 1))
        base = (0.5 + 0.02 * np.sin(t * 3), t * top)
        tip = (base[0] + side * spread * rng.uniform(0.5, 1), base[1] + size * rng.uniform(0.4, 1.0))
        pen.blade(base, tip, 0.1 * side, 0.009, trunk, 0.5, 0.3)
        pen.leaf(tip, (tip[0] + side * size * 0.9, tip[1] + size * 0.6), size * 0.38, leaf, rng.uniform(0.4, 0.8))
    pen.leaf((0.5, top), (0.52, top + 0.18), 0.045, leaf, 0.7)
    return pen.rgba()


def sapling():
    return _sapling(BARK, LEAF, 10, 13, 0.1, 0.14)


def birch_sapling():
    return _sapling(BIRCH, BIRCH_LEAF, 11, 14, 0.09, 0.12)


def pine_sapling():
    rng = np.random.default_rng(12)
    pen = Pen()
    pen.blade((0.5, 0.0), (0.5, 0.82), 0.01, 0.022, BARK, 0.4, 0.4)
    for i in range(6):
        y = 0.12 + i * 0.115
        reach = 0.3 * (1 - i / 7)
        for s in (-1, 1):
            for j in range(3):
                pen.blade((0.5, y + 0.02), (0.5 + s * reach * (1 - j * 0.22), y - 0.07 + j * 0.01), s * 0.12, 0.017, PINE, 0.35 + 0.13 * j + 0.05 * s, 0.7)
    pen.leaf((0.5, 0.78), (0.5, 0.96), 0.03, PINE, 0.6)
    return pen.rgba()


def rust_bloom():
    rng = np.random.default_rng(13)
    pen = Pen(smooth=True, seed=113)
    _quad_stem(pen, LEAF, 40, bow=0.02)
    _quad_basal(pen, LEAF, 41, count=7, wide=0.028)
    # Iron indicator: two vivid 6-petal burnt-orange/rust star blossoms with bright amber hearts.
    for cx, length, width in ((0.25, 0.20, 0.048), (0.75, 0.185, 0.046)):
        c = (cx, 0.75)
        start = rng.uniform(0.0, 0.5)
        for i in range(6):
            a = start + 2 * np.pi * i / 6 + rng.uniform(-0.04, 0.04)
            pen.petal(c, a, length * rng.uniform(0.92, 1.05), width, RUST_PETAL, 0.64 + 0.08 * np.sin(a), shape=0.92, vein=0.10)
        pen.ellipse(c, 0.048, 0.048, 0.0, AMBER, 0.65)
        pen.ellipse(c, 0.024, 0.024, 0.0, RUST_PETAL, 0.20)
    return pen.rgba()


def coal_shrub():
    rng = np.random.default_rng(14)
    pen = Pen(smooth=True, seed=114)
    # Coal indicator: dark woody branching stems (bottom) and lush dark charcoal-slate foliage sprigs with glossy black berries (top).
    for ox, seed in ((0.25, 50), (0.75, 51)):
        r = np.random.default_rng(seed)
        pen.blade((ox, 0.0), (ox + r.uniform(-0.02, 0.02), 0.42), r.uniform(-0.06, 0.06), 0.020, BARK, 0.28, taper=0.5)
        for side, y, ln in ((-1, 0.14, 0.16), (1, 0.20, 0.17), (-1, 0.28, 0.14), (1, 0.31, 0.13)):
            end = (ox + side * ln, y + 0.11 + r.uniform(-0.02, 0.02))
            pen.blade((ox, y), end, side * 0.08, 0.011, BARK, 0.32, taper=0.6)
    for cx in (0.25, 0.75):
        leaves = []
        for _ in range(44):
            a = rng.uniform(0, 2 * np.pi)
            d = rng.uniform(0.01, 0.13)
            base = (cx + np.cos(a) * d * 0.75, 0.75 + np.sin(a) * d * 0.75)
            ln = rng.uniform(0.085, 0.135)
            la = a + rng.uniform(-0.42, 0.42)
            tip = (np.clip(base[0] + np.cos(la) * ln, cx - 0.215, cx + 0.215),
                   np.clip(base[1] + np.sin(la) * ln, 0.535, 0.965))
            leaves.append((rng.uniform(0.24, 0.78), base, tip, rng.uniform(0.028, 0.042)))
        for lt, base, tip, w in sorted(leaves):
            pen.leaf(base, tip, w, SLATE, lt)
        for _ in range(9):
            a, d = rng.uniform(0, 2 * np.pi), rng.uniform(0.03, 0.15)
            bx, by = cx + np.cos(a) * d, 0.75 + np.sin(a) * d
            pen.ellipse((bx, by), 0.024, 0.024, 0.0, BERRY, 0.22)
            pen.ellipse((bx - 0.006, by + 0.007), 0.008, 0.007, 0.0, BERRY, 0.94)
    return pen.rgba()


def copper_bell():
    pen = Pen(smooth=True, seed=115)
    # Copper indicator: graceful upright-arching spire rooted at (0.24, 0.0) hung with vivid verdigris-teal bells.
    for tip, bow, w, lt in (
        ((0.06, 0.44), 0.16, 0.022, 0.48), ((0.48, 0.34), -0.16, 0.022, 0.44), ((0.16, 0.54), 0.08, 0.018, 0.55),
    ):
        pen.blade((0.24, 0.0), tip, bow, w, SAGE, lt, taper=0.88)
    pts = curve((0.24, 0.0), (0.78, 0.88), 0.26)
    pen.stroke(pts, np.linspace(0.013, 0.006, len(pts)), SAGE, 0.50)
    # Five well-spaced nodding bells hanging cleanly along the underside/flank of the arching stem.
    for i, t in enumerate((0.40, 0.53, 0.66, 0.79, 0.91)):
        x, y = pts[int(t * (len(pts) - 1))]
        ped = (x + 0.038, y - 0.028)
        pen.blade((x, y), ped, 0.16, 0.006, SAGE, 0.48, taper=0.4)
        scale = 1.05 - 0.08 * i
        pen.bell(ped, 0.145 * scale, 0.056 * scale, -0.14 - 0.03 * i, TEAL_BELL, 0.58 + 0.04 * i)
    return pen.rgba()


def pale_star():
    rng = np.random.default_rng(16)
    pen = Pen(smooth=True, seed=116)
    _quad_stem(pen, SAGE, 60, bow=-0.02)
    _quad_basal(pen, SAGE, 61, count=6, wide=0.020)
    # Limestone/quartz indicator: two crisp 5-pointed pale-lavender-white star blossoms with pale gold centres.
    for cx, length, width in ((0.25, 0.205, 0.042), (0.75, 0.19, 0.040)):
        c = (cx, 0.75)
        start = np.pi / 2 + rng.uniform(-0.18, 0.18)
        for i in range(5):
            a = start + 2 * np.pi * i / 5
            pen.petal(c, a, length * rng.uniform(0.95, 1.04), width, PALE_PETAL, 0.68 + 0.06 * np.sin(a), shape=1.25, vein=0.09)
        pen.ellipse(c, 0.036, 0.036, 0.0, AMBER, 0.78)
        pen.ellipse(c, 0.016, 0.016, 0.0, AMBER, 0.45)
    return pen.rgba()


def torch():
    pen = Pen(glow=True)
    pen.stroke(curve((0.5, 0.0), (0.5, 0.5), 0.0), 0.048, OAK, 0.35)
    pen.ellipse((0.5, 0.5), 0.085, 0.045, 0.0, BRONZE, 0.55)
    pen.ellipse((0.5, 0.45), 0.075, 0.02, 0.0, BRONZE, 0.25)
    pen.ellipse((0.5, 0.55), 0.06, 0.03, 0.0, COAL, 0.25)
    for r, h, tone in ((0.085, 0.3, 0.25), (0.062, 0.24, 0.5), (0.04, 0.17, 0.78), (0.022, 0.1, 1.0)):
        pen.ellipse((0.5, 0.57 + h * 0.5), r, h * 0.55, 0.0, FLAME, tone, glow=1.0)
    return pen.rgba()
