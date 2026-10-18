"""Natural block recipes in the stylized Ember & Aether look (docs/ART_STYLE.md): logs, planks, leaves, and the stylized
stone that bedrock and spent rock still use. The ground (stone, dirt, grass, sand) moved to the realistic `real.py`.

Every function returns an `N` x `N` x 4 uint8 tile. Tiles that repeat (stone, bark, planks) wrap at the edge.
`masonry.py`, `plants.py` and others take their colours from `RAMPS`.
"""
import numpy as np

from lib import N, cavity, coords, edges, fbm, hash2, paint, ramp, relief_light, rgba, roll, vnoise, worley

RAMPS = {
    "stone": ramp(["#0f1520", "#1e2431", "#323745", "#4b4f5e", "#666878", "#868390", "#a8a0a2", "#cbbdb6"]),
    "dirt": ramp(["#10080b", "#2a1114", "#4a231d", "#6b3a2b", "#8a5236", "#a86a42", "#c78a58"]),
    "grass": ramp(["#0c1a14", "#143022", "#1f4528", "#2f5e2c", "#46792f", "#66982f", "#8fb43a", "#c2d45e"]),
    "sand": ramp(["#2b2030", "#54403f", "#7e604f", "#a8836a", "#cfa985", "#e8cb9c", "#f6e6b8"]),
    "bark": ramp(["#0f080c", "#1f0f12", "#3b1b1a", "#5a2c22", "#78402f", "#9b573d", "#c07a55"]),
    "oak": ramp(["#2a1214", "#4a2420", "#6b3a2b", "#8c5236", "#ad6e46", "#c98d5c", "#e1b07e"]),
}


def stone(seed=0, tones=None, lift=0.0):
    """Fractured rock: angular plates of two sizes with chiselled edges, dark cracks, faint strata and glints.
    `tones` swaps the ramp (bedrock, spent rock) and `lift` shifts the whole value."""
    X, Y = coords()
    Xw = (X + (fbm(X, Y, 4, 3, seed + 11) - 0.5) * 0.2) % 1.0  # wobbly plate outlines, not straight cuts
    Yw = (Y + (fbm(X, Y, 4, 3, seed + 12) - 0.5) * 0.2) % 1.0
    f1, f2, pid = worley(Xw, Yw, 4, seed + 1, 0.9)
    g1, g2, qid = worley(Xw, Yw, 8, seed + 7, 0.9)
    edge = f2 - f1
    sub = (g2 - g1) * (qid > 0.45)  # the small fractures only split some plates
    h = np.minimum(edge / 0.3, 1) ** 0.6 * 0.6 + np.minimum(sub / 0.3, 1) * 0.12 + pid * 0.35
    h = h + fbm(X, Y, 10, 3, seed + 3) * 0.12
    light = relief_light(h * 7, 0.95)
    strata = np.sin((Y * 9 + fbm(X, Y, 3, 2, seed + 4) * 1.5) * 2 * np.pi) * 0.5 + 0.5
    grain = fbm(X, Y, 24, 2, seed + 5)
    v = 0.33 + lift + pid * 0.2 + qid * 0.06 + (light - 0.5) * 0.7 + (strata - 0.5) * 0.08 + (grain - 0.5) * 0.14
    v = v - cavity(h * 3, 2.5) * 1.1
    v = np.where(edge < 0.03, 0.03 + 0.1 * grain, v)
    v = np.where((sub < 0.025) & (qid > 0.45) & (edge >= 0.04), v - 0.18, v)
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    v = v + (hash2(ix, iy, seed + 6) > 0.994) * 0.3
    return rgba(paint(RAMPS["stone"] if tones is None else tones, v, 0.3))


def bark(seed=0, tones=None, moss=True):
    """Broad bark plates split by deep fissures, a few mossy patches on the lit side; tiles in both directions.
    `tones` swaps the ramp (dead wood) and `moss=False` leaves the moss off."""
    X, Y = coords()
    u = X * 6 + (fbm(X, Y, 3, 3, seed + 1) - 0.5) * 1.3
    plate = hash2(np.floor(u).astype(int) % 6, 0, seed + 9)
    ridge = 1 - np.abs((u % 1.0) - 0.5) * 2
    along = fbm(X * 1.0, Y * 0.5, 5, 2, seed + 5)
    h = (ridge ** 0.6) * (0.6 + plate * 0.3) + fbm(X, Y, 14, 3, seed + 2) * 0.25
    light = relief_light(h * 8, 1.0)
    fissure = (ridge < 0.12 + 0.1 * along) | (np.abs(fbm(X, Y, 5, 3, seed + 6, 0.5, 0.3) - 0.5) < 0.008)
    v = 0.16 + h * 0.5 + (light - 0.5) * 0.5 + plate * 0.08
    img = paint(RAMPS["bark"] if tones is None else tones, np.where(fissure, 0.03 + along * 0.08, v), 0.3)
    moss = moss & (fbm(X, Y, 3, 3, seed + 3) > 0.7) & (light > 0.48) & ~fissure & (ridge > 0.4)
    green = paint(RAMPS["grass"], 0.18 + h * 0.3 + (light - 0.5) * 0.5, 0.45)
    return rgba(np.where(moss[..., None], green, img))


def log_end(seed=0):
    """Growth rings round a pale heartwood, radial checks, a dark bark rim (a face, not a repeating tile)."""
    X, Y = coords()
    dx, dy = X - 0.5, Y - 0.5
    warp = fbm(X, Y, 4, 3, seed + 1) - 0.5
    r = np.hypot(dx, dy) * 2 + warp * 0.07
    rings = np.sin(r * 2 * np.pi * 7.5) * 0.5 + 0.5
    ang = np.arctan2(dy, dx)
    check = (np.abs(np.sin(ang * 3 + warp * 5)) < 0.03) & (r > 0.15) & (r < 0.85)
    rim = np.maximum(np.abs(dx), np.abs(dy)) * 2
    v = 0.7 - r * 0.28 + (rings - 0.5) * 0.3 + (fbm(X, Y, 20, 2, seed + 2) - 0.5) * 0.12
    img = paint(RAMPS["oak"], np.where(check, v - 0.35, v), 0.3)
    outer = paint(RAMPS["bark"], 0.2 + fbm(X, Y, 9, 3, seed + 3) * 0.35, 0.4)
    return rgba(np.where((rim > 0.84)[..., None], outer, img))


def planks(seed=0):
    """Four horizontal boards with grain, staggered butt joints, nail heads and a bevelled seam; tiles."""
    X, Y = coords()
    board = (Y * 4).astype(int)
    bid = hash2(board, 0, seed + 1)
    streak = vnoise((X * 2.0 + bid * 3) * 4, Y * 90, 4, seed + 2)
    fine = vnoise(X * 6 + bid * 5, Y * 160, 6, seed + 3)
    local = (Y * 4) % 1.0
    joint_x = hash2(board, 1, seed + 4)
    joint = np.abs(((X - joint_x + 0.5) % 1.0) - 0.5) < 0.012
    v = 0.38 + bid * 0.2 + (streak - 0.5) * 0.3 + (fine - 0.5) * 0.14
    v = v + np.where((local >= 0.045) & (local < 0.09), 0.14, 0.0)
    v = np.where((local < 0.045) | joint, 0.02, v)
    img = paint(RAMPS["oak"], v, 0.3)
    bright, dark = RAMPS["oak"][6], RAMPS["oak"][0]
    for b in range(4):  # a nail near each end of a board: dark head with a lit rim
        for off in (-0.07, 0.07):
            cx, cy = int(((joint_x[b * 32, 0] + off) % 1.0) * N), b * 32 + 16 + (b % 2) * 4
            x1, y1 = (cx + 1) % N, cy + 1
            img[cy, cx], img[y1, x1] = bright, dark
            img[cy, x1] = img[y1, cx] = RAMPS["oak"][1]
    return rgba(img)


def leaf_shape(dx, dy, ang, la, lb):
    """Inside-mask and (along, across) coordinates of a pointed leaf of half-length `la` and half-width `lb`."""
    u = dx * np.cos(ang) + dy * np.sin(ang)
    w = -dx * np.sin(ang) + dy * np.cos(ang)
    inside = np.abs(u) < la
    inside &= np.abs(w) < lb * np.clip(1 - (u / la) ** 2, 0, 1) ** 0.7
    return inside, u / la, w / lb


def leaves(seed=0, tones=None, width=0.42, scale=1.0, density=1.0):
    """Clustered leaves in three depths: dark back leaves, mid, lit front leaves; a cutout with holes. `tones`, `width`
    (leaf width over length; needles are thin), `scale` and `density` make the other trees' canopies."""
    rng = np.random.default_rng(seed + 50)
    tones = RAMPS["grass"] if tones is None else tones
    rgb = np.zeros((N, N, 3), np.uint8)
    rgb[:] = tones[2]
    cover = np.zeros((N, N), bool)
    ys, xs = np.mgrid[-16:17, -16:17]
    for count, size, lo, hi in ((38, 13, 0.08, 0.3), (36, 12, 0.3, 0.55), (24, 11, 0.5, 0.85)):
        size *= scale
        for _ in range(int(count * density)):
            cx, cy = rng.integers(0, N), rng.integers(0, N)
            ang, la = rng.uniform(0, np.pi), size * rng.uniform(0.8, 1.3)
            inside, u, w = leaf_shape(xs, ys, ang, la, la * width)
            base = rng.uniform(lo, hi)
            shade = base + 0.16 * (-(xs * np.cos(ang) * 0 + (xs + ys) / (2 * la))) - 0.1 * (np.abs(w) < 0.12) + 0.08 * (u + 1) / 2
            idx = (np.clip(shade, 0, 0.999) * len(tones)).astype(int)
            px, py = (cx + xs) % N, (cy + ys) % N
            rgb[py[inside], px[inside]] = tones[idx[inside]]
            cover[py[inside], px[inside]] = True
    rim = edges(cover) & ~roll(cover, 1, 0)
    rgb[rim] = tones[1]
    return rgba(rgb, alpha=np.where(cover, 255, 0).astype(np.uint8))
