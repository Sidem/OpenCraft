"""Bakes the real night sky for the game (web/src/render/sky/): stars.bin, milkyway.jpg.

Run once when the inputs change (the outputs are committed; see web/src/render/sky/CREDITS.md for sources and licences):

    python scripts/bake-sky/bake-sky.py <hyg_v44.csv.gz> <milkyway_2020_4k.exr>

Needs python 3 with numpy and Pillow, and ffmpeg on the PATH (decodes the EXR). Stops with a message when a
constellation line names a star that is not in the catalogue or looks wrong, and checks the Milky Way image
against two known places (the galactic centre and the Large Magnetic Cloud) so the orientation cannot drift.

stars.bin (little endian): u16 star count N, u16 line count P, then N stars of 6 bytes (u16 right ascension over the
full circle, i16 declination over -90..90 degrees, u8 magnitude as (mag + 2) * 25, u8 colour index B-V as
(ci + 0.5) * 100), brightest first, then P lines of two u16 star indices. To add a constellation: a row in
constellations.txt.
"""

import csv
import gzip
import io
import math
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).parent
OUT = HERE.parent.parent / "web" / "src" / "render" / "sky"
MAG_LIMIT = 6.5
MW_SIZE = (2048, 1024)


def load_stars(path):
    stars = []
    with gzip.open(path, "rt", encoding="utf8") as f:
        for row in csv.DictReader(f):
            if row["id"] == "0":  # the Sun
                continue
            mag = float(row["mag"])
            if mag > MAG_LIMIT:
                continue
            ci = float(row["ci"]) if row["ci"] else 0.6
            stars.append(
                dict(ra=float(row["ra"]) * 15.0, dec=float(row["dec"]), mag=mag, ci=ci,
                     bayer=row["bayer"], con=row["con"], name=row["proper"])
            )
    stars.sort(key=lambda s: s["mag"])
    return stars


def vec(s):
    ra, dec = math.radians(s["ra"]), math.radians(s["dec"])
    return (math.cos(dec) * math.cos(ra), math.cos(dec) * math.sin(ra), math.sin(dec))


def separation(a, b):
    d = sum(x * y for x, y in zip(vec(a), vec(b)))
    return math.degrees(math.acos(max(-1.0, min(1.0, d))))


def read_lines(stars):
    index = {}
    for i, s in enumerate(stars):
        # Sorted by magnitude: the brightest of a pair wins. "Bet-1" also answers to "Bet".
        for bayer in {s["bayer"], s["bayer"].split("-")[0]} - {""}:
            index.setdefault((bayer, s["con"]), i)
    lines = []
    for raw in (HERE / "constellations.txt").read_text(encoding="utf8").splitlines():
        raw = raw.split("#")[0].strip()
        if not raw:
            continue
        name, segments = [p.strip() for p in raw.split(":")]
        for seg in segments.split(","):
            ends = []
            for token in seg.split("-"):
                bayer, con = token.split()
                if (bayer, con) not in index:
                    sys.exit(f"{name}: no star {bayer} {con} (brighter than magnitude {MAG_LIMIT}) in the catalogue")
                ends.append(index[(bayer, con)])
            a, b = ends
            span = separation(stars[a], stars[b])
            if not 0.8 <= span <= 40:
                sys.exit(f"{name}: {seg.strip()} spans {span:.1f} degrees, which cannot be a line of a figure")
            lines.append((a, b))
    return lines


def write_stars(stars, lines):
    out = bytearray(struct.pack("<HH", len(stars), len(lines)))
    for s in stars:
        ra = round(s["ra"] / 360.0 * 65536) % 65536
        dec = round(s["dec"] / 90.0 * 32767)
        mag = round((s["mag"] + 2.0) * 25)
        ci = round((min(2.05, max(-0.5, s["ci"])) + 0.5) * 100)
        out += struct.pack("<HhBB", ra, dec, mag, ci)
    for a, b in lines:
        out += struct.pack("<HH", a, b)
    (OUT / "stars.bin").write_bytes(out)
    print(f"stars.bin: {len(stars)} stars (magnitude <= {MAG_LIMIT}), {len(lines)} line segments, {len(out)} bytes")


def bake_milky_way(exr):
    with tempfile.TemporaryDirectory() as tmp:
        png = Path(tmp) / "mw.png"
        subprocess.run(
            ["ffmpeg", "-v", "error", "-y", "-i", str(exr), "-vf", f"scale={MW_SIZE[0]}:{MW_SIZE[1]}:flags=area",
             "-pix_fmt", "rgb48le", str(png)],
            check=True,
        )
        a = np.array(Image.open(png)).astype(np.float32) / 65535.0
    hi = np.percentile(a.mean(axis=2), 99.8)
    x = np.clip(a / hi, 0, 1) ** 0.55
    img = Image.fromarray((x * 255).astype(np.uint8))
    img.save(OUT / "milkyway.jpg", quality=72, optimize=True, progressive=True)
    print(f"milkyway.jpg: {MW_SIZE[0]}x{MW_SIZE[1]}, {(OUT / 'milkyway.jpg').stat().st_size} bytes")
    return x.mean(axis=2)


def check_milky_way(lum):
    """The image is equirectangular with right ascension 0 in the middle and growing to the left, declination +90 on top."""
    h, w = lum.shape

    def at(ra, dec):
        u = (0.5 - ra / 360.0) % 1.0
        v = 0.5 - dec / 180.0
        r = 6
        x, y = int(u * w), int(v * h)
        return lum[max(0, y - r):y + r, max(0, x - r):x + r].mean()

    centre = at(266.4, -28.9)  # the galactic centre, in Sagittarius
    anti = at(86.4, 28.9)  # the opposite direction in the sky
    lmc = at(80.9, -69.8)
    quiet = at(150.0, 60.0)  # high above the galactic plane
    print(f"galactic centre {centre:.2f}, anticentre {anti:.2f}, LMC {lmc:.2f}, quiet sky {quiet:.2f}")
    if not (centre > 0.5 and centre > 1.5 * anti and centre > 5 * quiet and lmc > 5 * quiet):
        sys.exit("the Milky Way image is not where the galactic centre and the Large Magellanic Cloud should be")


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    OUT.mkdir(parents=True, exist_ok=True)
    stars = load_stars(sys.argv[1])
    write_stars(stars, read_lines(stars))
    check_milky_way(bake_milky_way(sys.argv[2]))


if __name__ == "__main__":
    main()
