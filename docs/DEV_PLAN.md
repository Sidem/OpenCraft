# OpenCraft development plan

**Status (2026-11-17, v0.8.8):** Milestones 1–13 (all but the balance step 13.6b), the play-test response R1–R8, S1–S4, U1–U3,
trains, the performance check, realistic trees (generator 12) and whole-tree felling are built and pushed. Open: **L3, L4, L6**
(mini-lessons; L2 and L5 done), **FIRE** (planned), **13.6b** (waits for the user to play Milestone 13). **Build in the order of
`docs/ROADMAP_ORDER.md`** (rated by impact and sessions, audited against the code and the user's saves 2026-11-16): the
(chemistry design CHEM0 and doc compaction and lessons L2 and L5 done 2026-11-16, MERGE and LOOK 2026-11-17), WebP atlas, the rest of the cheap wins (F-BEAT, the factory as an instrument, is parked in the backlog of `ROADMAP_ORDER.md`, 2026-11-16),
then **ROOM** (block ids become `u16`; only 40 cube layers are free); fire in the third wave; Milestone 14 (M14A, then M14B)
stays last (user, 2026-11-16). Dropped 2026-11-14: recorded sound effects, the
cheaper menu, skipping boxes behind the camera, the carry. What changed lately for players: `web/src/patch-notes.md` and `git log`.

> **This project is written entirely by AI coding agents.** Every session starts cold and every line read costs tokens.
> **Keeping the codebase small, modular and cheap to read is as important as any feature** (section 3.1, mandatory).

This is the plan of record: direction, rules, and what to build next. Settled decisions: `docs/DECISIONS.md`. Player's
manual: `docs/PLAYING.md`, `FACTORY.md`, `TOWNS.md`. Future milestones: `docs/ROADMAP.md`; tech tree: `docs/TECH_TREE.md`
(its section 8 is the content architecture) and `docs/TECH_ERAS.md` (each era's recipes). Read those only when planning.
Finished work and old change-log entries: `git log`; the removed `docs/history/` and the longer plan are at commit
`025edaf` (`git show 025edaf:docs/history/CHANGELOG_R.md`).

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
   `web/src/patch-notes.md` (newest `## date · vX.Y.Z · title` first); a new note bumps the version in `package.json` and
   `crates/engine/Cargo.toml` (+0.0.1 small, +0.1.0 a major addition; `docs/DECISIONS.md` "Version numbers").
   Structural change: `docs/CODEMAP.md` in the same commit.

Ground rules (details in `docs/WORKFLOW.md`): **never push to `main` without asking** (a push deploys the live site);
commit only when asked; PowerShell and the file tools on this Windows machine (the Bash tool fails); performance first,
dependencies justified; explain things to the user in plain language.

---

## 1. Where the project is heading

A browser game between **Minecraft** and **Satisfactory**: a fully editable voxel world that you mine, shape and
industrialise. Not a Minecraft clone; its conventions can be broken freely.

**Pillars.** (1) Everything is physical: items travel on belts, machines are blocks, the world is the material.
(2) Finite ground pushes you outward: deposits run dry; distance, depth and scale become the problems. (3) Terrain is
something you engineer: cut, fill, flatten, tunnel, flood, by drone. (4) Automation replaces your hands at every scale:
mining, then logistics, then construction. (5) Every tier adds a new kind of problem, not just recipes: quantity → power →
distance → terrain → fluids → scale.

**The decisions that bind new work** (all of them: `docs/DECISIONS.md`):

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

### Measured (2026-11-14, the user's 37.6 h save `artifacts/bench/factory.ocworld`: 6,813 belts, 102 machines working)

Ryzen 7 5800X, RTX 3060, Chrome headless 1904×985 uncapped, `node scripts/frame-bench.mjs <dist> chrome gpu <save>` with
`WARM=20000`, 8 s of `playing`. Heap: 12 MB JS after the first collection (150 MB while the atlas decodes), 63 MB wasm memory.

| | p50 | p95 | p99 | worst | frames in 8 s |
|---|---|---|---|---|---|
| before the pump fix (`025edaf`) | 4.1 ms | 9.9–10.2 | 11.7–12.4 | 17.2–17.7 | ~1,580 |
| after (section 6) | 3.5–3.9 ms | 4.5–4.8 | 5.2–5.3 | 11.9–14.7 | 2,040–2,230 |

- Steady scene: 451 draw calls, 211k quads, 186 of 435 chunk meshes in view. Headless engine, standing still at the base
  (`bench-flight.mjs <save> 60 0 8 0 still`): `update` 2.6 ms, no streaming work, 0 mesh events (it was 3.8 ms of work,
  43 meshes a second, worst frame 18.7 ms). Profile of a steady frame (`scripts/prof-top.mjs`): wasm 77%, JS 6%, GC 0.2%;
  `Game::update` self time 26%, `World::light_at` 9%, belt posing and models 14%, `bufferSubData` 5%.
- Download before the menu: JS 534 KB (175 KB gzipped), CSS 88 KB (18), wasm 2.24 MB (813), `art/pack.png` 4.52 MB and its
  glow map 42 KB; music (9.3 MB) loads later. Software rendering (SwiftShader) at this size draws 0.8 frames a second:
  fill-bound, not a target.
- A 30,000-belt, 3,000-processor base (2026-10-19, `bench_*`): a steady tick 112 µs; any belt placed relinks the whole
  factory (3.4 ms); `write_instances` writes 1 M floats (0.5 ms) a frame. Towns plan every stage in 92 ms at once.

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

- **Changes stay local.** One feature, one module or folder: new files plus at most one registration line per table. If
  one feature edits five central files, fix the structure. `Game` stays a thin facade; `api/*.rs` methods only forward.
  Content (blocks, items, recipes, machines, techs, ore settings) is data, code is per kind of behaviour; variants are
  data, never booleans or a block per tier. Simulation and presentation are separate modules: the core never reads
  presentation, hands or the API (`scripts/check-map.mjs` checks it). Boring, explicit code: no macro tricks, no ECS, no
  abstraction before its second instance.
- **Budgets** live in `scripts/check-size.mjs` (`RULES`, `STARTUP`), run by `npm run check`: lines for source and CSS and
  `CLAUDE.md`, bytes for docs and for the session-start read (root `CLAUDE.md`, this plan, `CODEMAP.md`). Split a file over
  its warn limit before adding to it.
- **Readability.** Tests live in `src/<module>/tests.rs`, never inline. Every module starts with a header of at most ~15
  lines (what it owns, invariants, how to extend), then public items, then private helpers. Unique, searchable names. One
  source of truth: never mirror a constant between Rust and TS (expose a getter) or between code and docs (point to it).
  Delete dead code at once. Comments say *why*.
- **Docs are maps, not history.** `CODEMAP.md` is an index, one line per module; detail lives in module headers. This plan
  holds the current work and the last few change-log entries; finished work is `git log`. Operational how-tos:
  `docs/WORKFLOW.md`.
- **Feedback loops fast and quiet.** `npm run check` before every commit. Prefer headless Rust scenario tests (build through
  actions, run ticks, assert on state or `describe` text) over the browser pane, which is for final visual proof.
- **Sessions.** One scoped step, finished green. Read line ranges and symbols, not whole files. Every milestone ends with a
  cleanup step: budgets clean, code map current, plan compressed, dead code gone.

### 3.2 Performance

Hot loops and state stay in Rust; bulk data passes zero-copy. Measure before and after anything that could cost frame time,
on the same save: `node scripts/bench-flight.mjs <save>` (engine share, headless), `scripts/frame-bench.mjs` (real browser),
`PROFILE=` plus `scripts/prof-top.mjs` (where the time goes; build with `npm run build:wasm -- --profiling`), `bench_*` tests.
**A scene that stands still must do no streaming work:** `bench-flight.mjs <save> 60 0 8 0 still` reports `meshes`, which
should stay near 0 (a loop that re-dirtied chunks every tick cost 39% of the frame until 2026-11-14). A world edit made by
the simulation every tick uses `set_block_anywhere_later` (the remesh waits for the streaming budget), never `set_block_anywhere`.

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

## 4. Now: open steps

Rules that bind every step: content architecture per `docs/TECH_TREE.md` section 8; each save-format change bumps
`SAVE_VERSION` with a migration and a fixture test; golden hashes are re-recorded only on purpose, noted in the step; new
ids append (`docs/EXTENDING.md` says where the next free one comes from); each new look gets a placeholder layer and a
`docs/ART_HANDOVER.md` request line; fluids other than water ride belts as barrel items.

- [ ] **13.6b Balance from the user's next play-test** (`scripts/inspect-save.mjs`): order sizes, upkeep rates, perk rates
  and the harbour's price, once the user has played Milestone 13.
- [ ] **L1 Mini-lessons.** Built: the rules, the machine-stopped lesson (`stall_lesson.rs`), need lessons
  (`need_lesson.rs`), no recipe (`recipe_lesson.rs`, L2), upgrade with kits (`upgrade_lesson.rs`, L5), guided first steps (`first_steps.rs`). L3, L4, L6 are one session each, in this order, each with a patch note.
  - *Rules (every lesson):* shown only for a need standing near the player now, never on a timeline; one card at a time in
    `ui/hints.ts` order, with `QUIET_TICKS` after each; done is a bit of `Sim::taught` (`lessons.rs`; saved, world-wide, set
    by doing the job or by H via `Action::LessonTaught`, never cleared); one escalation (card, then a prompt on the object,
    `key-prompt.ts`), no camera effects; a lesson whose job is already done never shows; a headless test completes it and
    `lessons/tests.rs` fuzzes the latches. Teach a stall once per cause, never nag; a lesson cannot create a need.
  - *Next:* **L3 Blueprint
    walk-through** (after the repeat prompt is accepted: prompts on the two corners; done: a blueprint saved). **L4 Planner**
    (planner crafted, no site; card only; done: first site). **L6 Map and pins** (the first 150 blocks out from spawn).
  - *Done when (each):* its bot test passes; `node scripts/check-lessons.mjs <saves>` stays quiet; seen in the browser pane.
- [ ] **FIRE A flamethrower, and fire that burns grown trees whole to charcoal** (user, 2026-11-14; four steps, FIRE1 core,
  FIRE2 flamethrower, FIRE3 look, FIRE4 manual). Full design, the six rules that keep it cheap and each step's done-when:
  `docs/ROADMAP.md` "Fire". Binding rules: fire burns only what grows (trees as `felling/tree.rs` finds them, saplings, ground
  cover), never a placed block, machine or house; a burning tree is one record, not a block automaton; a world with no fire
  does no fire work; every record and every tick's work is capped. Charcoal is treated as coal everywhere (2026-11-16).
- **Measured performance options, none built** (2026-11-14; section 2 has the baseline). In order of value:
  (1) The atlas as lossless WebP (`art/pack.png` 4.52 MB → 2.68 MB
  in four slices of at most 16,383 rows, bit-exact): needs a decode that keeps un-premultiplied alpha (`art-pack.ts`) and
  `tools/art/build.py` to write it. (2) Cache static machine geometry per chunk instead of rewriting it relative to the
  eye each frame (`Belt::frame`, `item_pose`, `model`, `light_at`: about 25% of a steady frame). (3) In the core, leave
  alone a pump's source cell when the sea refills it at once (`pumping.rs`, `sim/water.rs`): two fewer core edits and water
  checks per pump operation, but it changes the state hash, so re-record the pins on purpose.
- **Requested 2026-11-16, ordered in `docs/ROADMAP_ORDER.md` (`docs/ROADMAP_REQUESTS.md`):** XRAY goggles, worn in an equipment slot, that show buried builds; WIRES power lines hung from
  the poles' arms (XRAY's tech sits in blue science after Hoists); WORLDS upgrade old saves by generator layer (deposits only; no map pin for far deposits).
  Light and colour: GLASS stained glass and coloured lamps (pigments from ores in chemical plants); LED brighter lamps with colour
  control (a later tech, unpowered); STREET tall street lamps; CBPAL a colour-blind palette for tier colours, as a setting;
  MUSIC a sea piece and industrial and space pieces by era.
  In `docs/ROADMAP_CHEMISTRY.md`: CHEM the chemical tree (**design done 2026-11-16**: TEACH, DYE, GAS early; AIR, COOL, CARB beside the
  elevator; lubricant goes into rotating machines; no boil-off, gases stay barrels; the oil chain's jam is diesel); PUMPDOWN pumping out a sealed flooded shaft
  (analysed: feasible); CODEX a periodic table of discovered and made elements, with achievements (no rewards).
- **Parked, fit in where a session has room:** P3 cheaper box instances (see (2) above), P4 an incremental co-op
  checksum, P5 small TS savings (`git show 025edaf:docs/history/CHANGELOG_M12.md`); a power beam drawn from satellites to
  rectennas; incremental `relink` if big bases hitch on placing (measured in section 2).
- **The user's to do:** add the two TURN secrets (`docs/WORKFLOW.md` section 4), then test with `&relay` on both ends.
- **Only if placing feels laggy in co-op:** show your own edits at once in the render cache, undo them if the core
  disagrees, never touch the core (`interaction.rs`). A real resync logged by a client becomes a replayable test.

**Order:** `docs/ROADMAP_ORDER.md` ranks all of the above, the interlude (`docs/ROADMAP.md`) and Milestone 14 by impact per
session; the interlude's presentation items are cheap and sit in the first two waves. Art goes on in parallel
(`docs/ART_HANDOVER.md`).

## 5. Open questions for the user

Ask when the step that needs the answer comes up. Record answers in `docs/DECISIONS.md`.

None open (the 2026-11-16 answers are rows in `docs/DECISIONS.md`). Still to choose when planned, by Claude: the research for the
flamethrower (FIRE2) and for the LED lamps.

---

## 6. Recent changes

Older entries: `git show 025edaf:docs/DEV_PLAN.md` (entries before 2026-11-15 are in `git log -p -- docs/DEV_PLAN.md`).

- **2026-11-17: MERGE and LOOK, the merger and the router family look (v0.8.8, no save bump).** `RouterKind::Merger` (block
  `merger.rs`; the kind is the byte older saves wrote as the filter flag): inputs in turn, idle ones skipped (`router/tests.rs`).
  `router_model.rs`: one gatehouse, roof art per job (cyan in, gold out), a lit band per opening, a chevron over the served side.
  Machine ports are black doorways (`process/port_mouth.rs`, `render::mark_dark`) with a lamp when a belt is joined (`Processor::linked`).

- **2026-11-16: L5, the upgrade lesson (v0.8.7).** upgrade_lesson.rs: the crosshair on a bottleneck (Processor::is_bottleneck: working at full pace,
  every input piled up; found by Factory::bottleneck_at) with the next step's kits and research in hand brings a card and an Open prompt after
  2 s; bit UPGRADE_BIT (9), taught by upgrading that machine or H; saved worlds count it taught (KNOWN_BEFORE_74, still save 74). Tests:
  
eed_lesson/tests.rs (they aim the real camera: update_target reruns every tick, so a test cannot just set g.target).
- **2026-11-16: L2, the no-recipe lesson (v0.8.7, save 74).** `recipe_lesson.rs`: a `Pick::Chosen` machine with no recipe (found by
  `Factory::unchosen`, since it reports no efficiency sample) near the player for `STALL_TICKS` brings a card and an Open prompt;
  bit `NO_RECIPE_BIT` (8) in `Sim::taught`; choosing a recipe or H teaches it; save 74 writes nothing new, older worlds count it
  taught (`KNOWN_BEFORE_74`). Tests in `need_lesson/tests.rs`, `lessons/tests.rs`.
- **2026-11-16: DOCS, session-start read 57 → 38 KB (docs only).** `CODEMAP.md` 25.7 → 15.4 KB: entries are names plus a few words
  (detail lives in module headers; the map's own header says so); `check-map.mjs` still passes. This plan 27.6 → 20.1 KB: change-log
  entries before 2026-11-15 dropped. Keep new entries to two or three lines.

- **2026-11-16: CHEM0, the chemistry design (docs only).** Measured on the 37.6 h save: both refineries and the cracker stand on
  "Diesel Barrel has nowhere to go" (no diesel generator built). Families ranked by the need they meet; elevator-only ones moved
  next to it; decisions in `docs/DECISIONS.md`; the design and each step's done-when in `docs/ROADMAP_CHEMISTRY.md`. Code: only
  `recipes/tests.rs` `NO_USE_YET` shrank to lubricant. No patch note (players see nothing).
- **2026-11-16: the user's ten answers (direction hub).** Block ids `u16` everywhere (ROOM); M14A stays last; fire stays in
  wave 3; charcoal = coal; goggles worn; LED a later tech, unpowered; colour-blind palette as a setting (CBPAL); no pin for far
  deposits; three more music pieces (MUSIC); `web/public/replays/` git-ignored. Rows in `docs/DECISIONS.md`; CBPAL and MUSIC added
  to `docs/ROADMAP_ORDER.md`, M14A moved to the end of it.
- **2026-11-16: roadmap audit (`/deepen`).** Checked against the code and the user's six exported saves: new gate ROOM, M14
  split (M14A first, later kept last), LED row (64, 2) measured, gas fields placed apart from `ore_at`, codex from `Produced` events, multiplier
  sinks cut from chemistry. Details: `docs/ROADMAP_ORDER.md` "Audit".

- **2026-11-15: bolder kits, a true green Mk2 (user, v0.8.6).** Kit cases wear their own layers `tex::kit_paint` (468–471: whole
  face in the tier colour plus Mk pips; `item_models/components.rs` `kit`). Mk2's art colour is `kit.LEAF` (hue ~120), not aether.
- **2026-11-15: upgrade button drawn, mining note removed (user, v0.8.5).** `ui/machine-upgrade.ts` `UpgradeButton`: arrow and
  Mk, kit icon, held/needed (also the hauler pack's). New standing rule "show, don't write": `docs/DECISIONS.md`, `web/src/CLAUDE.md`.
