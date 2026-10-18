"""Wave 2 of the industry faces: logistics blocks (ramps, lift, underpass, pipes, docks), power (boiler, turbine, arc furnace,
solar, accumulator), crusher and silo. The process plants are in `process.py`. Same kit and colour roles as `industry.py`.
Every function returns a tile; `registry.py` maps them to layers.
"""
import numpy as np

from kit import (AETHER, BLUE, BRONZE, CONCRETE, COPPER, DARK, EMBER, FIREBRICK, GOLD, GREEN, MOSS, RED, STEEL, VERDIGRIS,
                 VIOLET, XS, YS, Face)
from lib import N, coords, fbm, paint, rgba
from machines import AMBER, arch_mask


def _recess(f, box=(14, 14, 114, 114)):
    return f.rect(box, DARK, 0.28, -1.0, 5)


def ramp_side(up):
    f = Face(33, STEEL, 7)
    _recess(f)
    wedge = (YS > (N - XS if up else XS)) & (XS > 14) & (XS < 114) & (YS > 14) & (YS < 114)
    f.fill(wedge, STEEL, 0.5, None, 0.3)
    f.bar((18, 110) if up else (110, 110), (110, 18) if up else (18, 18), 8, BRONZE, 0.55)
    f.chevrons((44, 56, 92, 100), AETHER, 2, True, 0.5, 9)
    f.bolts()
    return f.rgba()


def lift_side():
    f = Face(35, STEEL, 7)
    _recess(f)
    f.pipe((30, 14, 42, 114), True, BRONZE, 0.5)
    f.pipe((86, 14, 98, 114), True, BRONZE, 0.5)
    f.chevrons((46, 26, 82, 102), AETHER, 3, True, 0.7, 8)
    return f.rgba()


def underpass(entry):
    f = Face(36, STEEL, 7)
    _recess(f)
    arch = arch_mask(30, 98, 24, 106)
    f.fill(arch, DARK, 0.2, None, 0.1)
    rim = arch & ~(arch & np.roll(np.roll(arch, 4, 0), 0, 1) & np.roll(arch, -4, 0) & np.roll(arch, 4, 1) & np.roll(arch, -4, 1))
    f.fill(rim, BRONZE, 0.55, None, 0.2)
    f.chevrons((46, 44, 82, 96), GOLD, 2, not entry, 0.6, 9)
    return f.rgba()


def pipe_tile(tones, flow):
    """A pipe wall: a cylinder shaded across, flanges at the ends, a coloured band for what flows through it."""
    f = Face(178, STEEL, 1)
    f.pipe((0, 0, N, N), False, STEEL, 0.55)
    f.rect((0, 0, N, 14), BRONZE, 0.5, 1.0, 3, outline=False)
    f.rect((0, N - 14, N, N), BRONZE, 0.5, 1.0, 3, outline=False)
    band = (YS > 38) & (YS < 90)
    lum = f.img.mean(-1) / 255.0
    f.img[band] = paint(tones, np.clip(lum * 1.1, 0, 1), 0.2)[band]
    f.glow[band] = flow
    return f.rgba()


def pipe_water():
    return pipe_tile(BLUE, 0.2)


def pipe_steam():
    return pipe_tile(RED, 0.25)


def dock_side():
    f = Face(196, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.45, -1.0, 4)
    f.hazard((14, 90, 114, 108), GOLD, DARK, 18, 0.6)
    f.slots((20, 24, 108, 78), 3, False, DARK)
    f.bolts()
    return f.rgba()


def dock_top(load):
    f = Face(197, STEEL, 7)
    _recess(f)
    f.chevrons((34, 30, 94, 100), GOLD if load else BLUE, 2, not load, 0.8, 13)
    f.bolts()
    return f.rgba()


def signal():
    f = Face(199, STEEL, 7)
    _recess(f)
    f.bar((64, 114), (64, 60), 12, STEEL, 0.5)
    f.disc((64, 40), 22, BRONZE, 0.5, 1.0, 4)
    f.disc((64, 40), 15, RED, 0.7, -1.0, 6, 0.9, 0.5)
    return f.rgba()


# ---------------------------------------------------------------------------------------------------- power

def boiler_side():
    f = Face(141, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.42, 1.0, 4)
    for y in (30, 66, 98):
        f.rect((10, y, 118, y + 6), BRONZE, 0.5, 1.0, 2)
    f.window((24, 40, 54, 62), EMBER, 0.8)
    f.gauge((86, 50), 18, 0.55)
    f.slots((22, 78, 106, 94), 5, False, EMBER, 0.7)
    f.pipe((56, 10, 70, 30), True, COPPER, 0.5)
    return f.rgba()


def boiler_top():
    f = Face(142, STEEL, 7)
    _recess(f)
    f.port((64, 64), 40, EMBER, 0.85, BRONZE)
    f.pipe((56, 14, 72, 30), True, COPPER, 0.5)
    f.bolts()
    return f.rgba()


def turbine_side():
    f = Face(143, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.45, 1.0, 4)
    f.slots((20, 20, 108, 70), 6, False, DARK)
    f.coil((10, 78, 118, 102), COPPER, 7)
    f.rect((10, 74, 118, 80), BRONZE, 0.5, 1.0, 2)
    f.rect((10, 102, 118, 108), BRONZE, 0.5, 1.0, 2)
    f.bolts()
    return f.rgba()


def turbine_top():
    f = Face(144, STEEL, 7)
    _recess(f)
    f.fan((64, 64), 44, 8, STEEL, AETHER, 0.7)
    f.bolts()
    return f.rgba()


def crusher_side():
    f = Face(145, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.hazard((10, 92, 118, 112), GOLD, DARK, 18, 0.6)
    f.rect((26, 20, 102, 80), DARK, 0.25, -1.0, 5)
    for i in range(5):
        x = 34 + i * 16
        f.bar((x, 26), (x + 8, 72), 7, STEEL, 0.6)
    return f.rgba()


def crusher_top():
    f = Face(146, STEEL, 7)
    _recess(f)
    f.rect((26, 26, 102, 102), STEEL, 0.4, -1.0, 4)
    for i in range(4):
        for j in range(4):
            f.disc((40 + i * 16, 40 + j * 16), 6, DARK, 0.3, -1.0, 3)
    f.bolts()
    return f.rgba()


def silo_side():
    f = Face(147, CONCRETE, 7)
    f.pipe((0, 0, N, N), False, CONCRETE, 0.55)
    for y in (4, 62, 120):
        f.rect((0, y - 3, N, y + 4), STEEL, 0.45, 1.0, 2, outline=False)
    f.rect((50, 84, 78, 112), STEEL, 0.45, -1.0, 4)
    f.disc((64, 98), 8, DARK, 0.3, -1.0, 3)
    f.gauge((64, 34), 16, 0.7)
    return f.rgba()


def silo_top():
    f = Face(148, STEEL, 7)
    _recess(f)
    f.disc((64, 64), 44, CONCRETE, 0.5, 1.0, 8)
    f.disc((64, 64), 22, STEEL, 0.5, -1.0, 5)
    f.disc((64, 64), 8, BRONZE, 0.55, 1.0, 3)
    return f.rgba()


def arc_side():
    f = Face(151, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.stripe((10, 24, 118, 40), VIOLET)
    f.slots((24, 56, 104, 100), 5, False, VIOLET, 0.8)
    f.bolts()
    return f.rgba()


def arc_top():
    f = Face(152, STEEL, 7)
    _recess(f)
    for x in (40, 88):
        f.port((x, 64), 22, EMBER, 0.85, STEEL)
    f.bar((40, 64), (88, 64), 5, VIOLET, 0.7, 0.9)
    return f.rgba()


def solar_top():
    f = Face(158, STEEL, 7)
    f.rect((8, 8, 120, 120), STEEL, 0.4, 1.0, 3)
    for cx in range(3):
        for cy in range(3):
            x0, y0 = 14 + cx * 36, 14 + cy * 36
            f.rect((x0, y0, x0 + 32, y0 + 32), BLUE, 0.35 + 0.1 * ((cx + cy) % 2), -1.0, 4, 0.08)
            f.bar((x0 + 16, y0 + 3), (x0 + 16, y0 + 29), 2, STEEL, 0.8)
    return f.rgba()


def solar_side():
    f = Face(159, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.slots((24, 30, 104, 48), 2, False, DARK)
    f.lines((24, 66, 104, 104), 3, STEEL, 0.8)
    f.bolts()
    return f.rgba()


def accumulator_top():
    f = Face(160, STEEL, 7)
    _recess(f)
    for x in (40, 88):
        f.disc((x, 64), 20, BRONZE, 0.5, 1.0, 5)
        f.disc((x, 64), 11, EMBER, 0.6, -1.0, 4, 0.6)
    f.bolts()
    return f.rgba()


def accumulator_side():
    f = Face(161, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.rect((22, 36, 106, 92), DARK, 0.25, -1.0, 5)
    for i in range(4):
        f.rect((30 + i * 19, 44, 44 + i * 19, 84), GREEN, 0.65, -1.0, 3, 0.9)
    f.bolts()
    return f.rgba()
