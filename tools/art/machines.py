"""Machine and structure faces: dark-iron plates with bronze fittings and verdigris, the smelter's firebrick and ember mouth,
the generator's copper coil and aether slits, glass and the amber lamp. Glowing texels set `glow` (packed into alpha by `rgba`).

Palette roles: iron/steel is navy-to-pale-blue, bronze is the wood-brown ramp pushed to gold, energy is ember orange
(fuel, heat) or aether cyan (power, electrics). Faces are drawn straight on, lit from the top left like every tile.
"""
import numpy as np

from lib import N, coords, edges, fbm, grow, hash2, paint, ramp, relief_light, rgba, roll, vnoise
from masonry import bricks
from natural import RAMPS

STEEL = ramp(["#0b0f18", "#161c2a", "#232b3d", "#34405a", "#4a5a78", "#6d82a2", "#9fb4cc", "#d3e2ee"])
BRONZE = ramp(["#1c0e0d", "#3d2014", "#6b3b1c", "#9a5c28", "#c58a3c", "#e6b45a", "#fbe08f"])
FIREBRICK = ramp(["#1a0b0e", "#3b1618", "#5d2a22", "#82402c", "#a95a38", "#cc7a4a", "#ec996f"])
EMBER = ramp(["#1a0505", "#5a1408", "#b3300c", "#f0661a", "#ffa53a", "#ffe38a"])
AETHER = ramp(["#06202a", "#0d4a58", "#1b8a95", "#3fd0d0", "#9af0ea", "#e4fff8"])
COPPER = ramp(["#2b120d", "#5e2a1a", "#9d4a24", "#d07a3a", "#f0a860", "#ffd9a0"])
VERDIGRIS = ramp(["#0d2a31", "#1c5456", "#33887a", "#6fc4a8", "#b4ecd0"])
AMBER = ramp(["#4a1708", "#8f2c0c", "#d9561a", "#ff8a26", "#ffc04a", "#fff0b0"])


def rivet(img, cx, cy, r=4, tones=BRONZE):
    """A domed rivet head with a lit upper left and a dark rim, drawn wrapping."""
    for dy in range(-r - 1, r + 2):
        for dx in range(-r - 1, r + 2):
            d = np.hypot(dx, dy)
            x, y = (cx + dx) % N, (cy + dy) % N
            if d <= r:
                lit = np.clip(0.55 - (dx + dy) / (2.2 * r), 0, 1)
                img[y, x] = tones[int(np.clip(lit * len(tones), 1, len(tones) - 1))]
            elif d <= r + 1:
                img[y, x] = tones[0]


def plate(seed=0, border=7, tones=STEEL):
    """A brushed steel panel with a raised, bevelled rim; returns (rgb, rim mask, inner height)."""
    X, Y = coords()
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    d = np.minimum(np.minimum(ix, N - 1 - ix), np.minimum(iy, N - 1 - iy))
    brush = vnoise(X * 3, Y * 120, 3, seed + 1)
    wear = fbm(X, Y, 6, 3, seed + 2)
    h = np.minimum(d, border) / border
    light = relief_light(h * 6, 1.0)
    v = 0.4 + (brush - 0.5) * 0.12 + (wear - 0.5) * 0.18 + (light - 0.5) * 0.9
    v = np.where(d < 1, 0.05, v)
    return paint(tones, v, 0.25), d < border, d


def frame(seed=0):
    """Riveted dark-iron plate with bronze corner brackets and verdigris weeping from the rivets."""
    img, rim, d = plate(seed)
    X, Y = coords()
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    for cx, cy in ((14, 14), (113, 14), (14, 113), (113, 113), (64, 6), (64, 121)):
        for k in range(int(hash2(cx, cy, seed) * 8) + 8):
            yy = cy + 5 + k
            if 0 <= yy < N:
                img[yy, cx + int(hash2(k, cx, seed) * 3) - 1] = VERDIGRIS[1 + int(k > 8)]
        rivet(img, cx, cy)
    for sx in (0, 1):
        for sy in (0, 1):
            x0, y0 = 8 + sx * 82, 8 + sy * 82
            for t in range(28):  # an L bracket hugging the corner
                for w in range(5):
                    ax, ay = (x0 + t, y0 + w) if sx == 0 else (x0 + 27 - t, y0 + w)
                    bx, by = (x0 + w, y0 + t) if sy == 0 else (x0 + w, y0 + 27 - t)
                    shade = 3 + (w == 0) * 2 - (w == 4) * 2
                    for px, py in ((ax, ay if sy == 0 else y0 + 27 - w), (bx if sx == 0 else x0 + 27 - w, by)):
                        if 0 <= px < N and 0 <= py < N:
                            img[py, px] = BRONZE[int(np.clip(shade, 0, 6))]
    for cx, cy in ((20, 20), (107, 20), (20, 107), (107, 107)):
        rivet(img, cx, cy, 3)
    return rgba(img)


def steel(seed=0):
    """Plain brushed steel with a panel seam: the neutral metal used wherever a plate shows."""
    img, rim, d = plate(seed + 3, 5)
    for cx, cy in ((12, 12), (115, 12), (12, 115), (115, 115)):
        rivet(img, cx, cy, 3, STEEL)
    return rgba(img)


def arch_mask(x0=34, x1=94, top=48, bottom=120):
    ys, xs = np.mgrid[0:N, 0:N]
    cx, r = (x0 + x1) / 2, (x1 - x0) / 2
    box = (xs >= x0) & (xs < x1) & (ys >= top + r) & (ys < bottom)
    cap = (np.hypot(xs - cx, ys - (top + r)) < r) & (ys < top + r)
    return box | cap


def smelter_side(seed=0):
    """Firebrick wall with an iron-framed furnace mouth full of ember glow and soot above it."""
    X, Y = coords()
    img = bricks(seed + 20, heights=(32, 32, 32, 32), moss=False)[..., :3].copy()
    # recolour the courses with the firebrick ramp, keeping their light and shade
    lum = (img.astype(float) @ [0.3, 0.59, 0.11]) / 255.0
    img = paint(FIREBRICK, np.clip(lum * 1.05 + 0.05, 0, 1), 0.3)
    mouth = arch_mask()
    frame_m = grow(mouth, 6) & ~mouth
    soot = np.clip((np.arange(N)[:, None] - 0) / 1.0, 0, 1) * 0
    above = np.clip(1 - (np.arange(N)[:, None] / 70.0), 0, 1) * (np.abs(np.arange(N)[None, :] - 64) < 34) * (fbm(X, Y, 8, 2, seed + 4) + 0.3)
    img = np.where((above > 0.45)[..., None], paint(FIREBRICK, 0.08 + 0.1 * fbm(X, Y, 12, 2, seed + 5), 0.3), img)
    iron = paint(STEEL, 0.25 + 0.35 * relief_light(grow(mouth, 6).astype(float) * 4, 1.2) + 0.1 * fbm(X, Y, 20, 2, seed), 0.3)
    img = np.where(frame_m[..., None], iron, img)
    ys, xs = np.mgrid[0:N, 0:N]
    flame = 1 - ((120 - ys) / 72.0) * 0.9 + (fbm(X * 1.0, Y * 0.6, 7, 3, seed + 6) - 0.5) * 0.9
    heat = np.clip(flame, 0, 1)
    fire = paint(EMBER, 0.12 + heat * 0.88, 0.3)
    img = np.where(mouth[..., None], fire, img)
    for cx, cy in ((24, 56), (104, 56), (24, 112), (104, 112)):
        rivet(img, cx, cy, 3, STEEL)
    glow = np.where(mouth, np.clip(0.35 + heat * 0.65, 0, 1), 0.0)
    return rgba(img, glow=glow)


def smelter_top(seed=0):
    """Firebrick deck inside an iron rim, a glowing vent grate."""
    X, Y = coords()
    base = bricks(seed + 21, heights=(32, 32, 32, 32), moss=False)[..., :3]
    lum = (base.astype(float) @ [0.3, 0.59, 0.11]) / 255.0
    img = paint(FIREBRICK, np.clip(lum + 0.05, 0, 1), 0.3)
    rim_img, rim, d = plate(seed + 7, 9)
    img = np.where(rim[..., None], rim_img, img)
    ys, xs = np.mgrid[0:N, 0:N]
    vent = (np.abs(xs - 64) < 24) & (np.abs(ys - 64) < 24) & (((xs - 40) // 6) % 2 == 0)
    img = np.where(vent[..., None], paint(EMBER, 0.45 + 0.4 * fbm(X, Y, 6, 2, seed + 8), 0.3), img)
    wall = grow(vent | ((np.abs(xs - 64) < 26) & (np.abs(ys - 64) < 26)), 2) & ~vent & (np.abs(xs - 64) < 27) & (np.abs(ys - 64) < 27)
    img = np.where(wall[..., None] & ~vent[..., None], STEEL[2], img)
    for cx, cy in ((18, 18), (109, 18), (18, 109), (109, 109)):
        rivet(img, cx, cy, 3, STEEL)
    return rgba(img, glow=np.where(vent, 0.7, 0.0))


def generator_side(seed=0):
    """Dark-iron housing, a band of copper coil windings and three aether slits that glow."""
    img, rim, d = plate(seed + 11, 7)
    X, Y = coords()
    ys, xs = np.mgrid[0:N, 0:N]
    band = (ys >= 44) & (ys < 100) & (d >= 7)
    wind = (xs + ys // 14 * 2) % 7
    coil = paint(COPPER, 0.25 + 0.55 * np.abs(np.sin(wind / 7 * np.pi)) + 0.12 * (fbm(X, Y, 10, 2, seed) - 0.5), 0.2)
    coil = np.where(((xs + ys // 14 * 2) % 7 == 0)[..., None], COPPER[0], coil)
    img = np.where(band[..., None], coil, img)
    top_rail = (ys >= 40) & (ys < 44) & (d >= 7) | (ys >= 100) & (ys < 104) & (d >= 7)
    img = np.where(top_rail[..., None], BRONZE[3], img)
    slits = np.zeros((N, N), bool)
    for sx in (26, 56, 86):
        slits |= (xs >= sx) & (xs < sx + 16) & (ys >= 18) & (ys < 32)
    mid = paint(AETHER, 0.35 + 0.55 * np.clip(1 - np.abs(xs - 64) / 80, 0, 1) * 0 + 0.4 * fbm(X, Y, 8, 2, seed + 9), 0.3)
    img = np.where(slits[..., None], mid, img)
    rim_s = grow(slits, 2) & ~slits
    img = np.where(rim_s[..., None], STEEL[0], img)
    for cx, cy in ((14, 14), (113, 14), (14, 113), (113, 113)):
        rivet(img, cx, cy, 3)
    return rgba(img, glow=np.where(slits, 0.9, 0.0))


def generator_top(seed=0):
    """A round turbine grille over an aether glow, inside a riveted iron rim."""
    img, rim, d = plate(seed + 12, 9)
    X, Y = coords()
    ys, xs = np.mgrid[0:N, 0:N]
    dx, dy = xs - 63.5, ys - 63.5
    r = np.hypot(dx, dy)
    ang = np.arctan2(dy, dx)
    disc = r < 46
    well = paint(AETHER, np.clip(0.2 + (1 - r / 46) * 0.8 + (fbm(X, Y, 6, 2, seed) - 0.5) * 0.2, 0, 1), 0.3)
    img = np.where(disc[..., None], well, img)
    blade = disc & (np.abs(np.sin(ang * 6 + r * 0.06)) < 0.22) & (r > 9)
    hub = r < 9
    ring = (np.abs(r - 46) < 3) | (np.abs(r - 28) < 1.5)
    img = np.where((blade | ring)[..., None], paint(STEEL, 0.3 + 0.3 * relief_light(blade * 3.0, 1) + 0.1, 0.2), img)
    img = np.where(hub[..., None], BRONZE[4], img)
    for cx, cy in ((18, 18), (109, 18), (18, 109), (109, 109)):
        rivet(img, cx, cy, 3)
    glow = np.where(disc & ~(blade | ring) | hub * 0, 0.75 * (1 - r / 60.0), 0.0)
    return rgba(img, glow=np.clip(glow, 0, 1))


def glass(seed=0):
    """A bronze-framed pane: only the frame and two pale glints are solid, the rest is see-through (a cutout)."""
    X, Y = coords()
    ys, xs = np.mgrid[0:N, 0:N]
    d = np.minimum(np.minimum(xs, N - 1 - xs), np.minimum(ys, N - 1 - ys))
    frame_m = d < 9
    h = np.minimum(d, 8) / 8
    v = 0.3 + (relief_light(h * 6, 1.0) - 0.5) * 1.2 + (fbm(X, Y, 8, 2, seed) - 0.5) * 0.25
    img = paint(BRONZE, np.where(d < 1, 0.0, v), 0.25)
    glint = (((xs - ys) % 64) < 5) & (xs > 18) & (xs < 112) & (ys > 18) & (ys < 112) & (((xs + ys) % 128) > 40)
    glint2 = (((xs - ys + 22) % 64) < 3) & (xs > 18) & (xs < 112) & (ys > 18) & (ys < 112)
    pane = np.where((glint | glint2)[..., None], np.array([196, 238, 240], np.uint8), np.array([30, 70, 80], np.uint8))
    img = np.where(frame_m[..., None], img, pane)
    alpha = np.where(frame_m | glint | glint2, 255, 0).astype(np.uint8)
    for cx, cy in ((4, 4), (123, 4), (4, 123), (123, 123)):
        img[cy, cx] = BRONZE[6]
    return rgba(img, alpha=alpha)


def lamp(seed=0):
    """An aether lamp: a brass cage round a faceted amber crystal that lights itself."""
    X, Y = coords()
    ys, xs = np.mgrid[0:N, 0:N]
    dx, dy = xs - 63.5, ys - 63.5
    d = np.minimum(np.minimum(xs, N - 1 - xs), np.minimum(ys, N - 1 - ys))
    cage = d < 14
    bars = ((np.abs(dx) < 3) | (np.abs(dy) < 3)) & (d < 52)
    diamond = (np.abs(dx) + np.abs(dy)) < 52
    facet = ((np.arctan2(dy, dx) + np.pi) / (2 * np.pi) * 8).astype(int) % 8
    shade = 0.45 + 0.45 * (1 - np.hypot(dx, dy) / 60.0) + (facet % 2) * 0.12 - (facet >= 4) * 0.08
    crystal = paint(AMBER, np.clip(shade, 0, 1), 0.2)
    wall = paint(STEEL, 0.15 + 0.1 * fbm(X, Y, 8, 2, seed), 0.2)
    h = np.minimum(d, 13) / 13
    metal = paint(BRONZE, 0.3 + (relief_light(h * 5, 1) - 0.5) * 1.1 + 0.1 * fbm(X, Y, 10, 2, seed + 1), 0.25)
    img = np.where(diamond[..., None], crystal, wall)
    img = np.where((cage | (bars & ~(np.abs(dx) + np.abs(dy) < 36)))[..., None], metal, img)
    for cx, cy in ((9, 9), (118, 9), (9, 118), (118, 118)):
        rivet(img, cx, cy, 3)
    glow = np.where(diamond & ~cage & ~(bars & ~(np.abs(dx) + np.abs(dy) < 36)), np.clip(0.55 + shade * 0.45, 0, 1), 0.0)
    return rgba(img, glow=glow)
