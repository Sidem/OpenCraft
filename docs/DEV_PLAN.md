# OpenCraft development plan

**Status (2026-11-14):** Milestones 1–12, Milestone 13 (Settlements, all but its balance step 13.6b), the play-test
response R1–R7, S1–S4, U1–U3 and trains (T) are built and pushed; R8 is decided "none for now", the realistic art pass
is live and graded toward the key art. Open: **Q3** (a cheaper menu), **L2–L6** (mini-lessons), **13.6b** (waits for the user to play Milestone 13) and **FX** (recorded sounds,
held back on branch `sound-effects`). 2026-11-13 also: three-way filters, glowing lamps, the hand in first person, wire
props over hills, barrels; 2026-11-14 realistic trees and forest biomes, generator 12, then trees felled whole and every wood its own (section 6). What changed since 2026-10-22 (items, machines such as the 2×3×3 pumpjack, belts, maps, trains, playtester
notes): `web/src/patch-notes.md` and `git log`. **Next: L2–L6, then the feel-and-feedback interlude**, the carry's era
pieces, Milestone 14 (the space elevator).

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
- **Peaceful**, no combat or PvE; no pressure system for now (R8, parked in the roadmap). **Open-ended**: the megaproject unlocks, never ends.
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
placeholder layer and a `docs/ART_HANDOVER.md` request line; fluids other than water ride belts as barrel items.

- [x] R1–R7 and S1–S4 (talks, rivers, plants, ambient sound, music, ruins, decor, sky and weather): built and pushed.
  Not built from them: the era as a music context (waits for an industrial or space piece), rain splashes, lightning,
  snow on machines, decor slabs and stairs (need non-cube meshing).
- [x] **R8 Pressure without combat (talk).** Decided 2026-10-22: none for now. The options (groundwater, pollution,
  storms, subsidence) stay in `docs/ROADMAP.md`; reopen only if the user asks.
- [x] **Pod channels by name (small).** A launcher and a pad pick one of six named colours in their panel
  (`factory/process/pods/channels.rs`, `ui/pod-channel.ts`), not by facing. Save version 54: the facing becomes the colour.
- [x] **TR Realistic trees (user, `/deepen`, 2026-11-14).** Generator 12 grows eleven kinds (`worldgen/grow/`) in
  five new forest biomes (`biome.rs` `local_biome_at`); leaf cards and branching log rings (`mesher/foliage.rs`,
  `mesher/logs/`). Open: blind judging of the looks, and the GPU cost (section 6).
- [x] **TF Felling whole trees, species wood (user, `/deepen`, 2026-11-14).** Grown trees fall whole (`sim/felling.rs`);
  ten woods' logs, planks, floors and doors, any wood in any recipe (`wood.rs`, `block/woods.rs`).
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
- [ ] **L1 Mini-lessons: when and where they show** (user request 2026-10-24). Built: the rules and lesson 1; L2 to L6 are
  one session each, in this order. Every lesson also gets a patch note.
  - *Evidence* (the user's 34 h and 3.9 h saves, `scripts/check-lessons.mjs`): readouts already say what to do ("Output full:
    put a belt leading away"), so a lesson makes the first sighting hard to miss and adds no facts. 60 of 98 working machines
    in the 34 h base were output-blocked, which is normal: teach a stall once per cause, never nag. That base has 0 blueprints
    and 0 earthworks sites: a lesson cannot create a need (needs before techs). No early-session data exists, so the 30 s and
    60 s timings are guesses for the user to tune in play.
  - *Rules (every lesson):* (1) shown only for a need standing near the player now, never on a timeline; (2) one card at a
    time in `ui/hints.ts` order: hand-mining warning, blueprint prompt, opening, lesson, side tip, main tip, with
    `QUIET_TICKS` after each; (3) done is a bit of `Sim::taught` (`lessons.rs`; saved, world-wide, set by doing the job or by
    H via `Action::LessonTaught`, never cleared; older saves read their stall lessons as taught); (4) one escalation: the
    card, then a prompt on the object (`key-prompt.ts`), no camera effects; (5) a lesson whose job is already done when it
    would start never shows; (6) a headless test completes it, and `lessons/tests.rs` fuzzes the latches.
  - *L1 built:* the machine-stopped lesson (`stall_lesson.rs`). A machine within 24 blocks, stalled 30 s for a cause not yet
    taught (power, input, output, fuel, water, deposit, blocked) brings a card with the cause and the fix and a "Check" prompt
    on it; it works again, or H, and the cause is taught. The opening scene's machines are left alone. Side tips now stay
    done too (bits 16 up).
  - *Need lessons built (2026-11-06, `need_lesson.rs`):* a waiting or full machine's readout and card name the item and
    where it comes from or what takes it (`item_guide::source_line`, `use_line`: sand from a beach quarry or a crusher, slag
    into sand); a machine short of power 30 s brings a card naming [P] (bit 7; taught by full power, opening P, or H); a
    side tip on slag into sand after Ore Crushing. Save 68 marks them taught in older worlds. The cold-player benchmark
    (`need_lesson/tests.rs` `benchmark_screens`, Haiku agents shown only the screen) went from 5/17 to 17/19.
  - *First steps guided (2026-11-07, `first_steps.rs`, after the user's play-test):* opening steps CraftIron ("another
    coal miner, for iron") and BoxCoal (the line's first box set beside the lit coal miner, outlined); a goal flag on
    both maps (`MARK_GOAL`); the build menu makes the asked-for recipe and its tab glow (`guide_craft`); Go electric
    points to the starter copper patch (prompt, flag, bearing) until a miner digs copper. All presentation, not saved.
  - *Next:* **L2 No recipe** (an assembler with no recipe chosen: its sample is `None` today; add a bit and a prompt "Open").
    **L3 Blueprint walk-through** (once the player accepts the repeat prompt: prompts on the two corners; done: a blueprint
    saved). **L4 Planner** (planner crafted, no site; card only; done: first site). **L5 Upgrade with kits** (a bottleneck
    machine pointed at while kits are held; the 34 h save's 315 green kits sit beside 13 output-blocked Mk1 miners, so first
    check an upgrade would help). **L6 Map and pins** (first 150 blocks out from spawn).
  - *Done when (each):* its bot test passes; `node scripts/check-lessons.mjs <saves>` stays quiet; seen in the browser pane.
- [ ] **FX Sound effects from recordings** (decided 2026-10-27, `docs/DECISIONS.md`). FX1–FX4 are built and **held back** on the
  local branch `sound-effects` (user, 2026-10-28: not happy with it yet); `main` plays the synthesised effects. The steps,
  findings and tools: `docs/history/CHANGELOG_R.md` (moved 2026-10-30). Bring them back when the user is ready.
- [x] **T Trains people build and enjoy** (user, 2026-10-30, via `/deepen`; done 2026-10-31): track on the ground, docks
  as pieces of track (onto a node or laid in line), a modern diesel easing into stops, cars you aim at and open, the
  journey bot (`tests/train_journey.rs`). Steps, targets and results: `docs/history/CHANGELOG_R.md`.
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

- [x] **Q Performance check** (user, 2026-11-08, after a player's lag report): detection, a report to paste, quality settings, automatic
  quality. Measurements: `docs/history/CHANGELOG_R.md`. The reporting player has yet to try it.
  - [ ] **Q3 A cheaper menu** (the user left it out for now). The launcher costs a whole play frame: not drawing the world behind it took
    software Chrome from 0.6 to 60 fps; removing the blur gains only 5–10%. Skip `renderer.render` in `main.ts` `frame` while the menu is
    open and not `menu.previewing`; `advance` keeps running. *Done when:* software Chrome's launcher p95 is under 33 ms, settings still
    preview, the picture behind doesn't go blank.

## 5. Open questions for the user

Ask when the step that needs the answer comes up. Record answers in `docs/DECISIONS.md`.

| Needed by | Question |
|---|---|
| Any UI step | A colour-blind palette for tier colours (pips and "Mk" text are there regardless)? |
| A world-settings step | Should old worlds be able to opt into newer generation (rivers, trees, ruins)? Should far deposits seen once get a map pin? |
| Music | A sea piece; industrial and space pieces (with the era as a context). |

---

## 6. Recent changes

- **2026-11-14: tree lag spikes (user).** A sapling's growth remeshed per block (`set_block_anywhere`: 214 remeshes
  for one young oak, 0.4–1.3 s native); it now uses `set_block_anywhere_later` like felling and the forester. A fall
  checked the leaves round every removed log (`LEAF_CHECK_RADIUS` cube each, 3–15 ms); `Sim::logs_removed` now checks
  once over the tree's box, and `tree::take` stops reading at the first layer with no wood or leaves. Native, host
  frame with its 6 ms work budget (`tests/tree_spikes.rs` `bench_tree_spikes`): growth tick about 0.1 ms, the fall's
  frame 7–13 ms (was up to 65), the floor being one forest-chunk remesh (3–6 ms, `bench_forest_mesh`). Guarded by
  `growing_and_felling_leave_remeshing_to_the_budget`. Not measured in wasm.
- **2026-11-14: trees felled whole, and every wood its own (user, `/deepen`).** Grown trees' wood (`is_tree_wood`;
  grown oaks now `OAK_TREE_LOG`) never mines a block at a time: swings at the trunk's lowest `LOW` layers add work, a
  mark on the bark counts whole and moves, elsewhere half (`sim/felling.rs`, like the game Rust); the tree leaves the
  world at once, the host draws it tipping over (`render/falling.ts`), and after `FALL_TICKS` its logs land along the
  fall line. Placed logs and old worlds' trees still mine block by block. Ten woods (`wood.rs`): own logs (bamboo
  drops bamboo), planks, floors and doors in their colour (`block/woods.rs`, `tools/art/woods.py`); any wood serves
  where a recipe names the oak's, and the output keeps the wood used. Look fixes, each from a failing test: tree wood
  lets light through (black feet, the pine whorl), wood no longer digs a pit below the foot, runs of single logs bend
  smoothly (palm stems), living limb ends taper whichever way they point. Measured: 38 of 43 kind × tool runs fell
  within ±15% of mining the trunk (mean +5.5%); the rest are small trees with an axe, rounded up to a whole swing
  (worst +25%: 1.5 s against 1.2 s). Off the mark about twice as long; all logs land within 1.5 blocks of the line
  (tests). Wasm +18.3 KB gzipped against 2d32cf2. Save 73. Pins re-recorded: the scripted run (only the felling's
  bytes, checked), the mesh pin, generator 12 (not yet released). Gap: a craft or batch loaded mid-way forgets which
  wood it took and makes the oak's.
- **2026-11-14: realistic trees and forest biomes (user, `/deepen`; generator 12).** Trees grow from a species row
  (`worldgen/grow/kinds.rs`: oak, broadleaf, birch, pine, cherry, palm, baobab, acacia, redwood, eucalyptus, bamboo)
  as a trunk, drooping limbs and leaf clusters, built in their own box (`grow/mod.rs` `Plot`), so saplings
  (`sim/saplings.rs`, `grow_young`) match generation. Invariants tested in `grow/tests.rs`: no leaf within a block and a
  half of a trunk below its crown, none that would decay. `biome_at` keeps the five broad biomes (ores, towns, sound,
  weather); `local_biome_at` refines them for trees and ground cover only. Leaves are cards (`mesher/foliage/`), logs
  rings that follow limbs (`mesher/logs/`), bark and leaf layers from `tools/art/trees.py`. Measured against HEAD
  509ed60: wasm +18 KB gzipped; native forest-chunk meshing 1.99 → 2.07 ms; generation 1.66 → 2.48 ms a column;
  GPU frame (RTX 3060, forest save, `scripts/frame-bench.mjs`) p50 1.8 → 3.9 ms and p99 5.5 → 8.9 ms, over the
  +10% target: about 2.5× the shaped quads. Next if it matters: distance LOD for cards and limbs. Pins re-recorded:
  `world/tests.rs` mesh, `worldgen/tests.rs` version 12.

- **2026-11-14: lifts down, cliffs in drags, hover range (user).** `Shape::Drop` (appended, no save bump): a lift
  column fed over its top with an unfed foot carries items down (`belt_shape::derive_lift_ways`, run before ramps so
  rederiving is idempotent). A dragged line climbs cliffs over a block high on lifts and goes down them towards a lower
  end, while lifts in hand last (`belt_line/path.rs` `MAX_LIFT`). A pole charges a hover pack as far as it wires to its
  own kind (`Factory::charge_poles`, `pole::span`). Tests: `factory/tests/lift_ways.rs`, `belt_line/tests.rs`.

- **2026-11-13: five user suggestions (save 72).** Canisters are barrels everywhere (search still finds "canister",
  `ui/craft-terms.ts`). Filters sort three ways (`factory/filter_rules.rs`: picks per output, one output takes
  everything else; `FilterPick` / `FilterRest`, tags 73–74, tag 4 reads as a straight pick; old filters load their
  item straight on, the rest shared by both sides) with a panel of three direction cards (`ui/filter-panel.ts`).
  Lamps glow: `halos.rs` + `world/glows.rs` (render cache) feed an additive pass (`render/halos.ts`), and glowing
  texels are brighter. First person shows the hand and held item (`avatars/model.rs` `push_view_hand`; comfort
  setting "Hand in first person"). Tools are held at their grip and make their own stroke (`avatars/tool_hold.rs`:
  chop, dig, sweep; the avatar's arm follows it); avatars now hold the selected item in the right hand. Power wires that would cut through a hill get free props (`wire_props.rs`,
  presentation only, cached and rechecked). Golden hash re-recorded (`sim/tests.rs` note).
- **2026-11-13: the personal drone reworked (user request, save 71).** Orders instead of one errand
  (`helpers/drone/`: Fetch, Store, PutAway, Craft; a queue of `MAX_JOBS`; `Action::Drone`, tag 72; tag 38 reads as a
  fetch). A trip tours the nearest stops (`trip.rs`) through boxes and machine outputs (`factory/stock.rs`) within
  `RANGE` 64. Craft gathers what `crafting::fetches_for` lacks, then hand-crafts. Restock and HUD words in
  `drone_hands.rs`, the panel `ui/drone.ts` (Y; Shift+Y the old fetch). Flight `companion_view.rs`: spring follow, lean
  from thrust, eased legs. On the user's save the reach rise from 32 to 64 took the base centre from 56 to 85 boxes;
  put-away would clear 22 kinds (47 stacks) there. Fixture `helpers/drone/v70.ocworld`; golden hash re-recorded. Then
  (user notes): fetches and stores tie to a pack slot (`ANY_SLOT` for none); boxes on a belt line are never filled and
  empty labelled ones only by hand, except the `box_label::MISC` catch-all, filled last and mixed (`stock.rs` roles); slots are picked in the inventory (`ui/drone-slots.ts`) and wear
  badges from `drone_slots` there and on the HUD hotbar.

- **2026-11-12: a real pumpjack (playtester note).** 2×3×3 (`pump::FOOTPRINT`: empties in at the back, barrels out
  at the front), drawn by `pump/view.rs` (`Look::Jack`): motor, gearbox, crank arms with counterweights, pitman arms,
  samson post, walking beam, horsehead, bridle, polished rod and wellhead. The beam follows the crank through the real
  four-bar linkage (`view::pose`); the crank (`Pump::crank`, derived, not saved) eases to the power share and stops in
  place. The well is sought under every base cell. Old saves: none of the user's 5 pumpjacks (6 saves) had room to grow
  (each in a straight belt line), so a pumpjack from before save 70 loads `compact`: one cell, every side, drawn at a
  third (`Processor::footprint`). Fixture `pump/v69.ocworld` written by the commit before.
- **2026-11-12: ports that read, box labels (playtester notes).** A held multi-block machine draws a see-through ghost
  and, per port face, a belt stub, arrows and the item riding it (`factory/port_marks.rs`, `port_view.rs`; marks are a
  solid pass after the ghost). Judged blind from screenshots (`scripts/port-shots.mjs`): 44.8% of sides right with the
  hatches alone, 96.1% with the ghost (target 95%; every miss a near stub out of frame). A save scan found belts on dead
  faces rare (0–1 a save) and 83 of 89 filled boxes holding one kind, so a box shows its most common item unless a
  label is picked (`factory/box_label.rs`, `Action::LabelBox`, save version 69). Second round: built machines show a
  floor pad at each open port (`Factory::write_port_pads`): 75.3% of sides right blind from 2 blocks (far 29%: the
  machine hides the far pad). Signs over the roof for hidden pads read blind (95.0% vs 46.0% from 7.5 blocks,
  `port-shots.mjs … placed 7.5`) but the user found them distracting in play and they were removed. Box labels moved
  onto the lid, facing the camera and drawn as far as the box; the label picker is a searchable dropdown; heaps ride belts `belt::HEAP_BOOST` bigger.

- **2026-11-09: bigger belt items.** Items ride at `belt::ITEM_SIZE` (just under `ITEM_SPACING`), barrels `BARREL_BOOST` bigger
  (`belt::ride_unit`, `item::is_barrel`). Drawing only: `END_STOP` is fixed, so belt capacity and the state hash are unchanged.

- **2026-11-08: performance check.** The launcher warns when the browser draws without the graphics card (`render/gpu-probe.ts`), settings has a
  report to copy (`ui/perf-report.ts`), Picture quality settings with a live render distance (`render/quality.ts`, `World::set_view_radius`),
  and Auto lowers sharpness when frames are slow (`render/auto-quality.ts`). Measured in Chrome, Edge and Firefox (`scripts/frame-bench.mjs`);
  results in `docs/history/CHANGELOG_R.md`.
- **2026-11-07: round logs everywhere (user).** Log items are a lying cylinder (`item_models/stock.rs`), so loose, belt,
  held and icon logs match the trunks. The forester's lifted crown and sapling are round (`forester/crane.rs` `crown`),
  and a bucked stem slides whole through the head with world-scale bark (`render.rs` `Framed`, the round side's scroll
  in `box-glsl.ts`), losing a whole log at each cut instead of shrinking. Test: `a_bucked_stem_slides_whole_…`.
- **2026-11-07: round trees (user, `/deepen`).** Logs and leaves leave the cube mesher for a per-chunk shaped list
  (`mesher/shaped.rs`, drawn by `render/shaped.ts` in two passes of their own): logs are 12-sided cylinders
  (`logs.rs`), leaves one surface-nets shell over the leaves within `foliage::REACH`, Taubin-smoothed, with
  world-anchored lumps and three layers (`foliage.rs`, `foliage/shell.rs`). Leaf and log edits remesh the chunks within
  that reach. Tests: crack-free across 8 chunks, the same bits whatever was meshed before, caps only where an end
  shows. `bench_forest_mesh` (seed 1337): canopy surface facing an axis 14.7% (cubes 100%), trunk sides 0%; quads per
  tree chunk 1.21×, vertex bytes 2.2×, mesh time 1.28×. Flight bench on the user's save: engine frame average 4.70 ms
  (4.66 without), p99 11.4 (12.8). Wasm +20.2 KB raw, +9.8 KB gzip (the surface nets; vertex packing kept out of
  line). User's blind A/B: "looks pretty good".
- **2026-11-07: the forester (user).** Block 159, a processor with `Pick::Forest` and `Extra::Forest`
  (`factory/process/forester.rs` + `forester/`), unlocked by Forestry (tech 86: red packs, no prerequisites). It works an
  11×11 patch in front of it with a 3×3 grid of plots: looks for a tree, swings its crane, strips the crown top-down
  (one sapling per 25 leaves, deterministic), fells the trunk top-down into its output, replants, rests when a round
  finds nothing. Placing needs flat grass or dirt over the whole patch (`forester/patch.rs` `survey`; refusal in
  `put_block`, red preview in `forester_preview.rs`); Mk2/Mk3 (items 399, 400, kits like the quarry's, rows in
  `refit::REFIT_LATER` at the end of `MACHINE_RECIPES`) work 2× and 3× as fast on 14×17 and 17×23 patches as far as the
  ground is flat; a knuckle-boom crane with a harvester head grips, saws, fells, bucks, hauls to the bunk and replants
  (`work.rs`, `crane.rs`, `scene.rs`, `render::push_framed`); felling takes the trunk and only the leaves no other log
  holds (`fell.rs`). Measured (3 seeds × 75 min after 15 min warm-up): Mk1 6.06, Mk2 12.13, Mk3 18.07 logs a minute,
  CV ≤ 0.085, ≈200 kJ a log at every tier, no plot bare for want of saplings. Golden hash re-recorded (one more
  research entry).
- **2026-11-07: stray gold boxes, ghost mode later, stone tips (user).** The "random ghost" was the
  blueprint prompt's preset box (`blueprint/repeat.rs`, a third copy of a machine: the opening's two coal miners make
  a third likely), which `blueprint_boxes` drew outside ghost mode until B twice cleared it; it now draws only in ghost
  mode, and B also drops a half-dragged belt line. Ghost mode, holding blueprints and the prompt need
  `Feature::GhostMode` (Construction Drones; `docs/DECISIONS.md`). Tips: "Go electric" also says to dig down for stone;
  the quarry tip moved right after it, about stone; the drones tip now comes before the ghost-mode one, done when a
  port holds drones.
- **2026-11-06: teach at the moment of need (user, `/deepen`).** An audit found [P] taught nowhere, the scanner only in
  the item guide, the quarry tip silent on sand, and slag into sand only in a research blurb; "Waiting for 1 Quicklime,
  2 Sand, 2 Stone" read as all three missing. Now a stuck machine names the missing item and its source, the full one
  what its output is for, low power names [P], and Ore Crushing brings a slag tip (L1 above). Benchmark: every scene of
  the user's list passes on both agents; the one still failing is finding a deposit again (the map: L6).

Older entries: `docs/history/CHANGELOG_R.md`.
