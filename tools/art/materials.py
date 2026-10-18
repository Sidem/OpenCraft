"""Plain materials for the item models (`crates/engine/src/item_models`, `textures/materials.rs`): even surfaces in
real-world colours with only fine structure (brushing, mottle, weave, a soft sheen), never frames, rivets or a picture
of an item, so they read on parts of any size. Also the faces printed on such parts (a circuit board's traces, a
processor's lid, a wafer's dies) and the canisters' paints. Continuous tone: registry entries are `real` (no photo
references, so only the value rule judges them).
"""
import numpy as np

from lib import N, blur, coords, fbm, hash2, rgba

_I = np.arange(N)
YS, XS = np.meshgrid(_I, _I, indexing="ij")


def _hex(h):
    h = h.lstrip("#")
    return np.array([int(h[i:i + 2], 16) for i in (0, 2, 4)], float)


def _shade(colour, v):
    """`colour` (hex) scaled by the brightness field `v` (1 = as given)."""
    return np.clip(_hex(colour)[None, None] * v[..., None], 0, 255).astype(np.uint8)


def _sheen(X, Y, k):
    """A broad soft highlight toward the top left, so a flat face reads as a smooth solid."""
    return 1 + k * (0.5 - (X * 0.6 + Y * 0.4)) * 1.4


def brushed(colour, seed, depth=0.07):
    """Metal brushed along x: fine streaks the width of the tile, a slow wander in strength, a soft sheen."""
    X, Y = coords()
    rows = blur(hash2(np.zeros(N, int), _I, seed)[:, None].repeat(N, 1), 0.6)
    streak = (rows - 0.5) * 2 * (0.6 + 0.4 * fbm(X, Y, 2, 2, seed + 1))
    v = _sheen(X, Y, 0.08) + streak * depth + (fbm(X, Y, 6, 3, seed + 2) - 0.5) * 0.05
    return rgba(_shade(colour, v))


def rubber(seed=920):
    """Black rubber: matte, a faint mottle and a fine grain."""
    X, Y = coords()
    v = 1 + (fbm(X, Y, 5, 3, seed) - 0.5) * 0.12 + (hash2(XS, YS, seed + 1) - 0.5) * 0.06 + _sheen(X, Y, 0.05) - 1
    return rgba(_shade("#34343a", v))


def webbing(seed=940):
    """Woven nylon webbing: an over-under weave of 4-texel threads, each round-shaded, olive grey."""
    X, Y = coords()
    cell = ((XS // 4) + (YS // 4)) % 2 == 0
    across = np.where(cell, XS % 4, YS % 4)  # position across the thread on top
    thread = 0.86 + 0.14 * np.sin((across + 0.5) / 4 * np.pi)
    v = thread + (fbm(X, Y, 4, 2, seed) - 0.5) * 0.08 + (hash2(XS // 2, YS // 2, seed + 1) - 0.5) * 0.05
    return rgba(_shade("#66684f", v))


def paint(colour, seed, mottle=0.06, sheen=0.1):
    """Smooth paint on metal: a faint orange-peel mottle and a soft sheen."""
    X, Y = coords()
    v = _sheen(X, Y, sheen) + (fbm(X, Y, 8, 3, seed) - 0.5) * mottle + (fbm(X, Y, 24, 2, seed + 1) - 0.5) * mottle * 0.5
    return rgba(_shade(colour, v))


def plastic(seed=216):
    """White moulded plastic: an even neutral white with a glossy sheen across it, the faintest mottle, no grain."""
    X, Y = coords()
    v = _sheen(X, Y, 0.16) + (fbm(X, Y, 4, 2, seed) - 0.5) * 0.03
    gloss = np.exp(-(((X - 0.3) * 0.8 + (Y - 0.28)) / 0.14) ** 2) * 0.08  # a soft reflected band
    return rgba(_shade("#aeb1b6", v + gloss))


def drum(colour, seed, bare=False):
    """A canister's painted steel: its fluid's colour (bare brushed steel when empty), a darker rolled rim top and
    bottom, faint wear on the paint."""
    X, Y = coords()
    if bare:
        img = brushed(colour, seed, 0.05)[..., :3].astype(float)
    else:
        v = _sheen(X, Y, 0.1) + (fbm(X, Y, 6, 3, seed) - 0.5) * 0.08
        img = _shade(colour, v).astype(float)
        near_rim = np.minimum(YS, N - 1 - YS) / 24.0
        wear = np.clip((fbm(X, Y, 10, 3, seed + 1) - 0.45 - near_rim * 0.3) * 4, 0, 1)[..., None]
        img = img * (1 - wear * 0.55) + _hex("#8a9098") * wear * 0.55  # paint scuffed through to steel at the rims
    rim = (YS < 6) | (YS > N - 7)
    img[rim] *= 0.72
    return rgba(np.clip(img, 0, 255).astype(np.uint8))


def board(seed=154):
    """A circuit board's top: green solder mask over a woven fibre base, copper traces routed in straight runs with
    45-degree bends showing as lighter green, gold vias and pads."""
    X, Y = coords()
    base = 1 + (fbm(X, Y, 16, 2, seed) - 0.5) * 0.08 + (((XS + YS) % 6) < 3) * 0.02
    trace = np.zeros((N, N), bool)
    rng = np.random.default_rng(seed)
    pads = []
    for _ in range(26):
        x, y = int(rng.integers(6, N - 6)), int(rng.integers(6, N - 6))
        for _ in range(3):  # a run, a 45-degree bend, a run
            dx, dy = [(1, 0), (0, 1), (1, 1), (1, -1)][int(rng.integers(0, 4))]
            sign = 1 if rng.random() < 0.5 else -1
            for _ in range(int(rng.integers(6, 26))):
                x, y = (x + dx * sign) % N, (y + dy * sign) % N
                trace[y, x] = trace[y, (x + 1) % N] = True
        pads.append((x, y))
    img = _shade("#1d5e34", base).astype(float)
    img[trace] = _shade("#2f8a4c", base)[trace]
    gold = np.zeros((N, N), bool)
    for x, y in pads:
        r = np.hypot((XS - x + N / 2) % N - N / 2, (YS - y + N / 2) % N - N / 2)
        gold |= r < 2.6
    img[gold] = _hex("#d0aa52")
    img[gold & (np.hypot(XS % 3 - 1, YS % 3 - 1) < 0.5)] = _hex("#6a5428")
    return rgba(img.astype(np.uint8))


def chip_lid(seed=167):
    """A processor's nickel lid: fine matte grain, a bevelled edge, laser-etched lines of marking and a dot code."""
    X, Y = coords()
    v = _sheen(X, Y, 0.08) + (hash2(XS, YS, seed) - 0.5) * 0.05 + (fbm(X, Y, 6, 2, seed + 1) - 0.5) * 0.04
    edge = np.minimum(np.minimum(XS, N - 1 - XS), np.minimum(YS, N - 1 - YS))
    v = np.where(edge < 6, v * (0.8 + 0.04 * edge), v)
    mark = np.zeros((N, N), bool)
    rng = np.random.default_rng(seed)
    for row, y in enumerate((30, 42, 54)):
        x = 26
        while x < 92 - row * 14:
            w = int(rng.integers(3, 8))
            mark |= (XS >= x) & (XS < x + w) & (YS >= y) & (YS < y + 6) & (rng.random() < 0.9)
            x += w + 2
    code = (XS >= 76) & (XS < 100) & (YS >= 76) & (YS < 100)
    mark |= code & (hash2(XS // 3, YS // 3, seed + 3) > 0.5)
    img = _shade("#a6aaae", v)
    img[mark] = _shade("#6e7276", v)[mark]
    return rgba(img)


def dies(seed=247):
    """A wafer's face: square dies in narrow scribe lanes, each a few blocks of circuitry, the polished silicon
    shimmering from violet through green to gold across the wafer."""
    X, Y = coords()
    lane = ((XS % 16) < 2) | ((YS % 16) < 2)
    hue = fbm(X, Y, 2, 2, seed) * 2 + X * 0.6
    tint = np.stack([0.5 + 0.5 * np.cos(hue * 6.28 + k) for k in (0.0, 2.1, 4.2)], -1)
    block = ((XS % 16 > 3) & (XS % 16 < 9) & (YS % 16 > 3) & (YS % 16 < 13)) | \
            ((XS % 16 > 10) & (XS % 16 < 14) & (YS % 16 > 3) & (YS % 16 < 8))
    v = np.where(block, 0.78, 1.0) * (1 + (fbm(X, Y, 8, 2, seed + 1) - 0.5) * 0.1)
    img = (_hex("#5c6278")[None, None] * 0.84 + tint * 255 * 0.16) * v[..., None]
    img[lane] = _hex("#9aa0aa")
    return rgba(np.clip(img, 0, 255).astype(np.uint8))
