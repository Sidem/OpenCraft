"""The decor building set and the ruins (docs/ART_STYLE.md): five concretes, four brick and tile faces, framed glass, metal
panel, catwalk, railing, panel light; rusted plating and machine husks, a girder, the logbook and the vault door.
Concretes and bricks wrap (they repeat over big walls); sprites with holes (glass, catwalk, railing) are cutouts.
"""
import numpy as np

from draw import Pen, curve
from industry import concrete
from kit import AETHER, BLUE, BRONZE, DARK, GOLD, GREEN, RED, STEEL, XS, YS, Face
from lib import N, coords, fbm, paint, ramp, rgba
from masonry import bricks
from rocks import BASALT, GRANITE, SANDSTONE
from stains import RUST

BLACK = ramp(["#05060a", "#0e1118", "#1b1f2a", "#2b3140", "#40485a"])
WHITE = ramp(["#2a2e3c", "#5a5f72", "#9096a8", "#c8ccd8", "#eceef4"])
RED_C = ramp(["#1c0a0e", "#3d1618", "#6b2a26", "#94443a", "#b86650", "#d98d6e"])
GREEN_C = ramp(["#08140e", "#12281c", "#1f4a2c", "#346a3c", "#52885a", "#7aa87c"])
YELLOW_C = ramp(["#1e1408", "#4a3412", "#7a5a1c", "#a88428", "#cfa844", "#e8cd78"])
SLAG = ramp(["#08060c", "#14101c", "#241c30", "#383048", "#4e4462"])


def coloured_concrete(tones, seed):
    return concrete(seed, tones)


def basalt_tiles(seed=0):
    """Square basalt tiles, each its own tone, in pale grout; wraps."""
    X, Y = coords()
    tx, ty = XS // 32, YS // 32
    own = np.random.default_rng(seed + 1).random((4, 4))[ty, tx]
    local_x, local_y = XS % 32, YS % 32
    d = np.minimum(np.minimum(local_x, 31 - local_x), np.minimum(local_y, 31 - local_y))
    lit = ((local_x < 4) | (local_y < 4)) * 0.12 - ((local_x > 27) | (local_y > 27)) * 0.12
    v = 0.34 + own * 0.2 + lit + (fbm(X, Y, 16, 3, seed + 2) - 0.5) * 0.2
    img = paint(BASALT, v, 0.3)
    grout = d < 2
    img[grout] = paint(WHITE, np.full((N, N), 0.4 + 0.1 * fbm(X, Y, 20, 2, seed + 3)), 0.3)[grout]
    return rgba(img)


def brick_face(tones, seed, joint=None):
    return bricks(seed, (30, 34, 32, 32), 2, False, tones, joint)


def framed_glass():
    pen = Pen()
    pen.stroke(curve((0.0, 0.5), (1.0, 0.5), 0.0), 0.04, STEEL, 0.5)
    pen.stroke(curve((0.5, 0.0), (0.5, 1.0), 0.0), 0.04, STEEL, 0.5)
    for y in (0.035, 0.965):
        pen.stroke(curve((0.0, y), (1.0, y), 0.0), 0.04, STEEL, 0.5)
    for x in (0.035, 0.965):
        pen.stroke(curve((x, 0.0), (x, 1.0), 0.0), 0.04, STEEL, 0.5)
    for a, b in (((0.12, 0.3), (0.3, 0.12)), ((0.62, 0.9), (0.9, 0.62))):
        pen.stroke(curve(a, b, 0.0), 0.012, AETHER, 0.9)
    return pen.rgba()


def metal_panel(seed=0):
    f = Face(343, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.5, -1.0, 4)
    f.bar((64, 10), (64, 118), 4, STEEL, 0.3)
    f.bar((10, 64), (118, 64), 4, STEEL, 0.3)
    f.bolts(((20, 20), (108, 20), (20, 108), (108, 108), (76, 76), (52, 52), (76, 52), (52, 76)), 3, STEEL)
    return f.rgba()


def catwalk():
    """A steel grating: a frame with a mesh of crossing bars and open squares between them (cutout)."""
    pen = Pen()
    for t in (0.02, 0.98):
        pen.stroke(curve((0.0, t), (1.0, t), 0.0), 0.04, STEEL, 0.55)
        pen.stroke(curve((t, 0.0), (t, 1.0), 0.0), 0.04, STEEL, 0.55)
    for t in np.linspace(0.18, 0.82, 5):
        pen.stroke(curve((0.0, t), (1.0, t), 0.0), 0.014, STEEL, 0.45)
        pen.stroke(curve((t, 0.0), (t, 1.0), 0.0), 0.014, STEEL, 0.45)
    return pen.rgba()


def railing():
    pen = Pen()
    for x in (0.08, 0.5, 0.92):
        pen.stroke(curve((x, 0.0), (x, 0.82), 0.0), 0.035, STEEL, 0.5)
    for y in (0.82, 0.5, 0.2):
        pen.stroke(curve((0.0, y), (1.0, y), 0.0), 0.028 if y > 0.7 else 0.02, BRONZE if y > 0.7 else STEEL, 0.55)
    return pen.rgba()


def panel_light():
    f = Face(346, STEEL, 7)
    f.rect((8, 8, 120, 120), STEEL, 0.45, 1.0, 4)
    cream = ramp(["#5a4a3a", "#a8946a", "#e8d8a8", "#fff6d8", "#ffffff"])
    f.rect((18, 18, 110, 110), cream, 0.78, -1.0, 6, 1.0, 0.2)
    f.bar((64, 18), (64, 110), 3, cream, 0.5, 1.0)
    f.bar((18, 64), (110, 64), 3, cream, 0.5, 1.0)
    return f.rgba()


# ---------------------------------------------------------------------------------------------------- ruins

def _rust(seed, flakes=True):
    f = Face(seed, RUST, 7)
    X, Y = f.X, f.Y
    if flakes:
        m = fbm(X, Y, 7, 3, seed + 3) > 0.6
        f.fill(m & (f.d > 6), RUST, 0.25, None, 0.5)
    return f


def rust_plate():
    f = _rust(326)
    f.rect((12, 12, 116, 116), RUST, 0.5, -1.0, 4, grain=0.5)
    f.bolts(((20, 20), (108, 20), (20, 108), (108, 108)), 3, RUST)
    return f.rgba()


def rust_machine():
    f = _rust(327)
    f.rect((14, 14, 114, 80), DARK, 0.3, -1.0, 5)
    f.bar((20, 70), (60, 24), 5, RUST, 0.4)
    f.bar((60, 24), (104, 66), 4, RUST, 0.35)
    f.slots((20, 90, 108, 112), 3, False, DARK)
    return f.rgba()


def rust_girder():
    f = _rust(328)
    f.rect((8, 8, 120, 28), RUST, 0.6, 1.0, 4)
    f.rect((8, 100, 120, 120), RUST, 0.6, 1.0, 4)
    f.rect((52, 28, 76, 100), RUST, 0.45, -1.0, 4)
    return f.rgba()


def logbook_side():
    f = Face(330, GREEN, 7)
    f.rect((8, 8, 120, 120), GREEN, 0.4, 1.0, 5)
    f.rect((24, 40, 104, 60), GOLD, 0.55, 1.0, 3)
    f.lines((30, 72, 98, 100), 3, GOLD, 0.7)
    return f.rgba()


def logbook_top():
    page = ramp(["#3a2c22", "#8a7258", "#c8b088", "#ece0c0", "#fff8e4"])
    f = Face(331, page, 7)
    f.rect((8, 8, 120, 120), page, 0.6, 1.0, 5)
    for y in range(24, 112, 8):
        f.bar((14, y), (114, y), 2, page, 0.3)
    return f.rgba()


def vault_door():
    f = Face(332, STEEL, 7)
    f.rect((8, 8, 120, 120), STEEL, 0.4, 1.0, 5)
    f.disc((64, 64), 48, BRONZE, 0.5, 1.0, 6)
    f.disc((64, 64), 36, STEEL, 0.55, -1.0, 6)
    for a in range(8):
        ang = a * np.pi / 4
        f.bar((64, 64), (64 + np.cos(ang) * 34, 64 + np.sin(ang) * 34), 6, BRONZE, 0.55)
    f.disc((64, 64), 12, BRONZE, 0.6, 1.0, 4)
    f.disc((64, 64), 5, DARK, 0.3, -1.0, 2)
    return f.rgba()
