"""Planks, floors and doors of the woods after the oak (`block/woods.rs`, layers 432 + 4 * wood + 0..3: planks, floor,
door, open door). Each species generates its own sawn boards with botanical grain, knot frequency, board layout, hue
and lightness:
  - birch: pale creamy ivory-blond, fine tight satin grain, few small pin knots
  - pine: pale honey-straw softwood, bold contrasting amber latewood growth bands and prominent oval knots
  - palm: warm golden-tan matrix streaked with dense dark-mahogany vascular fibres ("porcupine wood"), no knots
  - cherry: rich rosy-cinnamon / reddish-amber heartwood, smooth fine wavy cathedral grain, dark gum flecks
  - baobab: pale chalky-taupe / soft creamy-grey fibrous wood, broad gentle low-contrast grain
  - acacia: dramatic high-contrast swirling heartwood/sapwood bands (golden caramel to deep espresso brown)
  - redwood: deep rich brick-red / terracotta-crimson straight tight vertical grain, virtually knot-free
  - eucalyptus: warm rose-blush tan and honey-apricot interlocked wavy ribbon figure
  - bamboo: pale golden-straw laminated narrow strips (8 strips) with crisp staggered horizontal node knuckles

Add a wood: a row in `SPECIES`, and its four entries in `registry.py`'s `WOODS` (it builds from `NAMES`).
"""
import numpy as np

import wood
from lib import N, blur, coords, fbm, hash2

# (name, dark_rgb, mid_rgb, light_rgb, board_count, knot_chance, grain_contrast)
SPECIES = [
    ("birch",      (148, 128,  94), (196, 178, 140), (218, 204, 170), 4, 0.15, 0.30),
    ("pine",       (156, 112,  52), (212, 174, 108), (232, 204, 144), 4, 0.80, 0.62),
    ("palm",       ( 72,  44,  24), (178, 146, 102), (208, 180, 134), 4, 0.00, 0.72),
    ("cherry",     ( 96,  42,  28), (158,  84,  56), (196, 126,  90), 4, 0.18, 0.45),
    ("baobab",     (140, 130, 116), (186, 176, 160), (214, 206, 192), 4, 0.08, 0.28),
    ("acacia",     ( 64,  34,  18), (168, 108,  52), (218, 174, 108), 4, 0.25, 0.82),
    ("redwood",    ( 76,  28,  20), (138,  56,  40), (172,  82,  58), 4, 0.06, 0.42),
    ("eucalyptus", (134,  86,  64), (196, 142, 112), (224, 178, 148), 4, 0.14, 0.44),
    ("bamboo",     (134, 106,  48), (196, 172,  96), (218, 198, 128), 8, 0.00, 0.36),
]
NAMES = [row[0] for row in SPECIES]
BRASS = np.array([214.0, 170.0, 70.0])
LEAF = N // 8  # the open door's leaf: 2 / 16 of the tile


def _soft(x, width):
    return np.clip(x / width + 0.5, 0, 1)


def _ramp3(dark, mid, light, t):
    """Smooth 3-stop timber ramp (dark latewood/groove -> mid tone -> bright earlywood)."""
    t = np.clip(t, 0.0, 1.0)[..., None]
    d, m, l = np.array(dark, float), np.array(mid, float), np.array(light, float)
    lo = d + (m - d) * np.clip(t * 2.0, 0.0, 1.0)
    hi = m + (l - m) * np.clip((t - 0.5) * 2.0, 0.0, 1.0)
    return np.where(t <= 0.5, lo, hi)


def _boards(w, seed):
    name, dark, mid, light, n_boards, knot_p, contrast = SPECIES[w]
    X, Y = coords()
    s0 = seed + w * 37
    board = np.floor(Y * n_boards).astype(int) % n_boards
    local = (Y * n_boards) % 1.0
    tone = (hash2(board, 0, s0 + 1) - 0.5) * (0.28 if name == "acacia" else 0.16)
    x_shift = hash2(board, 2, s0 + 7)
    Xs = (X + x_shift) % 1.0

    # Species-specific grain field centered near 0.5.
    if name == "birch":
        grain = wood.streaks(Xs, Y, 2, 64, 3, s0 + 2) * 0.55 + wood.streaks(Xs, Y, 4, 128, 2, s0 + 3) * 0.45
        field = 0.54 + tone + (grain - 0.5) * contrast
    elif name == "pine":
        # Bold wide cathedral latewood rings.
        warp = (wood.streaks(Xs, Y, 2, 8, 2, s0 + 4) - 0.5) * 0.35
        rings = np.sin((local * 2.2 + warp + wood.streaks(Xs, Y, 2, 18, 2, s0 + 2) * 0.9) * np.pi * 2.4)
        late = _soft(-rings - 0.35, 0.35)
        fine = wood.streaks(Xs, Y, 3, 64, 2, s0 + 3)
        field = 0.60 + tone - late * 0.32 + (fine - 0.5) * 0.22
    elif name == "palm":
        # Porcupine-wood vascular bundle dashes packed lengthwise.
        fibres = wood.streaks(Xs, Y, 8, 96, 3, s0 + 2)
        flecks = _soft(0.32 - wood.streaks(Xs, Y, 14, 128, 2, s0 + 5), 0.14)
        field = 0.58 + tone + (fibres - 0.5) * 0.32 - flecks * 0.46
    elif name == "cherry":
        # Smooth wavy satin grain + tiny dark gum flecks.
        wave = wood.streaks(Xs, Y, 2, 36, 3, s0 + 2)
        satin = wood.streaks(Xs, Y, 3, 96, 2, s0 + 3)
        fleck = _soft(0.18 - wood.streaks(Xs, Y, 18, 48, 2, s0 + 6), 0.08) * 0.22
        field = 0.52 + tone + (wave - 0.5) * 0.38 + (satin - 0.5) * 0.16 - fleck
    elif name == "baobab":
        # Broad soft spongy fibrous grain.
        broad = wood.streaks(Xs, Y, 3, 20, 3, s0 + 2)
        pores = fbm(Xs, Y, 32, 2, s0 + 3)
        field = 0.54 + tone + (broad - 0.5) * 0.24 + (pores - 0.5) * 0.14
    elif name == "acacia":
        # High-contrast swirling heartwood/sapwood bands.
        swirl = wood.streaks(Xs, Y, 2, 10, 3, s0 + 2) + (wood.streaks(Xs, Y, 3, 24, 2, s0 + 5) - 0.5) * 0.4
        band = np.sin(swirl * np.pi * 3.0 + hash2(board, 5, s0) * 6.0) * 0.5 + 0.5
        fine = wood.streaks(Xs, Y, 4, 80, 2, s0 + 3)
        field = 0.50 + tone + (band - 0.5) * contrast + (fine - 0.5) * 0.18
    elif name == "redwood":
        # Tight straight uniform vertical/horizontal cedar-like grain.
        straight = wood.streaks(Xs, Y, 1, 72, 3, s0 + 2)
        fine = wood.streaks(Xs, Y, 2, 128, 2, s0 + 3)
        field = 0.52 + tone + (straight - 0.5) * 0.34 + (fine - 0.5) * 0.16
    elif name == "eucalyptus":
        # Gently interlocked ribbon stripe + rosy-tan/honey patches.
        ribbon = np.sin((local * 2.0 + wood.streaks(Xs, Y, 2, 12, 2, s0 + 4) * 0.6) * np.pi * 2.0) * 0.5 + 0.5
        fine = wood.streaks(Xs, Y, 3, 64, 3, s0 + 2)
        field = 0.52 + tone + (ribbon - 0.5) * 0.24 + (fine - 0.5) * 0.28
    else:
        # Bamboo: laminated narrow strips with crisp staggered horizontal node knuckles.
        fibres = wood.streaks(Xs, Y, 4, 112, 2, s0 + 2)
        node_x = (hash2(board, 3, s0 + 8) * 0.7 + 0.15)
        nd = np.abs(((X - node_x + 0.5) % 1.0) - 0.5)
        knuckle = _soft(0.012 - nd, 0.01) * (0.75 + 0.25 * wood.streaks(X, Y, 16, 32, 2, s0 + 5))
        field = 0.58 + tone + (fibres - 0.5) * 0.24 - knuckle * 0.36

    # Knots (species-tuned frequency).
    if knot_p > 0.0:
        kx = hash2(board, 3, s0 + 8) * 0.78 + 0.11
        ky = (board + 0.5) / n_boards
        kd = np.hypot((X - kx + 0.5) % 1.0 - 0.5, (Y - ky) * (n_boards * 0.55))
        has_knot = (hash2(board, 4, s0 + 9) < knot_p).astype(float)
        k_rad = 0.042 if name == "pine" else 0.026
        knot = has_knot * _soft(k_rad - kd, 0.018)
        swirl_k = has_knot * np.sin(kd * 240) * _soft(k_rad * 2.4 - kd, 0.05) * 0.07
        field = field + swirl_k - knot * (0.48 if name == "pine" else 0.35)

    # Board seams, staggered butt joints & nails (bamboo uses fine lamination seams, no nails).
    joint_x = hash2(board, 1, s0 + 4)
    jd = np.abs(((X - joint_x + 0.5) % 1.0) - 0.5)
    seam_w = 0.055 if n_boards == 8 else 0.028
    gap = np.maximum(_soft(seam_w - local, seam_w) + _soft(local - (1.0 - seam_w * 0.5), seam_w * 0.8),
                     _soft(0.005 - jd, 0.007))
    field = field - blur(gap, 0.5) * (0.32 if name == "bamboo" else 0.46)
    if name != "bamboo":
        nail = np.zeros_like(X)
        for off in (-0.045, 0.045):
            nd = np.hypot((X - joint_x - off + 0.5) % 1.0 - 0.5, (local - 0.5) / n_boards)
            nail = np.maximum(nail, _soft(0.0065 - nd, 0.0045))
        field = field - nail * 0.35

    field = field + (fbm(X, Y, 64, 1, s0 + 11) - 0.5) * 0.06
    rgb = _ramp3(dark, mid, light, field)

    # Eucalyptus has broad rosy-tan to warm-honey tonal mottling across boards.
    if name == "eucalyptus":
        patch = fbm(X, Y, 4, 2, 610)[..., None]
        warm = _ramp3((126, 84, 52), (186, 138, 92), (218, 176, 130), field)
        rgb = rgb * (1.0 - patch * 0.55) + warm * (patch * 0.55)

    return rgb.clip(0, 255).astype(np.uint8)


def planks(w):
    return np.dstack([_boards(w, 0), np.full((N, N), 255, np.uint8), np.zeros((N, N), np.uint8)])


def floor(w):
    """Boards laid the other way round, shifted so no joint lines up with the planks'."""
    rgb = np.roll(_boards(w, 5), (N // 2, N // 3), axis=(0, 1))
    out = np.zeros((N, N, 5), np.uint8)
    out[..., :3] = (rgb * 0.92).astype(np.uint8)
    out[..., 3] = 255
    return out


def door(w, open_=False):
    rgb = np.rot90(_boards(w, 7)).copy().astype(float)  # upright boards
    ys, xs = np.mgrid[0:N, 0:N]
    edge = (xs < 6) | (xs >= N - 6) | (ys < 6) | (ys >= N - 6)
    brace = ((ys >= 22) & (ys < 36)) | ((ys >= N - 36) & (ys < N - 22))
    rgb[edge] *= 0.55
    rgb[brace & ~edge] *= 0.8
    hinge = (xs >= 6) & (xs < 14) & (((ys >= 16) & (ys < 30)) | ((ys >= N - 30) & (ys < N - 16)))
    rgb[hinge] = (52, 50, 54)
    handle = ((xs - (N - 18)) ** 2 + (ys - N // 2) ** 2) < 7 ** 2
    rgb[handle] = BRASS * (0.85 + 0.15 * ys[handle] / N)[:, None]
    out = np.zeros((N, N, 5), np.uint8)
    out[..., :3] = rgb.clip(0, 255).astype(np.uint8)
    out[..., 3] = 255
    if open_:
        out[:, LEAF:, 3] = 0
    return out


def door_open(w):
    return door(w, True)
