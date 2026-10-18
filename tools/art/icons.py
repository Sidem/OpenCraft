"""Item icons (docs/ART_STYLE.md): the items drawn as a picture on a deep navy backdrop, for the layers an item shows as its
whole tile (tools, gear, robot parts, canisters, satellites). They are built on the machine kit's `Face` (so parts, outlines
and glow work as for machines) plus polygons and curves in texel coordinates, y down, content inside 14..114. Materials that
only skin a model (ingots, packs, avatar) are in `items.py`.
"""
import numpy as np

from kit import BLUE, DARK, GOLD, GREEN, RED, VIOLET, XS, YS, Face, TIERS
from lib import N, fbm, paint, ramp, roll
from machines import AETHER, AMBER, BRONZE, COPPER, EMBER, STEEL

NAVY = ramp(["#070a14", "#0e1424", "#18223c", "#26345a"])
WOOD = ramp(["#1e120a", "#4a2c18", "#7a4c26", "#a87440", "#cfa066"])
STONE_HEAD = ramp(["#241c18", "#4a3e38", "#7a6a60", "#a8978a", "#d0c2b4"])
IRON_HEAD = ramp(["#1c2230", "#3a4458", "#66728a", "#98a4b8", "#cdd5e2"])
STEEL_HEAD = ramp(["#10162a", "#243456", "#42609a", "#6f92cc", "#acc6ee"])
LEATHER = ramp(["#1a0f0a", "#40261a", "#6e4228", "#9a6238", "#c08a54"])
TEAL = ramp(["#06161c", "#0f3a44", "#1b6a72", "#2fa0a0", "#78d8cc"])
PCB = ramp(["#04140c", "#0a3020", "#145c36", "#22884a", "#4cc070"])
WHITE = ramp(["#2a2e3c", "#5a6078", "#9aa2bc", "#d4d9e8", "#f6f8ff"])


def canvas(seed):
    """A Face whose plate is replaced by the icon backdrop: a navy vignette with a little cloud in it."""
    f = Face(seed)
    r = np.hypot(f.X - 0.5, f.Y - 0.45)
    v = 0.78 - r * 1.2 + (fbm(f.X, f.Y, 5, 3, seed + 1) - 0.5) * 0.3
    f.img[:] = paint(NAVY, np.clip(v, 0, 1), 0.4)
    f.glow[:] = 0
    return f


def poly(pts):
    """Mask of a polygon (even-odd rule) with corners `pts` in texel coordinates."""
    x, y = XS + 0.5, YS + 0.5
    inside = np.zeros((N, N), bool)
    for i in range(len(pts)):
        (x0, y0), (x1, y1) = pts[i], pts[(i + 1) % len(pts)]
        crosses = (y0 > y) != (y1 > y)
        at = x0 + (y - y0) * (x1 - x0) / ((y1 - y0) or 1e-9)
        inside ^= crosses & (x < at)
    return inside


def depth(m, k=5):
    """How many texels inside `m` each texel is, up to `k` (0 outside)."""
    d = np.zeros((N, N))
    cur = m.copy()
    for _ in range(k):
        d += cur
        cur = cur & roll(cur, 1, 0) & roll(cur, -1, 0) & roll(cur, 0, 1) & roll(cur, 0, -1)
    return d


def shape(f, m, tones, base=0.5, relief=1.0, b=5, glow=None, grain=0.15):
    """Paints the free-form mask `m` as a bevelled solid: lit on its top left edges, with a dark outline."""
    f._shade(m, np.maximum(depth(m, b) - 1, 0), tones, base, relief, b, grain, glow)
    return m


def curve_mask(pts, widths):
    """Mask of a ribbon along the polyline `pts` (n, 2) with half-width `widths` (one number or n)."""
    pts = np.asarray(pts, float)
    w = np.broadcast_to(np.asarray(widths, float), (len(pts),))
    d = np.hypot(XS[..., None] + 0.5 - pts[:, 0], YS[..., None] + 0.5 - pts[:, 1])
    return (d < w).any(-1)


def arc(p0, ctrl, p1, n=48):
    t = np.linspace(0, 1, n)[:, None]
    p0, ctrl, p1 = np.array(p0, float), np.array(ctrl, float), np.array(p1, float)
    return (1 - t) ** 2 * p0 + 2 * (1 - t) * t * ctrl + t ** 2 * p1


# ---------------------------------------------------------------------------------------------------- tools

def tool(kind, head, seed):
    """A pickaxe, axe or shovel standing on the diagonal, handle at the bottom left, the head in `head`'s metal."""
    f = canvas(seed)
    c = np.array([88.0, 38.0])
    d = np.array([0.707, -0.707])  # along the handle, toward the head
    p = np.array([0.707, 0.707])   # across it
    at = lambda a, b: tuple(c + p * a + d * b)
    f.bar((20, 108), tuple(c - d * 6), 9, WOOD, 0.5)
    if kind == "pick":
        pts = arc(at(-44, -16), at(0, 30), at(44, -16))
        t = np.linspace(0, 1, len(pts))
        shape(f, curve_mask(pts, 2.5 + 7.5 * (1 - np.abs(2 * t - 1) ** 2)), head, 0.55)
        shape(f, curve_mask(arc(at(-6, 0), at(0, 8), at(6, 0), 8), 8), head, 0.4)
    elif kind == "axe":
        blade = poly([at(a, b) for a, b in ((-6, -12), (-6, 14), (12, 16), (40, 28), (40, -30), (12, -16))])
        shape(f, blade, head, 0.55)
        f.bar(at(40, -28), at(40, 26), 3, head, 0.95)
    else:
        spade = poly([at(a, b) for a, b in ((-4, 0), (-15, 8), (-15, 34), (-8, 50), (0, 56), (8, 50), (15, 34), (15, 8), (4, 0))])
        shape(f, spade, head, 0.55)
        f.bar(at(0, 6), at(0, 46), 3, head, 0.3)
    return f.rgba()


# ---------------------------------------------------------------------------------------------------- parts

def gear_icon(seed=113):
    f = canvas(seed)
    d = np.hypot(XS - 63.5, YS - 63.5)
    ang = np.arctan2(YS - 63.5, XS - 63.5)
    tooth = ((ang / (2 * np.pi) * 10) % 1) < 0.5
    body = (d < 40) | ((d < 52) & tooth)
    shape(f, body, STEEL, 0.6, b=6)
    f.disc((63.5, 63.5), 30, STEEL, 0.4, -1.0, 4)
    f.disc((63.5, 63.5), 20, STEEL, 0.7, 1.0, 5)
    f.disc((63.5, 63.5), 8, DARK, 0.3, -1.0, 3)
    return f.rgba()


def chip(seed, die=AETHER, body=DARK, pins=GOLD):
    f = canvas(seed)
    for i in range(7):
        k = 26 + i * 12
        for box in ((k, 14, k + 6, 30), (k, 98, k + 6, 114), (14, k, 30, k + 6), (98, k, 114, k + 6)):
            f.rect(box, pins, 0.6, 1.0, 2)
    f.rect((26, 26, 102, 102), body, 0.4, 1.0, 6)
    f.rect((40, 40, 88, 88), die, 0.55, -1.0, 4, 0.7)
    f.bar((48, 64), (80, 64), 3, die, 0.9, 0.9)
    f.bar((64, 48), (64, 80), 3, die, 0.9, 0.9)
    return f.rgba()


def circuit(seed=154):
    """A green board: gold traces to a dark socket and a few solder pads."""
    f = canvas(seed)
    f.rect((10, 10, 118, 118), PCB, 0.5, 1.0, 5)
    rng = np.random.default_rng(seed)
    for _ in range(9):
        x, y = int(rng.integers(18, 100)), int(rng.integers(18, 100))
        f.bar((x, y), (x + int(rng.integers(8, 28)), y), 3, GOLD, 0.6)
        f.bar((x, y), (x, y + int(rng.integers(8, 26))), 3, GOLD, 0.6)
    f.rect((38, 38, 90, 90), DARK, 0.3, 1.0, 5)
    f.rect((50, 50, 78, 78), AETHER, 0.5, -1.0, 3, 0.6)
    return f.rgba()


def cylinder(f, box, vertical, tones, base=0.55):
    f.pipe(box, vertical, tones, base)


def servo(seed=168):
    f = canvas(seed)
    cylinder(f, (22, 40, 86, 88), True, STEEL)
    f.coil((38, 44, 70, 84), COPPER)
    f.rect((86, 54, 108, 74), STEEL, 0.8, 1.0, 3)
    f.rect((14, 48, 24, 80), BRONZE, 0.5, 1.0, 3)
    return f.rgba()


def actuator(seed=169):
    f = canvas(seed)
    cylinder(f, (14, 46, 70, 82), True, STEEL)
    f.bar((70, 64), (104, 64), 9, STEEL, 0.85)
    f.rect((100, 46, 114, 82), EMBER, 0.55, 1.0, 3, 0.4)
    f.rect((26, 38, 36, 90), BRONZE, 0.5, 1.0, 3)
    return f.rgba()


def drone_cell(seed=170):
    f = canvas(seed)
    f.rect((34, 28, 94, 108), VIOLET, 0.45, 1.0, 6)
    f.rect((44, 46, 84, 98), AETHER, 0.55, -1.0, 4, 0.8)
    f.rect((52, 14, 76, 28), STEEL, 0.7, 1.0, 3)
    f.rect((40, 74, 88, 98), AETHER, 0.8, -1.0, 3, 1.0)
    return f.rgba()


def guidance(seed=171):
    f = canvas(seed)
    for r in (46, 34, 22):
        f.disc((64, 64), r, STEEL, 0.5 + 0.1 * (r % 3), 1.0 if r != 34 else -1.0, 4)
    f.disc((64, 64), 12, AETHER, 0.85, 1.0, 5, 1.0)
    for a in range(4):
        ang = a * np.pi / 2
        f.bar((64 + np.cos(ang) * 44, 64 + np.sin(ang) * 44), (64 + np.cos(ang) * 56, 64 + np.sin(ang) * 56), 8, BRONZE, 0.5)
    return f.rgba()


def quad(seed, core, arms=STEEL):
    """A quad-rotor seen from above: four arms with rotor discs round a body."""
    f = canvas(seed)
    for sx in (-1, 1):
        for sy in (-1, 1):
            tip = (64 + sx * 38, 64 + sy * 38)
            f.bar((64, 64), tip, 7, arms, 0.5)
            f.disc(tip, 17, STEEL, 0.35, -1.0, 5)
            f.disc(tip, 4, DARK, 0.3, 1.0, 2)
    f.rect((44, 44, 84, 84), arms, 0.55, 1.0, 5)
    f.rect((54, 54, 74, 74), core, 0.65, -1.0, 4, 0.9)
    return f.rgba()


def jetpack(seed=175):
    f = canvas(seed)
    for x in (30, 70):
        f.rect((x, 22, x + 28, 82), STEEL, 0.5, 1.0, 5)
        f.rect((x + 4, 82, x + 24, 92), BRONZE, 0.5, 1.0, 3)
        flame = poly([(x + 5, 92), (x + 23, 92), (x + 14, 116)])
        shape(f, flame, EMBER, 0.75, b=3, glow=0.9)
    f.bar((58, 30), (70, 30), 5, BRONZE, 0.5)
    return f.rgba()


def personal_drone(seed=176):
    f = canvas(seed)
    f.disc((64, 66), 42, TEAL, 0.5, 1.0, 8)
    f.disc((64, 66), 24, DARK, 0.3, -1.0, 5)
    f.disc((64, 66), 15, AETHER, 0.8, 1.0, 5, 1.0)
    for sx in (-1, 1):
        f.rect((64 + sx * 36 - 8, 14, 64 + sx * 36 + 8, 28), STEEL, 0.6, 1.0, 3)
    return f.rgba()


def planner(seed=177):
    f = canvas(seed)
    f.rect((10, 22, 118, 106), BLUE, 0.35, 1.0, 5)
    for i in range(1, 6):
        f.bar((10 + i * 18, 26), (10 + i * 18, 102), 1, BLUE, 0.7, 0.2)
    for i in range(1, 4):
        f.bar((14, 22 + i * 21), (114, 22 + i * 21), 1, BLUE, 0.7, 0.2)
    f.rect((38, 44, 88, 84), EMBER, 0.6, -1.0, 3, 0.6)
    f.bar((38, 44), (88, 84), 2, EMBER, 0.9, 0.8)
    return f.rgba()


# ---------------------------------------------------------------------------------------------------- gear

def hauler(tier, seed):
    f = canvas(seed)
    f.rect((30, 32, 98, 108), LEATHER, 0.5, 1.0, 7)
    f.rect((30, 32, 98, 56), LEATHER, 0.7, 1.0, 5)
    f.rect((50, 62, 78, 92), LEATHER, 0.35, -1.0, 4)
    f.rect((60, 54, 68, 70), GOLD, 0.7, 1.0, 2, 0.2)
    if tier > 0:
        f.rect((30, 20, 98, 32), TIERS[tier], 0.6, 1.0, 3, 0.55)
        f.pips(min(tier, 4), 22, 64, 4, 4)
    f.bar((24, 40), (24, 100), 6, LEATHER, 0.3)
    f.bar((104, 40), (104, 100), 6, LEATHER, 0.3)
    return f.rgba()


def boots(seed, sole, accent):
    f = canvas(seed)
    for x in (10, 68):
        shape(f, poly([(x, 16), (x + 28, 16), (x + 28, 72), (x + 50, 82), (x + 50, 104), (x, 104)]), STEEL, 0.45, b=5)
        f.rect((x - 2, 98, x + 52, 112), sole, 0.5, 1.0, 3, 0.4 if sole is not STEEL else None)
        f.bar((x + 4, 34), (x + 24, 34), 4, accent, 0.7, 0.6)
        f.bar((x + 4, 50), (x + 24, 50), 4, accent, 0.7, 0.6)
    return f.rgba()


def exo_frame(seed=184):
    f = canvas(seed)
    f.rect((26, 14, 102, 114), STEEL, 0.4, 1.0, 5)
    for y in (40, 66, 92):
        f.bar((30, y), (98, y), 4, STEEL, 0.65)
    f.bar((64, 20), (64, 108), 5, STEEL, 0.7)
    f.rect((46, 40, 82, 68), AETHER, 0.6, -1.0, 4, 0.9)
    f.disc((64, 54), 7, AETHER, 0.95, 1.0, 3, 1.0)
    return f.rgba()


def mining_rig(seed=185):
    f = canvas(seed)
    f.rect((14, 14, 114, 40), BRONZE, 0.5, 1.0, 5)
    f.rect((54, 14, 74, 30), GOLD, 0.7, 1.0, 3, 0.4)
    drill = poly([(30, 44), (66, 44), (48, 108)])
    shape(f, drill, STEEL, 0.55, b=5)
    for k in range(4):
        f.bar((36 + k * 2, 52 + k * 13), (62 - k * 3, 58 + k * 13), 3, STEEL, 0.9)
    f.disc((92, 74), 18, EMBER, 0.6, 1.0, 5, 0.8)
    f.disc((92, 74), 8, AMBER, 0.9, -1.0, 4, 1.0)
    return f.rgba()


def gloves(seed=285):
    f = canvas(seed)
    palm = poly([(36, 100), (32, 62), (38, 50), (86, 50), (96, 62), (92, 100)])
    shape(f, palm, TEAL, 0.5, b=6)
    for i, x in enumerate((38, 54, 70, 84)):
        f.rect((x, 20 + abs(i - 1.5) * 6, x + 12, 56), TEAL, 0.55, 1.0, 4)
    f.rect((32, 90, 96, 108), STEEL, 0.6, 1.0, 4)
    f.rect((36, 94, 92, 104), AETHER, 0.7, -1.0, 2, 0.6)
    return f.rgba()


# ---------------------------------------------------------------------------------------------------- liquids and cans

def canister(liquid, seed, level=0.8, glow=0.45, dark_fill=False):
    """A steel can with a brass cap and a glass window: the fluid (`liquid` ramp, None when empty) shows at its level."""
    f = canvas(seed)
    f.rect((38, 36, 90, 112), STEEL, 0.5, 1.0, 7)
    shape(f, poly([(46, 36), (82, 36), (76, 24), (52, 24)]), STEEL, 0.6, b=3)
    f.rect((54, 12, 74, 26), BRONZE, 0.55, 1.0, 3)
    f.rect((46, 50, 82, 100), DARK, 0.3, -1.0, 4)
    if liquid is not None:
        top = int(100 - 50 * level)
        f.rect((49, top, 79, 97), liquid, 0.5 if not dark_fill else 0.25, -1.0, 3, glow if not dark_fill else 0.05)
        f.bar((50, top + 1), (78, top + 1), 2, liquid, 0.95, glow)
    f.bar((52, 60), (52, 92), 2, STEEL, 0.9)
    return f.rgba()
