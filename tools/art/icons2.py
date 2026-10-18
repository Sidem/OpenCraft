"""More item icons (see `icons.py` for the helpers): vehicles and the drone port, the rocket's parts and the satellites,
and the coin, wafer, accelerator, fuel cell and battery.
"""
import numpy as np

from icons import PCB, TEAL, WHITE, canvas, poly, shape
from kit import BLUE, DARK, GOLD, GREEN, RED, VIOLET, XS, YS, TIERS
from lib import N, fbm, paint, ramp
from machines import AETHER, AMBER, BRONZE, COPPER, EMBER, STEEL

LOCO = ramp(["#06140e", "#0f2e20", "#1a5236", "#2c7a4c", "#4ca470"])
GRAVEL = ramp(["#0c0e14", "#1c2030", "#2e3446", "#454d62", "#646e86"])
PANEL = ramp(["#050a22", "#0c1c5a", "#1a3a9c", "#3a68d0", "#86a8f0"])


def rail(seed=193):
    """A rail tile seen from above: ballast, wooden sleepers and two steel rails."""
    f = canvas(seed)
    X, Y = f.X, f.Y
    f.img[:] = paint(GRAVEL, 0.3 + 0.4 * fbm(X, Y, 28, 2, seed + 2), 0.4)
    for y in range(8, N, 22):
        f.rect((6, y, 122, y + 12), ramp(["#1e120a", "#4a2c18", "#7a4c26", "#a87440"]), 0.5, 1.0, 3)
    for x in (30, 90):
        f.bar((x, 0), (x, N), 8, STEEL, 0.65)
        f.bar((x - 2, 0), (x - 2, N), 2, STEEL, 0.95)
    return f.rgba()


def loco(seed=194):
    f = canvas(seed)
    f.rect((14, 52, 92, 88), LOCO, 0.5, 1.0, 6)           # the boiler
    f.rect((14, 52, 92, 62), BRONZE, 0.6, 1.0, 3)          # brass band
    f.rect((92, 38, 116, 88), LOCO, 0.45, 1.0, 5)          # the cab
    f.rect((98, 44, 110, 58), AMBER, 0.7, -1.0, 3, 0.9)
    f.rect((22, 34, 38, 52), STEEL, 0.4, 1.0, 3)           # chimney
    for x in (30, 58, 88):
        f.disc((x, 96), 13, STEEL, 0.4, 1.0, 4)
        f.disc((x, 96), 5, BRONZE, 0.6, 1.0, 2)
    return f.rgba()


def wagon(seed=195):
    f = canvas(seed)
    f.rect((14, 40, 114, 90), STEEL, 0.4, 1.0, 6)
    f.rect((14, 40, 114, 54), VIOLET, 0.6, 1.0, 3, 0.4)
    for x in (34, 54, 74, 94):
        f.bar((x, 58), (x, 86), 2, STEEL, 0.7)
    for x in (32, 96):
        f.disc((x, 98), 12, STEEL, 0.4, 1.0, 4)
    return f.rgba()


def hover_pack(seed=200):
    f = canvas(seed)
    f.rect((34, 18, 94, 80), WHITE, 0.55, 1.0, 7)
    f.rect((34, 34, 94, 42), GREEN, 0.7, 1.0, 2, 0.4)
    for x in (28, 76):
        f.rect((x, 82, x + 24, 96), STEEL, 0.5, 1.0, 3)
        shape(f, poly([(x + 3, 96), (x + 21, 96), (x + 12, 116)]), AETHER, 0.8, b=3, glow=1.0)
    return f.rgba()


def cargo_drone(seed=201):
    f = canvas(seed)
    for sx in (-1, 1):
        for sy in (-1, 1):
            tip = (64 + sx * 40, 64 + sy * 40)
            f.bar((64, 64), tip, 7, STEEL, 0.5)
            f.disc(tip, 15, AETHER, 0.5, -1.0, 4, 0.6)
    f.rect((40, 40, 88, 88), BRONZE, 0.5, 1.0, 5)
    f.rect((52, 52, 76, 76), EMBER, 0.65, -1.0, 4, 0.8)
    return f.rgba()


def port_top(seed=173):
    f = canvas(seed)
    f.rect((6, 6, 122, 122), STEEL, 0.3, 1.0, 5)
    f.disc((64, 64), 50, EMBER, 0.55, 1.0, 5, 0.7)
    f.disc((64, 64), 40, DARK, 0.25, -1.0, 4)
    for box in ((46, 36, 56, 92), (72, 36, 82, 92), (56, 58, 72, 68)):
        f.rect(box, WHITE, 0.8, 1.0, 2, 0.5)
    return f.rgba()


def port_side(seed=174):
    f = canvas(seed)
    f.rect((6, 6, 122, 122), STEEL, 0.3, 1.0, 5)
    f.hazard((6, 44, 122, 84), AMBER, DARK, 22, 0.6)
    return f.rgba()


# ---------------------------------------------------------------------------------------------------- the rocket

def rocket_engine(seed=289):
    f = canvas(seed)
    f.rect((38, 14, 90, 40), STEEL, 0.5, 1.0, 5)
    shape(f, poly([(42, 40), (86, 40), (108, 108), (20, 108)]), STEEL, 0.5, b=7)
    f.bar((44, 52), (36, 100), 2, STEEL, 0.9)
    f.rect((18, 100, 110, 112), EMBER, 0.6, 1.0, 3, 0.9)
    f.disc((64, 108), 14, AMBER, 0.9, -1.0, 4, 1.0)
    return f.rgba()


def fuel_tank(seed=290):
    """White tank skin: a cylinder shaded across its width with a red band and a seam."""
    f = canvas(seed)
    f.pipe((0, 0, N, N), True, WHITE, 0.5)
    f.rect((0, 56, N, 76), RED, 0.55, 1.0, 3, outline=False)
    return f.rgba()


def avionics(seed=291):
    f = canvas(seed)
    f.rect((10, 10, 118, 118), PCB, 0.4, 1.0, 5)
    for i in range(1, 6):
        f.bar((10 + i * 18, 14), (10 + i * 18, 114), 2, ramp(["#06200f", "#2a8a3a", "#7ae890"]), 0.8, 0.3)
        f.bar((14, 10 + i * 18), (114, 10 + i * 18), 2, ramp(["#06200f", "#2a8a3a", "#7ae890"]), 0.8, 0.3)
    f.rect((42, 42, 86, 86), GOLD, 0.6, 1.0, 5, 0.3)
    f.rect((54, 54, 74, 74), DARK, 0.3, -1.0, 3)
    return f.rgba()


def satellite(seed, body, light, bus_glow=0.8):
    f = canvas(seed)
    for sx in (-1, 1):
        x0 = 64 + sx * 14
        for k in range(2):
            lo, hi = sorted((x0 + sx * (k * 24), x0 + sx * (24 + k * 24 - 2)))
            f.rect((lo, 46, hi, 82), PANEL, 0.5, 1.0, 3)
    f.rect((46, 36, 82, 92), body, 0.55, 1.0, 5)
    f.rect((56, 56, 72, 72), light, 0.8, -1.0, 3, bus_glow)
    return f.rgba()


def comms_satellite(seed=295):
    f = canvas(seed)
    shape(f, poly([(24, 60), (64, 28), (104, 60), (64, 76)]), WHITE, 0.6, b=6)
    f.bar((64, 60), (64, 100), 5, STEEL, 0.6)
    f.rect((50, 96, 78, 112), STEEL, 0.5, 1.0, 4)
    f.disc((64, 52), 5, RED, 0.8, 1.0, 2, 0.9)
    return f.rgba()


def drop_pod(seed=302):
    f = canvas(seed)
    shape(f, poly([(64, 12), (92, 40), (100, 96), (28, 96), (36, 40)]), WHITE, 0.6, b=8)
    f.disc((64, 58), 14, DARK, 0.2, -1.0, 4)
    f.disc((64, 58), 9, AETHER, 0.7, -1.0, 3, 0.7)
    f.rect((24, 96, 104, 110), STEEL, 0.45, 1.0, 4)
    return f.rgba()


# ---------------------------------------------------------------------------------------------------- small things

def coin(seed=243):
    f = canvas(seed)
    f.disc((64, 64), 50, GOLD, 0.55, 1.0, 7)
    f.disc((64, 64), 40, GOLD, 0.45, -1.0, 5)
    for i in range(3):
        ang = i * 2 * np.pi / 3 - np.pi / 2
        a, b = (64 + np.cos(ang) * 24, 64 + np.sin(ang) * 24), (64 + np.cos(ang + 2.09) * 24, 64 + np.sin(ang + 2.09) * 24)
        f.bar(a, b, 6, GOLD, 0.95)
    return f.rgba()


def wafer(seed=247):
    f = canvas(seed)
    f.disc((64, 64), 50, ramp(["#10162a", "#34447a", "#6a82c0", "#a8bce8"]), 0.5, 1.0, 6)
    d = np.hypot(XS - 63.5, YS - 63.5)
    grid = (((XS % 16) < 2) | ((YS % 16) < 2)) & (d < 44)
    f.fill(grid, ramp(["#10162a", "#34447a", "#6a82c0"]), 0.15)
    f.rect((58, 98, 72, 108), DARK, 0.3, 1.0, 2)
    return f.rgba()


def accelerator(seed=248):
    f = canvas(seed)
    f.rect((10, 10, 118, 118), PCB, 0.45, 1.0, 5)
    f.disc((64, 64), 40, GOLD, 0.55, 1.0, 5, 0.3)
    f.disc((64, 64), 30, DARK, 0.2, -1.0, 4)
    f.disc((64, 64), 14, AETHER, 0.8, 1.0, 5, 1.0)
    for a in range(8):
        ang = a * np.pi / 4
        f.bar((64 + np.cos(ang) * 40, 64 + np.sin(ang) * 40), (64 + np.cos(ang) * 54, 64 + np.sin(ang) * 54), 4, GOLD, 0.6)
    return f.rgba()


def scrap(seed=329):
    f = canvas(seed)
    rust = ramp(["#1a0b0a", "#4a2018", "#7a3a22", "#a8603a", "#d09060"])
    for pts, tones in (([(20, 96), (56, 52), (74, 92)], rust), ([(50, 100), (82, 40), (110, 98)], STEEL),
                       ([(30, 70), (66, 34), (84, 70), (52, 84)], rust), ([(76, 104), (98, 70), (116, 104)], rust)):
        shape(f, poly(pts), tones, 0.5, b=4)
    f.bar((34, 78), (54, 56), 3, STEEL, 0.7)
    return f.rgba()


def fuel_cell(seed=236):
    f = canvas(seed)
    f.rect((22, 20, 106, 32), GOLD, 0.6, 1.0, 3)
    f.rect((22, 96, 106, 108), GOLD, 0.6, 1.0, 3)
    f.rect((30, 32, 98, 96), DARK, 0.3, -1.0, 5)
    f.rect((40, 36, 88, 92), ramp(["#0a2a08", "#2a8a10", "#80e030", "#d0ff80"]), 0.6, 1.0, 5, 0.9)
    return f.rgba()


def battery(seed=190):
    f = canvas(seed)
    f.rect((34, 28, 94, 112), GREEN, 0.45, 1.0, 7)
    f.rect((50, 14, 78, 28), STEEL, 0.7, 1.0, 3)
    f.rect((34, 28, 94, 44), WHITE, 0.6, 1.0, 3)
    f.rect((60, 56, 68, 92), DARK, 0.2, 1.0, 2, 0.0)
    f.rect((46, 68, 82, 80), DARK, 0.2, 1.0, 2, 0.0)
    return f.rgba()
