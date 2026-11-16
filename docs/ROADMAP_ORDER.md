# OpenCraft: order of work, rated by impact and cost (2026-11-16, audited the same day)

One ranked list of every open item in `docs/DEV_PLAN.md`, `ROADMAP.md`, `ROADMAP_REQUESTS.md` and
`ROADMAP_CHEMISTRY.md`. It is the order to build in unless the user says otherwise. The plan of record stays
`docs/DEV_PLAN.md`; this file only orders what it lists.

## How the numbers work

- **Impact 1–5** (my judgment; change a number and re-sort): 5 changes what the game is; 4 is something most players
  hit every session or a design decision that steers a lot of later work; 3 is clearly felt in a play session;
  2 is polish or helps some players; 1 is small.
- **Sessions** (one scoped step each, as in `CLAUDE.md`): the estimate to finish green and committed, from the
  design notes and how much core, save, art and test work the item has.
- **Value** = impact ÷ sessions. Items sort by value, except that an item waits for what it needs (**needs**), and a
  gate is ranked with what it unlocks (WORLDS is 3 ÷ 4 alone but 7 ÷ 7 with natural gas and helium).
- **Done** is the share of all impact (132 points over 118.5 sessions) delivered once that row is finished.

Not in the table: **13.6b** balance (needs the user to play Milestone 13 first) and the user's **TURN secrets**.
Do them when their turn comes; each is one session or less.

## Audit (2026-11-16): what the code and the user's saves say

**Room for content is the hidden gate.** `BlockId` is a `u8` (`block/mod.rs`) and item ids start at 256, so
`BLOCK_COUNT` 219 leaves **37 block ids**. Planned demand, counted per item: MERGE 1, LED 3, STREET 1, GLASS 16 (32 if
coloured lamps are ids too), PUMPDOWN 1, chemistry machines about 12 (air compressor, column, chiller, heat exchanger,
ammonia reactor, gas well, wellhead separator, gas plant, gas turbine, reformer, salt works, coke oven), the gas-field
ore 1, the thumper 1, the wind turbine 1, geothermal 2, the elevator about 4: **about 43 without slabs, about 85 with
them**. Texture layers: the cube vertex holds the layer in 9 bits (`mesher/quad.rs` `layer << 23`, read by
`shaders.ts`), so cube faces must use layers below 512; `tex::COUNT` is 472. Item and model layers can pass 512 by
widening one mask (`shaped.ts` reads `(w2 >> 16u) & 511u` of a field the engine fills as 16 bits; box instances carry
layers as floats), up to the device's `MAX_ARRAY_TEXTURE_LAYERS`. New row **ROOM** settles both before wave 2 fills it. **Decided (user, 2026-11-16): block ids become `u16`
everywhere** (no palette, no ration); ROOM measures the memory and save-size cost and settles where item ids sit.

**Corrections to items** (each in its roadmap file):
- **LED (measured):** the planned row (47, 1) fails. A throwaway test on the `light/tests.rs` harness, a lamp in a
  sealed tunnel: light stays 15 out to 32 blocks, then drops to 0, and at distance 33 one cell reads 0 and the next 14
  (whether the lamp lies in that chunk's field): seams at chunk borders. A row (64, 2) in the short field is seamless
  (cells at equal distance agree) and full-bright out to 24 blocks instead of 17 (twice the area), falling to 0 at 32.
- **CHEM-GAS / WORLDS:** a gas row in `EXTRAS` cannot keep old deposits: `ore_at` draws one pick over the biome's whole
  weight sum, so a new weight re-rolls every far deposit in those biomes (iron, copper, any ore), not only oil and
  uranium. Gas fields need their own placement (own hash and grid), with `ore_at` untouched and a pin test that every
  ore deposit of the bench save is identical before and after the upgrade.
- **CODEX:** fold the core's `SimEvent::Produced` (miners, quarry, processors, towns already emit it) into the set,
  plus hand crafts and pickups, instead of a scan every 30 ticks, which misses an item that leaves a machine in between.
- **CHEM families:** four planned sinks are multipliers, which the binding decision forbids: oxygen for blast-furnace
  yield, coke for blast-furnace yield, silicon as a higher-yield wafer path. Superconducting cable has no loss to remove:
  wires lose nothing (`factory/pole.rs`; only laser beams lose 10%, `laser.rs` `LOSS_DIVISOR`).
- **GLASS:** coloured lamps take the LED's colour record instead of 16 ids; stained glass keeps 16 ids and 16 layers.
- **Feasible as written, spot-checked:** XRAY1 (machines and belts are box instances, `factory/render.rs`; Hoists is
  tech 51), MERGE, WIRES, PUMPDOWN (one id). Not re-checked: lessons, FIRE, PERF, the F-* presentation items.

**The user's saves (6 exports, 0.4 h to 39 h) say demand, not supply, is missing.** Techs done: 11 at 10.6 h, 22 at
18.3 h, 43 at 24.9 h, 80 at 37.6 h, 81 of 87 at 39 h (the rest are endless multipliers, never started). From 10.6 h on,
about half the machines wait because their output has nowhere to go (20/21, 8/36, 26/53, 60/106, 53/102, 54/104). At
39 h the chemical plants, pumpjacks, chip fabs, centrifuge and launch site stand at 0%: the existing chemistry already
has no pull. The audit split the elevator (**M14A**: anchor and first sections as standing orders of what the player
already makes: steel, aluminium, plastic, chips, rocket fuel; **M14B**, the rest, asks for carbon fibre and liquid oxygen,
so chemistry answers the elevator's needs instead of adding stock) and proposed M14A for wave 2. **The user said no
(2026-11-16): both stay last**, so the idle chains wait for them. **CHEM0 (done 2026-11-16) started from the
elevator's bill of materials:** every family's product must end in the elevator, a town order or a player's want
(decor, light). Result: the families that only feed the elevator (AIR, COOL, CARB) moved to wave 4 beside it, since alone
they would be stock with no pull; PIGMENTS and GAS stay early; nitrogen, salt and the small families are parked
(`docs/ROADMAP_CHEMISTRY.md`).

**Estimates:** sessions 2026-11-12 to 16 (27 transcripts) made about 20 commits, among them items this table would
rate at 3 or more sessions (realistic trees and whole-tree felling, 2026-11-14). Absolute session counts look high,
the ratios hold; what the table missed was the unbudgeted gate above.

## What the numbers say

- The first 12 rows take 13.5 sessions and deliver 23% of the impact; the first 25 take 43.5 sessions for 53%.
- **Fire (FIRE1–4) sits at rows 27–37, and stays in wave 3 (user, 2026-11-16).** Each step rates 1–3; nothing else
  depends on it.
- **CHEM0 is done** (it decided the order of the chemistry families, the lubricant dead end and what GLASS, CODEX and
  the elevator need). **CHEM-TEACH** is the cheap first chemistry step; the oil chain's real jam is diesel, not lubricant.
- **PERF2 (cached machine geometry)** has the most hidden risk of the performance items: it touches the render hot
  path for about 25% of a steady frame. Measure first with `scripts/frame-bench.mjs`.
- **M14A** rates the highest value of any large item (5 ÷ 3: the game's missing goal and a sink for idle chains), but the
  user keeps Milestone 14 last (2026-11-16): M14A and M14B are the final rows. Raise M14A only if the user changes this.
- **CBPAL and MUSIC** are new rows from the user's 2026-11-16 answers (ratings are mine: change them and re-sort).
- **Slabs and stairs** (impact 4, 6 sessions) is the best of the "later" items for builders; it needs ROOM.

## Wave 1: cheap wins (one session each, mostly independent), then the gate

| # | Id | Item | Impact | Sessions | Value | Done | Needs |
|---|---|---|---|---|---|---|---|
| 1 | DOCS | Compact plan and code map under 48 KB (every future session reads them) | 3 | 1 | 3.00 | 2% | - |
| 2 | L2 | Lesson: no recipe chosen (most common stall) | 3 | 1 | 3.00 | 5% | - |
| 3 | L5 | Lesson: upgrade with kits (upgrading is new in v0.8.3) | 3 | 1 | 3.00 | 7% | - |
| 4 | F-BEAT | Factory as an instrument (presentation only) | 3 | 1 | 3.00 | 9% | - |
| 5 | MERGE | Merger building (new building, save bump, art request) | 4 | 2 | 2.00 | 12% | - |
| 6 | PERF1 | Atlas as lossless WebP (4.5 to 2.7 MB) (load time on slow links) | 3 | 1.5 | 2.00 | 14% | - |
| 7 | L3 | Lesson: blueprint walk-through | 2 | 1 | 2.00 | 16% | - |
| 8 | L4 | Lesson: planner | 2 | 1 | 2.00 | 17% | - |
| 9 | L6 | Lesson: map and pins | 2 | 1 | 2.00 | 19% | - |
| 10 | WIRES | Wires hang from the pole arms (visual only) | 2 | 1 | 2.00 | 20% | - |
| 11 | CBPAL | Colour-blind palette for tier colours, a setting (presentation only) | 2 | 1 | 2.00 | 22% | - |
| 12 | CHEM-TEACH | Chemistry cards teach: one-line real process per product, the diesel-jam lesson (`docs/ROADMAP_CHEMISTRY.md`) | 2 | 1 | 2.00 | 23% | - |
| 13 | ROOM | Room for content: block ids to `u16` everywhere (decided; measure memory and save size) and model layers past 512 | 4 | 3 | 1.33 | 27% | - |

## Wave 2: logistics, light and the first chemistry (two sessions each)

| # | Id | Item | Impact | Sessions | Value | Done | Needs |
|---|---|---|---|---|---|---|---|
| 14 | LOOK | Clearer splitter, filter, merger models (art-dependent) | 3 | 2 | 1.50 | 29% | MERGE |
| 15 | XRAY1 | X-ray goggles, worn in an equipment slot: buried logistics (tech in blue science after hoists) | 3 | 2 | 1.50 | 31% | - |
| 16 | LED | LED lamps, unpowered, a later tech; light row (64, 2), colour record (measured above) | 3 | 2 | 1.50 | 33% | ROOM |
| 17 | STREET | Tall street lamps (uses the led light row) | 2 | 1 | 2.00 | 35% | LED |
| 18 | CHEM-DYE | Chemistry: pigments (chemical-plant rows only, no new block id) | 3 | 2 | 1.50 | 37% | - |
| 19 | GLASS | Stained glass (16 ids); coloured lamps via the LED record | 3 | 2 | 1.50 | 39% | CHEM-DYE, LED |
| 20 | F-TAG | Follow one item (host-side watcher first) | 3 | 2 | 1.50 | 42% | - |
| 21 | F-TIME | Timelapse of the world (shareable) | 3 | 2 | 1.50 | 44% | - |
| 22 | F-SITE | Site bonuses: solar by biome, wind by height (shown on the ghost) | 3 | 2 | 1.50 | 46% | - |

## Wave 3: the heavier engine work, water, worlds, gas and fire

| # | Id | Item | Impact | Sessions | Value | Done | Needs |
|---|---|---|---|---|---|---|---|
| 23 | PERF2 | Cache static machine geometry (about 25% of a steady frame) (touches render hot path) | 3 | 3 | 1.00 | 48% | - |
| 24 | PUMPDOWN | Pump out a sealed flooded shaft (core change, lockstep search) | 3 | 3 | 1.00 | 51% | - |
| 25 | WORLDS | Upgrade old worlds by generator layer (deposits) (gates gas fields in old saves) | 3 | 4 | 0.75 | 53% | - |
| 26 | CHEM-GAS | Chemistry: natural gas (fields apart from `ore_at`, well, plant, turbine, reformer, polyethylene; helium and LNG wait for COOL) | 4 | 3 | 1.33 | 56% | WORLDS, ROOM |
| 27 | FIRE1 | Fire core: burning trees to charcoal | 3 | 3 | 1.00 | 58% | - |
| 28 | FIRE3 | Fire look | 2 | 1 | 2.00 | 60% | FIRE1 |
| 29 | FIRE2 | Flamethrower (research: Claude's pick; charcoal counts as coal) | 3 | 2 | 1.50 | 62% | FIRE1 |
| 30 | F-SEIS | Seismic prospecting | 3 | 3 | 1.00 | 64% | - |
| 31 | L-AWAY | Catch-up on return | 3 | 3 | 1.00 | 67% | - |
| 32 | XRAY2 | X-ray goggles: buried blocks (only if the cost measures well) | 2 | 2 | 1.00 | 68% | XRAY1 |
| 33 | L-BEAM | Power beam from satellites | 2 | 2 | 1.00 | 70% | - |
| 34 | L-SET | World settings screen | 2 | 2 | 1.00 | 71% | WORLDS |
| 35 | MUSIC | Music: a sea piece, industrial and space pieces by era (era is a new director context) | 2 | 2 | 1.00 | 73% | - |
| 36 | PERF3 | Pump leaves its source cell alone (changes the state hash) | 1 | 1 | 1.00 | 73% | - |
| 37 | FIRE4 | Fire manual and cleanup | 1 | 1 | 1.00 | 74% | FIRE2, FIRE3 |
| 38 | CODEX | Periodic table and achievements, set from `Produced` events (fills as chemistry grows) | 3 | 4 | 0.75 | 77% | CHEM-DYE |

## Wave 4: big and long-term, then the elevator and its chemistry last (user, 2026-11-16)

| # | Id | Item | Impact | Sessions | Value | Done | Needs |
|---|---|---|---|---|---|---|---|
| 39 | L-SLAB | Slabs and stairs (meshing, placement, collision) | 4 | 6 | 0.67 | 80% | ROOM |
| 40 | F-GEO | Geothermal on basalt vents (a worldgen feature) | 2 | 3 | 0.67 | 81% | WORLDS |
| 41 | L-TD | Tower-defence mode (still peaceful) | 3 | 6 | 0.50 | 83% | - |
| 42 | L-SRV | Dedicated server | 2 | 5 | 0.40 | 85% | - |
| 43 | L-MOD | Mod support | 2 | 6 | 0.33 | 86% | - |
| 44 | L-P345 | Cheaper box instances, incremental checksum, small TS savings (three parked items) | 1 | 3 | 0.33 | 87% | - |
| 45 | CHEM-AIR | Chemistry: air compressor and column (nitrogen, oxygen, argon), built beside the elevator it feeds | 3 | 2 | 1.50 | 89% | ROOM |
| 46 | CHEM-COOL | Chemistry: chiller, liquid oxygen, helium and LNG (M14B pull) | 3 | 3 | 1.00 | 92% | CHEM-AIR, CHEM-GAS |
| 47 | CHEM-CARB | Chemistry: carbon fibre from heavy oil, plastic and nitrogen (M14B pull) | 3 | 2 | 1.50 | 94% | CHEM-AIR |
| 48 | M14A | Space elevator phase 1: anchor and first tether sections as standing orders of existing goods (kept last by the user) | 5 | 3 | 1.67 | 98% | - |
| 49 | M14B | Space elevator, the rest (capstone, optional and never-ending) | 3 | 7 | 0.43 | 100% | M14A, CHEM-CARB, CHEM-COOL |

Ids: Lx lessons (DEV_PLAN L1); F-* the feel-and-feedback interlude (`ROADMAP.md`); CHEM-* the chemistry families and steps
(`ROADMAP_CHEMISTRY.md`); L-* items from "Later / maybe" and the parked list; ROOM and M14A from
the audit above; CBPAL and MUSIC from the user's 2026-11-16 answers (`ROADMAP_REQUESTS.md`).
