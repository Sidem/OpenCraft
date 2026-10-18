"""Wave 1 of the industry faces: belts and logistics, the miners, the first machines (constructor, lab, assembler and its
ports, blast furnace, motor, concrete). Each is a short list of `kit.Face` parts; accents follow the function: ember
orange for fuel and heat, aether cyan for power and data, teal for assembly, violet for the far tech. Tier machines wear
the tier ramp as a band with one pip per Mk. Recipes return `Face.rgba()` tiles; `registry.py` maps them to layers.
"""
import numpy as np

from kit import AETHER, BLUE, BRONZE, CONCRETE, COPPER, DARK, EMBER, FIREBRICK, GOLD, STEEL, TIERS, VERDIGRIS, VIOLET, XS, YS, Face
from lib import N, coords, fbm, paint, rgba, vnoise
from machines import arch_mask, rivet
from natural import RAMPS

TIER_BELT = [EMBER, AETHER, BLUE, VIOLET]


def belt(tier=0, seed=0):
    """A rubber belt between bronze rails, chevrons inlaid in the tier colour; wraps along its length (the texture scrolls)."""
    X, Y = coords()
    v = 0.17 + 0.12 * vnoise(X * 40, Y * 40, 40, seed + 1) - 0.05 * ((YS % 8) < 2)
    img = paint(STEEL, v, 0.3)
    dist = np.minimum(XS, N - 1 - XS)
    rail = dist < 12
    rv = 0.62 - np.abs(dist - 5.5) * 0.045 + np.where(XS < 64, 0.1, -0.12) * (dist > 6)
    img = np.where(rail[..., None], paint(BRONZE, np.where(dist < 1, 0.0, np.where(dist > 10, 0.05, rv)), 0.2), img)
    for k in range(4):
        rivet(img, 6, 16 + 32 * k, 3)
        rivet(img, N - 7, 16 + 32 * k, 3)
    dx = np.abs(XS - 63.5)
    phase = (YS - dx * 0.75) % 64
    chev = (phase < 18) & (dx < 46)
    tones = TIER_BELT[tier]
    img = np.where(chev[..., None], paint(tones, 0.78 - phase / 18 * 0.45, 0.15), img)
    edge = chev & ((phase < 2) | (phase > 16))
    img = np.where(edge[..., None], tones[1], img)
    return rgba(img, glow=np.where(chev, 0.35, 0.0))


def _router(arrows):
    f = Face(31, STEEL, 7)
    f.rect((18, 18, 110, 110), DARK, 0.3, -1.0, 4)
    f.bar((64, 106), (64, 70), 9, AETHER, 0.6, 0.7)
    for ox in (-1, 1):
        f.bar((64, 70), (64 + ox * 28, 40), 9, AETHER, 0.6, 0.7)
        f.bar((64 + ox * 28, 40), (64 + ox * 28, 26), 9, AETHER, 0.6, 0.7)
    if arrows:  # a filter: a bronze diamond sits in the throat
        f.disc((64, 70), 12, BRONZE, 0.6, 1.0, 4)
        f.disc((64, 70), 5, DARK, 0.3, -1.0, 2)
    f.bolts(((14, 14), (113, 14), (14, 113), (113, 113)), 3)
    return f.rgba()


def splitter_top():
    return _router(False)


def filter_top():
    return _router(True)


def _band(f, tier, y=82):
    f.stripe((8, y, 120, y + 18), TIERS[tier])
    f.pips(tier, y + 6)


def miner_side(tier=0):
    f = Face(15 + tier, STEEL, 7)
    f.rect((16, 16, 112, 74), STEEL, 0.55, -1.0, 5)
    f.slots((26, 26, 102, 64), 4, False, DARK)
    f.bolts(((14, 14), (113, 14), (14, 113), (113, 113)), 3)
    _band(f, tier)
    f.hazard((8, 104, 120, 114), TIERS[tier], DARK, 18, 0.7)
    return f.rgba()


def miner_top():
    f = Face(16, STEEL, 7)
    f.rect((16, 16, 112, 112), DARK, 0.3, -1.0, 5)
    f.slots((24, 24, 104, 104), 8, False, STEEL, None, 0.28)
    f.fan((64, 64), 26, 6, STEEL, DARK, 0.3)
    f.bolts()
    return f.rgba()


def drill():
    """Polished drill steel with a helical thread; wraps."""
    X, Y = coords()
    phase = ((XS + YS) % 32) / 32.0
    v = 0.42 + 0.34 * np.sin(phase * 2 * np.pi) + (vnoise(X * 16, Y * 16, 16, 3) - 0.5) * 0.1
    img = paint(STEEL, np.where(phase < 0.06, 0.0, v), 0.25)
    return rgba(img)


def box_side():
    """A timber crate: oak panels, bronze corner brackets, a latch."""
    f = Face(18, RAMPS["oak"], 7)
    f.rect((16, 16, 112, 112), RAMPS["oak"], 0.45, -1.0, 4)
    f.rect((16, 56, 112, 72), RAMPS["oak"], 0.55, 1.0, 4)
    for x0, y0, x1, y1 in ((8, 8, 30, 30), (98, 8, 120, 30), (8, 98, 30, 120), (98, 98, 120, 120)):
        f.rect((x0, y0, x1, y1), BRONZE, 0.5, 1.0, 4)
    f.rect((54, 52, 74, 78), BRONZE, 0.55, 1.0, 4)
    f.disc((64, 64), 4, DARK, 0.3, -1.0, 2)
    return f.rgba()


def box_top():
    f = Face(19, RAMPS["oak"], 7)
    f.rect((16, 16, 112, 112), RAMPS["oak"], 0.5, -1.0, 4)
    f.rect((16, 60, 112, 68), DARK, 0.2, -1.0, 2)
    for x0, y0, x1, y1 in ((8, 8, 30, 30), (98, 8, 120, 30), (8, 98, 30, 120), (98, 98, 120, 120)):
        f.rect((x0, y0, x1, y1), BRONZE, 0.5, 1.0, 4)
    return f.rgba()


def constructor_side():
    f = Face(29, STEEL, 7)
    f.window((18, 16, 110, 74), AETHER, 0.7)
    f.bar((30, 66), (60, 34), 7, STEEL, 0.6)
    f.bar((60, 34), (92, 56), 6, STEEL, 0.55)
    f.disc((60, 34), 6, BRONZE, 0.6, 1.0, 3)
    f.screen((18, 82, 70, 110), AETHER, 0.8, "wave")
    f.slots((78, 82, 110, 110), 3, False, DARK)
    f.bolts()
    return f.rgba()


def constructor_top():
    f = Face(30, STEEL, 7)
    f.rect((18, 18, 110, 110), STEEL, 0.5, -1.0, 5)
    f.rect((34, 34, 94, 94), DARK, 0.3, -1.0, 4)
    f.port((64, 64), 22, AETHER, 0.7)
    f.bolts()
    return f.rgba()


def lab_side():
    f = Face(41, STEEL, 7)
    f.window((16, 16, 112, 70), AETHER, 0.75)
    f.screen((24, 24, 104, 62), AETHER, 0.9, "grid")
    f.rect((16, 80, 52, 112), DARK, 0.3, -1.0, 3)
    f.rect((60, 80, 112, 112), STEEL, 0.55, -1.0, 4)
    f.lines((64, 86, 108, 108), 3, STEEL, 0.85)
    f.disc((34, 96), 8, AETHER, 0.6, -1.0, 3, 0.7)
    f.bolts()
    return f.rgba()


def lab_top():
    f = Face(42, STEEL, 7)
    f.rect((16, 16, 112, 112), BRONZE, 0.5, 1.0, 5)
    f.disc((64, 64), 38, AETHER, 0.55, -1.0, 8, 0.6)
    f.gauge((64, 64), 18, 0.7)
    f.bolts()
    return f.rgba()


def pole_side():
    """A utility pole: grey post, crossarm, glass insulators that glint (a cutout)."""
    from draw import Pen, curve
    pen = Pen(glow=True)
    pen.stroke(curve((0.5, 0.0), (0.5, 0.95), 0.0), 0.05, STEEL, 0.5)
    pen.stroke(curve((0.2, 0.8), (0.8, 0.8), 0.0), 0.03, STEEL, 0.6)
    for x in (0.24, 0.5, 0.76):
        pen.ellipse((x, 0.86), 0.035, 0.05, 0.0, AETHER, 0.7, glow=0.7)
    return pen.rgba()


def assembler_side():
    f = Face(121, STEEL, 7)
    f.window((16, 16, 112, 68), VERDIGRIS, 0.55)
    f.bar((30, 58), (50, 30), 6, STEEL, 0.6)
    f.bar((50, 30), (84, 30), 6, STEEL, 0.6)
    f.bar((84, 30), (98, 58), 6, STEEL, 0.55)
    f.screen((16, 78, 58, 110), VERDIGRIS, 0.8, "bars")
    f.slots((66, 78, 112, 110), 4, False, DARK)
    f.stripe((8, 114, 120, 120), VERDIGRIS)
    f.bolts()
    return f.rgba()


def assembler_top():
    f = Face(122, STEEL, 7)
    f.rect((16, 16, 112, 112), STEEL, 0.5, -1.0, 5)
    f.rect((30, 30, 98, 98), DARK, 0.3, -1.0, 4)
    for dx, dy in ((-1, -1), (1, -1), (-1, 1), (1, 1)):
        f.bar((64 + dx * 10, 64 + dy * 10), (64 + dx * 30, 64 + dy * 30), 6, VERDIGRIS, 0.6, 0.5)
    f.disc((64, 64), 12, BRONZE, 0.55, 1.0, 4)
    f.bolts()
    return f.rgba()


def _port(tones, up):
    f = Face(123 + int(up), STEEL, 7)
    f.rect((14, 14, 114, 114), DARK, 0.28, -1.0, 5)
    f.chevrons((26, 26, 102, 102), tones, 2, up, 0.8, 12)
    f.bolts(((12, 12), (115, 12), (12, 115), (115, 115)), 3)
    return f.rgba()


def port_in():
    return _port(AETHER, False)


def port_out():
    return _port(GOLD, True)


def port_side():
    f = Face(129, STEEL, 7)
    f.rect((14, 14, 114, 114), DARK, 0.28, -1.0, 5)
    f.chevrons((26, 26, 102, 102), VIOLET, 2, False, 0.8, 12)
    f.bolts(((12, 12), (115, 12), (12, 115), (115, 115)), 3)
    return f.rgba()


def concrete(seed=0, tones=CONCRETE):
    """Poured concrete: pale aggregate, tie-rod holes and board lines; wraps."""
    X, Y = coords()
    v = 0.5 + (fbm(X, Y, 8, 4, seed + 1) - 0.5) * 0.5 + (vnoise(X * 60, Y * 60, 60, seed + 2) - 0.5) * 0.2
    v = v - ((YS % 64) < 2) * 0.18 - ((XS % 64) < 2) * 0.1
    img = paint(tones, v, 0.3)
    for cx, cy in ((32, 32), (96, 96)):
        yy, xx = np.ogrid[0:N, 0:N]
        hole = np.hypot(xx - cx, yy - cy) < 5
        img[hole] = tones[0]
        rim = (np.hypot(xx - cx, yy - cy) < 7) & ~hole
        img[rim] = tones[2]
    return rgba(img)


def motor_side():
    f = Face(126, STEEL, 7)
    f.coil((8, 30, 120, 98), COPPER, 7)
    f.rect((8, 24, 120, 30), BRONZE, 0.55, 1.0, 2)
    f.rect((8, 98, 120, 104), BRONZE, 0.55, 1.0, 2)
    f.bolts()
    return f.rgba()


def blast_side():
    """A tall brick stack bound with iron hoops, an ember mouth low down."""
    f = Face(127, STEEL, 7)
    f.rect((8, 8, 120, 120), FIREBRICK, 0.4, 1.0, 3, grain=0.4)
    for y in (20, 60, 100):
        f.rect((8, y, 120, y + 10), STEEL, 0.4, 1.0, 3)
        f.bolts(((18, y + 5), (110, y + 5)), 2, STEEL)
    mouth = arch_mask(40, 88, 66, 98)
    f.fill(mouth, EMBER, 0.75, 0.9, 0.5)
    f.rect((36, 98, 92, 104), STEEL, 0.4, 1.0, 2)
    return f.rgba()


def blast_top():
    f = Face(128, STEEL, 7)
    f.rect((12, 12, 116, 116), FIREBRICK, 0.4, 1.0, 4, grain=0.3)
    f.port((64, 64), 40, EMBER, 0.9, STEEL)
    f.bolts()
    return f.rgba()
