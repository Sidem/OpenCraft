"""Painting primitives for the texture tool: periodic noise (every function wraps at the tile edge, so tiles repeat
seamlessly), cellular patterns, height-to-light, hue-shifted ramps and ordered-dither quantisation.

A tile is an `N` x `N` array. Coordinates `X, Y` run 0..1 across it; anything noise-like takes them times an integer
`cells`, which is its period, so it wraps. Shapes that must stay crisp (pixel-art edges) are thresholded, never blurred.
"""
import numpy as np

from style import oklab_to_srgb, srgb_to_oklab

N = 128


def coords(n=N):
    ys, xs = np.mgrid[0:n, 0:n]
    return (xs + 0.5) / n, (ys + 0.5) / n


# ---------------------------------------------------------------------------------------------------- noise

def hash2(ix, iy, seed):
    """Per-cell white noise in [0, 1) from integer coordinates (vectorised, wraps in uint32)."""
    with np.errstate(over="ignore"):
        a = (np.asarray(ix, np.int64) & 0xFFFFFFFF).astype(np.uint32)
        b = (np.asarray(iy, np.int64) & 0xFFFFFFFF).astype(np.uint32)
        h = a * np.uint32(374761393) + b * np.uint32(668265263) + np.uint32((seed * 2246822519 + 12345) & 0xFFFFFFFF)
        h = (h ^ (h >> np.uint32(13))) * np.uint32(1274126177)
        h = h ^ (h >> np.uint32(16))
    return h.astype(np.float64) / 4294967296.0


def vnoise(x, y, period, seed):
    """Smooth value noise at lattice coordinates (x, y), wrapping every `period` cells. Range 0..1."""
    x0, y0 = np.floor(x), np.floor(y)
    tx, ty = x - x0, y - y0
    tx, ty = tx * tx * (3 - 2 * tx), ty * ty * (3 - 2 * ty)
    ix0, iy0 = x0.astype(np.int64) % period, y0.astype(np.int64) % period
    ix1, iy1 = (ix0 + 1) % period, (iy0 + 1) % period
    a = hash2(ix0, iy0, seed) * (1 - tx) + hash2(ix1, iy0, seed) * tx
    b = hash2(ix0, iy1, seed) * (1 - tx) + hash2(ix1, iy1, seed) * tx
    return a * (1 - ty) + b * ty


def fbm(X, Y, cells, octaves=4, seed=0, gain=0.5, warp=0.0):
    """Fractal noise 0..1. `warp` bends the sample point by another noise (in tile fractions) for organic shapes."""
    if warp:
        wx = vnoise(X * 3, Y * 3, 3, seed + 91) - 0.5
        wy = vnoise(X * 3, Y * 3, 3, seed + 92) - 0.5
        X, Y = X + wx * warp, Y + wy * warp
    total, amp, norm = 0.0, 1.0, 0.0
    for o in range(octaves):
        c = cells * 2 ** o
        total = total + amp * vnoise(X * c, Y * c, c, seed + o * 7)
        norm += amp
        amp *= gain
    return total / norm


def worley(X, Y, cells, seed=0, jitter=1.0):
    """Cellular noise: (distance to nearest point, to second nearest, a 0..1 id of the nearest cell), in cell units."""
    gx, gy = X * cells, Y * cells
    ix, iy = np.floor(gx).astype(np.int64), np.floor(gy).astype(np.int64)
    f1 = np.full(gx.shape, 9.0)
    f2 = np.full(gx.shape, 9.0)
    ident = np.zeros(gx.shape)
    for oy in (-1, 0, 1):
        for ox in (-1, 0, 1):
            cx, cy = ix + ox, iy + oy
            px = cx + 0.5 + (hash2(cx % cells, cy % cells, seed) - 0.5) * jitter
            py = cy + 0.5 + (hash2(cx % cells, cy % cells, seed + 5) - 0.5) * jitter
            d = np.hypot(px - gx, py - gy)
            closer = d < f1
            f2 = np.where(closer, f1, np.minimum(f2, d))
            ident = np.where(closer, hash2(cx % cells, cy % cells, seed + 9), ident)
            f1 = np.where(closer, d, f1)
    return f1, f2, ident


def blur(a, sigma):
    """Periodic gaussian blur (sigma in texels) through the FFT, so it wraps at the tile edge."""
    n = a.shape[0]
    f = np.fft.fftfreq(n)
    k = np.exp(-2 * (np.pi * sigma) ** 2 * (f[:, None] ** 2 + f[None, :] ** 2))
    return np.real(np.fft.ifft2(np.fft.fft2(a) * k))


def roll(a, dx, dy):
    return np.roll(np.roll(a, dy, 0), dx, 1)


def edges(mask):
    """Texels of `mask` that touch a texel outside it (4-neighbourhood, wrapping)."""
    inner = mask & roll(mask, 1, 0) & roll(mask, -1, 0) & roll(mask, 0, 1) & roll(mask, 0, -1)
    return mask & ~inner


def grow(mask, r=1):
    out = mask.copy()
    for _ in range(r):
        out = out | roll(out, 1, 0) | roll(out, -1, 0) | roll(out, 0, 1) | roll(out, 0, -1)
    return out


# ---------------------------------------------------------------------------------------------------- light

def relief_light(height, strength=1.0, light=(-0.55, -0.55)):
    """Lambert-ish shade 0..1 (0.5 flat) of a height field lit from the top left; wraps at the edge."""
    gx = (roll(height, -1, 0) - roll(height, 1, 0)) * 0.5
    gy = (roll(height, 0, -1) - roll(height, 0, 1)) * 0.5
    return np.clip(0.5 + strength * (gx * -light[0] + gy * -light[1]), 0, 1)


def cavity(height, sigma=3.0):
    """Ambient occlusion-like darkening 0..1 (1 deep in a crevice) from how far a texel sits below its surroundings."""
    return np.clip(blur(height, sigma) - height, 0, None)


BAYER4 = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]) / 16.0 - 0.5


def bayer(n=N):
    return np.tile(BAYER4, (n // 4, n // 4))


# ---------------------------------------------------------------------------------------------------- colour

def hex_rgb(h):
    h = h.lstrip("#")
    return [int(h[i:i + 2], 16) for i in (0, 2, 4)]


def ramp(stops, k=9):
    """`k` tones interpolated in Oklab through hex colour stops (dark to light). Returns uint8 (k, 3)."""
    lab = srgb_to_oklab(np.array([hex_rgb(s) for s in stops], float))
    t = np.linspace(0, 1, k) * (len(stops) - 1)
    i = np.minimum(t.astype(int), len(stops) - 2)
    f = (t - i)[:, None]
    return oklab_to_srgb(lab[i] * (1 - f) + lab[i + 1] * f)


def paint(tones, v, dither=0.35, n=N):
    """Quantise a 0..1 field to the ramp's tones with ordered dither (dither 0 gives flat bands). Returns (n, n, 3)."""
    k = len(tones)
    idx = np.clip(np.floor(v * k + bayer(n) * dither), 0, k - 1).astype(int)
    return tones[idx].astype(np.uint8)


def snap(img, tones):
    """Each texel moved to the nearest of `tones` (RGB distance): shading by arithmetic keeps a painted palette this way."""
    d = ((img[..., None, :].astype(int) - tones[None, None].astype(int)) ** 2).sum(-1)
    return tones[d.argmin(-1)].astype(np.uint8)


def tone_at(tones, i):
    return tones[int(np.clip(i, 0, len(tones) - 1))]


def stroke(img, tones, x, y, dx, dy, length, shade, width=1):
    """Draws a short stroke into `img` (wrapping) darkest at its root and lightest at its tip, for blades and veins."""
    n = img.shape[0]
    for s in range(length):
        t = s / max(1, length - 1)
        px, py = int(round(x + dx * s)) % n, int(round(y + dy * s)) % n
        for w in range(width):
            img[py, (px + w) % n] = tone_at(tones, (shade + t * 3) * (len(tones) - 1) / 8)


def rgba(rgb, alpha=None, glow=None, n=N):
    """A finished tile: an (n, n, 5) uint8 array of R, G, B, coverage (255 unless a cutout) and glow (0..1 self-light,
    stored 0..255; the engine adds it as light that no daylight or shadow dims)."""
    out = np.zeros((n, n, 5), np.uint8)
    out[..., :3] = rgb
    out[..., 3] = 255 if alpha is None else alpha
    if glow is not None:
        out[..., 4] = np.clip(np.asarray(glow) * 255, 0, 255).astype(np.uint8)
    return out
