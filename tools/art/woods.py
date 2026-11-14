"""Planks, floors and doors of the woods after the oak (`block/woods.rs`, layers 432 + 4 * wood + 0..3: planks, floor,
door, open door) as recolours of the oak's sawn boards (`wood.planks`): the boards' lightness is remapped onto each
wood's tone, keeping the grain, knots and nails. The floor is the boards re-seeded and shifted; the door is the boards
stood upright inside a dark frame with two braces, hinges and a brass handle; the open door keeps only the leaf along
the left edge (two of sixteen texels, as the engine's), the rest cut out. The oak's own door and floor have no art
here (the engine's 16 px looks are upscaled), so these are placeholders until proper art (`docs/ART_HANDOVER.md`).
Add a wood: a row in `TONES`, and its four entries in `registry.py`'s `WOODS` (it builds from `NAMES`).
"""
import numpy as np

import wood
from lib import N, coords

# name, mid tone (what a texel of the oak's mid lightness becomes)
TONES = [
    ("birch", (206, 184, 138)), ("pine", (208, 166, 98)), ("palm", (190, 172, 138)), ("cherry", (150, 78, 56)),
    ("baobab", (172, 158, 138)), ("acacia", (190, 112, 56)), ("redwood", (128, 56, 42)),
    ("eucalyptus", (204, 148, 120)), ("bamboo", (200, 176, 96)),
]
NAMES = [name for name, _ in TONES]
BRASS = np.array([214, 170, 70])
LEAF = N // 8  # the open door's leaf: 2 / 16 of the tile


def _light(rgb):
    return rgb @ np.array([0.299, 0.587, 0.114])


def recolour(tile, w):
    """The oak tile's colours moved onto wood `w`'s tone by lightness: the mean lightness becomes the tone itself,
    darker texels scale it down, lighter ones lift it toward white."""
    rgb = tile[..., :3].astype(float)
    k = (_light(rgb) / _light(rgb).mean())[..., None]
    tone = np.array(TONES[w][1], float)
    if NAMES[w] == "palm":  # fibres along the boards
        X, Y = coords()
        k = k * (1 + 0.12 * (wood.streaks(X, Y, 2, 48, 3, 600) - 0.5))[..., None]
    if NAMES[w] == "eucalyptus":  # patches of pinkish tan and light brown
        from lib import fbm
        X, Y = coords()
        t = fbm(X, Y, 4, 2, 610)[..., None]
        tone = tone * (1 - t) + np.array([186, 134, 90.0]) * t
    out = np.where(k <= 1, tone * k, tone + (255 - tone) * np.clip((k - 1) * 0.8, 0, 1))
    return out.clip(0, 255).astype(np.uint8)


def _boards(w, seed):
    return recolour(wood.planks(seed)[..., :4], w)


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
