# Art style: Ember & Aether (pilot signed off 2026-10-18; natural batch built)

> **Direction change, 2026-10-18 (later the same day):** the user found the look too stylized and asked for realism
> back, with copper ore green. Natural blocks and building materials move to the **realistic look** in section 12
> (photo-referenced colour, continuous tone, no outlines, true mineralogy). Machines, items and icons keep Ember &
> Aether. Sections 1-11 stay as the record and still govern the machine faces.

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
rust nodules, copper green malachite crusts along cracks, realistic since section 12); masonry = uneven courses, per-brick tone, moss in the joints; metal =
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
119 scores 0.00, the pack scores 0.65–1.0), **colours** (≤ 96; `real` tiles are judged by **realism** instead, section 12), **value** (mean L 0.18–0.78, 0.12 for glowing tiles),
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
   samples on the fixed scene (target about 0.032, 0.0864 at the pilot). The style distance no longer applies to natural
   blocks (section 12): it rewards closeness to fantasy paintings.

## 12. The realistic look for natural blocks (2026-10-18)

**Why the old look read as stylized (measured, not felt).** The rules themselves made it: "6-30 colours a tile", hard
aliased edges, ink outlines, and a 96-colour cap in `check.py`. Stone had 9 colours, sand 6, water 5. Against CC0 photo
scans of the same materials (ambientCG, lit by their own normal and AO maps from the top left like `relief_light`,
cropped to 1 m and reduced to 128 px), the tiles had **2.5-4x the photos' lightness spread** and **3-10x their
hard-edge share** (stone 33% vs 6%, grass 65% vs 9%). Noise was not the cause: spectral slopes were already natural
(-1.8 to -2.3). The causes are posterising, outlines, one sticker motif per tile and invented hues.

**The look.** Structure stays procedural (it must wrap, stay small and stay seeded); colour comes from photos.
`real.colour(mat, field)` maps any structure field onto the fit photos' lightness quantiles and their a, b by
lightness, with patchy hue variation of the photos' size. Continuous tone, no outlines, no ordered dither. Relief may be
up to twice the photos' lightness spread, since a block must read at a distance without normal maps. Minerals take their
real colours: copper is malachite green with azurite and brassy chalcopyrite, uranium is black pitchblende with yellow
autunite (which fluoresces, so only it glows).

**The judge.** `python tools/art/realism.py fetch` (photos to `artifacts/art2/refs/`, git-ignored), `reference`
(writes `tools/art/reference/materials.json`: fit-photo palette, holdout-photo cloud, photo metrics), `score`. Fit photos
give the palette and holdout photos judge it, so a tile cannot pass by copying its judge. `check.py` applies to every
`real=True` entry with a material: colour distance at most 1.5x the photo-to-photo floor, hard edges at most 5 points
over the photos', lightness spread at most 2x. A deposit rule now compares ores as colour clouds under all four vision
types (pairs under 0.03 fail; pairs under 0.08 are printed, they lean on shape for colour-blind players).
Limits: the floor is tiny for planks and sand (0.005, 0.009), so those will need a looser absolute limit; scan sizes are
stated for 6 of 23 photos (others taken as 150 cm); `colour` reproduces the fit photos' value distribution, so the
distance mostly checks hue and the holdout match, while edges and spread check the structure.

**Done in this step.** Copper ore (4 looks) as malachite crusts along cracks with a green stain fading into the rock;
crushed and washed copper follow it; uranium as pitchblende with autunite. Copper vs uranium is 0.086-0.100 (cloud
distance) under every vision type, up from 0.064. Cost: copper vs iron under deutan fell from 0.087 to 0.069, as a
green ore must; the crusts are lighter than iron's nodules so lightness carries part of it. The minimap mark was
already green (`0x3cc8a0`). Prototypes `real.stone`, `real.dirt`, `real.grass_top` pass the judge:

| Tile | Distance (limit) | Hard edges (limit) | Lightness spread (limit) | Old tile |
|---|---|---|---|---|
| stone | 0.030 (0.042) | 7.5% (11.0%) | 0.069 (0.080) | 0.051, 33%, 0.143 |
| dirt | 0.033 (0.047) | 9.3% (27.6%) | 0.093 (0.107) | 0.075, 20%, 0.095 |
| grass top | 0.047 (0.068) | 10.1% (14.2%) | 0.054 (0.065) | 0.042, 65%, 0.107 |

Stone's distance is 41% better, not the 50% targeted: 50% would sit below the photo-to-photo floor (0.028).
Grass's colour was already within the photos' range; its stylization was contrast and edges only.
In game (`ART_REAL=1 python tools/art/build.py`, fixed pit scene, `artifacts/art2/ab_A_day.jpg` vs `ab_B_day.jpg`): the
realistic stone and dirt read as rock and soil, but two problems show that the review sheet hides. **The stone's repeat
grid shows on a large floor**, and **the old ore stickers look pasted on** against a realistic body. Pack: 1.78 MB now,
2.07 MB with the prototypes (about 15 KB a realistic tile against 5 KB); about 90 realistic tiles extrapolate to about
2.7 MB (budget 4 MB). Decode and merge stayed within 110-250 ms in the pane, inside the 300 ms budget.

**Evaluation of the other natural tiles** (colours / hard edges / lightness spread; photo distance where photos exist):

| Tile | Measured | What reads stylized | Suggested change |
|---|---|---|---|
| grass side | 35 / 33% / 0.105 | dithered shadow band, old dirt | real dirt + real grass, ragged lip, soft shadow |
| sand | 6 / 12% / 0.079, d 0.024 (0.013) | 6 tones, pinkish | photo palette (Ground054/080), same ripples at lower contrast |
| log side | 14 / 20% / 0.124, d 0.125 (0.056) | red-orange bark, moss stripes; worst distance | grey-brown fissured plates in the Bark012/014 palette, moss only in a few fissures |
| log top | 11 / 19% / 0.124 | perfect concentric orange rings | irregular rings, paler heartwood, radial checks (needs an end-grain photo) |
| planks | 8 / 20% / 0.089, d 0.057 | orange oak, black seams, pixel nails | greyer brown from Planks023A/037A, soft seams, continuous grain |
| stone bricks | 20 / 17% / 0.125 | pink and grey alternating, moss in every joint | real stone cut into courses, light mortar, rare moss |
| leaves (3 trees) | 9 / 19% / 0.13-0.15 | clean sprite leaves; pine is teal (173°) | smaller, denser leaves in continuous tone; pine toward 150-160° |
| bedrock, spent rock | 9-10 / 11-44% / 0.07-0.19 | outlined cells | darker and paler `real.stone` variants |
| granite | 32 / 55% / 0.181, d 0.067 (0.040) | grains right, too contrasty and pink | Granite001A palette, finer grains, half the contrast |
| sandstone | 8 / 23% / 0.100 | saturated orange bands | bedding in a sandstone photo palette (needs a photo) |
| basalt | 9 / 15% / 0.096, d 0.040 (0.040) | outlined cells | fine dark rock, a few vesicles, joints as cracks only |
| limestone | 8 / 27% / 0.083, ink 10% | cartoon spiral fossils | keep the palette (passes), fossils as faint shell fragments |
| coal, iron, quartz, bauxite, oil sand | 15-27 / 23-45% / 0.15-0.18 | outlined stickers on cobble | on `real.stone`: coal as dull black seams with a sooty halo, iron as red-brown and ochre staining, quartz as milky veins, bauxite as small soft pisoliths in red earth, oil sand as dark oily sand |
| water | 5 / 24% / 0.077 | cyan caustic cells | after the water renderer work in progress: deeper blue-green, soft ripples |
| birch bark, dead wood, cactus | 5-9 / 12-22% / 0.11-0.19 | few tones, ink lines (cactus 9.7%) | more tones, varied lenticels, no outlines; follows the bark |
| hint stains (12) | 10-21 / 26-69% / 0.09-0.14 | blobs on the stylized ground | follow their ground; a soft colour shift plus small mineral grains, keeping the shape rules |
| concrete, masonry bricks, basalt tiles | 7-11 / 14-19% / 0.07-0.18 | black mortar, loud per-brick tone | light mortar, subtle per-brick tone (needs photos) |

**Suggested rollout, each a reviewed step with an in-game A/B:**

1. **Ground (done, below):** stone (and so every ore body), dirt, grass top and side, sand, their 12 hint variants.
2. **Ores and rocks:** the five sticker ores redone on the real body, then bedrock, spent rock, granite, basalt,
   limestone, sandstone.
3. **Wood:** bark, log end, planks, leaves of the three trees, birch, dead wood, cactus.
4. **Built materials:** stone and masonry bricks, concrete, basalt tiles (fetch brick and concrete photos first).

**Batch 1 done (2026-10-18): the ground.** `real.py` now paints stone (so every ore body, quartz included), dirt, grass
top, grass side (real turf over real soil, a soft shadow, no dither) and sand (asymmetric ripples). The 12 hints are
`stains.apply` over them: blended in Oklab with soft edges, in mineral colours (ochre oxide, coal dust, muted copper
green, chalk), each keeping its shape language. Iron is big ragged patches, coal angular Voronoi chips (no longer leaf
shapes), copper broken crusts, chalk small chips. Lightness still carries each stain for colour-blind players: rust and
chalk lift more (protan reads red darker), and on grass rust goes darker and muted, as bare rusty soil, since bright
orange on turf read as paint and darkened full-chroma oxide read as blood. The old stylized ground recipes are gone
from `natural.py`. Measured: stone 0.030 / 7.5% / 0.069, dirt 0.033 / 9.3% / 0.093, grass 0.046 / 10.1% / 0.054,
sand 0.013 / 8.2% / 0.061 (limit 0.014: sand's two fit photos are nearly alike, so its floor is tight); every hint
stands out on 8-37% of its tile under all four vision types. Pack 2.56 MB (35 continuous-tone tiles at about 22 KB
each, not the 15 KB I estimated). The rest of the rollout extrapolates to about 3.8 MB against the 4 MB budget, so
batch 2 should measure it and may need palette-reduced PNGs for tiles whose detail allows it. Decode and merge
measured 170-241 ms warm. In game: `artifacts/art2/batch1_pit.jpg` and `batch1_meadow.jpg`. Still showing: the stone
floor's repeat grid (the engine step below, in progress in its own session), and the coal, iron, bauxite and oil sand
deposits as stylized stickers on the realistic body (batch 2).

**Batch 2 done (2026-10-18): ores and rocks.** Deposits now grow into the rock instead of sitting on it: soft-edged
masks, and the body's own relief multiplied through the deposit (`ores._relief`). Coal is a ragged black seam with faint
laminae, cleat cracks and a soot halo; iron ragged hematite-to-limonite masses with a rust stain, sized by quantile so
every alternate shows about a quarter of deposit; bauxite pale laterite crowded with pisoliths of three sizes; oil sand
dark granular sand with glossy pools and loose grains. Copper's crust and azurite and uranium's masses and autunite got
the same soft edges. The rocks are in `rocks.py` through `real.colour`: granite a flat crystal mosaic (Granite001A),
sandstone soft cross-bedded strata (new photo Rock029), basalt rough dark rock with vesicles and a rare joint
(Rock035), limestone pale beds with fading seams, long fractures and pits (Rock023, the cartoon fossils are gone),
bedrock big dark shards with soft steps (basalt colours, a shade darker, so it reads apart from basalt by structure),
spent rock bleached, pitted `real.stone` with faint oxide seams. Judge (distance / edges / spread): granite 0.002 /
15.2% / 0.075, sandstone 0.028 / 12.7% / 0.075 (edge limit 13.8%), basalt 0.006 / 13.2% / 0.075, limestone 0.006 /
15.4% / 0.095. Deposit colours: every ore pair is at least 0.034 at its worst vision. The ores sit on a lightness ladder
so colour-blind players can tell them apart (coal 0.17, oil sand 0.23, iron 0.30, bauxite 0.54 under protan), since
coal, oil sand, iron and pitchblende are all dark; closest pairs: coal vs oil sand 0.035, iron vs oil sand 0.034. Pack
2.78 MB (+0.22 MB for 22 tiles, about 10 KB each, so the smooth ore deposits compress better than ground). Decode and
merge 175-225 ms. Still stylized among natural blocks: quartz ore's crystal fans (outlined), then wood (batch 3) and
built materials (batch 4).

**Engine follow-up** (done, b462dea): natural tiles are turned per block, so large surfaces no longer show the grid.
