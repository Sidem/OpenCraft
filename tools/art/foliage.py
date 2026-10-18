"""Tree canopies in the realistic look (docs/ART_STYLE.md section 12): oak (four looks), birch and pine leaf blocks.
Cutouts (alpha 0 or 255) that wrap. Leaves are placed back to front in three depths until the block is covered enough;
each leaf is lit from the top left across its blade, with a paler midrib, and front leaves cast a soft shadow on the
ones behind. No outlines. Colour comes from leaf photos through `real.colour` (materials `leaves` and `needles` in
`realism.py`, palettes only: the photos are flat scans), darkened, since most of a canopy is in its own shade.
Add a tree: a recipe here and its row in `registry.py`.
"""
import numpy as np

from lib import N, blur, coords, fbm, rgba, roll
from real import colour
from style import oklab_to_srgb, srgb_to_oklab

LIGHT = np.array([-0.7071, -0.7071])  # toward the light, in texel coordinates (x right, y down)


def _shadowed(depth, cover):
    """Darkening 0..1 where a nearer leaf lies just toward the light, so it shades this one."""
    near = roll(np.where(cover, depth, 0.0), 2, 2)
    return np.clip(blur(near, 1.2) - np.where(cover, depth, 0.0), 0, 1)


def _finish(field, depth, cover, mat, lift, seed, cool=0.04):
    """Colour the leaves; the darker a texel, the more it is lit by the blue sky instead of the sun, so shade turns
    from the photos' sunlit yellow-green toward blue-green (`cool`, in Oklab b)."""
    X, Y = coords()
    field = field - _shadowed(depth, cover) * 1.2 + (fbm(X, Y, 3, 3, seed + 90) - 0.5) * 0.3
    rgb = colour(mat, field, 2.2, hue_var=1.2, seed=seed, lift=lift, mask=cover)
    lab = srgb_to_oklab(rgb.reshape(-1, 3).astype(float)).reshape(N, N, 3)
    shade = np.clip((0.6 - lab[..., 0]) / 0.35, 0, 1)
    lab[..., 2] -= cool * shade
    lab[..., 1] -= cool * 0.3 * shade
    rgb = oklab_to_srgb(lab.reshape(-1, 3)).reshape(N, N, 3)
    return rgba(rgb, alpha=np.where(cover, 255, 0).astype(np.uint8))


def leaves(seed=0, size=7.0, width=0.5, cover_at=(0.4, 0.58, 0.72), lift=-0.2, mat="leaves"):
    """A broadleaf canopy: pointed oval leaves of half-length about `size` texels and `width` (width over length),
    laid in three depths until the block's covered share reaches each of `cover_at`."""
    rng = np.random.default_rng(seed + 50)
    field, depth = np.zeros((N, N)), np.zeros((N, N))
    cover = np.zeros((N, N), bool)
    r = int(size * 1.6) + 2
    ys, xs = np.mgrid[-r:r + 1, -r:r + 1]
    for layer, target in enumerate(cover_at):
        while cover.mean() < target:
            cx, cy = rng.integers(0, N, 2)
            ang, la = rng.uniform(0, 2 * np.pi), size * rng.uniform(0.75, 1.3)
            u = (xs * np.cos(ang) + ys * np.sin(ang)) / la  # -1 at the stalk, +1 at the tip
            w = (-xs * np.sin(ang) + ys * np.cos(ang)) / (la * width)
            shape = np.clip(1 - u ** 2, 0, 1) ** 0.7 * (1 - 0.25 * u)  # widest a little before the middle
            inside = (np.abs(u) < 1) & (np.abs(w) < shape)
            tilt = rng.uniform(-0.25, 0.25)  # how squarely the leaf faces the light
            across = (xs * LIGHT[0] + ys * LIGHT[1]) / la  # brighter toward the light, a curved blade
            v = (0.3 + layer * 0.25 + tilt + across * 0.12 - np.abs(w) * 0.08
                 + 0.06 * (np.abs(w) < 0.12) * (u < 0.85))
            px, py = (cx + xs) % N, (cy + ys) % N
            field[py[inside], px[inside]] = v[inside]
            depth[py[inside], px[inside]] = (layer + 1) / 3 + rng.uniform(0, 0.05)
            cover[py[inside], px[inside]] = True
    return _finish(field, depth, cover, mat, lift, seed)


def oak_leaves(alt=0):
    return leaves(seed=alt * 7)


def birch_leaves():
    """Small, light, toothy-oval birch leaves, a sparser crown."""
    return leaves(seed=3, size=5.5, width=0.62, cover_at=(0.36, 0.52, 0.64), lift=-0.12)


def pine_leaves(seed=4):
    """Pine needles: twigs crossing the block, each set with long thin needles in pairs to both sides, the tips
    paler; three depths, the far twigs darker."""
    rng = np.random.default_rng(seed + 50)
    field, depth = np.zeros((N, N)), np.zeros((N, N))
    cover = np.zeros((N, N), bool)
    for layer, target in enumerate((0.3, 0.46, 0.58)):
        while cover.mean() < target:
            x, y = rng.uniform(0, N, 2)
            ang = rng.uniform(0, 2 * np.pi)
            tilt = rng.uniform(-0.2, 0.2)
            for s in range(int(rng.integers(14, 30))):  # walk along the twig
                tx, ty = x + np.cos(ang) * s * 1.5, y + np.sin(ang) * s * 1.5
                for side in (-1, 1):
                    a = ang + side * rng.uniform(0.5, 0.9)
                    length = int(rng.integers(7, 12))
                    lit = 0.12 * (np.cos(a) * LIGHT[0] + np.sin(a) * LIGHT[1])
                    for k in range(length):
                        px, py = int(tx + np.cos(a) * k) % N, int(ty + np.sin(a) * k) % N
                        field[py, px] = 0.3 + layer * 0.25 + tilt + lit + 0.15 * k / length
                        depth[py, px] = (layer + 1) / 3
                        cover[py, px] = True
                ang += rng.uniform(-0.08, 0.08)
    return _finish(field, depth, cover, "needles", -0.12, seed, cool=0.06)
