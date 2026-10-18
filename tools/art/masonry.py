"""Cut stone and metal-bound walls: stone bricks (and later the decor bricks, concretes and tiles)."""
import numpy as np

from lib import N, coords, edges, fbm, grow, hash2, paint, ramp, rgba, roll
from natural import RAMPS

WARM = ramp(["#161722", "#2c2b3a", "#443f50", "#625564", "#836f76", "#a58a88", "#c9a899", "#e8cdb5"])


def bricks(seed=0, heights=(30, 34, 32, 32), mortar=2, moss=True, tones=None, joint_tones=None):
    """Ashlar courses of uneven height and brick width, bevelled and variegated, with moss creeping in the joints."""
    rng = np.random.default_rng(seed + 3)
    X, Y = coords()
    dl = np.zeros((N, N))
    dr = np.zeros((N, N))
    dt = np.zeros((N, N))
    db = np.zeros((N, N))
    bid = np.zeros((N, N))
    y0 = 0
    for row, h in enumerate(heights):
        n = int(rng.integers(2, 4))
        cuts = np.sort(rng.choice(np.arange(24, N - 24), n - 1, replace=False)) if n > 2 else np.array([rng.integers(44, 84)])
        cuts = np.concatenate([[0], cuts, [N]])
        start = int(rng.integers(0, N))  # stagger the joints between courses
        for j in range(len(cuts) - 1):
            a, b = cuts[j], cuts[j + 1]
            for x in range(a, b):
                col = (x + start) % N
                dl[y0:y0 + h, col] = x - a
                dr[y0:y0 + h, col] = b - 1 - x
                bid[y0:y0 + h, col] = hash2(row * 7 + j, 0, seed + 5)
        dt[y0:y0 + h] = np.arange(h)[:, None]
        db[y0:y0 + h] = (h - 1 - np.arange(h))[:, None]
        y0 += h
    d = np.minimum(np.minimum(dl, dr), np.minimum(dt, db))
    grain = fbm(X, Y, 20, 3, seed + 1)
    mott = fbm(X, Y, 5, 3, seed + 2)
    lit = ((dl < 8) | (dt < 8)) * (1 - np.minimum(np.minimum(dl, dt), 8) / 8)
    shade = ((dr < 8) | (db < 8)) * (1 - np.minimum(np.minimum(dr, db), 8) / 8)
    dome = np.minimum(d, 14) / 14
    v = 0.34 + bid * 0.2 + (mott - 0.5) * 0.18 + (grain - 0.5) * 0.14 + lit * 0.3 - shade * 0.3 + dome * 0.1
    crack = (np.abs(fbm(X, Y, 4, 3, seed + 6, 0.5, 0.3) - 0.5) < 0.008) & (bid > 0.75)
    v = np.where(crack, v - 0.25, v)
    face = np.where((bid > 0.5)[..., None], paint(WARM, v, 0.3), paint(RAMPS["stone"], v + 0.05, 0.3))
    if tones is not None:  # one family of brick (the decor set) instead of warm and grey mixed
        face = paint(tones, v + 0.1, 0.3)
    joint = d < mortar + (hash2((X * N).astype(int) // 4, (Y * N).astype(int) // 4, seed + 4) > 0.8)
    joint_img = paint(RAMPS["stone"] if joint_tones is None else joint_tones, 0.1 + grain * 0.14, 0.3)
    if moss:
        green = (fbm(X, Y, 6, 3, seed + 8) > 0.5) & joint
        joint_img = np.where(green[..., None], paint(RAMPS["grass"], 0.2 + grain * 0.3, 0.4), joint_img)
        fringe = grow(green, 2) & ~joint & (fbm(X, Y, 14, 2, seed + 9) > 0.55) & ((dl < 6) | (dr < 6) | (dt < 6) | (db < 6))
        face = np.where(fringe[..., None], paint(RAMPS["grass"], 0.3 + grain * 0.35, 0.4), face)
    return rgba(np.where(joint[..., None], joint_img, face))
