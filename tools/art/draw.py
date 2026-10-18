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


def curve(p0, p1, bow=0.0, n=80):
    """Points of a quadratic curve from `p0` to `p1` bowed sideways by `bow` (fraction of its length)."""
    p0, p1 = np.array(p0, float), np.array(p1, float)
    d = p1 - p0
    ctrl = (p0 + p1) / 2 + np.array([-d[1], d[0]]) * bow
    t = np.linspace(0, 1, n)[:, None]
    return (1 - t) ** 2 * p0 + 2 * (1 - t) * t * ctrl + t ** 2 * p1


class Pen:
    def __init__(self, glow=False):
        self.rgb = np.zeros((N, N, 3), np.uint8)
        self.cover = np.zeros((N, N), bool)
        self.glow = np.zeros((N, N)) if glow else None

    def fill(self, mask, tones, shade, glow=None):
        idx = np.clip(np.round(shade * (len(tones) - 1)), 0, len(tones) - 1).astype(int)
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

    def rgba(self, glow_scale=1.0):
        rim = edges(self.cover) & self.cover
        rgb = self.rgb.copy()
        rgb[rim] = (rgb[rim] * 0.62).astype(np.uint8)
        a = np.where(self.cover, 255, 0).astype(np.uint8)
        glow = None if self.glow is None else self.glow * glow_scale
        return rgba(rgb, alpha=a, glow=glow)
