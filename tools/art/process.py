"""Wave 2b of the industry faces: the process plants (electrolytic cell, pumpjack, refinery, cracker, chemical plant, diesel
generator, electrolyser, washer, hoist and winch, centrifuge, reactor, recycler, chip fab). Same kit and colour roles as
`industry.py`; the logistics and power blocks are in `heavy.py`.
"""
import numpy as np

from kit import (AETHER, BLUE, BRONZE, CONCRETE, COPPER, DARK, EMBER, FIREBRICK, GOLD, GREEN, MOSS, RED, STEEL, VERDIGRIS,
                 VIOLET, XS, YS, Face)
from lib import N, coords, fbm, paint, rgba
from machines import AMBER, arch_mask


def _recess(f, box=(14, 14, 114, 114)):
    return f.rect(box, DARK, 0.28, -1.0, 5)


def cell_side():
    f = Face(191, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.hazard((10, 14, 118, 30), AMBER, DARK, 18, 0.6)
    f.window((22, 42, 106, 86), AETHER, 0.75)
    f.bar((34, 82), (34, 50), 6, COPPER, 0.6)
    f.bar((94, 82), (94, 50), 6, COPPER, 0.6)
    f.slots((22, 96, 106, 112), 3, False, DARK)
    return f.rgba()


def cell_top():
    f = Face(192, STEEL, 7)
    _recess(f)
    f.rect((24, 24, 104, 104), AETHER, 0.5, -1.0, 6, 0.7)
    f.bar((34, 34), (94, 94), 4, COPPER, 0.6)
    f.bar((94, 34), (34, 94), 4, COPPER, 0.6)
    return f.rgba()


def pumpjack_side():
    f = Face(206, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.pipe((52, 10, 76, 118), True, STEEL, 0.5, 4)
    f.rect((20, 80, 108, 108), DARK, 0.25, -1.0, 4)
    f.bar((30, 100), (98, 88), 6, BRONZE, 0.55)
    return f.rgba()


def pumpjack_top():
    f = Face(207, STEEL, 7)
    _recess(f)
    f.port((64, 64), 36, DARK, 0.0, BRONZE)
    f.disc((64, 64), 16, STEEL, 0.5, 1.0, 4)
    f.bolts()
    return f.rgba()


def refinery_side():
    f = Face(212, STEEL, 7)
    # Flat paint: the round body the game draws it on shades itself (the box shader, `factory::push_round`).
    f.fill(np.ones((N, N), bool), STEEL, 0.5)
    for y in (24, 64, 104):
        f.rect((0, y, N, y + 8), BRONZE, 0.5, 1.0, 3, outline=False)
    f.rect((44, 36, 84, 56), DARK, 0.25, -1.0, 4)
    f.slots((48, 40, 80, 52), 2, False, EMBER, 0.6)
    f.gauge((64, 82), 12, 0.4)
    return f.rgba()


def refinery_top():
    f = Face(213, STEEL, 7)
    _recess(f)
    f.disc((64, 64), 42, STEEL, 0.45, 1.0, 6)
    f.port((64, 64), 26, EMBER, 0.7, BRONZE)
    f.bolts()
    return f.rgba()


def cracker_side():
    f = Face(214, COPPER, 7)
    # Flat paint: the round body the game draws it on shades itself (the box shader, `factory::push_round`).
    f.fill(np.ones((N, N), bool), COPPER, 0.5)
    for y in (22, 62, 102):
        f.rect((0, y, N, y + 8), STEEL, 0.45, 1.0, 3, outline=False)
    f.window((44, 34, 84, 54), EMBER, 0.85)
    f.gauge((64, 82), 12, 0.7)
    return f.rgba()


def cracker_top():
    f = Face(215, STEEL, 7)
    _recess(f)
    f.disc((64, 64), 42, COPPER, 0.5, 1.0, 6)
    f.fan((64, 64), 30, 8, STEEL, DARK, 0.2)
    return f.rgba()


def chem_side():
    f = Face(219, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.window((20, 20, 70, 60), GREEN, 0.8)
    f.disc((94, 40), 16, BRONZE, 0.5, 1.0, 4)
    f.disc((94, 40), 9, GREEN, 0.6, -1.0, 4, 0.7)
    f.slots((20, 74, 108, 108), 4, False, DARK)
    return f.rgba()


def chem_top():
    f = Face(220, STEEL, 7)
    _recess(f)
    for cx, cy in ((44, 44), (84, 44), (44, 84), (84, 84)):
        f.port((cx, cy), 18, GREEN, 0.6, STEEL)
    return f.rgba()


def diesel_side():
    f = Face(223, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.slots((22, 20, 106, 58), 4, False, DARK)
    f.rect((22, 66, 106, 108), BRONZE, 0.5, 1.0, 4)
    f.disc((64, 87), 14, EMBER, 0.6, -1.0, 4, 0.7)
    return f.rgba()


def diesel_top():
    f = Face(224, STEEL, 7)
    _recess(f)
    f.pipe((24, 52, 104, 76), False, STEEL, 0.5, 4)
    f.fan((64, 96), 14, 6, STEEL, DARK)
    f.fan((64, 32), 14, 6, STEEL, DARK)
    return f.rgba()


def electrolyser_side():
    f = Face(225, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.window((20, 22, 108, 70), BLUE, 0.7)
    for x in (34, 54, 74, 94):
        f.bar((x, 70), (x, 30), 5, AETHER, 0.65, 0.8)
    f.rect((20, 82, 108, 108), DARK, 0.25, -1.0, 4)
    f.lines((26, 88, 102, 104), 2, AETHER, 0.8)
    return f.rgba()


def electrolyser_top():
    f = Face(226, STEEL, 7)
    _recess(f)
    f.port((46, 64), 24, BLUE, 0.7, STEEL)
    f.port((90, 64), 24, RED, 0.5, STEEL)
    return f.rgba()


def washer_side():
    f = Face(231, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.window((18, 20, 110, 70), BLUE, 0.6)
    f.pipe((14, 78, 114, 96), False, STEEL, 0.5, 4)
    f.slots((22, 102, 106, 114), 2, False, DARK)
    return f.rgba()


def washer_top():
    f = Face(232, STEEL, 7)
    _recess(f)
    f.rect((22, 22, 106, 106), BLUE, 0.45, -1.0, 6, 0.35)
    for i in range(5):
        f.bar((30, 34 + i * 16), (98, 34 + i * 16), 4, STEEL, 0.6)
    return f.rgba()


def hoist_frame():
    """A lattice shaft: steel posts and cross braces, open between them (a cutout)."""
    from draw import Pen, curve
    pen = Pen()
    for x in (0.08, 0.92):
        pen.stroke(curve((x, 0.0), (x, 1.0), 0.0), 0.06, STEEL, 0.5)
    for y in (0.06, 0.5, 0.94):
        pen.stroke(curve((0.08, y), (0.92, y), 0.0), 0.04, BRONZE, 0.55)
    pen.stroke(curve((0.1, 0.08), (0.9, 0.5), 0.0), 0.03, STEEL, 0.55)
    pen.stroke(curve((0.9, 0.5), (0.1, 0.92), 0.0), 0.03, STEEL, 0.55)
    return pen.rgba()


def hoist_top():
    from draw import Pen, curve
    pen = Pen()
    for x in (0.1, 0.9):
        pen.stroke(curve((x, 0.0), (x, 1.0), 0.0), 0.07, STEEL, 0.5)
    for y in (0.1, 0.9):
        pen.stroke(curve((0.0, y), (1.0, y), 0.0), 0.07, STEEL, 0.5)
    pen.ellipse((0.5, 0.5), 0.2, 0.2, 0.0, BRONZE, 0.55)
    return pen.rgba()


def winch_side():
    f = Face(235, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.hazard((10, 14, 118, 30), GOLD, DARK, 18, 0.6)
    f.disc((64, 70), 30, BRONZE, 0.5, 1.0, 6)
    f.disc((64, 70), 16, DARK, 0.3, -1.0, 4)
    f.disc((64, 70), 6, STEEL, 0.5, 1.0, 2)
    return f.rgba()


def centrifuge_side():
    f = Face(237, STEEL, 7)
    # Flat paint: the round body the game draws it on shades itself (the box shader, `factory::push_round`).
    f.fill(np.ones((N, N), bool), STEEL, 0.5)
    f.rect((0, 56, N, 72), VIOLET, 0.55, 1.0, 3, 0.4, outline=False)
    f.window((44, 24, 84, 48), AETHER, 0.7)
    f.hazard((0, 96, N, 112), GOLD, DARK, 18, 0.6)
    return f.rgba()


def centrifuge_top():
    f = Face(238, STEEL, 7)
    _recess(f)
    f.disc((64, 64), 42, STEEL, 0.45, 1.0, 6)
    f.fan((64, 64), 32, 4, VIOLET, DARK, 0.4)
    return f.rgba()


def reactor_side():
    f = Face(239, CONCRETE, 7)
    f.rect((8, 8, 120, 120), CONCRETE, 0.5, 1.0, 4)
    f.hazard((8, 14, 120, 30), GOLD, DARK, 18, 0.6)
    f.window((34, 44, 94, 84), GREEN, 0.9)
    f.disc((64, 64), 12, GREEN, 0.8, -1.0, 4, 1.0)
    f.rect((8, 96, 120, 112), STEEL, 0.4, 1.0, 3)
    f.slots((16, 98, 112, 110), 6, True, DARK)
    return f.rgba()


def reactor_top():
    f = Face(240, CONCRETE, 7)
    _recess(f)
    f.disc((64, 64), 44, CONCRETE, 0.5, 1.0, 6)
    for dx, dy in ((0, 0), (-20, -12), (20, -12), (-20, 12), (20, 12), (0, 24), (0, -24)):
        f.disc((64 + dx, 64 + dy), 7, GREEN, 0.6, -1.0, 3, 0.9)
    return f.rgba()


def recycler_side():
    f = Face(244, MOSS, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.disc((64, 56), 34, GREEN, 0.4, 1.0, 6, 0.3)
    f.disc((64, 56), 18, DARK, 0.3, -1.0, 4)
    f.slots((22, 98, 106, 112), 2, False, DARK)
    return f.rgba()


def recycler_top():
    f = Face(245, STEEL, 7)
    _recess(f)
    f.chevrons((30, 30, 98, 98), GREEN, 2, True, 0.7, 12)
    return f.rgba()


def fab_side():
    f = Face(249, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.window((18, 20, 110, 60), VIOLET, 0.8)
    f.screen((18, 70, 66, 108), AETHER, 0.9, "grid")
    f.slots((74, 70, 110, 108), 4, False, DARK)
    return f.rgba()


def fab_top():
    f = Face(250, STEEL, 7)
    _recess(f)
    f.rect((26, 26, 102, 102), DARK, 0.3, -1.0, 4)
    for i in range(3):
        for j in range(3):
            f.rect((34 + i * 22, 34 + j * 22, 50 + i * 22, 50 + j * 22), VIOLET, 0.6, 1.0, 3, 0.5)
    return f.rgba()
