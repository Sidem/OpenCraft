"""Style measurement for the art pipeline: colour statistics in Oklab and a distance between two images.

Used two ways:
  python tools/art/style.py reference <image>...   -> writes tools/art/reference/style.json from the sample images
  python tools/art/style.py compare <image>        -> prints the style distance of a screenshot to the reference
Everything works on a subsample of pixels so it stays fast. The distance is a sliced Wasserstein distance between the
two pixel clouds in Oklab (lower is closer); `compare` also prints the headline statistics next to the reference's.
"""
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).parent
REF = HERE / "reference" / "style.json"
SAMPLE = 30000
PROJECTIONS = 96
CLOUD = 5000  # points of the reference cloud kept in style.json (a random subsample, so still representative)

# The sample sheets are 1920x1080 with four panels each on a blue-grey mat; these are the panels' pixel boxes.
PANELS = [(17, 17, 938, 509), (982, 17, 1903, 509), (17, 575, 938, 1062), (982, 575, 1903, 1062)]


def srgb_to_oklab(rgb):
    c = rgb / 255.0
    lin = np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
    m1 = np.array([[0.4122214708, 0.5363325363, 0.0514459929],
                   [0.2119034982, 0.6806995451, 0.1073969566],
                   [0.0883024619, 0.2817188376, 0.6299787005]])
    lms = np.cbrt(lin @ m1.T)
    m2 = np.array([[0.2104542553, 0.7936177850, -0.0040720468],
                   [1.9779984951, -2.4285922050, 0.4505937099],
                   [0.0259040371, 0.7827717662, -0.8086757660]])
    return lms @ m2.T


def pixels(path, panels):
    img = np.asarray(Image.open(path).convert("RGB"))
    boxes = panels and PANELS or [(0, 0, img.shape[1], img.shape[0])]
    return np.concatenate([img[y0:y1, x0:x1].reshape(-1, 3) for x0, y0, x1, y1 in boxes])


def subsample(px, n=SAMPLE, seed=1):
    rng = np.random.default_rng(seed)
    return px[rng.choice(len(px), min(n, len(px)), replace=False)]


def stats(lab):
    L, a, b = lab[:, 0], lab[:, 1], lab[:, 2]
    C = np.hypot(a, b)
    h = np.degrees(np.arctan2(b, a)) % 360

    def mean_hue(mask):
        if mask.sum() < 50:
            return None
        return float(np.degrees(np.arctan2(b[mask].mean(), a[mask].mean())) % 360)

    colourful = C > 0.06
    return {
        "L_mean": float(L.mean()), "L_std": float(L.std()),
        "L_pct": [float(v) for v in np.percentile(L, [5, 25, 50, 75, 95])],
        "C_mean": float(C.mean()), "C_pct95": float(np.percentile(C, 95)),
        "dark_share": float((L < 0.3).mean()), "bright_share": float((L > 0.75).mean()),
        "hue_shadow": mean_hue((L < 0.3) & colourful), "hue_mid": mean_hue((L >= 0.3) & (L < 0.65) & colourful),
        "hue_light": mean_hue((L > 0.65) & colourful),
        "hue_hist24": [float(v) for v in np.histogram(h[colourful], bins=24, range=(0, 360),
                                                      weights=C[colourful])[0] / max(1e-9, C[colourful].sum())],
    }


def palette(lab, k=16):
    rng = np.random.default_rng(3)
    centres = lab[rng.choice(len(lab), k, replace=False)]
    for _ in range(25):
        idx = ((lab[:, None, :] - centres[None]) ** 2).sum(-1).argmin(1)
        for j in range(k):
            if (idx == j).any():
                centres[j] = lab[idx == j].mean(0)
    share = np.bincount(idx, minlength=k) / len(lab)
    order = np.argsort(-share)
    return centres[order], share[order]


def oklab_to_srgb(lab):
    m2i = np.linalg.inv(np.array([[0.2104542553, 0.7936177850, -0.0040720468],
                                  [1.9779984951, -2.4285922050, 0.4505937099],
                                  [0.0259040371, 0.7827717662, -0.8086757660]]))
    m1i = np.linalg.inv(np.array([[0.4122214708, 0.5363325363, 0.0514459929],
                                  [0.2119034982, 0.6806995451, 0.1073969566],
                                  [0.0883024619, 0.2817188376, 0.6299787005]]))
    lin = (lab @ m2i.T) ** 3 @ m1i.T
    lin = np.clip(lin, 0, 1)
    c = np.where(lin <= 0.0031308, lin * 12.92, 1.055 * lin ** (1 / 2.4) - 0.055)
    return np.clip(np.round(c * 255), 0, 255).astype(np.uint8)


def sliced_wasserstein(a, b, seed=7):
    rng = np.random.default_rng(seed)
    n = min(len(a), len(b))
    a, b = a[:n], b[:n]
    d = rng.normal(size=(PROJECTIONS, 3))
    d /= np.linalg.norm(d, axis=1, keepdims=True)
    pa, pb = np.sort(a @ d.T, axis=0), np.sort(b @ d.T, axis=0)
    return float(np.abs(pa - pb).mean())


def load_ref():
    return json.loads(REF.read_text())


def main():
    cmd, files = sys.argv[1], sys.argv[2:]
    if cmd == "reference":
        px = subsample(np.concatenate([pixels(f, True) for f in files]), SAMPLE * 3)
        lab = srgb_to_oklab(px.astype(float))
        cent, share = palette(lab, 24)
        out = {"stats": stats(lab), "palette_srgb": oklab_to_srgb(cent).tolist(), "palette_share": share.tolist(),
               "cloud": lab[:CLOUD].round(3).tolist()}
        REF.parent.mkdir(exist_ok=True)
        REF.write_text(json.dumps(out, separators=(",", ":")))
        print(json.dumps(out["stats"], indent=1))
    else:
        ref = load_ref()
        lab = srgb_to_oklab(subsample(pixels(files[0], False)).astype(float))
        s = stats(lab)
        print("style distance (sliced Wasserstein, lower is closer):", round(sliced_wasserstein(lab, np.array(ref["cloud"])), 4))
        for k in ("L_mean", "L_std", "C_mean", "C_pct95", "dark_share", "bright_share", "hue_shadow", "hue_light"):
            print(f"  {k:12} this {s[k] if s[k] is None else round(s[k], 3)}   reference {round(ref['stats'][k], 3)}")


if __name__ == "__main__":
    main()
