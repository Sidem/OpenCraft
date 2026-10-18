# Art style: Ember & Aether (pilot signed off 2026-10-18; natural batch built)

Direction chosen 2026-10-18 from the user's brief ("move away from Minecraft low-detail pixel art to at least 128 × 128,
a sci-fi fantasy fusion, like the attached sample backgrounds"). **It supersedes "B · Alpine" and the "calmer textures"
note in `ART_HANDOVER.md` 2b**, and lifts two of that file's rules for the art pack only: textures stay 16 × 16 pixel art,
and "procedural stays procedural" (the pack ships as PNG, like the music ships as audio). Everything else in the handover
(what you may touch, budgets, tests) still holds. Status: 85 of 347 layers are painted (the pilot's 33, then the natural batch); industry, items and the remaining blocks are not.

## 1. What the samples actually show (measured, `tools/art/style.py`)

Five sample sheets, 20 panels. The pixel statistics decided the palette more than the first impression did:

- **Low key:** mean Oklab lightness 0.34; 53% of pixels darker than 0.3, 6% brighter than 0.75. The game by day was 0.52 with
  3% dark pixels. Light comes from a few warm sources (windows, lanterns, a sun) against deep shadow.
- **Shadows are navy ink, not black or purple:** darkest tones `#0d1421 #1e2a3b #10212a`. Mids are plum and terracotta
  (`#4b3b50 #72536c #5d2e24 #783f30 #9b553e`), highlights salmon and cream (`#ec996f #efe09d`). The hue path is
  navy → plum → red-brown → orange → cream, warm light, cool dark. (My first guess, shadows toward violet, was wrong:
  dark coloured pixels average hue 10°, red-brown.)
- **Teal is the complement and the accent:** `#2d5664 #48797c #6f9f90 #86d4d5` (water, mountains, glowing doorways,
  moss). Green is deep (`#324828`) with lime highlights; red-coral foliage appears as a feature, never the base.
- **Glow carries the mood:** lit windows, fire, crystals, cyan light. Texture resolution is secondary to this.

## 2. The look

1. **Warm light, cool dark.** Every material ramp runs from navy-tinted shadow to a warm highlight; hue shifts along it.
2. **Low-key values.** Albedo mostly in L 0.25–0.65; the game's daylight brightens it. No pure black or white tile.
3. **Painted clusters, selective detail.** Big shapes first (plates, boards, ridges), then medium (cracks, nodules, veins),
   then grain. Hard, aliased edges; ordered dither (Bayer 4 × 4) only inside gradients; about 6–30 colours a tile.
4. **Old stone, living metal.** Cut and fractured rock, moss in the joints; bronze and dark iron fittings with rivets and
   verdigris; energy is ember orange (fuel, heat) or aether cyan (power, electrics), violet kept for the far tech.
5. **Light only where something makes it.** Ember mouths, aether slits and turbine wells, lamp crystals: the glow array.
   Nothing glows for decoration.
6. **Shapes before textures** (the 2026-10-16 review still holds): textures cannot give a cube a roof or an arch. The
   biggest remaining lever for "buildings" is non-cube geometry, listed under follow-ups.

Material families (recipes in `tools/art`): rock = blue-grey plates with wobbly outlines and warm highlights; earth = umber
clods with grey pebbles; grass = deep green with lime tips; sand = ochre ripples; wood = oak (rings, boards, nails) and
bark (broad plates, moss); ores = one shape language each so they differ without colour (coal dark angular shards, iron fat
rust nodules, copper thin veins with a teal crust); masonry = uneven courses, per-brick tone, moss in the joints; metal =
navy steel, bronze (the wood ramp pushed to gold), firebrick, copper coil.

Ramps (dark → light, interpolated in Oklab to 7–9 tones; defined in `natural.py` and `machines.py`):

| Ramp | Stops |
|---|---|
| stone | `#0f1520 #1e2431 #323745 #4b4f5e #666878 #868390 #a8a0a2 #cbbdb6` |
| dirt | `#10080b #2a1114 #4a231d #6b3a2b #8a5236 #a86a42 #c78a58` |
| grass | `#0c1a14 #143022 #1f4528 #2f5e2c #46792f #66982f #8fb43a #c2d45e` |
| sand | `#2b2030 #54403f #7e604f #a8836a #cfa985 #e8cb9c #f6e6b8` |
| oak / bark | `#2a1214 … #e1b07e` / `#0f080c … #c07a55` |
| steel | `#0b0f18 #161c2a #232b3d #34405a #4a5a78 #6d82a2 #9fb4cc #d3e2ee` |
| bronze, firebrick, ember, aether, amber | see `machines.py` |

## 3. Texture spec

- 128 × 128 RGBA, painted at true 128 (no 2×/4× upscales). Tiles that repeat over greedy-merged faces (stone, dirt, grass top,
  sand, bark, planks, bricks) wrap seamlessly; faces and machine panels are marked `tiles=False` in `registry.py`.
- Cutouts (leaves, glass) have alpha 0 or 255 and keep a colour under the transparent texels.
- **Glow** is a separate grey layer (0..255), not alpha: alpha stays coverage, and the HUD icons and box models are unchanged.
- Light baked into a tile is relief only (bevels and cavities, lit from the top left, like every pixel-art tile), never a
  cast shadow: faces are seen from six directions and the game's own lighting does the rest.
- Ores keep a shared body (`natural.stone(0)`), so alternates and neighbours match. Alternates (ore and leaves, layers
  73–90) move the deposit, not the body.

## 4. Pipeline

```
tools/art (Python, numpy + Pillow)  →  web/src/art/pack.png, pack_glow.png, pack.json  →  render/art-pack.ts  →  Renderer
```

| Command | What it does |
|---|---|
| `python tools/art/build.py [name …]` | Renders the tiles to `artifacts/art2/tiles/`, a review sheet (`artifacts/art2/sheet.png`, repeating tiles shown 2 × 2); a full run also writes the pack |
| `python tools/art/check.py` | The rules below; exit 1 on a failure |
| `python tools/art/compare.py artifacts/art2/current.bin` | Before/after sheet against the engine's generated layers |
| `python tools/art/style.py compare <screenshot>` | Style distance of a screenshot to the samples |
| `OC_DUMP=<file> cargo test -q --release dump_layers -- --ignored` | Dumps the engine's current layers (for `compare.py`, `dump.py`) |

Add a layer: write a recipe returning `lib.rgba(...)` (sprites: `draw.Pen`), append an `Entry` in `registry.py` (engine layer ids are in
`block/tex.rs`), run `build.py` and `check.py`. The game picks the pack up on reload; `?art=off` shows the old look.
Reproducible: every recipe is seeded, so a rebuild gives the same pixels.

## 5. Engine hooks (small, presentation only)

`render/art-pack.ts` (decoder and merge), `Renderer.setTextures(pixels, size, layers, glow)` (second `R8` array on texture
unit 4), `u_glow` in `litFrag` (`shown = mix(lit, albedo × 1.1, glow)`, so a glowing texel ignores daylight and shadow; fog
still applies), `main.ts` (one call; `?art=off`). The engine, saves and `GOLDEN_HASH` are untouched. The engine's own layers
are still generated at 64 × 64 and doubled to fill gaps until every layer has been repainted.

## 6. The outside judge (`check.py`) and its limits

Rules, per tile: **seam** (the wrap edge is no harsher than the harshest line already inside the tile), **phase** (detail at
all four texel phases alike, which an upscaled 64 or 32 grid fails; negative control: upscaling the engine's layers 0, 2, 109,
119 scores 0.00, the pack scores 0.65–1.0), **colours** (≤ 96), **value** (mean L 0.18–0.78, 0.12 for glowing tiles),
**cutout** (alpha 0/255, leaves 55–85% covered, glass 10–45%), **ores** (each deposit covers 6–45% of the tile and stands out
from the stone by > 0.12 Oklab under normal, protan, deutan and tritan vision; ores differ by colour or edge density).
Known weak spots, so nobody over-trusts it: it cannot see taste; the seam test misses a drawn border whose two edges
match (I tried a band test and removed it, it could not tell borders from bricks), so such tiles are marked `tiles=False`
and judged on the review sheet; and the **style distance** is dominated by the sky and global exposure, so darkening the
whole frame would "win". Use it for before/after on one fixed scene, never alone.

## 7. Pilot results (2026-10-18, signed off)

33 layers: stone, dirt, grass top and side, sand, log side and end, planks, stone bricks, leaves (4 looks), coal, iron, copper
(4 looks each), glass, lamp, frame, steel, smelter side and top, generator side and top. All rules pass.

- Style distance to the samples, fixed scene at about 13:00 (`artifacts/art2/before_day.jpg` / `pilot_day.jpg`): 0.1065 → 0.0864
  with 10% of the layers repainted and the sky and lighting untouched. The target for the finished set is 70% closer than
  the starting 0.1065 (about 0.032); the sky and a light grade will have to carry part of that, see follow-ups.
- Size: the pack is 300 KB for 33 tiles (about 9 KB a tile); 347 layers extrapolate to about 3.2 MB (target ≤ 4 MB).
- GPU memory: 347 × 128² RGBA is 22.7 MB, plus 5.7 MB of glow, about 38 MB with mips (target ≤ 40 MB; tight, and the glow
  array could shrink to the layers that glow).
- Load: decoding the pilot pack and merging 347 layers took 75 ms in the browser pane; the full pack's decode is the part to
  re-measure (target ≤ 300 ms extra; a Web Worker or a palette PNG are the levers).
- Wasm size and `generate()` time: unchanged (the engine's generator still runs for the layers not yet repainted).

## 8. Natural batch (2026-10-18, after the user signed off the pilot)

52 more layers: bedrock, spent rock, granite, sandstone, basalt, limestone (+3 alternates), quartz ore (+3), the four
stained soils, grass tops, grass sides and sands (`stains.py`: rust blotches, coal shards, copper veins, chalk chips, each lighter or
darker than the ground so colour-blind players still see them), water, the birch, pine, cactus and dead-wood
blocks, and 16 cutout sprites drawn with `draw.Pen` (`plants.py`: ground cover, saplings, indicator plants, the torch with a
glow flame). `check.py` now judges each variant family against its plain tile (ores, soils, grass tops, sands).
All rules pass; `npm run check` is green. Pack: 744 KB for 85 tiles (about 8.8 KB each, 347 would be about 3 MB). Decode and
merge of the 347 layers measured 80 ms warm (264 ms the first, uncached time on the dev server) with 85 painted. Not verified
in game: water's look (it needs a lake; the tile itself is checked), dusk and night views.
Known soft spots: the coal shrub reads as a bubble cluster, rust stains on sand are flat, cactus spines are tiny.

## 9. Industry batch (2026-10-18)

141 more layers, 226 of 347 painted: belts (tier chevrons glowing), miners, routers, constructor, lab, assembler, motor,
ramps, lift, underpass, pipes, docks, power, process plants, hoist and winch (cutouts), the grid colours, compute campus,
lasers, launch set, locomotive paint, and the decor set and ruins (concretes, bricks, tiles, framed glass, catwalk, railing,
rusted plating, logbook, vault door). All are `kit.Face` recipes: a face is a short list of parts in texel coordinates
(`rect`, `disc`, `port`, `fan`, `coil`, `pipe`, `slots`, `hazard`, `window`, `screen`, ...) over a bevelled plate, so a new
machine is 5-10 lines in the matching module. Tier colours: ember, aether, blue, violet, gold (`kit.TIERS`).
`check.py` changes: the phase rule now needs 0.15 (hard-edged machine tiles score 0.2-0.5, an upscaled grid still 0.0),
and each entry may declare its own `cover` range (hoist frame, glass, pine leaves).
Pack: 1.18 MB for 226 tiles (5.2 KB each: panels compress well); 347 would be about 1.8 MB. `npm run check` is green.
Decode and merge was 175-195 ms warm at 226 layers; the PNG unfilter loops dominate (rewritten per filter type, output
checked byte-identical to Pillow). Extrapolated 250-400 ms for 347, so the 300 ms budget is at risk: a worker or a palette
PNG is the fix. In game only belts, crates and the smelter were looked at closely; the rest are judged on the review sheet.
Soft spots: day and night glow untested on the big machines; water still unseen in a lake.

Review fixes (2026-10-18, after the user played it): rust stains were fat orange blobs that read as fallen leaves, now ragged
iron-oxide patches with a dithered halo (`stains._rust`); verdigris was neon teal veins, now a pale sage crust in broken
patches; granite was a pink gravel, now interlocking pink feldspar, grey quartz and cream plagioclase grains with black mica
flecks (`rocks.granite`). Rule of thumb they taught: a stain must not have a clean round or leaf outline.

## 10. Items batch (2026-10-18): the last 121 layers, 347 of 347 painted

Two kinds of layer. **Materials** that skin small item models and machine lights (`items.py`): metals as bevelled slabs with
a stamp and brushed grain (ingots, plates, wire, rod, screw, beam, forged head, slag, silicon), rubble fields for crushed and
washed ores, tailings, quicklime, sulfur and plastic (`items.rubble`), glowing flask liquid under a glass meniscus for the
five science packs, status lens lights, tier stripes with pips, scanner screens, the avatar's seven parts, the ladder and
smoke cutouts, and three ore blocks in `ores.py` (bauxite pebbles, tar-soaked oil sand, glowing uranium prisms).
**Pictures** the item shows as its whole tile (`icons.py`, `icons2.py`): tools by metal, gear, robot parts, canisters (steel
can, brass cap, a window of the fluid), vehicles, the rocket parts, satellites, drop pod, coin, wafer. They are drawn on the
machine kit's `Face` over a navy backdrop (`icons.canvas`), plus `poly` and `shape` for free-form bevelled solids.
`check.py` passes (three tiles needed darkening: flask glass, avatar skin, fuel tank).
Pack: 1.71 MB for 347 tiles (4.9 KB each; the 3-4 MB target is met with room). Decode and merge of all 347 layers measured
140-220 ms warm in the browser pane, inside the 300 ms budget (my 250-400 ms extrapolation was pessimistic). In game the
hotbar icons are too small to judge; the review sheets are the evidence. Soft spots: the canisters differ by liquid colour
only (their names tell the rest), the boots read as boots only because of the sole, the uranium prisms are thin.

## 11. Next, in order (each one green and its own commit; ask before pushing)

1. **Engine follow-ups the look needs, each its own step:** the crack overlay and minimap still assume 16 cells; a
   warm-light / cool-shadow grade in `litFrag` (the samples' signature: warm highlights, navy shadows); non-cube meshing for
   roofs, arches and slabs; glow on boxes' status lamps from the pack; retire the engine's own 64 px generators once the pack
   is the only source (wasm size).
2. **Judgement pass in game:** night and dusk glow on the big machines, water in a real lake, the style distance to the
   samples on the fixed scene (target about 0.032, 0.0864 at the pilot).
