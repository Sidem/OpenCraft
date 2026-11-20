"""A painter's pen for cutout sprites (plants, torch): shapes in plant space, crisp and shaded, layered back to front.

Plant space: x 0..1 left to right, y 0..1 bottom to top (the sprite stands on y = 0, the centre of the bottom edge is
(0.5, 0)). Every shape is thresholded (no anti-aliasing: a texel is covered or not) and shaded by its own form lit from the
top left, with colours from a ramp (dark to light). `Pen.rgba()` darkens the silhouette's rim, so sprites hold together
against any ground, and returns the tile with coverage 0 or 255.
"""
import numpy as np

from lib import N, edges, rgba, roll

_YY, _XX = np.mgrid[0:N, 0:N]
PX, PY = (_XX + 0.5) / N, 1 - (_YY + 0.5) / N
LIGHT = np.array([-0.5, 0.6, 0.62])


def _grain(seed=0):
    """Fine deterministic per-texel value noise in -1..1 for continuous botanical shading."""
    rng = np.random.default_rng(seed)
    g = rng.uniform(-1.0, 1.0, (N, N))
    return (g + roll(g, 1, 0) + roll(g, -1, 0) + roll(g, 0, 1) + roll(g, 0, -1)) / 5.0


def _bleed(rgb, cover, steps=8):
    """Fills transparent texels from their nearest opaque neighbours so mipmaps never get dark fringes."""
    out = rgb.astype(float)
    filled = cover.copy()
    for _ in range(steps):
        if filled.all():
            break
        acc = np.zeros_like(out)
        cnt = np.zeros(cover.shape, float)
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
            m = np.roll(filled, (dy, dx), axis=(0, 1))
            if dy == 1:
                m[0, :] = False
            elif dy == -1:
                m[-1, :] = False
            if dx == 1:
                m[:, 0] = False
            elif dx == -1:
                m[:, -1] = False
            acc += np.roll(out, (dy, dx), axis=(0, 1)) * m[..., None]
            cnt += m
        grow = (~filled) & (cnt > 0)
        out[grow] = acc[grow] / cnt[grow, None]
        filled |= grow
    if (~filled).any() and cover.any():
        out[~filled] = rgb[cover].mean(0)
    return np.clip(np.round(out), 0, 255).astype(np.uint8)


def curve(p0, p1, bow=0.0, n=80):
    """Points of a quadratic curve from `p0` to `p1` bowed sideways by `bow` (fraction of its length)."""
    p0, p1 = np.array(p0, float), np.array(p1, float)
    d = p1 - p0
    ctrl = (p0 + p1) / 2 + np.array([-d[1], d[0]]) * bow
    t = np.linspace(0, 1, n)[:, None]
    return (1 - t) ** 2 * p0 + 2 * (1 - t) * t * ctrl + t ** 2 * p1


class Pen:
    def __init__(self, glow=False, smooth=False, seed=0):
        self.rgb = np.zeros((N, N, 3), np.uint8)
        self.cover = np.zeros((N, N), bool)
        self.glow = np.zeros((N, N)) if glow else None
        self.smooth = smooth
        self.noise = _grain(seed) if smooth else None

    def fill(self, mask, tones, shade, glow=None):
        if not np.any(mask):
            return
        t = np.clip(shade, 0.0, 1.0)
        if self.smooth:
            t = np.clip(t + self.noise * 0.04, 0.0, 1.0)
            pos = t[mask] * (len(tones) - 1)
            lo = np.floor(pos).astype(int)
            hi = np.minimum(lo + 1, len(tones) - 1)
            f = (pos - lo)[:, None]
            col = tones[lo] * (1.0 - f) + tones[hi] * f
            self.rgb[mask] = np.clip(np.round(col), 0, 255).astype(np.uint8)
        else:
            idx = np.clip(np.round(t * (len(tones) - 1)), 0, len(tones) - 1).astype(int)
            self.rgb[mask] = tones[idx][mask]
        self.cover |= mask
        if glow is not None and self.glow is not None:
            self.glow[mask] = glow

    def stroke(self, pts, widths, tones, light=0.5, glow=None):
        """A tapered ribbon along `pts` with half-width `widths` (one number or an array along the curve). The left edge
        is lit, the right in shade, and the shade rises toward the tip."""
        w = np.broadcast_to(np.asarray(widths, float), (len(pts),))
        dx, dy = PX[..., None] - pts[:, 0], PY[..., None] - pts[:, 1]
        d = np.hypot(dx, dy)
        k = d.argmin(-1)
        near = np.take_along_axis(d, k[..., None], -1)[..., 0]
        mask = near < w[k]
        tan = np.gradient(pts, axis=0)
        tan /= np.hypot(tan[:, 0], tan[:, 1])[:, None] + 1e-9
        across = (np.take_along_axis(dx, k[..., None], -1)[..., 0] * -tan[k, 1]
                  + np.take_along_axis(dy, k[..., None], -1)[..., 0] * tan[k, 0]) / (w[k] + 1e-9)
        shade = light + 0.28 * across + 0.18 * (k / (len(pts) - 1))
        self.fill(mask, tones, shade, glow)
        return mask

    def blade(self, p0, p1, bow, width, tones, light=0.5, taper=0.85):
        pts = curve(p0, p1, bow)
        return self.stroke(pts, width * (1 - taper * np.linspace(0, 1, len(pts)) ** 1.3) + 0.002, tones, light)

    def ellipse(self, c, rx, ry, rot, tones, light=0.55, glow=None):
        """A domed ellipse (a leaf, a petal, a berry): lit from the top left like a lump."""
        dx, dy = PX - c[0], PY - c[1]
        u = dx * np.cos(rot) + dy * np.sin(rot)
        v = -dx * np.sin(rot) + dy * np.cos(rot)
        q = (u / rx) ** 2 + (v / ry) ** 2
        mask = q < 1
        rm = (rx + ry) / 2
        nx, ny = dx / rm, dy / rm
        nz = np.sqrt(np.clip(1 - np.minimum(q, 1), 0, 1))
        lam = (LIGHT[0] * nx + LIGHT[1] * ny) * 0.5 + LIGHT[2] * nz
        self.fill(mask, tones, light + (lam - 0.6) * 0.9, glow)
        return mask

    def leaf(self, base, tip, width, tones, light=0.55):
        """A pointed leaf from `base` to `tip` with a lit half and a shaded half split along its midrib."""
        (bx, by), (tx, ty) = base, tip
        length = np.hypot(tx - bx, ty - by)
        ang = np.arctan2(ty - by, tx - bx)
        dx, dy = PX - bx, PY - by
        u = (dx * np.cos(ang) + dy * np.sin(ang)) / length
        v = (-dx * np.sin(ang) + dy * np.cos(ang)) / width
        mask = (u > 0) & (u < 1) & (np.abs(v) < np.sin(np.pi * np.clip(u, 0, 1)) ** 0.8)
        self.fill(mask, tones, light + 0.28 * np.sign(v) * (np.abs(v) > 0.06) + 0.12 * (u - 0.5))
        return mask

    def petals(self, c, count, length, width, tones, light=0.6, start=0.0):
        for i in range(count):
            a = start + 2 * np.pi * i / count
            mid = (c[0] + np.cos(a) * length * 0.55, c[1] + np.sin(a) * length * 0.55)
            self.ellipse(mid, length * 0.5, width, a, tones, light + 0.1 * np.sin(a))

    def petal(self, c, angle, length, width, tones, light=0.65, shape=0.75, ruffle=0.0, vein=0.08):
        """A botanical petal radiating from `c` along `angle`, shaded darker at the throat and veined along its length."""
        dx, dy = PX - c[0], PY - c[1]
        ca, sa = np.cos(angle), np.sin(angle)
        u = (dx * ca + dy * sa) / length
        v = (-dx * sa + dy * ca) / width
        u_c = np.clip(u, 0.0, 1.0)
        # Widest past the middle (obovate) for flowers like poppies/daisies; sharper when shape > 1.
        env = np.sin(np.pi * u_c ** shape) ** 0.72
        if ruffle:
            env = env * (1.0 + ruffle * np.sin(v * 6.0 + u * 11.0))
        mask = (u > 0.0) & (u < 1.0) & (np.abs(v) < env)
        # Base-to-tip glow, gentle top-left light, subtle longitudinal veining.
        sun = 0.14 * (ca * LIGHT[0] + sa * LIGHT[1])
        crease = -vein * np.cos(v * 5.0) * (1.0 - 0.5 * u_c)
        shade = light - 0.22 * (1.0 - u_c) ** 1.6 + 0.16 * u_c + sun + crease - 0.08 * np.abs(v) / np.maximum(env, 0.05)
        self.fill(mask, tones, shade)
        return mask

    def bell(self, top, length, width, tilt, tones, light=0.6):
        """A hanging bellflower from its stem attachment `top`, flaring into a scalloped lip at the bottom."""
        dx, dy = PX - top[0], PY - top[1]
        ct, st = np.cos(tilt), np.sin(tilt)
        # Along the bell downward (u: 0 at crown, 1 at flared lip) and across (v).
        u = (-dy * ct + dx * st) / length
        v = (dx * ct + dy * st) / width
        u_c = np.clip(u, 0.0, 1.0)
        # Rounded shoulder widening gently, then flaring at the mouth with gentle scalloped tips.
        flare = np.sin(np.pi * np.minimum(u_c / 0.72, 0.5)) ** 0.75 * 0.78 + 0.36 * np.maximum(u_c - 0.65, 0.0) / 0.35
        lip = 1.0 - 0.08 * np.cos(v * 5.5) * (u_c > 0.75)
        mask = (u > 0.0) & (u < lip) & (np.abs(v) < flare)
        across = v / np.maximum(flare, 0.05)
        # Lit left side, darker underside/throat, pale flared lip.
        shade = light - 0.24 * across + 0.22 * np.maximum(u_c - 0.68, 0.0) / 0.32 - 0.12 * (1.0 - u_c) ** 2
        self.fill(mask, tones, shade)
        return mask

    def blob(self, c, rx, ry, seed, tones, light=0.55, wobble=0.22):
        """An organic ruffled lobe (lichen thallus, mossy cushion) with layered wavy harmonics."""
        rng = np.random.default_rng(seed)
        dx, dy = PX - c[0], PY - c[1]
        ang = np.arctan2(dy, dx)
        rad = np.hypot(dx / rx, dy / ry)
        r = np.ones_like(ang)
        for k in (2, 3, 5, 7):
            r += rng.uniform(0.3, 1.0) * (wobble / (k * 0.5)) * np.sin(k * ang + rng.uniform(0, 6.28))
        q = rad / np.maximum(r, 0.35)
        mask = q < 1.0
        # Soft radial height with bright ruffled margins and directional shading.
        edge_lift = 0.16 * np.clip((q - 0.65) / 0.35, 0.0, 1.0)
        dir_l = 0.14 * (LIGHT[0] * dx / max(rx, 1e-3) + LIGHT[1] * dy / max(ry, 1e-3))
        shade = light + 0.12 * (1.0 - q) + edge_lift + dir_l
        self.fill(mask, tones, shade)
        return mask

    def rgba(self, glow_scale=1.0):
        rgb = self.rgb.copy()
        if self.smooth:
            rgb = _bleed(rgb, self.cover)
        else:
            rim = edges(self.cover) & self.cover
            rgb[rim] = (rgb[rim] * 0.62).astype(np.uint8)
        a = np.where(self.cover, 255, 0).astype(np.uint8)
        glow = None if self.glow is None else self.glow * glow_scale
        return rgba(rgb, alpha=a, glow=glow)
