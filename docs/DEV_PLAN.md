# OpenCraft development plan

**Status (2026-10-19):** Milestones 1–12, Milestone 13 (Settlements, all but its balance step 13.6b) and the play-test
response R1–R7 plus S1–S4 (sky and weather) are built and pushed; the realistic art pass is live. R8 is decided
as "none for now" (2026-10-22) and pod channels by name are done (2026-10-20); 13.6b waits for the user to play
Milestone 13. Map hover and filters (U1), the telescope (U2) and the sickle for ground plants (U3) are done (2026-10-22), and
belt items tell where they came from and can be clicked to stay selected (2026-10-24). Tips stay done, and the
machine-stopped lesson is built (L1, 2026-10-24); the coal miner bridges to electricity and hand-crafted items can be taken apart, a torch in the hand lights the way and a left hand holds a second tool, and houses get doors, roof tiles and more, and a playtester's five notes are done (2026-10-25); the item guide answers how to get any item and the belt ejector drops a belt's items on the ground, with loose items merging into stacks (2026-10-26). Recorded sound effects (FX1–FX4) are held back on branch `sound-effects` (2026-10-28). Machines show what they make and wear their Mk as studs, and item icons fill their box (2026-10-28); the product turns on top of each machine, and round things are real cylinders and the long-distance pole a 2×2×4 transmission tower (2026-10-29); analytics' machines can be shown on the map (2026-10-29); round pipes, a round wire spool and two UI fixes, and 16 machine icons drawn after the real things, and blueprint keys that do what the label says and library cards with a picture and contents, and rates on machines, belts and the item guide (2026-10-30); a playtester's six notes: bendable belts with curved corners, sort by kind, power per grid, note signs, Mk names on the map and worked-out deposits marked (2026-10-31); trains: track lies on the ground, docks are pieces of track, a modern diesel that eases into its stops (T1–T3, 2026-10-31). **Next: T4 and T5, then the feel-and-feedback interlude**, then the carry's era pieces, Milestone 14 (the space elevator).

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
  - *Next:* **L2 No recipe** (an assembler with no recipe chosen: its sample is `None` today; add a bit and a prompt "Open").
    **L3 Blueprint walk-through** (once the player accepts the repeat prompt: prompts on the two corners; done: a blueprint
    saved). **L4 Planner** (planner crafted, no site; card only; done: first site). **L5 Upgrade with kits** (a bottleneck
    machine pointed at while kits are held; the 34 h save's 315 green kits sit beside 13 output-blocked Mk1 miners, so first
    check an upgrade would help). **L6 Map and pins** (first 150 blocks out from spawn).
  - *Done when (each):* its bot test passes; `node scripts/check-lessons.mjs <saves>` stays quiet; seen in the browser pane.
- [ ] **FX Sound effects from recordings** (decided 2026-10-27, `docs/DECISIONS.md`). FX1–FX4 are built and **held back** on the
  local branch `sound-effects` (user, 2026-10-28: not happy with it yet); `main` plays the synthesised effects. The steps,
  findings and tools: `docs/history/CHANGELOG_R.md` (moved 2026-10-30). Bring them back when the user is ready.
- [ ] **T Trains people build and enjoy** (user, 2026-10-30, via `/deepen`; one commit per step, in order). Evidence: the
  36 h save researched Rails, Trains and Freight and built none; rails float, docks stand beside the track, cars can't be
  aimed at, the schedule is built by a hidden click. Decided: **strict ground** (track goes only where ground is under it;
  the player fills or cuts), docks only on straight north-south or east-west track. Not: fuel, signal types, wait
  conditions, rail tiers, riding, research, sound files.
  - [x] **T1 Grounded track** (2026-10-30): track only where ground is under it; `factory/rail/ground.rs`.
  - [x] **T2 Docks are track** (2026-10-31): a dock is a piece of straight rail; `factory/rail/pieces.rs`.
  - [x] **T3 A modern train** (2026-10-31)**.** Streamlined freight locomotive (raked windscreen, lamps, roof gear, two three-axle
    bogies), wagons to match, turning wheels, a gauge the cars fit, easing into and out of stops (core, save bump).
    *Done when:* `fake_round` and `face_fight` pass; a blind judge prefers it in 5 of 6; `bench_trains` within ±5%.
  - [ ] **T4 Trains you can read.** Aim at a car: its fill (items, slots of `CAR_SLOTS`); use it: a train panel (each
    car's bar and contents, take and put like a box, state, schedule edit). Open wagons show a heap of their main item
    as high as they are full. *Done when:* 48 rays pick the right car; readouts equal the cargo; panel seen in the pane.
  - [ ] **T5 The journey.** A bot doing only what the labels say builds a two-dock line on uneven ground: no dead ends,
    ≥ 30% fewer clicks than HEAD. Instances for 20 trains × 6 wagons within +10%, wasm ≤ +15 KB gz; manual, patch notes.
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

- **2026-10-31: a modern train (T3).** `trains/model.rs`: a 37-box diesel (hood, raked windscreen with `Form::Lean`,
  headlamps, roof fans, two three-axle bogies, handrails) and 17-box wagons in one livery (`textures/transport.rs`,
  art pack `loco_body` in `tools/art/later.py`); wheels are spoked cylinders turned by the train's `odo`.
  `trains/motion.rs`: trains speed up by `ACCEL` and brake by `BRAKE` to stop exactly where they halt (at a dock,
  the first wagon over it: `STOP_PAST`); `halts` caches the edge's verdict, so `bench_trains` stayed within +2.4%.
  Save 65 adds each train's speed and distance. A blind judge (6 agents, 3 views × both orders) preferred it 6 of 6;
  all six called it dark on dark ground, so the blue was lifted and the exhaust made smaller after the vote.

Older entries: `docs/history/CHANGELOG_R.md`.
