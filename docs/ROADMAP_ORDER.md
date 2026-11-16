# OpenCraft: order of work, rated by impact and cost (2026-11-16)

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
- **Done** is the share of all impact (129 points over 117.5 sessions) delivered once that row is finished.

Not in the table: **13.6b** balance (needs the user to play Milestone 13 first) and the user's **TURN secrets**.
Do them when their turn comes; each is one session or less.

## What the numbers say

- The first 11 rows take 12.5 sessions and deliver 24% of the impact; the first 23 take 35.5 sessions for 51%.
- **Fire (FIRE1–4) drops from "next" to rows 29–40.** It was asked for as the next build after L2–L6, and each step
  rates 1–3. Move it up by raising its impact if the user wants it sooner; nothing else depends on it.
- **CHEM0 is first** because it costs one session and decides the order of the eight chemistry families,
  the lubricant dead end and what GLASS, CODEX and the space elevator need.
- **PERF2 (cached machine geometry)** has the most hidden risk of the performance items: it touches the render hot
  path for about 25% of a steady frame. Measure first with `scripts/frame-bench.mjs`.
- **M14** has the highest impact (5) and the lowest value among planned work (10 sessions). It is the destination, not
  a quick win. It needs carbon fibre (CHEM-CARB) and liquid oxygen (CHEM-COOL).
- **Slabs and stairs** (impact 4, 6 sessions) is the best of the "later" items for builders; it is the first
  candidate to promote.

## Wave 1: cheap wins (one session each, mostly independent)

| # | Id | Item | Impact | Sessions | Value | Done | Needs |
|---|---|---|---|---|---|---|---|
| 1 | CHEM0 | Chemistry design talk, lubricant decision (steers all families below) | 4 | 1 | 4.00 | 3% | - |
| 2 | DOCS | Compact plan and code map under 48 KB (every future session reads them) | 3 | 1 | 3.00 | 5% | - |
| 3 | L2 | Lesson: no recipe chosen (most common stall) | 3 | 1 | 3.00 | 8% | - |
| 4 | L5 | Lesson: upgrade with kits (upgrading is new in v0.8.3) | 3 | 1 | 3.00 | 10% | - |
| 5 | F-BEAT | Factory as an instrument (presentation only) | 3 | 1 | 3.00 | 12% | - |
| 6 | MERGE | Merger building (new building, save bump, art request) | 4 | 2 | 2.00 | 16% | - |
| 7 | PERF1 | Atlas as lossless WebP (4.5 to 2.7 MB) (load time on slow links) | 3 | 1.5 | 2.00 | 18% | - |
| 8 | L3 | Lesson: blueprint walk-through | 2 | 1 | 2.00 | 19% | - |
| 9 | L4 | Lesson: planner | 2 | 1 | 2.00 | 21% | - |
| 10 | L6 | Lesson: map and pins | 2 | 1 | 2.00 | 22% | - |
| 11 | WIRES | Wires hang from the pole arms (visual only) | 2 | 1 | 2.00 | 24% | - |

## Wave 2: logistics, light and the first chemistry (two sessions each)

| # | Id | Item | Impact | Sessions | Value | Done | Needs |
|---|---|---|---|---|---|---|---|
| 12 | LOOK | Clearer splitter, filter, merger models (art-dependent) | 3 | 2 | 1.50 | 26% | MERGE |
| 13 | XRAY1 | X-ray goggles: buried logistics (tech in blue science after hoists) | 3 | 2 | 1.50 | 29% | - |
| 14 | LED | Brighter LED lamps with colour control (block ids plus a colour record) | 3 | 2 | 1.50 | 31% | - |
| 15 | STREET | Tall street lamps (uses the led light row) | 2 | 1 | 2.00 | 33% | LED |
| 16 | CHEM-DYE | Chemistry: dyes and pigments | 3 | 2 | 1.50 | 35% | CHEM0 |
| 17 | GLASS | Stained glass and coloured lamps (needs pigments) | 3 | 2 | 1.50 | 37% | CHEM-DYE, LED |
| 18 | CHEM-AIR | Chemistry: air and gas capture (feeds cooling, nitrogen, helium) | 3 | 2 | 1.50 | 40% | CHEM0 |
| 19 | CHEM-N | Chemistry: Haber-Bosch, nitrogen compounds | 3 | 2 | 1.50 | 42% | CHEM-AIR |
| 20 | CHEM-CARB | Chemistry: carbon and silicon (carbon fibre for the elevator tether) | 3 | 2 | 1.50 | 44% | CHEM0 |
| 21 | F-TAG | Follow one item (host-side watcher first) | 3 | 2 | 1.50 | 47% | - |
| 22 | F-TIME | Timelapse of the world (shareable) | 3 | 2 | 1.50 | 49% | - |
| 23 | F-SITE | Site bonuses: solar by biome, wind by height (shown on the ghost) | 3 | 2 | 1.50 | 51% | - |

## Wave 3: the heavier engine work, water, worlds, gas and fire

| # | Id | Item | Impact | Sessions | Value | Done | Needs |
|---|---|---|---|---|---|---|---|
| 24 | PERF2 | Cache static machine geometry (about 25% of a steady frame) (touches render hot path) | 3 | 3 | 1.00 | 53% | - |
| 25 | CHEM-COOL | Chemistry: cooling, cryogenics, dry ice (liquid oxygen for rockets) | 3 | 3 | 1.00 | 56% | CHEM-AIR |
| 26 | PUMPDOWN | Pump out a sealed flooded shaft (core change, lockstep search) | 3 | 3 | 1.00 | 58% | - |
| 27 | WORLDS | Upgrade old worlds by generator layer (deposits) (gates gas fields in old saves) | 3 | 4 | 0.75 | 60% | - |
| 28 | CHEM-GAS | Chemistry: natural gas and helium (gas fields need the deposits layer) | 4 | 3 | 1.33 | 64% | CHEM-AIR, WORLDS |
| 29 | FIRE1 | Fire core: burning trees to charcoal | 3 | 3 | 1.00 | 66% | - |
| 30 | FIRE3 | Fire look | 2 | 1 | 2.00 | 67% | FIRE1 |
| 31 | FIRE2 | Flamethrower (open: which research) | 3 | 2 | 1.50 | 70% | FIRE1 |
| 32 | F-SEIS | Seismic prospecting | 3 | 3 | 1.00 | 72% | - |
| 33 | L-AWAY | Catch-up on return | 3 | 3 | 1.00 | 74% | - |
| 34 | XRAY2 | X-ray goggles: buried blocks (only if the cost measures well) | 2 | 2 | 1.00 | 76% | XRAY1 |
| 35 | CHEM-SALT | Chemistry: salt, alkali, Solvay glass | 2 | 2 | 1.00 | 78% | CHEM0 |
| 36 | L-BEAM | Power beam from satellites | 2 | 2 | 1.00 | 79% | - |
| 37 | L-SET | World settings screen | 2 | 2 | 1.00 | 81% | WORLDS |
| 38 | PERF3 | Pump leaves its source cell alone (changes the state hash) | 1 | 1 | 1.00 | 81% | - |
| 39 | CHEM-SMALL | Chemistry: smaller recipes | 1 | 1 | 1.00 | 82% | CHEM0 |
| 40 | FIRE4 | Fire manual and cleanup | 1 | 1 | 1.00 | 83% | FIRE2, FIRE3 |
| 41 | CODEX | Periodic table and achievements (fills as chemistry grows) | 3 | 4 | 0.75 | 85% | CHEM-AIR |

## Wave 4: big and long-term

| # | Id | Item | Impact | Sessions | Value | Done | Needs |
|---|---|---|---|---|---|---|---|
| 42 | L-SLAB | Slabs and stairs (meshing, placement, collision) | 4 | 6 | 0.67 | 88% | - |
| 43 | F-GEO | Geothermal on basalt vents (a worldgen feature) | 2 | 3 | 0.67 | 90% | WORLDS |
| 44 | M14 | Space elevator megaproject (capstone, optional and never-ending) | 5 | 10 | 0.50 | 94% | CHEM-CARB, CHEM-COOL |
| 45 | L-TD | Tower-defence mode (still peaceful) | 3 | 6 | 0.50 | 96% | - |
| 46 | L-SRV | Dedicated server | 2 | 5 | 0.40 | 98% | - |
| 47 | L-MOD | Mod support | 2 | 6 | 0.33 | 99% | - |
| 48 | L-P345 | Cheaper box instances, incremental checksum, small TS savings (three parked items) | 1 | 3 | 0.33 | 100% | - |

Ids: Lx lessons (DEV_PLAN L1); F-* the feel-and-feedback interlude (`ROADMAP.md`); CHEM-* the chemistry families
(`ROADMAP_CHEMISTRY.md`, numbered there 1–8); L-* items from "Later / maybe" and the parked list.
