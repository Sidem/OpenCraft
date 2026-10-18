"""Surface hints: the tinted ground over each ore (docs/ART_STYLE.md section 12). A stain blends part of a realistic ground
tile (dirt, grass, sand from `real.py`) toward the mineral's colour in Oklab, keeping the tile's own light and shade, and
each ore has its own SHAPE so the four read apart without colour (the same language as the ores): iron is big ragged
oxide patches, coal clustered dark shards, copper small broken green crusts, chalk fine pale chips. `apply` is called by
`hints.py`. Add a stain: a ramp, a mask function and a row in `KINDS`.

`RUST`, `COAL` and `VERDIGRIS` are the older stylized ramps other recipes still use (iron ore, coal ore, plants).
"""
import numpy as np

from lib import N, blur, coords, fbm, hash2, ramp, smooth, worley
from style import oklab_to_srgb, srgb_to_oklab

RUST = ramp(["#241012", "#4a1f1a", "#6e2c20", "#8f3d28", "#b05a38", "#cc7e52"])
COAL = ramp(["#05060b", "#0c0f19", "#171b2a", "#272d42", "#3f4a68", "#6f86ad"])
VERDIGRIS = ramp(["#1a2a2a", "#38504a", "#5d7f6c", "#8aa890", "#b4c8ae"])
# The realistic stains: limonite and hematite ochre, coal dust, copper's green salts, chalk.
OXIDE = ramp(["#4a2a10", "#7a4818", "#a8682a", "#c88c3c", "#e0b060"])
SOOT = ramp(["#0a0a0b", "#151516", "#212122", "#303032", "#46464a"])
COPPER_SALTS = ramp(["#1e3a2a", "#33604a", "#4f8466", "#7aa68a", "#a8c8b2"])
CHALK = ramp(["#8a867c", "#aeaa9e", "#cdc9bd", "#e4e1d6", "#f4f2ea"])


def _rust():
    """Iron staining: big ragged patches of oxide fading out at their edge, never round or leaf-shaped. 0..1."""
    X, Y = coords()
    n = fbm(X, Y, 3, 4, 103, 0.55, 0.35)
    return np.clip((n - 0.57) / 0.08, 0, 1)


def _coal():
    """Coal fragments: small angular chips (Voronoi cells, so straight broken sides) in loose clusters."""
    X, Y = coords()
    f1, f2, pid = worley(X, Y, 18, 105, 1.0)
    cluster = fbm(X, Y, 4, 3, 106)
    return (pid > 0.78 - 0.5 * np.clip(cluster - 0.5, 0, 1)) & (f2 - f1 > 0.12)


def _verdigris():
    """Copper salts weathering out of the soil: a pale sage crust in small broken patches, speckled where it is thin."""
    X, Y = coords()
    ix, iy = (X * N).astype(int), (Y * N).astype(int)
    n = fbm(X, Y, 7, 4, 107, 0.5, 0.3)
    grain = hash2(ix, iy, 108)
    return np.clip((n - 0.57) / 0.05, 0, 1) * (grain > 0.15)


def _chalk():
    """Chalk and limestone chips: small irregular pale fragments, scattered in loose drifts."""
    X, Y = coords()
    f1, _, pid = worley(X, Y, 28, 110, 1.0)
    r = 0.25 + 0.2 * fbm(X, Y, 20, 2, 111)
    return (f1 < r) & (pid > 0.55) & (fbm(X, Y, 5, 2, 112) > 0.4)


KINDS = {"rusty": (_rust, OXIDE), "dark": (_coal, SOOT), "green": (_verdigris, COPPER_SALTS), "pale": (_chalk, CHALK)}


def apply(img, kind, grass=False):
    """`img` (n, n, 3) with the stain `kind` blended in where its shape falls (soft edges, continuous tone). The stain
    always differs in lightness from the ground, not just in hue (so colour-blind players see it): coal is darker, the
    others lighter on dark ground and darker on light ground (sand). On `grass` rust is darker too: rusty bare soil
    showing through the turf, not orange paint on it."""
    mask_fn, tones = KINDS[kind]
    m = np.clip(blur(mask_fn().astype(float), 0.6), 0, 1)[..., None]
    lab = srgb_to_oklab(img.reshape(-1, 3).astype(float)).reshape(img.shape)
    L = lab[..., 0]
    shade = (L - L.min()) / (np.ptp(L) + 1e-9)  # the ground's own light and shade, kept inside the stain
    tone = srgb_to_oklab(smooth(tones, shade).reshape(-1, 3)).reshape(img.shape)
    lift = {"rusty": 0.16, "green": 0.15, "pale": 0.25}  # green on brown or green ground leans on lightness
    if kind == "dark":
        tone[..., 0] = L * 0.5
    elif grass and kind == "rusty":
        tone[..., 0] = L - 0.16
        tone[..., 1:] *= 0.55  # darkened oxide turns blood red at full chroma; rusty soil is a muted brown
    elif L.mean() > 0.6:
        tone[..., 0] = L - (0.2 if kind == "pale" else 0.17)
    else:
        tone[..., 0] = L + lift[kind]
    out = lab * (1 - m) + tone * m
    return oklab_to_srgb(out.reshape(-1, 3)).reshape(img.shape)