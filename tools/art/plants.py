"""Plants on crossed transparent quads (docs/ART_STYLE.md): ground cover, saplings, the indicator plants over stained
ground, and the torch. Drawn with `draw.Pen` in plant space (0..1, standing on y = 0); each recipe is seeded, so a
rebuild gives the same plant. Colours stay muted and cool-shadowed so a meadow reads as one place; flowers are the
accents. To add one: a function returning `pen.rgba()` and a row in `registry.py` (`cutout=True`).
"""
import numpy as np

from draw import Pen, curve
from lib import ramp
from natural import RAMPS
from stains import COAL, RUST, VERDIGRIS

GREEN = RAMPS["grass"]
LEAF = ramp(["#0c1a14", "#1c3a26", "#2f5e2c", "#4a7f34", "#6f9f3c", "#a8c658"])
FERN = ramp(["#0a1a1a", "#17382f", "#2a5e3c", "#417f42", "#66a050", "#a0c872"])
PINE = ramp(["#07141a", "#0e2a2a", "#17463a", "#25664a", "#3a865a", "#62a878"])
BIRCH_LEAF = ramp(["#10201a", "#26452a", "#43703a", "#6e9c48", "#a4c46a", "#d4e098"])
BARK = RAMPS["bark"]
BIRCH = ramp(["#1a1a24", "#4a4a5a", "#8a8a98", "#c8c6c8", "#f0eee8"])
DRY = ramp(["#1e1210", "#4a2e22", "#7a5632", "#a98348", "#d3b36d"])
CREAM = ramp(["#3c3f5c", "#8b8fb0", "#cfd2e6", "#f4f2ee"])
AMBER = ramp(["#6b2f10", "#b8651a", "#e9a22c", "#ffd96a"])
CRIMSON = ramp(["#2b0a14", "#6b1226", "#b32030", "#e84a3a", "#ff8d63"])
BLUE = ramp(["#14173f", "#2a3f8f", "#5a7fe0", "#a9c4ff"])
SLATE = ramp(["#0a0d16", "#1a2030", "#2c354a", "#434f68", "#62718e", "#8d9db8"])
LICHEN = ramp(["#2a3a24", "#5a7a48", "#8fb068", "#c4d890", "#ecf0b8"])
FLAME = ramp(["#5a0f10", "#b32030", "#f0662a", "#ffb43c", "#fff2b0"])
BRONZE = ramp(["#1e0f10", "#4a2a1c", "#8a5a2a", "#c8923c", "#f0c86a"])
OAK = RAMPS["oak"]


def tuft(seed=1):
    rng = np.random.default_rng(seed)
    pen = Pen()
    blades = []
    for i in range(15):
        f = i / 14
        x0 = 0.1 + 0.8 * f + rng.uniform(-0.03, 0.03)
        h = (0.22 + 0.4 * rng.uniform() * (1 - abs(f - 0.5) * 1.1))
        blades.append((rng.uniform(0.12, 0.55), x0, h, rng.uniform(-0.12, 0.12), rng.uniform(-0.1, 0.1)))
    for light, x0, h, lean, bow in sorted(blades):
        pen.blade((x0, 0.0), (x0 + lean, h), bow, 0.024 + 0.012 * light, GREEN, light)
    return pen.rgba()


def _stem(pen, top, bow, rng, leaves=2):
    pen.blade((0.5, 0.0), top, bow, 0.017, LEAF, 0.45, 0.3)
    for i in range(leaves):
        side = -1 if i % 2 == 0 else 1
        y = 0.12 + 0.12 * i
        pen.leaf((0.5 + (top[0] - 0.5) * y / top[1], y), (0.5 + side * 0.17, y + 0.12 + 0.03 * i), 0.034, LEAF, 0.5)


def daisy():
    rng = np.random.default_rng(2)
    pen = Pen()
    _stem(pen, (0.55, 0.62), 0.05, rng)
    pen.petals((0.55, 0.66), 14, 0.2, 0.026, CREAM, 0.65)
    pen.ellipse((0.55, 0.66), 0.058, 0.058, 0, AMBER, 0.55)
    pen.ellipse((0.55, 0.66), 0.026, 0.026, 0, AMBER, 0.2)
    return pen.rgba()


def poppy():
    rng = np.random.default_rng(3)
    pen = Pen()
    _stem(pen, (0.5, 0.56), -0.06, rng)
    for dx, dy, rot, r in ((-0.075, 0.0, 0.5, 0.085), (0.075, 0.0, -0.5, 0.085), (0.0, 0.06, 0.0, 0.08), (0.0, -0.05, 0.0, 0.08)):
        pen.ellipse((0.5 + dx, 0.62 + dy), r, r * 0.92, rot, CRIMSON, 0.58)
    pen.ellipse((0.5, 0.62), 0.026, 0.026, 0, CRIMSON, 0.0)
    return pen.rgba()


def bluebell():
    pen = Pen()
    for base, tip in (((0.3, 0.0), (0.12, 0.42)), ((0.4, 0.0), (0.66, 0.32))):
        pen.leaf(base, tip, 0.03, LEAF, 0.5)
    pts = curve((0.4, 0.0), (0.8, 0.58), 0.35)
    pen.stroke(pts, 0.012, LEAF, 0.45)
    for t in (0.55, 0.68, 0.8, 0.92):
        x, y = pts[int(t * 79)]
        pen.ellipse((x + 0.015, y - 0.04), 0.036, 0.055, 0.15, BLUE, 0.55)
        pen.ellipse((x + 0.018, y - 0.09), 0.048, 0.02, 0.0, BLUE, 0.7)
    return pen.rgba()


def fern():
    rng = np.random.default_rng(5)
    pen = Pen()
    fronds = [(-0.42, 0.55), (-0.28, 0.66), (-0.12, 0.74), (0.04, 0.8), (0.2, 0.7), (0.34, 0.62), (0.46, 0.5)]
    for k, (dx, h) in enumerate(sorted(fronds, key=lambda f: -abs(f[0]))):
        pts = curve((0.5, 0.0), (0.5 + dx * 0.75, h * 0.95), -dx * 0.5 if abs(dx) > 0.3 else dx * 0.15)
        pen.stroke(pts, 0.008, LEAF, 0.4)
        for i in range(10, 78, 4):
            x, y = pts[i]
            t = i / 79
            ang = np.arctan2(*(pts[i + 1] - pts[i])[::-1])
            ln = 0.12 * (1 - t) ** 0.6 + 0.02
            for s in (-1, 1):
                a = ang + s * 1.0
                pen.leaf((x, y), (x + np.cos(a) * ln, y + np.sin(a) * ln * 0.9), 0.016, FERN, 0.4 + 0.3 * t + 0.1 * s)
    return pen.rgba()


def reed():
    rng = np.random.default_rng(6)
    pen = Pen()
    for x0, lean, h in ((0.3, -0.18, 0.5), (0.4, 0.2, 0.62), (0.62, -0.2, 0.55), (0.72, 0.22, 0.46), (0.5, 0.05, 0.7)):
        pen.blade((x0, 0.0), (x0 + lean, h), rng.uniform(-0.1, 0.1), 0.026, LEAF, rng.uniform(0.4, 0.65), 0.95)
    for x, top in ((0.36, 0.92), (0.55, 0.8)):
        pen.blade((x, 0.0), (x + 0.02, top), 0.03, 0.011, LEAF, 0.5, 0.2)
        pen.ellipse((x + 0.02, top - 0.1), 0.032, 0.095, 0.05, DRY, 0.4)
        pen.blade((x + 0.02, top - 0.01), (x + 0.022, top + 0.07), 0.0, 0.006, LEAF, 0.6, 0.8)
    return pen.rgba()


def dry_brush():
    rng = np.random.default_rng(7)
    pen = Pen()

    def branch(p, ang, length, w, depth):
        end = (p[0] + np.cos(ang) * length, p[1] + np.sin(ang) * length)
        pen.blade(p, end, rng.uniform(-0.15, 0.15), w, DRY, rng.uniform(0.3, 0.6), 0.6)
        if depth:
            for f in (0.45, 0.8):
                q = (p[0] + (end[0] - p[0]) * f, p[1] + (end[1] - p[1]) * f)
                branch(q, ang + rng.choice((-1, 1)) * rng.uniform(0.35, 0.8), length * 0.6, w * 0.6, depth - 1)

    for a in (np.pi / 2 - 0.5, np.pi / 2 + 0.45, np.pi / 2 - 0.05):
        branch((0.5, 0.0), a, 0.34, 0.018, 3)
    return pen.rgba()


def lichen():
    rng = np.random.default_rng(8)
    pen = Pen()
    for _ in range(11):
        pen.ellipse((rng.uniform(0.12, 0.88), rng.uniform(0.05, 0.13)), rng.uniform(0.07, 0.12), rng.uniform(0.04, 0.07), 0.0, LICHEN, rng.uniform(0.35, 0.7))
    for _ in range(9):
        pen.ellipse((rng.uniform(0.15, 0.85), rng.uniform(0.06, 0.15)), 0.014, 0.014, 0.0, AMBER, 0.65)
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
    pen = Pen()
    for i in range(9):
        a = np.pi / 2 + (i - 4) * 0.22
        pen.blade((0.5, 0.0), (0.5 + np.cos(a) * 0.4, np.sin(a) * 0.42), rng.uniform(-0.1, 0.1), 0.03, GREEN, rng.uniform(0.35, 0.65), 0.9)
    for x, y, r in ((0.3, 0.52, 0.07), (0.62, 0.64, 0.085), (0.78, 0.44, 0.06)):
        pen.blade((x - 0.03 + (0.5 - x) * 0.5, 0.0), (x, y - 0.03), 0.05, 0.012, LEAF, 0.4, 0.2)
        pen.petals((x, y), 6, r * 2.3, r * 0.4, RUST, 0.62)
        pen.ellipse((x, y), r * 0.45, r * 0.45, 0, AMBER, 0.5)
    return pen.rgba()


def coal_shrub():
    rng = np.random.default_rng(14)
    pen = Pen()
    pen.blade((0.5, 0.0), (0.5, 0.36), 0.02, 0.04, BARK, 0.35, 0.3)
    discs = [(rng.uniform(0.18, 0.82), rng.uniform(0.3, 0.78), rng.uniform(0.06, 0.11)) for _ in range(30)]
    for x, y, r in sorted(discs, key=lambda d: d[1] - d[0] * 0.1):
        pen.ellipse((x, y), r, r * 0.9, 0.0, SLATE, 0.3 + 0.5 * y)
    return pen.rgba()


def copper_bell():
    pen = Pen()
    pts = curve((0.5, 0.0), (0.52, 0.78), 0.03)
    pen.stroke(pts, 0.013, LEAF, 0.45)
    for base, tip in (((0.5, 0.02), (0.18, 0.3)), ((0.5, 0.02), (0.84, 0.26)), ((0.5, 0.05), (0.3, 0.45))):
        pen.leaf(base, tip, 0.04, FERN, 0.5)
    for i, t in enumerate((0.45, 0.58, 0.7, 0.84)):
        x, y = pts[int(t * 79)]
        side = 1 if i % 2 else -1
        pen.blade((x, y), (x + side * 0.05, y - 0.02), 0.2 * side, 0.006, LEAF, 0.5, 0.1)
        pen.ellipse((x + side * 0.065, y - 0.06), 0.034, 0.055, 0.1 * side, VERDIGRIS, 0.6)
        pen.ellipse((x + side * 0.065, y - 0.115), 0.044, 0.018, 0.0, VERDIGRIS, 0.8)
    return pen.rgba()


def pale_star():
    pen = Pen()
    for x0, top, lean in ((0.26, 0.5, -0.06), (0.5, 0.82, 0.03), (0.74, 0.58, 0.05)):
        pen.blade((x0, 0.0), (x0 + lean, top), 0.03 * lean * 10, 0.012, LEAF, 0.45, 0.3)
        pen.petals((x0 + lean, top + 0.02), 5, 0.2, 0.02, CREAM, 0.65, start=np.pi / 2)
        pen.ellipse((x0 + lean, top + 0.02), 0.025, 0.025, 0, AMBER, 0.5)
    for base, tip in (((0.4, 0.0), (0.2, 0.2)), ((0.6, 0.0), (0.84, 0.18))):
        pen.leaf(base, tip, 0.03, LEAF, 0.5)
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
