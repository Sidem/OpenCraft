"""The art tool's outside judge: python tools/art/check.py   (exit code 1 when a rule fails)

Rules, each measured on the rendered tiles (docs/ART_STYLE.md section 6):
  seam    repeating tiles: the wrap edge is no harsher than the harshest line inside the tile (ratio <= 1.3)
  phase   texel phases look alike (>= 0.5), so no layer is an upscaled 64 or 32 grid
  colours at most 96 distinct colours in a tile (a painted palette, not noise)
  value   mean Oklab lightness 0.18..0.78: no tile near black or white (glowing tiles may go down to 0.12)
  cutout  alpha is only 0 or 255; leaves 55-85% covered, glass 10-45%
  ores    every ore's deposit covers 6-45% of its tile and differs from the stone body by more than 0.12 Oklab in normal
          vision and under protan, deutan and tritan simulation; ores differ from each other by shape (edge density) or colour
Also prints each tile's warm-light / cool-shadow hue shift for information.
"""
import sys

import numpy as np

import registry
from stains import KINDS
from style import srgb_to_oklab

CVD = {
    "protan": [[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]],
    "deutan": [[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.011820, 0.042940, 0.968881]],
    "tritan": [[1.255528, -0.076749, -0.178779], [-0.078411, 0.930809, 0.147602], [0.004733, 0.691367, 0.303900]],
}


def to_lab(rgb, cvd=None):
    c = rgb.astype(float) / 255.0
    lin = np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
    if cvd:
        lin = np.clip(lin @ np.array(CVD[cvd]).T, 0, 1)
    srgb = np.where(lin <= 0.0031308, lin * 12.92, 1.055 * lin ** (1 / 2.4) - 0.055)
    return srgb_to_oklab(srgb.reshape(-1, 3) * 255).reshape(rgb.shape)


def seam_ratio(lab):
    """The wrap edge's mean jump over the largest mean jump at any interior row or column boundary (<= 1 means the
    seam is no harsher than a line the tile already has, so planks and bricks may have lines but a drawn border fails)."""
    worst = 0.0
    for axis in (0, 1):
        a = np.moveaxis(lab, axis, 0)
        inner = np.abs(a[1:] - a[:-1]).sum(-1).mean(1)
        seam = np.abs(a[0] - a[-1]).sum(-1).mean()
        worst = max(worst, seam / max(inner.max(), 1e-9))
    return worst


def upscale_phase(lab):
    """Texture detail (small colour steps: dither, grain, shading) must turn up at all four texel phases alike. An image
    upscaled 2x or 4x has none between the texels of one block, so a phase is empty (0.0); a true 128 painting gives
    about 1.0. Strong edges are left out (they sit on one phase by chance); flat tiles with almost no detail pass."""
    worst = 1.0
    for axis in (0, 1):
        a = np.moveaxis(lab, axis, 0)
        jump = np.abs(np.roll(a, -1, 0) - a).sum(-1)
        small = ((jump > 1e-6) & (jump < 0.12)).mean(1)
        phase = [small[p::4].mean() for p in range(4)]
        if max(phase) > 0.02:
            worst = min(worst, min(phase) / max(phase))
    return worst

def edge_density(lab):
    gx = np.abs(np.diff(lab[..., 0], axis=1))[:-1]
    gy = np.abs(np.diff(lab[..., 0], axis=0))[:, :-1]
    return float(((gx + gy) > 0.12).mean())


def hue_shift(rgb):
    lab = to_lab(rgb).reshape(-1, 3)
    order = np.argsort(lab[:, 0])
    k = len(order) // 5
    hue = lambda p: np.degrees(np.arctan2(p[:, 2].mean(), p[:, 1].mean())) % 360
    return hue(lab[order[:k]]), hue(lab[order[-k:]])


def main():
    fails, tiles = [], {}
    print(f"{'layer':>5} {'name':16} {'seam':>5} {'phase':>6} {'cols':>5} {'L':>5}  shadow->light hue")
    for e in registry.LAYERS:
        tile = e.make()
        tiles[e.name] = (e, tile)
        rgb, alpha, glow = tile[..., :3], tile[..., 3], tile[..., 4]
        lab = to_lab(rgb)
        solid = alpha > 127
        seam = seam_ratio(lab) if e.tiles else float("nan")
        det = upscale_phase(lab)
        cols = len(np.unique(rgb.reshape(-1, 3), axis=0))
        mean_l = float(lab[..., 0][solid].mean())
        lo = 0.12 if glow.max() > 8 else 0.18
        hs, hl = hue_shift(rgb[solid].reshape(-1, 1, 3))
        print(f"{e.layer:5} {e.name:16} {seam:5.2f} {det:6.2f} {cols:5} {mean_l:5.2f}  {hs:3.0f} -> {hl:3.0f}")
        if e.tiles and seam > 1.3:
            fails.append(f"{e.name}: wrap seam {seam:.2f}")
        if det < 0.5:
            fails.append(f"{e.name}: texel phases are {det:.2f} alike: an upscaled grid?")
        if cols > 96:
            fails.append(f"{e.name}: {cols} colours")
        if not lo <= mean_l <= 0.78:
            fails.append(f"{e.name}: mean lightness {mean_l:.2f}")
        if e.cutout:
            if not np.isin(alpha, (0, 255)).all():
                fails.append(f"{e.name}: alpha is not 0/255")
            cover = float(solid.mean())
            lo_c, hi_c = e.cover or ((0.10, 0.45) if "glass" in e.name else (0.55, 0.85))
            if not lo_c <= cover <= hi_c:
                fails.append(f"{e.name}: {cover:.0%} covered")
    groups = [
        ("stone", ["coal_ore", "iron_ore", "copper_ore", "quartz_ore"]),
        ("dirt", [f"soil_{k}" for k in KINDS]),
        ("grass_top", [f"grass_top_{k}" for k in KINDS]),
        ("sand", [f"sand_{k}" for k in KINDS]),
    ]
    for base, members in groups:
        print(f"\nvariants of {base} (share of texels that stand out, worst case over colour-vision types):")
        ref = tiles[base][1][..., :3]
        shapes = {}
        for name in members:
            rgb = tiles[name][1][..., :3]
            worst = min(float((np.linalg.norm(to_lab(rgb, c) - to_lab(ref, c), axis=-1) > 0.12).mean())
                        for c in (None, "protan", "deutan", "tritan"))
            shapes[name] = edge_density(to_lab(rgb))
            print(f"  {name:16} deposit {worst:5.1%}   edge density {shapes[name]:.3f}")
            if not 0.06 <= worst <= 0.45:
                fails.append(f"{name}: deposit covers {worst:.1%} of the tile (worst case)")
        for i, a in enumerate(members):
            for b in members[i + 1:]:
                la, lb = (to_lab(tiles[n][1][..., :3]).reshape(-1, 3) for n in (a, b))
                colour = float(np.linalg.norm(la.mean(0) - lb.mean(0)))
                shape = abs(shapes[a] - shapes[b])
                ok = colour > 0.012 or shape > 0.02
                if not ok:
                    fails.append(f"{a} and {b} are too alike (colour {colour:.3f}, edges {shape:.3f})")
    print("\n" + ("FAIL\n  " + "\n  ".join(fails) if fails else "all rules pass"))
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
