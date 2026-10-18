"""The cactus and the water tile in the stylized Ember & Aether look (docs/ART_STYLE.md). The trees moved to the
realistic look: trunks (birch bark, dead wood) in `wood.py`, canopies in `foliage.py`.
"""
import numpy as np

from lib import N, coords, fbm, paint, ramp, rgba, worley
CACTUS = ramp(["#0a1c1c", "#14382e", "#225a3c", "#34804a", "#52a45a", "#86c880", "#c4e8a8"])
WATER = ramp(["#06141f", "#0b2a3a", "#12475a", "#1d6f80", "#2c93a0", "#5cbcc0", "#9fe6e0"])


def cactus_side(seed=0):
    """Vertical ribs, a shaded groove between each, and a spine cluster on every crest; tiles."""
    X, Y = coords()
    rib = np.sin(X * 6 * 2 * np.pi) * 0.5 + 0.5
    v = 0.25 + rib * 0.5 + (fbm(X, Y, 20, 2, seed + 1) - 0.5) * 0.12
    img = paint(CACTUS, np.where(rib < 0.1, 0.05, v), 0.3)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    for cx in range(6):
        x0 = int((cx + 0.5) / 6 * N + 0.5) % N
        for y0 in range(6, N, 16):
            for dx, dy in ((0, 0), (-2, -2), (2, -2), (-2, 2), (2, 2), (0, -3), (0, 3)):
                img[(y0 + dy + (cx % 2) * 8) % N, (x0 + dx) % N] = CACTUS[6]
            img[(y0 + (cx % 2) * 8) % N, x0] = CACTUS[0]
    return rgba(img)


def cactus_top(seed=0):
    """The cut end: a ring of ribs round a pale pith (a face, not a repeating tile)."""
    X, Y = coords()
    dx, dy = X - 0.5, Y - 0.5
    r = np.hypot(dx, dy) * 2
    ang = np.arctan2(dy, dx)
    rib = np.sin(ang * 10) * 0.5 + 0.5
    v = np.where(r > 0.86, 0.1, 0.35 + (1 - r) * 0.3 + rib * 0.15 * (r > 0.5))
    v = v + (fbm(X, Y, 14, 2, seed + 1) - 0.5) * 0.14
    img = paint(CACTUS, v, 0.3)
    img[(r < 0.3)] = paint(CACTUS, np.full((N, N), 0.88), 0.0)[(r < 0.3)]
    return rgba(img)


def water(seed=0):
    """Deep teal with a net of caustic lines; it drifts in the shader, so it only needs to wrap. Translucent, a little
    clearer where the caustics fall."""
    X, Y = coords()
    f1, f2, _ = worley(X, Y, 6, seed + 1, 1.0)
    g1, g2, _ = worley((X + 0.31) % 1.0, (Y + 0.17) % 1.0, 9, seed + 2, 1.0)
    line = np.maximum(np.clip(1 - (f2 - f1) / 0.16, 0, 1), np.clip(1 - (g2 - g1) / 0.12, 0, 1) * 0.7)
    swell = fbm(X, Y, 5, 3, seed + 3)
    v = 0.22 + swell * 0.28 + line ** 2 * 0.3
    alpha = (172 + np.clip(v, 0, 1) * 60).astype(np.uint8)
    return rgba(paint(WATER, v, 0.3), alpha=alpha)
