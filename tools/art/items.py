"""Item materials (docs/ART_STYLE.md): the surfaces that skin small models and machine lights, not pictures. Metals are
bevelled slabs with a brushed grain and a stamp; rubble (crushed, washed ores, tailings, sulfur, plastic) is a field of lit
grains; science packs are glowing liquid under a glass meniscus; the avatar's parts, the status lamps, the tier stripes, the
ladder and smoke are small recipes of their own. Pictures (tools, canisters, robot parts) are in `icons.py`.
"""
import numpy as np

from draw import Pen, curve
from icons import IRON_HEAD, STEEL_HEAD, TEAL, WOOD, WHITE
from kit import BLUE, DARK, GOLD, GREEN, RED, VIOLET, XS, YS, TIERS, Face
from lib import N, bayer, coords, fbm, hash2, paint, ramp, relief_light, rgba, roll, worley
from machines import AETHER, AMBER, BRONZE, COPPER, EMBER, STEEL

ALUMINIUM = ramp(["#2a3346", "#56657f", "#8ea0bc", "#c4d2e6", "#f2f8ff"])
SMOKE_TONES = ramp(["#2a2e3a", "#555b6c", "#8a90a2", "#bcc0cc", "#e4e6ee"])


# ---------------------------------------------------------------------------------------------------- metals

def ingot(tones, seed):
    """A cast bar's surface: bevelled, a pressed panel in the top, a bright edge, a few stamped marks."""
    f = Face(seed, tones, 6)
    f.rect((18, 20, 110, 108), tones, 0.5, -1.0, 4)
    f.bar((12, 12), (116, 12), 3, tones, 0.95)
    f.lines((28, 40, 100, 90), 3, tones, 0.25)
    return f.rgba()


def plate_item(tones, seed):
    f = Face(seed, tones, 5)
    f.rect((12, 12, 116, 116), tones, 0.5, -1.0, 3, grain=0.25)
    f.bolts(((20, 20), (108, 20), (20, 108), (108, 108)), 3, tones)
    return f.rgba()


def wire(seed=28):
    """Copper windings: each turn a round tube, lit on its left, with a dark gap and a little wander."""
    X, Y = coords()
    phase = ((XS + 3 * np.sin(Y * 6 + seed)) % 12) / 12
    tube = 0.25 + 0.65 * np.sin(np.pi * phase) ** 0.8
    img = paint(COPPER, tube + (fbm(X, Y, 12, 2, seed) - 0.5) * 0.12, 0.25)
    img[phase < 0.07] = COPPER[0]
    return rgba(img)


def rod(seed=70):
    f = Face(seed, IRON_HEAD, 1)
    f.pipe((0, 0, N, N), False, IRON_HEAD, 0.6)
    groove = ((XS + YS) % 22) < 3
    f.fill(groove, IRON_HEAD, 0.12, grain=0.05)
    return f.rgba()


def screw(seed=71):
    f = Face(seed, IRON_HEAD, 1)
    f.pipe((0, 0, N, N), True, IRON_HEAD, 0.62)
    thread = ((XS + YS * 2) % 20) < 7
    f.fill(thread, IRON_HEAD, 0.14, grain=0.05)
    return f.rgba()


def beam(seed=133):
    f = Face(seed, STEEL_HEAD, 4)
    f.rect((0, 0, N, 34), STEEL_HEAD, 0.65, 1.0, 5)
    f.rect((0, 94, N, N), STEEL_HEAD, 0.55, 1.0, 5)
    f.rect((0, 40, N, 88), STEEL_HEAD, 0.4, -1.0, 4)
    f.bolts(((20, 17), (64, 17), (108, 17), (20, 111), (64, 111), (108, 111)), 3, STEEL_HEAD)
    return f.rgba()


def forged(seed=137):
    """The steel tool head: hammered blue steel, shallow dents and a bright worked edge."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, 9, seed, 1.0)
    dent = (f1 < 0.3) & (pid > 0.45)
    h = np.where(dent, f1 / 0.3, 1.0) * 0.6
    light = relief_light(h * 3, 1.0)
    v = 0.5 + (light - 0.5) * 0.7 + (fbm(X, Y, 18, 3, seed + 1) - 0.5) * 0.3
    v = np.where(dent & (f1 > 0.27), v - 0.2, v)
    return rgba(paint(STEEL_HEAD, v, 0.3))


def slag(seed=130):
    """Black glassy slag, crazed with violet flecks that glimmer."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, 8, seed, 1.0)
    light = relief_light((np.minimum((f2 - f1) / 0.5, 1) + pid * 0.2) * 4, 1.0)
    v = 0.3 + (light - 0.5) * 0.6 + (fbm(X, Y, 20, 2, seed + 1) - 0.5) * 0.2
    img = paint(ramp(["#05040a", "#0f0b1a", "#1c1530", "#2e2250", "#4a3a7a"]), v, 0.3)
    fleck = hash2((X * N).astype(int) // 2, (Y * N).astype(int) // 2, seed + 2) > 0.93
    img[fleck] = ramp(["#6a4ab0", "#b090f0"])[1]
    glow = np.where(fleck, 0.35, 0.0)
    return rgba(img, glow=glow)


def silicon(seed=153):
    """Crystalline silicon: big faceted grains of blue-grey, each its own sheen."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, 4, seed, 0.8)
    rings = np.sin(f1 * 70) * 0.5 + 0.5  # growth lines inside each crystal
    v = 0.28 + pid * 0.4 + rings * 0.1 + (fbm(X, Y, 20, 2, seed) - 0.5) * 0.1
    img = paint(ramp(["#0c1224", "#243252", "#4c6290", "#8098c8", "#d0def8"]), np.where(f2 - f1 < 0.03, 0.05, v), 0.3)
    glint = (hash2((X * N).astype(int), (Y * N).astype(int), seed + 3) > 0.985) & (f2 - f1 > 0.05)
    img[glint] = 255
    return rgba(img)


# ---------------------------------------------------------------------------------------------------- rubble

def rubble(base, fleck, seed, cells=12, fleck_frac=0.25, shine=0.0, gap=0.07):
    """A field of lit grains in `base`, a share of them in `fleck`; `shine` adds bright glints (wet or crystalline)."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, cells, seed, 1.0)
    edge = f2 - f1
    light = relief_light((np.minimum(edge / 0.4, 1) + pid * 0.25) * 5, 1.1)
    v = 0.3 + pid * 0.3 + (light - 0.5) * 0.75 + (fbm(X, Y, 26, 2, seed + 1) - 0.5) * 0.15
    is_fleck = hash2((pid * 977).astype(int), 3, seed + 2) < fleck_frac
    img = np.where(is_fleck[..., None], paint(fleck, v, 0.3), paint(base, v, 0.3))
    img = np.where((edge < gap)[..., None], base[0], img)
    if shine:
        img[(light > 0.82) & (hash2((X * N).astype(int), (Y * N).astype(int), seed + 3) > 1 - shine)] = base[-1]
    return rgba(img)


# ---------------------------------------------------------------------------------------------------- science

def pack(tones, seed):
    """The liquid inside a science flask: a glass meniscus at the top, dark glass at the sides, light in the middle."""
    X, Y = coords()
    f = Face(seed, tones, 1)
    swirl = fbm(X, Y, 5, 3, seed + 1, 0.5, 0.5)
    v = 0.55 + (swirl - 0.5) * 0.45 - (np.abs(X - 0.5) * 0.9) ** 2 * 1.1
    img = paint(tones, np.clip(v, 0, 1), 0.3)
    side = (XS < 10) | (XS > N - 11)
    img[side] = tones[1]
    img[(XS < 4) | (XS > N - 5)] = tones[0]
    b1, _, bid = worley(X, Y, 7, seed + 4, 0.8)
    bubble = (b1 < 0.1) & (bid > 0.55)
    img[bubble & ~side & (YS > 30) & (b1 > 0.06)] = tones[-2]
    img[bubble & ~side & (YS > 30) & (b1 <= 0.06)] = tones[2]
    top = YS < 22
    glass = paint(ramp(["#2c4a54", "#7ab0b8", "#c4eaee", "#f0ffff"]), 0.45 + 0.45 * np.cos(X * 9) ** 2, 0.2)
    img[top] = glass[top]
    img[(YS == 22) & ~side] = tones[-1]
    glow = np.where(top, 0.0, 0.4) * (~side)
    return rgba(img, glow=glow)


def flask_glass(seed=72):
    X, Y = coords()
    v = 0.35 + 0.35 * np.cos(X * 14) ** 4 - (np.abs(Y - 0.5)) * 0.2 + (fbm(X, Y, 8, 2, seed) - 0.5) * 0.12
    return rgba(paint(ramp(["#16323a", "#4a8c98", "#9ad0d8", "#d8f6f8", "#ffffff"]), np.clip(v, 0, 1), 0.3))


# ---------------------------------------------------------------------------------------------------- lights and bands

def lamp_light(tones, seed):
    """A status lens: lit from within, brightest at the middle, a dark bezel at the edge."""
    X, Y = coords()
    r = np.hypot(X - 0.5, Y - 0.5) * 2
    v = np.clip(1.0 - r * 0.9 + (fbm(X, Y, 6, 2, seed) - 0.5) * 0.2, 0, 1)
    img = paint(tones, v, 0.2)
    rim = np.minimum(np.minimum(XS, N - 1 - XS), np.minimum(YS, N - 1 - YS)) < 3
    img[rim] = tones[0]
    return rgba(img, glow=np.where(rim, 0.0, 0.95))


def stripe(tier, seed):
    f = Face(seed, STEEL, 6)
    f.rect((6, 40, 122, 88), TIERS[tier], 0.62, 1.0, 5, 0.45)
    f.pips(tier, 58, 64, 8, 5)
    return f.rgba()


def scanner(tones, seed):
    """A scanner screen: concentric sweep arcs above a ping, on a dark glass."""
    f = Face(seed, STEEL, 1)
    f.img[:] = paint(ramp(["#02060a", "#061218", "#0c2028"]), 0.3 + 0.3 * fbm(f.X, f.Y, 5, 2, seed), 0.3)
    d = np.hypot(XS - 63.5, YS - 112.0)
    up = YS < 112
    for r in (30, 54, 78):
        f.fill((np.abs(d - r) < 2.4) & up, tones, 0.7, 0.9)
    f.bar((64, 112), (100, 40), 2, tones, 0.8, 0.8)  # the sweep
    f.disc((86, 70), 5, tones, 0.9, 1.0, 3, 1.0)
    bezel = np.minimum(np.minimum(XS, N - 1 - XS), np.minimum(YS, N - 1 - YS)) < 5
    f.fill(bezel, tones, 0.12, None, 0.05)
    return f.rgba()


def core_drill(seed=69):
    f = Face(seed, COPPER, 5)
    f.rect((10, 10, 118, 118), COPPER, 0.5, 1.0, 5)
    f.rect((10, 10, 118, 26), WHITE, 0.6, 1.0, 3)
    f.rect((10, 102, 118, 118), WHITE, 0.55, 1.0, 3)
    f.bolts(((22, 44), (106, 44), (22, 84), (106, 84)), 3, COPPER)
    return f.rgba()


def fog(seed=165):
    X, Y = coords()
    return rgba(paint(ramp(["#10141e", "#1a2030", "#262e44"]), 0.45 + (fbm(X, Y, 4, 2, seed) - 0.5) * 0.2, 0.3))


# ---------------------------------------------------------------------------------------------------- the avatar

def suit(seed=47):
    """Deep teal fabric: woven, with a seam down the middle and darker stitched panels."""
    X, Y = coords()
    weave = ((XS % 4 < 2) ^ (YS % 4 < 2)) * 0.06
    v = 0.4 + weave + (fbm(X, Y, 6, 3, seed) - 0.5) * 0.25
    img = paint(TEAL, v, 0.3)
    seam = (np.abs(XS - 63) < 2) | (np.abs(YS - 63) < 2)
    img[seam] = TEAL[1]
    img[(np.abs(XS - 63) < 1) & (YS % 6 < 3)] = TEAL[4]
    return rgba(img)


def skin(seed=48):
    """The robot's body shell: matt cream ceramic with soft panel lines."""
    X, Y = coords()
    v = 0.44 + (fbm(X, Y, 5, 3, seed) - 0.5) * 0.2
    img = paint(ramp(["#4a4a42", "#8a8878", "#c4c0a8", "#e6e2c8", "#f8f4dc"]), v, 0.3)
    img[(XS % 64 == 0) | (YS % 64 == 0)] = ramp(["#4a4a42", "#8a8878"])[0]
    return rgba(img)


def helmet(seed=49):
    f = Face(seed, BRONZE, 6)
    f.rect((12, 12, 116, 116), ramp(["#4a1c08", "#a8480c", "#e87a20", "#ffb050", "#ffe8b0"]), 0.55, 1.0, 6)
    f.bar((22, 100), (100, 24), 6, ramp(["#a8480c", "#e87a20", "#ffe8b0"]), 0.9)
    return f.rgba()


def visor(seed=50):
    X, Y = coords()
    v = 0.18 + 0.5 * np.clip(np.sin((X + Y) * 7) ** 8, 0, 1) * 0.5 + (fbm(X, Y, 5, 2, seed) - 0.5) * 0.2
    img = paint(ramp(["#02080c", "#06202a", "#0d4a58", "#3fd0d0", "#9af0ea"]), np.clip(v, 0, 1), 0.25)
    return rgba(img, glow=np.where(v > 0.5, 0.4, 0.05))


def joint(seed=162):
    X, Y = coords()
    rib = 0.5 + 0.3 * np.cos(YS * 0.5)
    img = paint(ramp(["#05070c", "#0f141e", "#1c2432", "#2c384c"]), np.clip(rib * 0.8 + (fbm(X, Y, 8, 2, seed) - 0.5) * 0.2, 0, 1), 0.3)
    return rgba(img)


def trim(seed=163):
    f = Face(seed, GOLD, 6)
    f.rect((8, 8, 120, 120), GOLD, 0.65, 1.0, 6, 0.55)
    return f.rgba()


def avatar_pack(seed=164):
    f = Face(seed, TEAL, 5)
    f.rect((10, 10, 118, 118), TEAL, 0.45, 1.0, 5)
    for y in (40, 84):
        f.rect((10, y, 118, y + 10), DARK, 0.3, -1.0, 2)
    f.bolts(((22, 24), (106, 24), (22, 104), (106, 104)), 3, STEEL)
    return f.rgba()


# ---------------------------------------------------------------------------------------------------- wood and smoke cutouts

def ladder_side():
    pen = Pen()
    for x in (0.2, 0.8):
        pen.stroke(curve((x, 0.0), (x, 1.0), 0.0), 0.05, WOOD, 0.55)
    for y in (0.15, 0.5, 0.85):
        pen.stroke(curve((0.2, y), (0.8, y), 0.0), 0.035, WOOD, 0.5)
    return pen.rgba()


def ladder_top():
    pen = Pen()
    for x in (0.12, 0.88):
        for y in (0.12, 0.88):
            pen.ellipse((x, y), 0.07, 0.07, 0.0, WOOD, 0.6)
    return pen.rgba()


def smoke(thin):
    pen = Pen()
    spots = ((0.5, 0.5, 0.2), (0.35, 0.58, 0.14), (0.66, 0.6, 0.13)) if thin else ((0.5, 0.5, 0.34), (0.3, 0.6, 0.2), (0.7, 0.6, 0.2), (0.5, 0.72, 0.18))
    for x, y, r in spots:
        pen.ellipse((x, y), r, r * 0.9, 0.0, SMOKE_TONES, 0.7)
    return pen.rgba()


def stick(seed=112):
    X, Y = coords()
    grain = np.sin((Y + 0.2 * fbm(X, Y, 3, 2, seed)) * 80) * 0.5 + 0.5
    return rgba(paint(WOOD, 0.4 + 0.2 * grain + (fbm(X, Y, 10, 2, seed + 1) - 0.5) * 0.2, 0.3))


def handle(seed=103):
    X, Y = coords()
    grain = np.sin((X + 0.1 * fbm(X, Y, 3, 2, seed)) * 90) * 0.5 + 0.5
    return rgba(paint(WOOD, 0.62 + 0.14 * grain + (fbm(X, Y, 10, 2, seed + 1) - 0.5) * 0.14, 0.3))
