"""Wave 3 of the industry faces: the power grid's colours, the compute campus (fibre node, datacenter, cooling tower, AI lab,
optimizer), lasers and data links, the swarm hub, the orbital launch set, and the locomotive's paint. Same kit as
`industry.py`; violet and aether are the far-tech accents, red marks the laser, gold the launch pad.
"""
import numpy as np

from kit import AETHER, BLUE, BRONZE, CONCRETE, DARK, EMBER, GOLD, GREEN, RED, STEEL, VIOLET, XS, YS, Face
from lib import N, paint, ramp, rgba
from machines import AMBER

PINK = ramp(["#2a0a1e", "#6b1650", "#b02a84", "#e860b4", "#ffa8d8", "#ffe4f2"])
GRID_RAMPS = (BLUE, EMBER, PINK, GREEN, GOLD, RED)


def _recess(f, box=(14, 14, 114, 114)):
    return f.rect(box, DARK, 0.28, -1.0, 5)


def grid_colour(i):
    """A plate in a grid's colour, lit from within so a line of them reads at night."""
    f = Face(251 + i, STEEL, 7)
    f.rect((12, 12, 116, 116), GRID_RAMPS[i], 0.6, 1.0, 8, 0.5, 0.6)
    f.disc((64, 64), 22, GRID_RAMPS[i], 0.85, -1.0, 8, 0.8)
    return f.rgba()


def node_side():
    f = Face(257, STEEL, 7)
    _recess(f)
    for i in range(4):
        f.bar((26, 30 + i * 22), (102, 30 + i * 22), 8, AETHER, 0.7, 0.9)
    f.bolts()
    return f.rgba()


def node_top():
    f = Face(258, STEEL, 7)
    _recess(f)
    f.port((64, 64), 40, AETHER, 0.95, STEEL)
    return f.rgba()


def fibre():
    """The light of a fibre line: pale cyan, banded, fully self-lit."""
    f = Face(259, STEEL, 1)
    f.rect((0, 0, N, N), AETHER, 0.6, 0.0, 2, 1.0, 0.3, outline=False)
    for y in range(8, N, 16):
        f.rect((0, y, N, y + 4), AETHER, 0.85, 0.0, 2, 1.0, 0.0, outline=False)
    return f.rgba()


def dc_side():
    f = Face(260, STEEL, 7)
    _recess(f)
    rng = np.random.default_rng(260)
    for r in range(5):
        for c in range(5):
            on = rng.random() > 0.35
            f.rect((24 + c * 17, 24 + r * 17, 34 + c * 17, 30 + r * 17), GREEN if on else STEEL, 0.7 if on else 0.2, 1.0, 2, 0.9 if on else None)
    f.bolts()
    return f.rgba()


def dc_top():
    f = Face(261, STEEL, 7)
    _recess(f)
    f.rect((22, 22, 106, 106), STEEL, 0.35, -1.0, 4)
    for i in range(6):
        f.bar((26, 28 + i * 13), (102, 28 + i * 13), 5, DARK, 0.3)
        f.bar((26 + i * 15, 26), (26 + i * 15, 102), 3, STEEL, 0.6)
    return f.rgba()


def tower_side():
    f = Face(262, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.slots((18, 18, 110, 110), 9, False, DARK, None, 0.28)
    return f.rgba()


def tower_top():
    f = Face(263, STEEL, 7)
    _recess(f)
    f.fan((64, 64), 46, 5, STEEL, DARK)
    return f.rgba()


def ai_side():
    f = Face(264, STEEL, 7)
    _recess(f)
    pts = [(34, 38), (34, 64), (34, 90), (64, 30), (64, 56), (64, 82), (64, 106 - 6), (94, 44), (94, 74)]
    for a in pts[:3]:
        for b in pts[3:7]:
            f.bar(a, b, 2, AETHER, 0.6, 0.6)
    for a in pts[3:7]:
        for b in pts[7:]:
            f.bar(a, b, 2, AETHER, 0.6, 0.6)
    for p in pts:
        f.disc(p, 6, AETHER, 0.8, 1.0, 3, 1.0)
    return f.rgba()


def ai_top():
    f = Face(265, STEEL, 7)
    _recess(f)
    f.disc((64, 64), 42, AETHER, 0.4, 1.0, 5, 0.6)
    f.disc((64, 64), 32, DARK, 0.2, -1.0, 4)
    f.disc((64, 64), 14, AETHER, 0.85, 1.0, 6, 1.0)
    return f.rgba()


def opt_side():
    f = Face(266, STEEL, 7)
    _recess(f)
    for i in range(3):
        f.stripe((20, 24 + i * 30, 108, 40 + i * 30), AMBER, 0.6)
    f.bolts()
    return f.rgba()


def opt_top():
    f = Face(267, STEEL, 7)
    _recess(f)
    f.disc((64, 64), 40, GOLD, 0.5, 1.0, 5, 0.3)
    f.disc((64, 64), 24, DARK, 0.2, -1.0, 4)
    f.disc((64, 64), 12, AMBER, 0.85, 1.0, 5, 1.0)
    return f.rgba()


def laser_side(receiver):
    f = Face(268 + 2 * int(receiver), STEEL, 7)
    _recess(f)
    if receiver:
        f.rect((26, 26, 102, 102), BLUE, 0.35, -1.0, 4, 0.3)
        for i in range(4):
            f.bar((26 + i * 25, 26), (26 + i * 25, 102), 3, AETHER, 0.7, 0.6)
            f.bar((26, 26 + i * 25), (102, 26 + i * 25), 3, AETHER, 0.7, 0.6)
    else:
        f.bar((14, 64), (114, 64), 22, RED, 0.5, 0.9)
        f.bar((14, 64), (114, 64), 6, RED, 0.95, 1.0)
    f.bolts()
    return f.rgba()


def laser_top(receiver):
    f = Face(269 + 2 * int(receiver), STEEL, 7)
    _recess(f)
    f.port((64, 64), 42, BLUE if receiver else RED, 0.95, STEEL)
    f.disc((64, 64), 12, STEEL, 0.9, 1.0, 4, 1.0)
    return f.rgba()


def beam():
    f = Face(272, STEEL, 1)
    f.rect((0, 0, N, N), RED, 0.6, 0.0, 2, 1.0, 0.3, outline=False)
    f.rect((40, 0, 88, N), RED, 0.8, 0.0, 2, 1.0, 0.1, outline=False)
    return f.rgba()


def mirror():
    f = Face(276, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.75, -1.0, 4, None, 0.5)
    for k in (-30, 20, 70):
        f.bar((10, 40 + k), (118, k - 20), 14, STEEL, 0.95)
    return f.rgba()


def data_side():
    f = Face(277, STEEL, 7)
    _recess(f)
    f.rect((24, 24, 104, 104), STEEL, 0.4, -1.0, 4)
    for i in range(4):
        f.rect((32, 32 + i * 18, 96, 42 + i * 18), AETHER if i % 2 == 0 else GREEN, 0.7, 1.0, 2, 0.85)
    return f.rgba()


def data_top():
    f = Face(278, STEEL, 7)
    _recess(f)
    f.disc((64, 64), 40, STEEL, 0.5, 1.0, 5)
    f.disc((64, 64), 28, BLUE, 0.5, -1.0, 6, 0.7)
    f.disc((64, 64), 10, AETHER, 0.9, 1.0, 4, 1.0)
    return f.rgba()


def hub_side():
    f = Face(279, STEEL, 7)
    _recess(f)
    for i in range(3):
        f.bar((18, 34 + i * 30), (110, 34 + i * 30), 6, AETHER, 0.6, 0.9)
        for x in (30, 64, 98):
            f.disc((x, 34 + i * 30), 7, DARK, 0.2, -1.0, 3)
    return f.rgba()


def hub_top():
    f = Face(280, STEEL, 7)
    _recess(f)
    f.port((64, 64), 42, AETHER, 1.0, STEEL)
    f.disc((64, 64), 12, STEEL, 0.9, 1.0, 4, 1.0)
    return f.rgba()


def launch_side():
    f = Face(286, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.45, 1.0, 4)
    f.hazard((10, 96, 118, 114), GOLD, DARK, 18, 0.6)
    f.bar((20, 20), (108, 90), 4, STEEL, 0.6)
    f.bar((108, 20), (20, 90), 4, STEEL, 0.6)
    return f.rgba()


def launch_top():
    f = Face(287, STEEL, 7)
    _recess(f)
    f.port((64, 64), 44, EMBER, 0.9, GOLD)
    f.disc((64, 64), 16, DARK, 0.2, -1.0, 4)
    return f.rgba()


def light_frame():
    f = Face(288, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.65, 1.0, 5)
    f.bar((16, 16), (112, 112), 6, STEEL, 0.5)
    f.bar((112, 16), (16, 112), 6, STEEL, 0.5)
    return f.rgba()


def rocket_hull():
    f = Face(293, STEEL, 1)
    f.pipe((0, 0, N, N), True, STEEL, 0.75)
    f.rect((0, 20, N, 40), RED, 0.55, 1.0, 3, outline=False)
    f.rect((0, 88, N, 108), RED, 0.55, 1.0, 3, outline=False)
    return f.rgba()


def rectenna_top():
    f = Face(297, STEEL, 7)
    _recess(f)
    for i in range(7):
        f.bar((20 + i * 15, 18), (20 + i * 15, 110), 3, AMBER, 0.6, 0.5)
        f.bar((18, 20 + i * 15), (110, 20 + i * 15), 3, AMBER, 0.6, 0.5)
    return f.rgba()


def rectenna_side():
    f = Face(298, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.hazard((10, 94, 118, 112), AMBER, DARK, 18, 0.6)
    f.slots((22, 24, 106, 80), 4, False, DARK)
    return f.rgba()


def ground_top():
    f = Face(300, STEEL, 7)
    _recess(f)
    for r in (46, 34, 22, 10):
        f.disc((64, 64), r, STEEL, 0.45 + (r % 20) * 0.01, 1.0 if r % 24 else -1.0, 4)
    return f.rgba()


def ground_side():
    f = Face(301, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.4, 1.0, 4)
    f.hazard((10, 94, 118, 112), BLUE, DARK, 18, 0.6)
    f.slots((22, 24, 106, 80), 3, False, DARK)
    return f.rgba()


def pod_side():
    f = Face(303, STEEL, 7)
    f.rect((10, 10, 118, 118), STEEL, 0.45, 1.0, 4)
    f.hazard((10, 94, 118, 112), AETHER, DARK, 18, 0.6)
    f.slots((22, 24, 106, 80), 3, False, DARK)
    return f.rgba()


def pod_launch_top():
    f = Face(304, STEEL, 7)
    _recess(f)
    f.port((64, 64), 42, AETHER, 0.9, STEEL)
    f.disc((64, 64), 14, DARK, 0.2, -1.0, 4)
    return f.rgba()


def pod_pad_top():
    f = Face(305, STEEL, 7)
    _recess(f)
    for r in (46, 32, 18):
        f.disc((64, 64), r, AETHER if r == 32 else STEEL, 0.5, 1.0 if r != 32 else -1.0, 4, 0.6 if r == 32 else None)
    f.disc((64, 64), 8, STEEL, 0.9, 1.0, 3, 1.0)
    return f.rgba()


def loco_body():
    f = Face(283, STEEL, 1)
    f.pipe((0, 0, N, N), False, GREEN, 0.45)
    f.rect((0, 52, N, 76), GOLD, 0.55, 1.0, 3, outline=False)
    return f.rgba()


def soot():
    f = Face(284, STEEL, 1)
    f.rect((0, 0, N, N), DARK, 0.75, 0.0, 2, None, 0.8, outline=False)
    return f.rgba()
