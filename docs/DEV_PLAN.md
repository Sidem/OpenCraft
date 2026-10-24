# OpenCraft development plan

**Status (2026-10-19):** Milestones 1–12, Milestone 13 (Settlements, all but its balance step 13.6b) and the play-test
response R1–R7 plus S1–S4 (sky and weather) are built and pushed; the realistic art pass is live. R8 is decided
as "none for now" (2026-10-22) and pod channels by name are done (2026-10-20); 13.6b waits for the user to play
Milestone 13. Map hover and filters (U1), the telescope (U2) and the sickle for ground plants (U3) are done (2026-10-22), and
belt items tell where they came from and can be clicked to stay selected (2026-10-24). **Next: the feel-and-feedback interlude**, then the carry's era pieces, Milestone 14 (the space elevator).

> **This project is written entirely by AI coding agents.** Every session starts cold and every line read costs tokens.
> **Keeping the codebase small, modular and cheap to read is as important as any feature** (section 3.1, mandatory).

This is the plan of record: direction, rules, and what to build next. Settled decisions: `docs/DECISIONS.md`. Finished
work: `docs/history/` (read only to learn how something came to be). Player's manual: `docs/PLAYING.md`, `FACTORY.md`,
`TOWNS.md`. Future milestones: `docs/ROADMAP.md`; tech tree: `docs/TECH_TREE.md` (concept, its section 8 is the content
architecture) and `docs/TECH_ERAS.md` (each era's recipes). Read those only when planning.

---

## 0. Handover: read this first

A working browser factory game, live at <https://sidem.github.io/OpenCraft/>: a Rust → wasm engine and a thin
TypeScript/WebGL2 host. It has belts, power, research with five science packs, tiers Mk1–Mk5, steam, chemistry, nuclear,
compute, trains, drones, terraforming, orbit, towns, co-op for 2–4 players, biomes, rivers, plants, ruins, weather and music.

Before you change code:

1. Read this file, then `docs/CODEMAP.md`, then the nested `CLAUDE.md` of the area you work in.
2. `npm run build:wasm` (if `web/src/wasm` is missing), then `npm run check` for a green baseline.
3. Do one step of section 4, or one clean part of it, per session; stop green.
4. When a step is done: tick it here, update the status line, add a line to section 6. Players will notice it: a line in
   `web/src/patch-notes.md` (newest `## date · title` first). Structural change: `docs/CODEMAP.md` in the same commit.

Ground rules (details in `docs/WORKFLOW.md`): **never push to `main` without asking** (a push deploys the live site);
commit only when asked; PowerShell and the file tools on this Windows machine (the Bash tool fails); performance first,
dependencies justified; explain things to the user in plain language.

---

## 1. Where the project is heading

A browser game between **Minecraft** and **Satisfactory**: a fully editable voxel world that you mine, shape and
industrialise. Not a Minecraft clone; its conventions can be broken freely.

### Pillars

1. **Everything is physical.** Items travel on belts, machines are blocks in the world, the world is the material.
2. **Finite ground pushes you outward.** Deposits run dry after a long time; distance, depth and scale become the problems.
3. **Terrain is something you engineer.** Cut, fill, flatten, tunnel, flood: by drone, the genre's signature feature here.
4. **Automation replaces your hands at every scale:** mining, then logistics, then construction.
5. **Every tier adds a new kind of problem, not just recipes:** quantity → power → distance → terrain → fluids → scale.

### The decisions that bind new work (all of them: `docs/DECISIONS.md`)

- **Needs before techs** (2026-10-16): new work gives the player a problem its tech solves. No more multiplier layers
  (compute, orbit) and no more perk layers. Depth in existing systems beats new systems.
- **Peaceful**, no combat or PvE; pressure comes without enemies (R8). **Open-ended**: the megaproject unlocks, never ends.
- **Co-op 2–4 players**, lockstep over WebRTC with a deterministic core. **Old saves keep loading.** New generation rules
  apply to new worlds only (a world keeps its generator version).
- **Content is data:** tiers index tables, processing machines are spec rows (`docs/TECH_TREE.md` section 8).
- **Art moves away from Minecraft:** natural blocks photo-referenced, machines stylized (`docs/ART_HANDOVER.md`).

---

## 2. Architecture in brief

Rust (`crates/engine`) owns all game state and hot loops; TypeScript (`web/src`) does input, WebGL2, DOM and Web Audio,
reading bulk data (meshes, instances, sounds, textures) zero-copy from wasm memory. `Game` (`lib.rs`) is the facade, its
JS API split by area into `api/*.rs`. The deterministic core is `Sim` (`sim.rs`): `Game` changes it only by queuing
`Action`s, applied at the next 60 Hz tick; the core answers with `SimEvent`s. `Sim::state_hash` fingerprints the core
through the same canonical bytes saves use (`bytes.rs`, `save.rs`). Co-op is lockstep: the host stamps every action for
`tick + INPUT_DELAY` and clients compare state hashes every `CHECKSUM_TICKS`; bodies and loose items are replicated, not
simulated. Budgets for 4 players: under ~10 KB/s per client outside snapshots, the host's frame under 2 ms more than solo.

### Measured (2026-10-19, `bench_*` and `scripts/bench-flight.mjs` on the user's 32-hour save)

- A 30,000-belt, 3,000-processor base: a steady tick 112 µs (median of 6; 180 µs the same day before the per-kind
  passes walked index lists, `process/kinds.rs`), but any belt placed relinks the whole factory (3.4 ms) and
  `write_instances` writes 1 M floats (0.5 ms) a frame. Towns plan every stage in 92 ms at once, one stage in 16 ms.
- Flying over the 32-hour base: the engine's share of a frame averages 5.5 ms, p99 13.4 ms (streaming and meshing),
  2 frames in 1,200 over 16.6 ms. Wasm 575 KB gzipped, JS 125 KB.

### Known limitations

1. Two tabs on the same world overwrite each other's saves (the later save wins).
2. TypeScript still mirrors a few engine constants (`web/src/CLAUDE.md` lists them); replace with getters when touched.
3. Item and belt instances aren't interpolated between ticks (only the camera is).
4. In co-op every action applies about 70 ms plus the round trip after it is made; a name change applies at the next
   session; hosting can't be stopped without leaving the page; the TURN relay is untested (the user adds its secrets).
5. The explored map, pins and blueprints live in one browser per world: co-op clients keep none, exports carry none.
6. Swimming and pumps treat every flowing-water cell as a whole block (the mesher draws them finer).

---

## 3. Engineering rules for all new work

### 3.1 Keep the codebase small and cheap to work on (TOP PRIORITY, mandatory)

Tokens go to reading code, exploring for where things live, failed build-test loops and re-learning the project each
session. Structure decides all four. **Treat context cost like frame time: a budget you measure and defend.** If a change
would break these rules, restructure first, in its own commit.

**Changes stay local.** One feature, one module or folder: new files plus at most one registration line per table. If one
feature edits five central files, fix the structure. `Game` stays a thin facade; `api/*.rs` methods only forward. Content
(blocks, items, recipes, machines, techs, ore settings) is data, code is per kind of behaviour. Variants are data, never
booleans or a block per tier. Simulation and presentation are separate modules: the core never reads presentation, hands
or the API (`scripts/check-map.mjs` checks it). Boring, explicit code: no macro tricks, no ECS, no abstraction before its
second instance.

**Budgets** (`scripts/check-size.mjs`, part of `npm run check`):

| Kind | Warn | Fail |
|---|---|---|
| Rust / TS source file, tests excluded | 400 lines | 600 lines |
| CSS file (one per UI component) | 300 lines | 500 lines |
| Any `CLAUDE.md` | 60 lines | 100 lines |
| A doc in `docs/` (not `history/`) | 24 KB | 40 KB |
| The session-start read: root `CLAUDE.md`, this plan, `CODEMAP.md` | 48 KB | 64 KB |

Docs are budgeted in bytes: a line budget only made lines longer. Split a file over its warn limit before adding to it.

**Readability.** Tests live in `src/<module>/tests.rs`, never inline. Every module starts with a header of at most ~15
lines (what it owns, invariants, how to extend), then public items, then private helpers. Unique, searchable names. One
source of truth: never mirror a constant between Rust and TS (expose a getter) or between code and docs (point to it).
Delete dead code at once. Comments say *why*.

**Docs are maps, not history.** `CODEMAP.md` is an index, one line per module; detail lives in module headers. This plan
holds only the current work; a finished milestone moves to `docs/history/`, the next comes in from `docs/ROADMAP.md`
detailed to steps with where, how and done-when. Operational how-tos live in `docs/WORKFLOW.md`.

**Feedback loops fast and quiet.** `npm run check` before every commit. Prefer headless Rust scenario tests (build through
actions, run ticks, assert on state or `describe` text) over the browser pane, which is for final visual proof. Quiet
flags, filtered output. Newtypes and exhaustive `match`es let the compiler list what a change must touch.

**Sessions.** One scoped step, finished green. Read line ranges and symbols, not whole files. Every milestone ends with a
cleanup step: budgets clean, code map current, plan compressed, dead code gone.

### 3.2 Performance

Hot loops and state stay in Rust; bulk data passes zero-copy; measure before and after anything that could cost frame
time (`bench_*`, `scripts/bench-flight.mjs`).

### 3.3 Wasm size

Report gzipped sizes when the build grows (`npx vite build`). The rules (no float formatting, no new std sorts, no new
crates without reason) are in `crates/engine/CLAUDE.md`.

### 3.4 Determinism (required for co-op)

The core (world edits, factory, deposits, inventories, crafting, research, towns) produces identical state on every
machine from the same seed and actions. Core code never depends on frame time, on which chunks are loaded (use the
`*_anywhere` accessors), on the camera, the local player, UI or presentation queries, on hash-map iteration order, on
wall-clock time or anything from JS. Queries never create or modify core state. Everything that changes the core is an
action applied at a tick boundary; presentation never feeds back. (A future native server will need `libm` on both sides.)

### 3.5 Style and docs to keep in sync

Match the surrounding code. Every engine feature gets tests in its module. Anything players see: the player's manual
(`docs/PLAYING.md`, `FACTORY.md`, `TOWNS.md`) and the patch notes. README.md is the pitch and setup only.

---

## 4. Now: the rest of the play-test response

The 2026-10-16 review and the user's 32-hour play-test became steps R1–R8 (`docs/history/CHANGELOG_R.md` has the
findings and every finished step). Rules that bind every step: content architecture per `docs/TECH_TREE.md` section 8;
each save-format change bumps `SAVE_VERSION` with a migration and a fixture test; golden hashes are re-recorded only on
purpose, noted in the step; new ids append (EXTENDING.md says where the next free one comes from); each new look gets a
placeholder layer and a `docs/ART_HANDOVER.md` request line; fluids other than water ride belts as canister items.

- [x] R1–R7 and S1–S4 (talks, rivers, plants, ambient sound, music, ruins, decor, sky and weather): built and pushed.
  Not built from them: the era as a music context (waits for an industrial or space piece), rain splashes, lightning,
  snow on machines, decor slabs and stairs (need non-cube meshing).
- [x] **R8 Pressure without combat (talk).** Decided 2026-10-22: none for now. The options (groundwater, pollution,
  storms, subsidence) stay in `docs/ROADMAP.md`; reopen only if the user asks.
- [x] **Pod channels by name (small).** A launcher and a pad pick one of six named colours in their panel
  (`factory/process/pods/channels.rs`, `ui/pod-channel.ts`), not by facing. Save version 54: the facing becomes the colour.
- [ ] **13.6b Balance from the user's next play-test** (`scripts/inspect-save.mjs`): order sizes, upkeep rates, perk
  rates and the harbour's price, once the user has played Milestone 13. Their latest save (2026-10-17, 32.4 h) has not
  yet researched Settlements.
- [x] **U1 Map hover and filters** (built 2026-10-22; do U1, U2, U3 in this order, one session each). `world_map_describe`
  answers up to eight marks, nearest first, so the host can skip filtered ones; every mark is a `Subject`
  (`minimap/marks.rs`), so drawing and describing cannot drift. U2's pins gain `y` in `ui/pins.ts` (`Pin`); filter
  category `pin:<kind id>` already covers them.
  Hovering a map or minimap mark shows an info box saying what it is; filter chips hide marks and pins by category.
  - *Where:* one engine query `world_map_describe(x, z, reach)` next to `minimap/marks.rs` (machines by block name, known
    deposits by ore and tier only, towns, sites, satellite cover, launch sites, pod channels by colour); never reveals
    what the player has not learned. New TS files `ui/map-info.ts` and `ui/map-filter.ts` (`worldmap.ts` is near its
    400-line budget: do not grow it). Filters: Pins (by kind), Ores (by ore), Machines, Towns, Sites, Satellite cover;
    kept in `localStorage`; applied to both the map and the corner minimap; the pin list follows the pin filter.
  - *Done when:* a test enumerates every mark shape and fails if one has no description; the query takes under 0.2 ms
    (a `bench_`); the filter logic is a pure function with `web/test/*.test.mjs` tests; seen in the browser pane;
    patch note and `docs/PLAYING.md` updated.
- [x] **U2 Telescope** (built 2026-10-22; needs U1's pin filter to exist). A hotbar item. Right-click toggles looking through it, the wheel
  zooms, left-click pins the spot under the crosshair. Looking through it shows every pin as a glowing column in its
  colour (reuse `factory::mark_beacon`, as `power_tools/long_pole.rs` does).
  - *Where:* a `ToolKind` row and item (`tools.rs`, as the scanner), a held-device module for the scope, the web
    renderer's `fovY` for zoom, an engine setter that TS calls when pins change. Pins gain an optional `y` (old pins
    keep working). A ray-march over the generator's heightfield (`worldgen/`) finds ground that is not loaded yet.
  - *Done when:* over 500 random rays the pinned height is within 1 block of the true surface on loaded ground and
    within 3 on unloaded ground (the test generates the chunk afterwards and compares); beacons show beyond 1,000
    blocks; a `bench_` shows the frame cost of 200 beacons; placeholder texture, `docs/ART_HANDOVER.md` line, patch note.
- [x] **U3 Sickle: clear ground plants** (built 2026-10-22; user: ground plants only, a sickle tool). Ground cover (`mesher/cover.rs`) is
  presentation drawn from a hash, so clearing needs saved state: a bitset of cleared columns per touched chunk, kept in
  the core so saves and co-op agree. `Action::ClearCover` (a codec tag), a 3×3 patch per click, the chunk remeshed, no
  drops, no regrowth. Save version bump with a migration and a fixture test; sickle item, recipe, placeholder texture
  and `docs/ART_HANDOVER.md` line.
  - *Done when:* untouched chunks mesh byte-identically to before; `bench_meshing` slows by under 2%; a two-player
    `net` test keeps equal state hashes after a clear; save and load keep cleared patches; wasm size change reported.
- **Parked, fit in where a session has room:** P3 cheaper box instances, P4 an incremental co-op checksum, P5 small TS
  savings (`docs/history/CHANGELOG_M12.md`); a power beam drawn from satellites to rectennas; incremental `relink` if
  big bases hitch on placing (measured above).
- **The user's to do:** add the two TURN secrets (`docs/WORKFLOW.md` section 4), then test with `&relay` on both ends.
- **Only if placing feels laggy in co-op:** show your own edits at once in the render cache, undo them if the core
  disagrees, never touch the core (`interaction.rs`). A real resync logged by a client becomes a replayable test.

**After these:** the feel-and-feedback interlude (`docs/ROADMAP.md`: the instrument with few voices and a distance cap,
follow one item, timelapse; then seismic prospecting and site bonuses), the carry's era pieces (splitter addresses and the
tally only), Milestone 14. Art goes on in parallel (`docs/ART_HANDOVER.md`).

---

## 5. Open questions for the user

Ask when the step that needs the answer comes up. Record answers in `docs/DECISIONS.md`.

| Needed by | Question |
|---|---|
| Any UI step | A colour-blind palette for tier colours (pips and "Mk" text are there regardless)? |
| A world-settings step | Should old worlds be able to opt into newer generation (rivers, trees, ruins)? Should far deposits seen once get a map pin? |
| Music | A sea piece; industrial and space pieces (with the era as a context). |

---

## 6. Recent changes

- **2026-10-24: click to keep a belt item selected** (user request). The mining button's press (`set_mining`, rising edge)
  selects the item under the crosshair or, over anything else, lets go (`item_select.rs`, `Game::selected_item`; presentation
  only, the core never sees it). Each frame `Factory::find_item` finds the item again by `ItemKey` (item, maker cell, birth
  tick = now minus ticks on belts, slack 2; of several fits, nearest the last sighting, for old saves' shared tags): one scan
  of the belts and routers, only while something is selected. To keep the birth tick steady a router now ages the tag of an item
  it holds (`Router::step`; the golden hash did not move). The readout shows the pointed-at item, else the selected one, and
  `I` takes the shown item unless it is selected and out of reach or held in a splitter (`target_item_state`: chip text in
  `ui/hud.ts`); an amber outline (`selected_item_box`, `FrameInfo.selectedItem`, `render/outlines.ts`) marks the selected one.
  No save-format change. Seen in the browser pane: select, look away, 5 s of belts followed, out of reach, taken. Not verified:
  a real click with pointer lock; holding the button still mines the belt as before.
- **2026-10-24: belt items have a story.** Pointing at an item on a belt names it, outlines it and says how long it has been on
  belts, which machine put it there and where, and the recipe if a machine made it (user request). Every `BeltItem` and a
  router's held item carry a `Tag` (`factory/provenance.rs`: ticks on belts, the maker's cell; 16 bytes), core state, so in
  saves (version 56; older items read as origin unknown) and the state hash. Producers (miner, quarry, box, processor) start a
  tag, `belt_step` ages it, `deliver` and routers pass it on, a machine that takes the item in drops it. No tick is threaded
  through the machines: the tag counts its own ticks. Picking: `factory/belt_items.rs` (`aim_boxes` adds the items to a
  belt's hit boxes so a tall item's top still selects the belt, `belt_item_under`, `belt_item_report`); `Game` keeps
  `target_item`; `target_item_name` and `target_detail` (`api/hud.rs`) feed `ui/hud.ts`; the outline is the item's box.
  Measured on the 34 h save (6,347 belts, 14,554 items, 4,339 belts backed up with 3): a whole tick 146 to 158 us (medians of
  35 interleaved runs, +8%, about 0.07% of a frame); a save 1,025,484 to 1,258,684 bytes raw, 243,688 to 270,898 gzipped
  (+11%; stuck items share one age here, so real play may add up to about twice that). The golden hash in `sim/tests.rs` was
  re-recorded. Wasm 1,630,288 to 1,639,380 bytes (+9.1 KB; `vite build` reports 592.98 KB gzipped). Seen in the browser pane
  (a smelter's ingot: name, 27 s on belts, "Made in a Smelter from ...", 2 west from here, outline). Not verified: the feel of
  aiming with real pointer lock. Not done: a tally per belt of how old its items are;
  following one item along its way (the roadmap's "follow one item" can now build on the tag).
- **2026-10-24: pick an item off a belt (key I)** (user request). `Action::TakeBeltItem { pos, item, at }` (codec tag 62): of the
  belt's items of that kind the one nearest progress `at` (255ths) goes to the player's pack, or stays when the pack is full
  (`Factory::take_belt_item`, `factory/belt_items.rs`; a tool keeps the uses that did not fit). The action names the item by kind
  and progress, not list index, because the list shifts while an action travels; on a fast belt in co-op the item can have
  moved on by then and nothing happens. No save-format change. `Game::take_target_item` (`api/hud.rs`) queues it for the item
  under the crosshair; `ui/hud.ts` shows an "I Pick up" chip under the readout (`#target-key`), `input.ts` binds the key, the
  controls page and `PLAYING.md` list it. A dedicated key, not right-click, because right-click places a held block against
  the belt. Seen in the browser pane (0 to 1 ingots); the real key press and pointer lock are not verified.
- **2026-10-22: sickle (U3).** Item 398 (`ToolKind::Sickle`, never wears; two iron ingots and two sticks, a hand recipe in
  `recipes/tooling.rs`). Right-click on natural ground (`mesher::bears_cover`) queues `Action::ClearCover` (codec tag 61),
  hold to sweep (0.25 s a swing; a patch already cleared sends nothing); anything else falls through, so panels still open
  (`sickle.rs`, with a green outline of the patch and a HUD label). The core: `world/cover_clear.rs`, one bit per world
  column, a 32 × 32 bitset (128 bytes) per touched chunk column in a sorted `Vec`, saved after the edited chunks (save
  version 55; older saves have none) and so in the state hash; the golden hash in `sim/tests.rs` was re-recorded for the
  four extra bytes. A 3 × 3 patch sets its bits whether or not the columns bore cover (the core cannot see light or
  biome); the bit stays when the ground is broken or replaced. The mesher gets the chunk column's rows in `Cover::cleared`
  and does one bit test per candidate column; loaded chunks with a cover cell in a newly cleared column remesh, the rest
  mesh bare when they load; a co-op resync redraws columns whose rows differ. No drops, no regrowth. Measured: 39 chunks
  around a seed-1337 spawn mesh byte for byte as before (hash recorded from the old mesher; `world/tests.rs`);
  `bench_meshing` median of 9 interleaved runs 0.602 ms a chunk before, 0.587 after (no slowdown); a scenario test clears
  a patch of the busiest 3 × 3 of a generated meadow, and across a chunk border, and compares every plant's words; a
  two-player `net` test keeps equal hashes. Wasm 1,620,447 → 1,630,288 bytes (+9.8 KB), gzipped 579,770 → 584,218
  (+4.4 KB, same `gzipSync` on the optimised builds). Seen in the browser pane (outline, cut, label). Not verified: real
  pointer lock, the feel of sweeping.
- **2026-10-22: telescope (U2).** Item 397 (`ToolKind::Telescope`, brass tube model, recipe from glass, plates and rods).
  Selected: right-click toggles the view (first person; eased 2-8x zoom, wheel steps it, never a jump; mouse look is divided
  by the zoom), left-click pins the ground under the crosshair as an empty-note Note (`ui/scope.ts`, `ui/pins.ts` gains
  `y`). Looking shows every pin as a flat-coloured light column (kind colour): `factory::mark_beacon_tinted` (glow flag
  3 + 12-bit colour, one `boxVert` line). Far plane: columns beyond 1.5 view distances are scaled about the eye to that
  distance, so the angle is kept (`telescope/beacons.rs`). Aim: `telescope/ground.rs`, an adaptive heightfield march
  (6,000 blocks) over loaded blocks then `height_at`. Measured: 500 rays on loaded ground, 499 within 1 block of their
  column's surface (500 within 3); 500 rays at unloaded ground (156-4,593 blocks, 228 beyond 1,000), 500 equal the
  generated surface, 499 within 3 of an exact block walk. Release bench: 200 beacons 78 us a frame (7 without pins; the
  aim alone mean 79 us, worst 1,081 us). Engine limit `MAX_PINS` 200 (`scope_pin_limit`). Save version unchanged (pins
  live in TS). Not verified: real pointer lock, feel and comfort. Pins are per browser, not shared in co-op.
- **2026-10-22: map hover and filters (U1).** Every map mark is now a `Subject` (`minimap/marks.rs`: shape, colour and,
  in `minimap/describe.rs`, words come from it), so `world_map_describe(x, z, reach)` (a query, nothing created) tells what
  the nearest marks are, only what the player has learned: a deposit by ore and tier, never its reserve. A test reads
  every `MARK_*` constant from the source and fails for one without words. Measured on 3,000 processors, 30,000 belts, 96
  satellites and 30 towns at a 224-block reach: 20 µs a query (61 µs worst; the budget was 200). TS: `ui/map-filter.ts`
  (pure rules and the saved set of hidden categories, `web/test/map-filter.test.mjs`), `map-filter-panel.ts` (chips beside
  the map), `map-info.ts` (the box, on the map and the minimap). Filters apply to both maps; the pin list follows.
  The minimap box works only while the cursor is free (menu or panel): pointer lock hides it in play.
- **2026-10-20: pod channels by name.** A pod launcher's or landing pad's channel is now one of the power-grid palette's
  six colours (`grid_colour::PALETTE`, `pods::CHANNELS`), set in its panel by `Action::SetChannel` (codec tag 60) and kept in
  `Extra::Channel`, apart from the facing. The 3D mark and the map marks wear the colour (no new texture layers). Save
  version 54 stores it; older saves read the facing as the channel (blue, orange, pink, green). Blueprints do not copy it.

- **2026-10-19: docs restructure.** The session-start read went from about 188 KB to under 48 KB: `CODEMAP.md` is an
  index (detail lives in module headers), decisions moved to `docs/DECISIONS.md`, finished work to `docs/history/`,
  stale sections dropped from this plan and `WORKFLOW.md`. `npm run check` gained byte budgets for docs and
  `scripts/check-map.mjs` (map paths exist, every module is mapped, the core reads no presentation). Then the 12 source
  files over 400 lines were split along their seams (table rows into set or era files, `frame.rs`, `factory/tick.rs`,
  `pause.ts`, overlays into `outlines.ts`), no behaviour change. Last, what only one kind of processor keeps (well,
  research unit, debt, launches, depot wants, drone fleet) moved from six `Processor` fields into one enum
  (`process/extra.rs`): 528 to 464 bytes, save bytes unchanged. That made the tick slower (136 to 148 µs) through
  layout alone: about ten passes walked every processor to find a few of one kind. They now walk index lists rebuilt
  with the links (`process/kinds.rs`), same order: the big base's tick went from 180 to 112 µs (median of 6, interleaved).
- **2026-10-18:** real 3D item models, the water rework (smooth surfaces, still, flowing, rapid and falling water),
  per-block texture turning, and the realistic art batches (records in `docs/history/CHANGELOG_R.md`).
