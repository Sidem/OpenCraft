# OpenCraft development plan

**Status (2026-10-19):** Milestones 1–12, Milestone 13 (Settlements, all but its balance step 13.6b) and the play-test
response R1–R7 plus S1–S4 (sky and weather) are built and pushed; the realistic art pass is live. R8 is decided
as "none for now" (2026-10-22) and pod channels by name are done (2026-10-20); 13.6b waits for the user to play
Milestone 13. Map hover and filters (U1), the telescope (U2) and the sickle for ground plants (U3) are done (2026-10-22), and
belt items tell where they came from and can be clicked to stay selected (2026-10-24). Tips stay done, and the
machine-stopped lesson is built (L1, 2026-10-24); the coal miner bridges to electricity and hand-crafted items can be taken apart, a torch in the hand lights the way and a left hand holds a second tool, and houses get doors, roof tiles and more, and a playtester's five notes are done (2026-10-25); the item guide answers how to get any item and the belt ejector drops a belt's items on the ground, with loose items merging into stacks (2026-10-26). Recorded sound effects (FX1–FX4) are held back on branch `sound-effects` (2026-10-28). Machines show what they make and wear their Mk as studs, and item icons fill their box (2026-10-28); the product turns on top of each machine, and round things are real cylinders and the long-distance pole a 2×2×4 transmission tower (2026-10-29); analytics' machines can be shown on the map (2026-10-29); round pipes, a round wire spool and two UI fixes, and 16 machine icons drawn after the real things (2026-10-30). **Next: the feel-and-feedback interlude**, then the carry's era pieces, Milestone 14 (the space elevator).

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
- [ ] **FX Sound effects from recordings** (decided 2026-10-27, `docs/DECISIONS.md`). One step per session, in order. Tools
  stay outside the game: earshot in a local venv (`pip install` of github.com/Leonxlnx/earshot plus CPU torch), source
  recordings and their licences in a git-ignored folder; only the rendered effects and a credits list are committed.
  **Held back (user, 2026-10-28: not happy with it yet):** FX1–FX4 live on the local branch `sound-effects`, not on
  `main`; the game still plays the synthesised effects. Bring them back from that branch when the user is ready.
  - [x] **FX1 Non-material sounds** (done 2026-10-27): earshot's top 3 bank sounds per action plus the current sound, blind.
    earshot won 4 of 11, never with its own first choice: drop `swish/zing`, scan `pro_notify/ding_2045`, launch
    `impact/blast`, pod up `whoosh_big/jet` (render with `bank.render(id, seed)`; v0 seed `None` is the take picked, v1 seed
    11; all clean in `sfx check`, 69 KB as 64 kbit/s Opus). The current sound won the other 7 (pickup, click, splash, power
    up/down, pod down, denied): FX4 renders those from the synth as they are. The v1 takes are not yet heard by the user.
  - [x] **FX2 Recordings** (done 2026-10-27): 154 Freesound searches (CC0, up to 12 s) found 891 recordings. Pairs like
    grass break or sand place found almost nothing, so the sources are pooled per material (steps also give land, hits
    give dig, break and place): the 8 most used footsteps and 8 hits naming the material, 110 in all, all CC0. HQ Ogg
    previews, the scripts and `credits.csv` (id, author, licence, link) are in git-ignored `artifacts/fx2/`; the key
    lives in `~/.freesound.env` (`FREESOUND_CLIENT_SECRET`) and is never printed. A few picks are off-topic (a roof
    slide under grass): the blind pick filters them.
  - [x] **FX3 Material foley** (2026-10-27, results in `artifacts/fx3/`): blind pick, a recording won 26 of 35 (the
    chance rate; earshot's first choice 12 of 26). Naming test: game sounds 19 of 35, held-out real recordings (the
    ceiling) only 7 of 13, so the bar moved to the ceiling. Re-test: 20 of 35, at the ceiling; sand 0 of 5 every time
    (real sand 0 of 2 too). *Then ear tests ended* (user): sounds are chosen by label, two per material (`docs/DECISIONS.md`). Built:
    140 files in `artifacts/fx3l/` (`credits.csv`, 482 KB as Opus); `sfx check` 113 clean, 14 ring (pickaxe and
    metal, the material itself), 13 footsteps cut from walking recordings still end above -45 dB.
  - [x] **FX4 In the game** (done 2026-10-27): `audio/effects.ts` plays files found by name (variants = files present,
    none = silent); `effect-levels.ts` levels each to the synth sound it replaced (K-weighted loudest 100 ms), so old
    volumes hold; the designer keeps one Volume dial; synth effects removed. 156 files, 742 KB (budget ~600; the user kept 742 KB). Fixed after
    the user's listen: footsteps are whole steps (the cuts ended mid-sound; sand and metal from single takes, the
    closest to their label's consensus spectrum), power up/down without crackle (128 kbit/s); `sfx check` flags only
    the stone and metal hits' ring.
  - *Done when:* every effect comes from a file (met); in the blind naming test the game sounds are named right at least as
    often as held-out real recordings (the ceiling) and no material under 2 of 5 (user, 2026-10-27: real recordings
    scored 7 of 13, so 11 of 14 was out of reach; not run on the label set, ear tests ended); `npm run check` passes;
    credits list (`audio/effects/CREDITS.md`) and patch note (met).
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

- **2026-10-30: a steel two-stage rocket, a capsule pod, exhaust with shock diamonds** (user: the rocket and pods did not
  look beautiful and the climb was jagged). `process/rocket.rs` draws the launch site's rocket as round parts (steel
  booster and ship, heat tiles, grid fins, flaps, seven bells) by a tower with catch arms; it climbs from the batch's
  progress blended into the tick at last tick's speed, so it no longer steps once a tick. Drop pods fly as a capsule
  (`factory/capsule.rs`) on four canted plumes; `factory/plume.rs` is the shared exhaust (glowing cells, a pale diamond
  at each pinch). New layers in `textures/flight.rs`. Tests: `launch/tests.rs` (cold, firing, an even rise between
  ticks), `plume/tests.rs`. Presentation only: no save or core change.

- **2026-10-30: kits upgrade underpasses to their own tier** (user: a green kit took an underpass to Mk4). The host
  sends `using` every frame, so a held click on a non-belt piece re-sent `Upgrade`, and the core took whatever kit the
  next step needed. `Action::Upgrade` now names the kit's tier (the codec grew a byte: co-op peers must match). Shift
  lines include underpass pairs (`belt_chain`), budgeted per piece (`Factory::upgrade_kits`, 2 kits an underpass).
  Tests: `belt_line/tests.rs` holds the button 30 frames with every kit in stock; `belt_chain/tests.rs`.

- **2026-10-30: machine icons after the real things** (user: 16 icons hard to recognise). 13 are built by hand in
  `item_models/machines.rs` + `machines_plant.rs` (a waisted cooling tower, a turbine rotor, a headframe, a winch and
  hook, a cable coil and plug, a horsehead pumpjack, a robot arm…); pipe, rail and fibre node draw their placed model
  (`placed.rs` via `factory::shape_of`). A tier's item shares its machine's icon. `icon-paint.ts` reset a transform a box
  leaked into the next round part (17 other icons misdrew one: gear, motor, servo, satellites…). Judge: 16 at 40 px, 3
  shuffles: cubes 2.7, placed models 5.3, final Opus 16/16/16; Sonnet 9.0, but it binds names to wrong letters in closed
  cycles and two retests matched. Test: no two outlines overlap over 80%, none a cube. 75 of 111 machines are cubes.

- **2026-10-30: round pipes, wire spool, two UI fixes** (user: four bugs). Pipes are cylinders ending at their faces (a
  straight one is one length, a junction a hub), with a flange drawn by the joint's low side only; joined arms had
  overlapped 0.2 block and flickered. `factory/tests/face_fight.rs` fails on two faces sharing a plane and a facing
  across all 2,304 pipework layouts (at HEAD it flagged the first joint; it also lowered the pump's fan out of a top
  flange). Not fixed, found by the same check: cables fight in 54 of 64 layouts, fibre in 48. Copper wire is a round
  spool. The map filter's chips had shared `.mf-chip` with the focus chip (now `fo-chip`); `scripts/check-css.mjs` fails
  on a class defined in two stylesheets. A filter's choice is in its panel's redraw key (`ui/machine.ts`).
- **2026-10-29: "Show on map" from analytics** (a playtester's idea). Every kind row and slowest row of the Machines tab
  opens the world map with each machine of that kind ringed by health (`analytics_machines_of`, `Machines::of_kind`;
  `ui/map-focus.ts` fits the view, `map-focus-layer.ts` draws rings, chip and hover lines; the pin list moved to
  `map-pin-list.ts` to keep `worldmap.ts` under budget). Presentation only. Test: every kind's list matches its row's
  counts and every ring sits on a map mark (`analytics/tests.rs`); `bench_machines_of_kind` 13 µs for 1,513 of 3,025
  machines. On the 13.5 h play-test save, Quarry ×4 (3 output full, 1 flooded) all fall inside the fitted view.
- **2026-10-29: real round bodies and a transmission tower** (user: canisters, wafer, silicon, fuel cell looked like crossed
  boxes; the long pole should look like a real tower). The box pipeline gained a cylinder pass (`render/round-mesh.ts`,
  `boxes.ts`; the instance's `ROUND` mode bit and taper, `factory::push_round`), icons paint round bodies and leaned boxes
  (`ui/icon-paint.ts`, `icon-geometry.ts`), and every faked round model moved to `Form::Round` (items, processors,
  pump, train). `factory/tests/fake_round.rs` fails on any model faking one (at HEAD it flagged 32 items, 12 machines).
  The long pole is 2×2×4 (`pole_tower.rs`, `Form::Lean` legs on the item); save 61 fills old towers' new top with machine
  parts where it is air (`raise_old_towers`; the play-test save: 14 filled, 1 under stone kept). Baked round shading was flattened on round bodies' textures. `write_instances` +2.4%, wasm +4 KB.
- **2026-10-29: product displays turn on top** (user: the sign faced one way and the product vanished from the side).
  Every `Look::Product` slot is now square and stands on the model's top (roof, tank lid, column top), and `display.rs`
  draws a light plate with the product standing on it (flat items stood on edge), turning at `SPIN` while working and
  holding its angle otherwise (a `Cell` on `Processor`, presentation only). Smelter chimney shortened, blast chimney
  moved back, a bar across the crusher's hopper. Tests: products fit the turning circle at every angle; each slot stands
  on a part; from 16 level directions at least two of five sight lines across it pass every part (a tall smelter
  chimney fails it).
