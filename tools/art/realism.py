"""Realism of the natural tiles, judged against CC0 photo references (ambientCG): python tools/art/realism.py <command>

  fetch      downloads each reference's 1K colour map to artifacts/art2/refs/ (not committed; the ids are below)
  reference  reduces the photos to block scale (128 px, area average) and writes tools/art/reference/materials.json:
             per material an Oklab cloud of the holdout photos, the fit photos' lightness quantiles and colour profile
             (mean and spread of a, b by lightness; `real.py` paints with it) and the photos' own metrics
  score      prints each judged tile's distance to its material, next to the photo-to-photo distance (the floor)

Fit photos give the palette, holdout photos judge it, so a tile cannot match its judge by copying it. Metrics:
`dist` sliced Wasserstein in Oklab (style.py), `edges` share of texels on a hard lightness step (check.edge_density),
`Lsd` lightness spread. Add a material: a row in MATERIALS, then `fetch` and `reference`.
"""
import io
import json
import sys
import urllib.request
import zipfile
from pathlib import Path

import numpy as np
from PIL import Image

from style import sliced_wasserstein, srgb_to_oklab

ROOT = Path(__file__).resolve().parents[2]
PHOTOS = ROOT / "artifacts" / "art2" / "refs"
OUT = Path(__file__).parent / "reference" / "materials.json"
BINS = 16
# The scans' physical size in cm where ambientCG states it (others are taken as 150); a block is 100 cm.
SIZE_CM = {"Grass001": 140, "Grass002": 140, "Grass004": 140, "Ground048": 140, "Ground054": 350, "Bark014": 120}

# material: (tiles it judges, fit photos, holdout photos); an empty holdout judges against the fit photos
MATERIALS = {
    "stone": (["stone"], ["Rock058", "Rock060"], ["Rock030"]),
    "dirt": (["dirt"], ["Ground048", "Ground082S"], ["Ground103"]),
    "grass": (["grass_top"], ["Grass001", "Grass002"], ["Grass004"]),
    "sand": (["sand"], ["Ground054", "Ground080"], ["Ground079S"]),
    "bark": (["log_side"], ["Bark012", "Bark014"], ["Bark001"]),
    "planks": (["planks"], ["Planks023A", "Planks037A"], ["Planks012"]),
    "granite": (["granite"], ["Granite001A"], []),
    "basalt": (["basalt"], ["Rock035"], []),
    "limestone": (["limestone"], ["Rock023"], []),
    "sandstone": (["sandstone"], ["Rock029"], ["Rock009"]),
}


def edge_share(lab):
    gx = np.abs(np.diff(lab[..., 0], axis=1))[:-1]
    gy = np.abs(np.diff(lab[..., 0], axis=0))[:, :-1]
    return float(((gx + gy) > 0.12).mean())


def metrics(lab):
    flat = lab.reshape(-1, 3)
    return {"edges": edge_share(lab), "Lsd": float(flat[:, 0].std()), "L": float(flat[:, 0].mean()),
            "C": float(np.hypot(flat[:, 1], flat[:, 2]).mean())}


def photo(pid):
    """The material as a tile would show it: albedo lit by its own normal and AO maps from the top left (the light
    `lib.relief_light` bakes in; flat ground gives 1.0), cropped to 1 m and area-averaged to 128 px. Oklab, (128, 128, 3)."""
    load = lambda k: np.asarray(Image.open(PHOTOS / f"{pid}_{k}.jpg").convert("RGB")).astype(float) / 255.0
    albedo = load("color")
    lin = np.where(albedo <= 0.04045, albedo / 12.92, ((albedo + 0.055) / 1.055) ** 2.4)
    n = load("normal") * 2 - 1
    n[..., 1] = -n[..., 1]  # GL normals: +green points up the image, rows run down
    light = np.array([-0.5, -0.5, 0.71])
    shade = 0.35 + 0.65 * np.clip(n @ light, 0, None) / light[2] / np.linalg.norm(n, axis=-1).clip(1e-6)
    if (PHOTOS / f"{pid}_ao.jpg").exists():
        shade = shade * load("ao")[..., 0]
    lin = lin * shade[..., None]
    crop = int(round(lin.shape[0] * min(1.0, 100 / SIZE_CM.get(pid, 150))))  # one block is 1 m
    lin = lin[:crop, :crop]
    small = np.stack([np.asarray(Image.fromarray(lin[..., c].astype(np.float32)).resize((128, 128), Image.BOX))
                      for c in range(3)], -1)
    srgb = np.where(small <= 0.0031308, small * 12.92, 1.055 * small.clip(0) ** (1 / 2.4) - 0.055)
    return srgb_to_oklab((srgb * 255).reshape(-1, 3)).reshape(128, 128, 3)


def profile(lab):
    """Lightness quantiles (0..1 in 33 steps) and, per lightness bin, mean a, b and their spread."""
    flat = lab.reshape(-1, 3)
    L = flat[:, 0]
    q = np.quantile(L, np.linspace(0, 1, 33))
    edges_ = np.quantile(L, np.linspace(0, 1, BINS + 1))
    idx = np.clip(np.searchsorted(edges_, L, side="right") - 1, 0, BINS - 1)
    rows = []
    for b in range(BINS):
        p = flat[idx == b]
        rows.append([float(p[:, 0].mean()), float(p[:, 1].mean()), float(p[:, 2].mean()),
                     float(p[:, 1].std()), float(p[:, 2].std())])
    return {"quantiles": q.round(4).tolist(), "bins": np.round(rows, 4).tolist()}


def fetch():
    PHOTOS.mkdir(parents=True, exist_ok=True)
    ids = sorted({p for _, fit, hold in MATERIALS.values() for p in fit + hold})
    for pid in ids:
        if (PHOTOS / f"{pid}_color.jpg").exists():
            continue
        req = urllib.request.Request(f"https://ambientcg.com/get?file={pid}_1K-JPG.zip", headers={"User-Agent": "opencraft-art"})
        z = zipfile.ZipFile(io.BytesIO(urllib.request.urlopen(req, timeout=120).read()))
        for key, suffix in (("color", "_color.jpg"), ("normal", "_normalgl.jpg"), ("ao", "_ambientocclusion.jpg")):
            name = next((n for n in z.namelist() if n.lower().endswith(suffix)), None)
            if name:
                data = z.read(name)
                Image.open(io.BytesIO(data)).verify()
                (PHOTOS / f"{pid}_{key}.jpg").write_bytes(data)
        print(pid)


def reference():
    out = {}
    rng = np.random.default_rng(5)
    for mat, (_, fit, hold) in MATERIALS.items():
        fit_lab = np.concatenate([photo(p) for p in fit], axis=1)
        hold_lab = [photo(p) for p in (hold or fit)]
        cloud = np.concatenate([h.reshape(-1, 3) for h in hold_lab])
        cloud = cloud[rng.choice(len(cloud), min(1500, len(cloud)), replace=False)]
        floor = sliced_wasserstein(fit_lab.reshape(-1, 3)[rng.permutation(fit_lab.size // 3)], cloud) if hold else None
        out[mat] = {"fit": fit, "holdout": hold, "profile": profile(fit_lab), "cloud": cloud.round(3).tolist(),
                    "photo": {p: metrics(photo(p)) for p in fit + hold}, "floor": floor}
        print(f"{mat:10} floor {floor if floor is None else round(floor, 4)}  " +
              "  ".join(f"{p}: edges {m['edges']:.1%} Lsd {m['Lsd']:.3f}" for p, m in out[mat]["photo"].items()))
    OUT.write_text(json.dumps(out, separators=(",", ":")))


def load():
    return json.loads(OUT.read_text())


def score(tile_rgb, mat, ref=None):
    """Distance of a tile (rgb uint8) to a material's holdout cloud, plus its metrics."""
    ref = ref or load()
    lab = srgb_to_oklab(tile_rgb.reshape(-1, 3).astype(float))
    cloud = np.array(ref[mat]["cloud"])
    rng = np.random.default_rng(6)
    d = sliced_wasserstein(lab[rng.permutation(len(lab))], cloud[rng.permutation(len(cloud))])
    return d, metrics(lab.reshape(tile_rgb.shape))


def main():
    cmd = sys.argv[1]
    if cmd == "fetch":
        return fetch()
    if cmd == "reference":
        return reference()
    import registry
    ref = load()
    by_name = {e.name: e for e in registry.LAYERS}
    print(f"{'tile':12} {'material':10} {'dist':>6} {'floor':>6} {'edges':>6} {'photo':>6} {'Lsd':>5} {'photo':>5} {'cols':>5}")
    for mat, (tiles, _, _) in MATERIALS.items():
        pm = list(ref[mat]["photo"].values())
        pe, pl = np.mean([m["edges"] for m in pm]), np.mean([m["Lsd"] for m in pm])
        for name in tiles:
            rgb = by_name[name].make()[..., :3]
            d, m = score(rgb, mat, ref)
            cols = len(np.unique(rgb.reshape(-1, 3), axis=0))
            fl = ref[mat]["floor"]
            print(f"{name:12} {mat:10} {d:6.4f} {'-' if fl is None else round(fl, 4):>6} {m['edges']:6.1%} {pe:6.1%} "
                  f"{m['Lsd']:5.3f} {pl:5.3f} {cols:5}")


if __name__ == "__main__":
    main()
