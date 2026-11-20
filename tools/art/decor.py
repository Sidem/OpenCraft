"""The decor building set and the ruins (docs/ART_STYLE.md): five concretes, four brick and tile faces, framed glass, metal
panel, catwalk, railing, panel light; rusted plating and machine husks, a girder, the logbook and the vault door.
Concretes and bricks wrap (they repeat over big walls); sprites with holes (glass, catwalk, railing) are cutouts.
"""
import numpy as np

from draw import Pen, curve
from industry import concrete
from kit import AETHER, BLUE, BRONZE, DARK, GOLD, GREEN, RED, STEEL, XS, YS, Face
from lib import N, coords, fbm, paint, ramp, relief_light, rgba
from masonry import bricks
from rocks import BASALT, GRANITE, SANDSTONE
from stains import RUST

BLACK = ramp(["#05060a", "#0e1118", "#1b1f2a", "#2b3140", "#40485a"])
WHITE = ramp(["#2a2e3c", "#5a5f72", "#9096a8", "#c8ccd8", "#eceef4"])
RED_C = ramp(["#1c0a0e", "#3d1618", "#6b2a26", "#94443a", "#b86650", "#d98d6e"])
GREEN_C = ramp(["#08140e", "#12281c", "#1f4a2c", "#346a3c", "#52885a", "#7aa87c"])
YELLOW_C = ramp(["#1e1408", "#4a3412", "#7a5a1c", "#a88428", "#cfa844", "#e8cd78"])
BLUE_C = ramp(["#0c1428", "#1a2b4d", "#2c4678", "#4268b0", "#688ecc", "#9bb8e4"])
ORANGE_C = ramp(["#261208", "#542a10", "#8c4a1c", "#d07c34", "#e4a058", "#f4c88c"])
PURPLE_C = ramp(["#1a0e24", "#361e48", "#583470", "#804e98", "#a474bc", "#c8a2da"])
CYAN_C = ramp(["#0a2024", "#163e44", "#26686e", "#4098a0", "#6cbcc2", "#a2dce0"])
LIME_C = ramp(["#1a2408", "#384c12", "#627c22", "#96b840", "#b8d46c", "#d8ec9c"])
PINK_C = ramp(["#381a22", "#6c3844", "#a2626e", "#dc96a0", "#eab8c0", "#f6d8dc"])
SKY_C = ramp(["#1a283c", "#344e6e", "#5a7ea6", "#8cb0d6", "#b2cce6", "#d6e4f4"])
GREY_C = ramp(["#181a1e", "#36383e", "#5a5c64", "#808288", "#a6a8ae", "#cccdd2"])
BROWN_C = ramp(["#1a100a", "#362216", "#543824", "#745238", "#987458", "#bc9c82"])
MAROON_C = ramp(["#1c080e", "#3a101c", "#5e1c2c", "#822c42", "#a84860", "#ce7488"])
NAVY_C = ramp(["#0c1226", "#1a2448", "#2a386e", "#3c5094", "#5c72b6", "#869cd4"])
SLAG = ramp(["#08060c", "#14101c", "#241c30", "#383048", "#4e4462"])

# The 11 mixed-hue concretes in `block/tints.rs` / `recipes/dye.rs` `HUES` order (layers 480–490).
TINT_CONCRETES = (
    ("blue", BLUE_C), ("orange", ORANGE_C), ("purple", PURPLE_C), ("cyan", CYAN_C), ("lime", LIME_C),
    ("pink", PINK_C), ("sky", SKY_C), ("grey", GREY_C), ("brown", BROWN_C), ("maroon", MAROON_C), ("navy", NAVY_C),
)

# All 16 hues in `block/glass.rs` order (layers 491–506): the five primary hues, then `TINT_CONCRETES`.
GLASS_HUES = (
    ("red", RED_C), ("green", GREEN_C), ("black", BLACK), ("white", WHITE), ("yellow", YELLOW_C), *TINT_CONCRETES
)


def coloured_concrete(tones, seed):
    return concrete(seed, tones)


def _jewel(tones, boost=1.42):
    """Pushes a concrete ramp's chroma so stained glass reads jewel-bright while keeping its hue and palette size."""
    f = tones.astype(float)
    luma = (f[:, 0] * 0.2126 + f[:, 1] * 0.7152 + f[:, 2] * 0.0722)[:, None]
    return np.clip(luma + (f - luma) * boost, 0, 255).astype(np.uint8)


def stained_glass(tones, seed=0):
    """A translucent tinted cathedral-glass pane in a thin bevelled bronze-lead frame (all 16 hues share this look)."""
    X, Y = coords()
    d = np.minimum(np.minimum(XS, N - 1 - XS), np.minimum(YS, N - 1 - YS))
    frame_m = d < 6
    h = np.minimum(d, 5) / 5.0
    fv = 0.24 + (relief_light(h * 5.0, 1.0) - 0.5) * 0.9 + (fbm(X, Y, 8, 2, seed) - 0.5) * 0.2
    frame_img = paint(BRONZE, np.where(d < 1, 0.05, fv), 0.2)
    # Soft rippled glass body using a jewel-boosted ramp so light reads luminous through the pane.
    glass_tones = _jewel(tones)
    ripple = fbm(X, Y, 4, 3, seed + 11)
    edge_deep = np.clip((18.0 - d) / 14.0, 0.0, 1.0) * 0.16
    pv = np.clip(0.56 + (ripple - 0.5) * 0.26 - edge_deep, 0.22, 0.94)
    # Two clean / diagonal specular glints plus a subtle inner catch-light on the top-left inside edge.
    diag1 = (np.abs((XS + YS) - 84) < 3) & (XS > 18) & (XS < 74) & (YS > 12) & (YS < 68)
    diag2 = (np.abs((XS + YS) - 164) < 2) & (XS > 54) & (XS < 110) & (YS > 54) & (YS < 110)
    catch = ((XS == 7) | (YS == 7)) & (d >= 6) & (d <= 8)
    glint = diag1 | diag2 | catch
    pane_img = paint(glass_tones, np.where(glint, 0.96, pv), 0.18)
    img = np.where(frame_m[..., None], frame_img, pane_img)
    alpha = np.where(frame_m, 255, np.where(glint, 224, (148 + ripple * 16).astype(np.uint8))).astype(np.uint8)
    for cx, cy in ((3, 3), (124, 3), (3, 124), (124, 124)):
        img[cy, cx] = BRONZE[5]
    return rgba(img, alpha=alpha)


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
