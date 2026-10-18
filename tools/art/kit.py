"""The machine kit: a `Face` is a bevelled steel panel you build a machine side or top on by stacking parts (inset
windows, grilles, ports, coils, gauges, hazard bands, tier stripes). Every part is a crisp, shaded shape lit from the top
left; parts that give light set `glow` (0..1), which the pack turns into self-light.

Coordinates are texels, (x0, y0, x1, y1) with x1 and y1 excluded; the panel's rim is 7 texels, so content lives in 8..120.
Recipes in `industry.py`, `heavy.py` and `later.py` are short lists of parts. Add a part here when a third recipe needs it.
"""
import numpy as np

from lib import N, bayer, coords, fbm, grow, hash2, paint, ramp, relief_light, rgba, roll, vnoise
from machines import AETHER, AMBER, BRONZE, COPPER, EMBER, FIREBRICK, STEEL, VERDIGRIS, plate, rivet

YS, XS = np.mgrid[0:N, 0:N]
DARK = ramp(["#030409", "#080b13", "#0f1420", "#171d2e"])
MOSS = ramp(["#0c1a14", "#143022", "#1f4528", "#2f5e2c", "#46792f"])
CONCRETE = ramp(["#10141c", "#252a35", "#3d4350", "#5a606c", "#7b7f88", "#a0a1a6", "#c4c2c0"])
VIOLET = ramp(["#120a24", "#2a1650", "#4d2a8a", "#7e4fc4", "#b88cf0", "#ecd8ff"])
GOLD = ramp(["#2a1606", "#5e3a0c", "#a2701a", "#d9a42e", "#f4cf5c", "#fff3b0"])
BLUE = ramp(["#07102a", "#102a5e", "#1f4fa8", "#3f7fe0", "#8cb8ff", "#dcebff"])
GREEN = ramp(["#06200f", "#0f4a22", "#1f8a3a", "#4cc866", "#9af0a8", "#e0ffe4"])
RED = ramp(["#2a0508", "#661016", "#b32030", "#e84a3a", "#ff8d63", "#ffd9c0"])
TIERS = [EMBER, AETHER, BLUE, VIOLET, GOLD]  # Mk1 to Mk5 accents (the belts' chevron colours, the miners' stripes)


class Face:
    def __init__(self, seed=0, tones=STEEL, border=7):
        self.seed = seed
        self.img, _, self.d = plate(seed, border, tones)
        self.glow = np.zeros((N, N))
        self.alpha = None
        self.X, self.Y = coords()

    # -------------------------------------------------------------------------------------------- shapes
    def _shade(self, m, dist, tones, base, relief, b, grain, glow, dither=0.25, outline=True):
        h = np.where(m, np.clip(dist / b, 0, 1), 0.0)
        light = relief_light(h * 6, 1.0)
        v = base + relief * (light - 0.5) * 1.4 + (fbm(self.X, self.Y, 9, 2, self.seed + 5) - 0.5) * grain
        px = paint(tones, np.where(m & (dist < 1) & outline, 0.0, v), dither)
        self.img[m] = px[m]
        if glow is not None:
            self.glow[m] = np.maximum(self.glow[m], glow if np.isscalar(glow) else np.asarray(glow)[m])

    def rect(self, box, tones=STEEL, base=0.4, relief=1.0, b=4, glow=None, grain=0.15, outline=True):
        x0, y0, x1, y1 = box
        m = (XS >= x0) & (XS < x1) & (YS >= y0) & (YS < y1)
        dist = np.minimum(np.minimum(XS - x0, x1 - 1 - XS), np.minimum(YS - y0, y1 - 1 - YS))
        self._shade(m, dist, tones, base, relief, b, grain, glow, outline=outline)
        return m

    def disc(self, c, r, tones=STEEL, base=0.4, relief=1.0, b=4, glow=None, grain=0.15):
        d = np.hypot(XS - c[0] + 0.5, YS - c[1] + 0.5)
        m = d < r
        self._shade(m, r - d, tones, base, relief, b, grain, glow)
        return m

    def fill(self, m, tones, base, glow=None, grain=0.2):
        """A free-form mask in one tone family (no bevel)."""
        v = base + (fbm(self.X, self.Y, 9, 2, self.seed + 6) - 0.5) * grain
        self.img[m] = paint(tones, v, 0.3)[m]
        if glow is not None:
            self.glow[m] = np.maximum(self.glow[m], glow)

    # -------------------------------------------------------------------------------------------- parts
    def slots(self, box, count, vertical=False, inner=DARK, glow=None, base=0.3):
        """`count` recessed slots filling `box`, a lit lip under each; `inner` may be a glowing ramp."""
        x0, y0, x1, y1 = box
        span = (x1 - x0) if vertical else (y1 - y0)
        pitch = span / count
        for i in range(count):
            a = int(i * pitch + pitch * 0.25)
            z = int(i * pitch + pitch * 0.75)
            b = (x0 + a, y0, x0 + z, y1) if vertical else (x0, y0 + a, x1, y0 + z)
            self.rect(b, inner, base + 0.2 * (i % 2) * (glow is not None), -1.0, 2, glow, 0.2)

    def hazard(self, box, a=GOLD, b=DARK, period=18, base=0.55):
        x0, y0, x1, y1 = box
        self.rect(box, STEEL, 0.3, 1.0, 2)
        m = (XS > x0) & (XS < x1 - 1) & (YS > y0) & (YS < y1 - 1)
        band = ((XS + YS) % period) < period // 2
        self.img[m & band] = a[int(base * (len(a) - 1))]
        self.img[m & ~band] = b[1]

    def stripe(self, box, tones, glow=0.35):
        """A tier band: bevelled, in the tier ramp, faintly lit (an LED strip)."""
        self.rect(box, tones, 0.62, 1.0, 3, glow)

    def window(self, box, tones, glow=0.8, frame=BRONZE, wall=3):
        x0, y0, x1, y1 = box
        self.rect(box, frame, 0.5, 1.0, wall + 1)
        inner = (x0 + wall, y0 + wall, x1 - wall, y1 - wall)
        sheen = 0.45 + 0.4 * (fbm(self.X, self.Y, 4, 2, self.seed + 9))
        self.rect(inner, tones, 0.5, -1.0, 3, glow * sheen, 0.4)

    def port(self, c, r, inner=EMBER, glow=0.9, ring=BRONZE):
        self.disc(c, r, ring, 0.5, 1.0, 4)
        self.disc(c, r - 5, DARK, 0.3, -1.0, 3)
        self.disc(c, r - 9, inner, 0.55, -1.0, 4, glow, 0.5)

    def fan(self, c, r, blades=6, tones=STEEL, inner=DARK, glow=None):
        self.disc(c, r, STEEL, 0.35, 1.0, 4)
        self.disc(c, r - 5, inner, 0.45, -1.0, 4, glow, 0.4)
        ang = np.arctan2(YS - c[1] + 0.5, XS - c[0] + 0.5)
        d = np.hypot(XS - c[0] + 0.5, YS - c[1] + 0.5)
        blade = (d < r - 6) & (d > 5) & (np.abs(np.sin(ang * blades / 2 + d * 0.05)) < 0.35)
        self.fill(blade, tones, 0.45)
        self.disc(c, 6, BRONZE, 0.6, 1.0, 3)

    def coil(self, box, tones=COPPER, period=7):
        x0, y0, x1, y1 = box
        m = (XS >= x0) & (XS < x1) & (YS >= y0) & (YS < y1)
        phase = (XS - x0 + (YS - y0) // 14 * 2) % period
        v = 0.25 + 0.6 * np.abs(np.sin(phase / period * np.pi)) + (fbm(self.X, self.Y, 10, 2, self.seed) - 0.5) * 0.12
        px = paint(tones, np.where(phase == 0, 0.0, v), 0.2)
        self.img[m] = px[m]

    def pipe(self, box, vertical=False, tones=STEEL, base=0.5, rings=0):
        """A round pipe: shaded across its width, optional flange rings along its length."""
        x0, y0, x1, y1 = box
        m = (XS >= x0) & (XS < x1) & (YS >= y0) & (YS < y1)
        t = ((XS - x0) / max(x1 - x0 - 1, 1)) if vertical else ((YS - y0) / max(y1 - y0 - 1, 1))
        v = base + 0.42 * np.cos((t - 0.32) * np.pi) * 0.9 - 0.15 + (fbm(self.X, self.Y, 8, 2, self.seed + 3) - 0.5) * 0.1
        px = paint(tones, np.where((t < 0.02) | (t > 0.98), 0.0, v), 0.25)
        self.img[m] = px[m]
        for i in range(rings):
            pos = (i + 0.5) / rings
            a = int((x0 if vertical else y0) * 0 + ((y0 + pos * (y1 - y0)) if vertical else (x0 + pos * (x1 - x0))))
            ring = (np.abs((YS if vertical else XS) - a) < 3) & m
            self.img[ring] = BRONZE[3]
            edge = (np.abs((YS if vertical else XS) - a) == 3) & m
            self.img[edge] = BRONZE[0]

    def gauge(self, c, r, needle=0.6, glow=0.5):
        self.disc(c, r, BRONZE, 0.5, 1.0, 3)
        self.disc(c, r - 4, STEEL, 0.7, -1.0, 3)
        for k in range(9):
            a = np.pi * (1.1 + 0.8 * k / 8)
            x, y = int(c[0] + np.cos(a) * (r - 7)), int(c[1] + np.sin(a) * (r - 7))
            self.img[y - 1:y + 1, x - 1:x + 1] = DARK[1]
        a = np.pi * (1.1 + 0.8 * needle)
        for s in range(r - 9):
            x, y = int(c[0] + np.cos(a) * s), int(c[1] + np.sin(a) * s)
            self.img[y, x] = EMBER[4]
            self.glow[y, x] = glow
        self.disc(c, 3, BRONZE, 0.6, 1.0, 2)

    def screen(self, box, tones=AETHER, glow=0.8, style="bars"):
        x0, y0, x1, y1 = box
        self.rect(box, DARK, 0.3, -1.0, 3, None, 0.1)
        m = (XS > x0 + 3) & (XS < x1 - 4) & (YS > y0 + 3) & (YS < y1 - 4)
        if style == "bars":
            h = vnoise((XS // 8) * 0.9, np.zeros_like(XS), 16, self.seed + 4)
            lit = m & ((y1 - 5 - YS) < h * (y1 - y0 - 10))
            lit &= (XS % 8) < 6
        elif style == "wave":
            lit = m & (np.abs(YS - ((y0 + y1) / 2 + np.sin(XS * 0.22) * (y1 - y0) * 0.25)) < 2.5)
        else:
            lit = m & (((XS - x0) % 10 < 2) | ((YS - y0) % 10 < 2))
        self.fill(lit, tones, 0.65, glow)

    def chevrons(self, box, tones=EMBER, count=2, up=True, glow=0.4, thick=9):
        x0, y0, x1, y1 = box
        h = (y1 - y0) / count
        cx = (x0 + x1) / 2
        for i in range(count):
            top = y0 + i * h
            dy = (YS - top) if up else (top + h - 1 - YS)
            band = np.abs(dy - np.abs(XS - cx) * 0.7 - h * 0.15)
            m = (band < thick / 2) & (np.abs(XS - cx) < (x1 - x0) / 2) & (YS >= y0) & (YS < y1)
            self.fill(m, tones, 0.55, glow, 0.1)
            self.img[m & (band > thick / 2 - 2)] = tones[1]

    def bar(self, p0, p1, w=4, tones=STEEL, base=0.55, glow=None):
        """A straight bar from `p0` to `p1` (a strut, an arrow shaft), shaded across its width."""
        (ax, ay), (bx, by) = p0, p1
        dx, dy = bx - ax, by - ay
        L = max(np.hypot(dx, dy), 1e-6)
        u = ((XS - ax) * dx + (YS - ay) * dy) / L
        v = (-(XS - ax) * dy + (YS - ay) * dx) / L
        m = (u >= 0) & (u <= L) & (np.abs(v) <= w / 2)
        shade = base + 0.35 * np.clip(-v / (w / 2), -1, 1) * 0.8
        px = paint(tones, np.where(np.abs(v) > w / 2 - 1, 0.0, shade), 0.2)
        self.img[m] = px[m]
        if glow is not None:
            self.glow[m] = np.maximum(self.glow[m], glow)
        return m

    def pips(self, tier, y, x_mid=64, size=6, gap=4):
        """One bright square per Mk on a tier band, so tier is never only a colour."""
        n = tier + 1
        x = x_mid - (n * size + (n - 1) * gap) // 2
        for _ in range(n):
            self.img[y:y + size, x:x + size] = 245
            self.glow[y:y + size, x:x + size] = 0.6
            x += size + gap

    def bolts(self, pts=None, r=3, tones=BRONZE):
        for cx, cy in pts or ((14, 14), (113, 14), (14, 113), (113, 113)):
            rivet(self.img, cx, cy, r, tones)

    def lines(self, box, count, tones=STEEL, base=0.7):
        """Short dashes of small print: a panel label."""
        x0, y0, x1, y1 = box
        rng = np.random.default_rng(self.seed + 40)
        for i in range(count):
            y = y0 + int((i + 0.5) * (y1 - y0) / count)
            x = x0 + 2
            while x < x1 - 8:
                w = int(rng.integers(4, 14))
                self.img[y:y + 2, x:min(x + w, x1 - 2)] = tones[int(base * (len(tones) - 1))]
                x += w + 4

    def rgba(self, cutout=None):
        return rgba(self.img, alpha=cutout, glow=np.clip(self.glow, 0, 1))

