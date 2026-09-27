# OpenCraft development plan

**Status:** 2026-09-27 · Milestones 1–4 done (co-op tested across machines by the user; no TURN for
now) · **Next up: Milestone 5, once the user answers its questions (section 6)** · The `art` branch is
superseded; art work continues from `main` (`docs/ART_HANDOVER.md`).

> **This project is written entirely by AI coding agents.** Every session starts cold, and every line an
> agent has to read costs tokens and time. **Keeping the codebase small, modular and cheap to read is as
> important as any feature.** A sprawling codebase makes every future feature slower and more expensive,
> and that cost compounds. The rules in **section 3.1 are mandatory** and override convenience. When a
> change would break them, restructure first.

This is the plan of record. It says where the game is heading, what exists today, and exactly what to build
next. README.md covers setup and how the game works for players; this file covers direction and work.

---

## 0. Handover: read this first

You are picking up a working browser factory game (Rust → wasm engine, TypeScript/WebGL2 host), live at
<https://sidem.github.io/OpenCraft/>.

- **Milestone 1 (Foundation) is done** (section 8): a fixed 60 Hz tick, a deterministic core changed only
  by actions, several players in the engine, state hashes, and worlds that save in the browser.
- **Milestone 2 (Make it a game) is done** (section 8): items, smelter, constructor, belt logistics, power,
  research with science packs, Mk2 upgrades and onboarding tips.
- **Milestone 3 (Co-op) is done** (section 8): 2–4 players in one world over WebRTC, the host's browser as
  the authority, a Cloudflare Worker to connect them, hosting and joining from the menu.
- **Milestone 4 (Reasons to explore) is done** (section 8): biomes and rock provinces, ores that follow
  geology, surface hints, prospecting, map marks, day and night, sky light and lamps, a new build menu.
- **Now: Milestone 5: Scale and terrain** (section 4). It is outlined, not detailed: ask the user the
  questions in section 6 first, then write its steps.
- **Art** (`docs/ART_HANDOVER.md`) says who owns which looks. When gameplay needs a new look, append a
  `tex` layer with a plain placeholder pattern and add a line to that file's request list.

Before you change code:

1. Read sections 0–4 of this file (section 3.1 carefully), then `docs/CODEMAP.md`, then the nested
   `CLAUDE.md` of the area you work in. Skim README.md only if you need the player's view.
2. Run `npm run build:wasm` (if `web/src/wasm` is missing) and `npm run check` to confirm a green baseline
   (193 engine tests).
3. Work through the current milestone in step order. Each step lists where, how and when it's done. Do
   one step, or one clean part of a step, per session, and stop in a green, committed state.
4. When a step is done, tick its checkbox here, update the **Status** line at the top, and add a line to
   the change log (section 8). Update `docs/CODEMAP.md` in the same commit as any structural change. Keep
   this file truthful; the next session relies on it.

Ground rules (details in `docs/WORKFLOW.md`):

- **Never push to `main` without asking the user.** Every push deploys the live site.
- Commit only when the user asks. So far the user has had finished work committed and pushed straight
  to `main` when they asked for a deploy.
- Windows machine: use PowerShell and the file tools; the Bash tool fails here.
- **Keep the codebase agent-friendly (section 3.1).** File budgets, tests in separate files, docs as maps.
- Performance comes first, and dependencies must be justified. Explain things to the user in plain language.

---

## 1. Where the project is heading

A browser game between **Minecraft** and **Satisfactory**: a fully editable voxel world that you mine,
shape and industrialise. Not a Minecraft clone; its conventions can be broken freely.

### Pillars

1. **Everything is physical.** Items travel on belts, machines are blocks in the world, the world is the
   material. No magic teleporting storage.
2. **Finite ground pushes you outward.** Deposits run dry after a long time. Local depletion drives
   expansion, and distance, depth and scale become the problems each tier solves.
3. **Terrain is something you engineer.** Cut, fill, flatten, tunnel, flood. Terraforming machines are the
   game's signature feature; no other game in the genre has them.
4. **Automation replaces your hands at every scale:** mining first, then logistics, then construction.
5. **Every tier adds a new kind of problem, not just recipes:** quantity → power → distance → depth and
   terrain → fluids → scale.

### Decisions (settled with the user)

| Date | Decision |
|---|---|
| 2026-09-25 | **Finite but long-lived deposits.** A factory lasts a long time but not forever. |
| 2026-09-25 | **Efficiency model.** Hand mining is lossy, machines are efficient; recovery rate is a progression axis. Numbers will need tuning. |
| 2026-09-25 | **Peaceful** for the foreseeable future. No enemies or combat. A tower-defence-style mode may come much later, as a world setting. |
| 2026-09-25 | **Open-ended.** An optional final construction (e.g. a rocket ship) needing enormous resources and advanced research. Completing it must **not** end the game; it should unlock something. |
| 2026-09-25 | **Built entirely by AI coding agents.** Keeping the codebase from growing unwieldy is of utmost importance; token efficiency and fast iteration drive architecture choices (section 3.1). |
| 2026-09-25 | **Co-op multiplayer must be possible**: other people can join a world. |
| 2026-09-25 | **Research is Factorio-style**: labs use science packs crafted from increasingly complex parts (<https://wiki.factorio.com/Science_pack>). |
| 2026-09-25 | **Upgrades raise recovery**, not just speed (Mk2 miner ≈ 75%), so upgrading extends a deposit's life. |
| 2026-09-25 | **Bulk materials** (stone, sand, clay, gravel) stay effectively infinite for quarries. |
| 2026-09-25 | **Approach for co-op** (proposed by Claude, adopted with this plan): player-hosted over WebRTC; a deterministic simulation core driven by tick-stamped actions (Factorio-style); player movement and loose items replicated from the host. Revisit only if Milestone 3 prototyping shows a real problem. |
| 2026-09-25 | **Co-op hosting on Cloudflare**: a Cloudflare Worker for signalling and Cloudflare TURN as the relay. The user owns the account. |
| 2026-09-25 | **Co-op size: 2–4 players.** This sets the bandwidth and performance budgets. |
| 2026-09-26 | **No TURN relay for now**: STUN only. Players whose networks block direct connections can't join; adding the two TURN secrets later (`docs/WORKFLOW.md` section 4) needs no code change. |
| 2026-09-26 | **Tools wear out and come in tiers.** A better pickaxe keeps slightly more ore by hand, but nowhere near what machines recover (step P5). |
| 2026-09-26 | **Ores follow geology.** Coal, iron and copper are placed by rock type and biome (for example copper in mountains, coal in lowlands); limestone and quartz are added; visible rock types (granite, sandstone, basalt) say what lies beneath. |
| 2026-09-26 | **Deposit rates stay as today** (how many outcrops, veins and lodes); tune after play. |
| 2026-09-26 | **The world pauses while the game is closed.** No simulating missed time. |
| 2026-09-26 | **New generation rules for new worlds only.** Worlds made before Milestone 4 keep generator version 1 and play as before. |

### Proposed, not yet confirmed by the user

- The Miner Mk1 and the smelter stay unpowered (a burner tier); newer machines need power. Built this way.
- Old saves keep loading across format changes where a migration is cheap. Every save since version 1
  still loads.

---

## 2. Where the code stands (after Milestone 4)

### Architecture

Rust owns all game state and hot loops (`crates/engine`, compiled to wasm with wasm-bindgen). TypeScript
(`web/src`) is a thin platform layer: input, WebGL2 rendering, DOM UI, Web Audio. Bulk data (chunk meshes,
box instances, sound events, textures) is read zero-copy from wasm memory through `*_ptr` / `*_count`
accessors. `Game` (`lib.rs`) is a thin facade: its JS-facing API is split by area into `api/*.rs` and acts
for the local player (`Game.local`). The deterministic core is `Sim` (`sim.rs`: tick, world, factory with
deposits and research, each player's inventory, rng); `Sim::state_hash` fingerprints it through the
canonical encoding in `bytes.rs`, which `save.rs` also reads back for saves. The authority
(`authority.rs`) owns every player's body and the loose items. The local player's hands (mining, placing,
footsteps) live in `interaction.rs`. `Game` changes the core only by queuing `Action`s (`action.rs`),
applied at the next tick; the core answers with `SimEvent`s (`events.rs` reacts). `docs/CODEMAP.md` maps
every module.

Each frame, `web/src/main.ts`:

1. forwards input (`set_move`, `look`, `set_mining`, `set_using`, actions from `input.ts`),
2. calls `game.update(dt)`. This runs streaming, then as many fixed 60 Hz ticks as the frame time adds up to
   (`Game::run_tick`: every body's physics, the local player's targeting, mining and placing, item
   entities, all queuing actions, then the core's `Sim::step`, then `handle_sim_events` for item spawns,
   sounds and toasts), then interpolates the camera between the last two ticks and writes box instances,
3. runs `begin_work()` + `work_step()` under a time budget (generate or mesh one chunk per step),
4. drains mesh and unload events to the renderer, plays sound events, renders, updates the HUD.

**Co-op** is lockstep: only actions travel. The host stamps every action (its own too) for
`tick + INPUT_DELAY` (4) and sends one frame per tick; clients step their core only through the ticks
they have frames for, and compare state hashes every 60 ticks (a mismatch resyncs from a snapshot).
Bodies are each machine's own (states at 20 Hz); loose items run on the host only (views at 10 Hz). The
core never learns about the network: `net/` (Rust) moves bytes, `web/src/net/` runs the session over a
`Transport` (WebRTC between machines, BroadcastChannel between tabs). Budgets for 4 players: under
~10 KB/s per client outside snapshots, and the host's frame under 2 ms more than solo.

### What exists

- **World:** 32³ chunks, 256 tall; seeded terrain with cliffs, caves and trees; greedy mesher with AO;
  streaming nearest-first; edits kept when chunks unload (`World.saved`). Generator versions: a world
  keeps its own (`WorldGen::version`); version 2 adds biomes (`worldgen/biome.rs`: plains, desert over
  sandstone, highlands over granite, lowlands, basalt fields; spawn always plains) and ores by biome
  (`worldgen/geology.rs`, including limestone and quartz) with stained soil hints above veins and lodes.
- **Day and light:** a 20-minute day from the core tick (`daytime.rs`), sky, sun, moon and stars
  (`render/sky.ts`); sky and block light 0–15 computed while a chunk meshes (`light.rs`: the chunk plus a
  15-block margin), smoothed per vertex (a byte after the vertices); lamps (block 44) give light 15.
- **Deposits** (`deposits.rs`, placed by `worldgen/ore.rs`): outcrops, veins and lodes of coal, iron and
  copper (100 / 1,000 / 2,000 units per block, shared draw caps 60 / 240 / 1,200 per minute). A pool is
  shared per deposit, output tapers over the last 20%, and blocks turn to `SPENT_ROCK` as it drains, even
  in unloaded chunks. Hand mining keeps `HAND_YIELD` = 3 per block and costs the deposit one block.
- **Factory** (`factory/`, one file per machine kind, the `MACHINES` table and a `Machine` trait; one kind
  can serve several blocks through extra table rows):
  - Miners: Mk1 (1 unit/s, 60% recovery, unpowered) and Mk2 (2 units/s, 75%, 20 kW).
  - Belts (1 block/s; fast belts 2) with side-joins, corners, back-pressure, ramps, lifts and underpasses
    (`belt_shape.rs`); splitter and filter (`router.rs`); storage boxes (24 slots, open like a chest).
  - Smelter (ore plus coal or logs → ingots) and constructor (ingots → plates, rods, screws, wire), with
    machine recipes and fuels as data (`recipes.rs`). Right-click opens a machine panel (`panel.rs`,
    `ui/machine.ts`): status, buffers, recipe or filter choice, put-in and take buttons.
  - Power (`power.rs`): coal generators, poles that link within 10 blocks into grids, machines on the
    nearest pole within 5, brownouts as a speed factor.
  - Research labs (`lab.rs`) working through the tech tree (`research.rs`, key R) with red and green
    science packs; six techs unlock routing, climbing, underpasses, green packs, Mk2 and fast belts.
  - Models are instanced boxes; status readouts come from `describe`.
- **Inventory** (`inventory.rs`): 36 slots (hotbar 0–8), a cursor stack, click, shift-click and
  quick-move. **Crafting** (`recipes.rs`): hand recipes in the build menu (key E, `ui/crafting.ts`): a grid by `recipes::Group` with search, filters (can craft, missing, locked) and a hover card.
- **Onboarding tips** (`hints.rs`, `ui/hints.ts`, key H skips): seven tips from finding ore to research.
- **Minimap** (`minimap.rs`, `ui/minimap.ts`, key N): loaded terrain around the player, north up, arrows
  for every player; rings for prospected veins and lodes (kept in the browser's world record, dropped
  when dry) and squares for machines (`minimap/marks.rs`); presentation only.
- **Block timers** (`sim/timers.rs`, core state): leaves of a felled tree decay (half-life 5 s), grass
  spreads onto bare dirt beside it (30 s) and turns to dirt under a solid block (15 s), saplings grow.
  Timers start only from block changes, never from scanning chunks.
- **Saplings** (`sim/saplings.rs`): leaves drop one 1 time in 25 (broken or decayed); planted on dirt or
  grass, it grows after 60 s plus a 90 s half-life into a tree shaped like generated ones
  (`worldgen::tree_blocks`); drawn as crossed quads (`Render::Plant`, mesher faces 6 and 7).
- **Items** (`item.rs`): `ItemId(u16)`. Ids below 256 are the blocks with the same number (0–44, see
  `block/mod.rs`; 29 is the sapling, 30–43 Milestone 4's rocks, ores, glass, stained soils and sand, 44 the lamp); from 256: iron and copper ingots, iron plate, iron rod, screws, copper wire, red and green
  science packs, the six tools (264–269), scanner (270) and core drill (271). Every item is drawn as a textured box.
- **Sound:** procedural foley, 7 materials including metal, and a sound designer (key O).
- **Debug API** on `window.opencraft.game`: `give`, `teleport`, `run_ticks`, `skip_time`, `find_deposit`,
  `block_at`, `toggle_fly`, `set_look`, `add_player`, `remove_player`, `state_hash`, `debug_desync`.
- **Saving** (`save.rs`, `web/src/save/`): worlds autosave to IndexedDB; the menu lists, creates,
  exports and imports them. Save version 12; every version since 1 loads.
- **Co-op** (`net/`, `web/src/net/`, `signal/`, `ui/coop.ts`, `ui/players.ts`): "Play together" in the
  menu hosts the open world (a room code and link from the signalling Worker) or joins from a pasted link
  or code; up to 4 players; a returning player gets their things back (player keys, `Sim.away`). Avatars
  with name tags, a Tab player list with ping, join and leave notices, readable refusals (full, another
  version, no such game, no connection), a hidden co-op tab keeps ticking, a silent peer times out after
  10 s, and a client that differs from the host or falls 5 s behind resyncs in place. URL shortcuts for
  testing: `?host`, `?host=<room>` / `?join=<room>` (tabs of one browser), `?join=<code>`, `?relay`.
- **Tools** (`tools.rs`, recipes at the end of `RECIPES`): pickaxe, axe and shovel in stone (150 uses, 2×)
  and iron (600, 4×, an iron pickaxe keeps 4 ore); a tool's stack count is its uses left, shown as a wear
  bar (`showAmount` in `ui/hud.ts`). Placeholder looks in `textures/tools.rs`.
- **Prospecting** (`prospect.rs`, `ui/prospect.ts`): the scanner lists deposits within 48 blocks (ore,
  tier, live bearing and distance, depth, size band); the core drill (3 s hold) gives a column's exact
  figures. Queries only: no actions, the state hash never moves. Devices are tools that never wear.
- **Size:** about 212 KB gzipped in total (wasm 161.4 KB, JS 44.9 KB, CSS 5.8 KB).

### Known limitations and technical debt

1. Only the local player has hands: another player's mining and placing reach this machine only as
   their results (by design; co-op sends actions, not hands).
2. Two tabs on the same world overwrite each other's saves (the later save wins).
3. ~~Veins and lodes can only be found by digging~~: fixed by prospecting (4.6) and map marks (4.7).
4. The outcrop nearest spawn (about 11, 62, 2 with seed 1337) is buried under 1–2 blocks.
5. TypeScript mirrors a few engine constants: `INSTANCE_FLOATS`, the 6 floats per sound event, and the
   order of sound materials and event kinds. Replace them with getters when touching that code.
6. Item and belt instances aren't interpolated between ticks (only the camera is); optional polish.
7. Balance is untested by real play: pack costs, research times and Mk2 costs will need tuning
   (`docs/WORKFLOW.md` section 6 lists the numbers).
8. In co-op every action applies about 70 ms plus the round trip after it's made; your own block edits
   aren't shown early (section 4, "instant feedback"). A name change applies the next time you host or
   join. Hosting can't be stopped without leaving the page.
9. Co-op over the internet has been tested by the user on two machines (STUN only); the relay is untested.
10. A block timer that fires or checks next to a chunk nobody loaded or edited generates that chunk for
    each read (`block_anywhere_or_generate`); rare (the player walked off right after felling a tree),
    but slow if it happens. A small cache of generated chunks would fix it.
11. Light is recomputed per meshed chunk (0.49 ms per chunk against 0.36 without); an edit relights up to
    about 30 chunks (within 15 blocks, and all below) over the following frames. A working smelter
    doesn't glow (its light would depend on factory state).

---

## 3. Engineering rules for all new work

### 3.1 Keep the codebase small and cheap to work on (TOP PRIORITY, mandatory)

**Why this matters more than usual:** only AI agents develop this project, and each session starts with no
memory of the code. Tokens go to four things:

1. reading code to understand it (the biggest cost),
2. exploring to find where things live,
3. failed build and test iterations, and the output they print,
4. re-learning the project every session.

Structure decides all four. A 400-line module behind a clear interface is cheap to change forever; a
1,000-line god object taxes every feature that touches it. **Treat context cost like frame time: a budget
you measure and defend.** If a change would break these rules, restructure first, in its own commit.

**Architecture: changes stay local**

- **One feature, one module or folder.** Adding a machine, item, recipe or UI panel means new files plus at
  most one registration line in a central place. If you find yourself editing five files for one feature,
  the structure is wrong; fix it.
- **No god objects.** The wasm facade (`Game`) is thin: its API is split by area into `api/*.rs` files
  (several `#[wasm_bindgen] impl Game` blocks are allowed), and each method only forwards to a module.
- **Content is data.** Blocks, items, recipes, machines, tech tree and ore settings are tables. Code is
  written per *kind* of behaviour, never per individual item.
- **Simulation and presentation are separate modules** (section 3.4). A UI change never requires reading
  simulation code, and the reverse.
- **Boring, explicit code.** Shallow call chains, no macro tricks, no deep generic towers, no ECS framework.
  Plain data-oriented modules: typed storage in `Vec`s plus one function per system. Don't build
  abstractions ahead of need. Introduce a registry when the second instance of a kind arrives (e.g. the
  machine registry arrived with the smelter).

**Size budgets** (enforced by `scripts/check-size.mjs`, part of `npm run check`)

| Kind | Soft limit (warn) | Hard limit (fail) |
|---|---|---|
| Rust / TS source file, excluding tests | 400 lines | 600 lines |
| CSS file (one per UI component) | 300 lines | 500 lines |
| Any `CLAUDE.md` | 60 lines | 100 lines |
| This plan (read every session) | 600 lines | 800 lines |
| Other docs (`CODEMAP`, `ROADMAP`, `WORKFLOW`) | 300 lines | 500 lines |

Count all lines, blank ones included: `(Get-Content f).Count`, not `Measure-Object -Line`.

- Before adding to a file over its soft limit, split it.
- **Tests never live inline in implementation files.** Declare `#[cfg(test)] mod tests;`, which resolves to
  `src/foo/tests.rs` from both `src/foo.rs` and `src/foo/mod.rs`. Reading a module must not pull in its
  tests.

**Readability: cheap to skim**

- **Every module starts with a header** (`//!` in Rust, a top comment in TS) of at most ~15 lines: what it
  owns, its key invariants, and how to extend it ("to add a machine: …").
- **Public items first, private helpers after**, so the top of a file is its interface.
- **Unique, searchable names** (`belt_step`, not a tenth `update`), so one search finds the right spot.
- **One source of truth.** Never mirror constants between Rust and TS; expose a getter (like `hand_yield()`)
  or generate it.
- **Delete dead code immediately.** No commented-out code, no old and new paths living side by side after a
  refactor.
- **Comments explain *why*,** briefly. Don't narrate what the code already says.

**Docs: maps, not history**

- `docs/CODEMAP.md`: one line per module (what it owns) plus "how to add X" recipes.
  **Update it in the same commit as any structural change.** It replaces most exploration.
- **Nested `CLAUDE.md` files** in `crates/engine/` and `web/src/` hold subsystem conventions. Claude Code loads
  them only when working in that folder, so they cost nothing elsewhere. Keep them short.
- **This plan details only the current milestone.** When a milestone finishes, compress it into section 8
  (a few lines) and move the next milestone in from `docs/ROADMAP.md`, detailed to the level of the
  current one (steps with where, how and done-when). Future milestones stay as short bullet lists in the
  roadmap, which is read only when planning.
- **Operational reference lives in `docs/WORKFLOW.md`** (commands, verification, gotchas); read only the
  section you need.
- The root `CLAUDE.md` holds rules and pointers only.

**Feedback loops: fast and quiet**

- **`npm run check`** runs format check, clippy with warnings as errors, engine tests, typecheck
  and size budgets, and prints only a summary and the failures. Run it before every commit.
- **Prefer headless tests over the browser.** Scenario tests in Rust (build a layout through actions, run
  ticks, assert on state or `describe` text) and text debug dumps (e.g. an ASCII map of an area with
  machines and belt contents) are far cheaper than driving the browser pane. The browser is for final
  visual proof only. From Milestone 1 on, bugs should become replayable action scripts.
- **Keep tool output small:** quiet flags (`cargo test -q`), and filter long output to what matters.
- **Let the compiler do the searching:** newtypes (`ItemId`, `PlayerId`) and exhaustive `match`es make the
  compiler list everything a change must touch.

**Session habits**

- One scoped step per session; finish green and committed. Big sessions are where tokens disappear.
- Read line ranges and symbols, not whole files. Use the code map. For a broad search, use a search
  sub-agent so only its conclusion enters the main context.
- End every session by updating this plan (checkboxes, status, change log) and the code map.
- **Every milestone ends with a cleanup step:** size check clean, code map current, plan compressed, dead
  code gone, the next milestone moved in from the roadmap.

### 3.2 Performance

- Hot loops and all game state stay in Rust. TypeScript stays thin.
- Pass bulk data zero-copy (views on wasm memory); don't serialise per frame.
- Measure before and after anything that could cost frame time.

### 3.3 Wasm size

- Report gzipped sizes when the build grows noticeably (`npx vite build` prints them).
- Avoid formatting floats in Rust (`{:.1}` pulls in about 25 KB). Round to integers, or format in
  TypeScript.
- Avoid new instantiations of the std sorts (about 5–9 KB each). Use the insertion sort
  `math::sort_small_by_key`.
- No new crates without a reason worth their size. Hand-written serialisation beats serde+bincode here.

### 3.4 Determinism (required for co-op; enforced from Milestone 1)

The **deterministic core** (world edits, factory, deposits, inventories, crafting, research) must produce
identical state on every machine given the same seed and the same actions. Therefore core code must not
depend on:

- frame time: advance only in fixed ticks,
- which chunks happen to be loaded or meshed locally: use the `*_anywhere` accessors, which generate or
  read the stored copy,
- the camera, the local player, UI state or presentation queries: queries must never create or modify core
  state,
- iteration order of hash maps: iterate `Vec`s or sorted keys wherever order affects results,
- wall-clock time, `Math.random`, or anything from JS,
- (for a future native server) platform math: `sin`/`cos`/`powf` may differ between wasm and native. The
  `libm` crate on both sides fixes that when the time comes.

Everything that changes the core goes through an **action** applied at a tick boundary. Things that are only
presentation (camera, sounds, particles, meshes, HUD, readouts) never feed back into the core.

### 3.5 Code style

- Section 3.1 comes first. Beyond it, match the surrounding code: comment density, naming, idioms.
- Every engine feature gets unit tests in its module. Run `cargo test --workspace --release`.
- Keep README.md in sync for anything player-visible, and keep this plan in sync for everything else.

---

## 4. Now: Milestone 5: Scale and terrain

Milestone 4 is done (summary in section 8). Milestone 5 moved here from `docs/ROADMAP.md` and is **not yet
detailed**: ask the user the questions in section 6, record the answers in section 1, then write its steps
here (where, how, done when), in the style of Milestone 4's (git history of this file).

The outline:

- **Terraforming machines** (the signature feature). Start with "mark an area, pick a job"; no
  programming language needed: an excavator digs a marked area and hauls the spoil to a box or dump
  site, a grader flattens to a height, a filler places fill, a borer cuts tunnels and shafts.
- **Byproducts as fill:** slag from smelting and tailings from washing become fill material, so waste
  feeds the terrain system.
- **Blueprints and construction drones:** copy a region of machines, place a ghost, and drones build it
  from storage (or tear it down).
- **Transport:** personal (hoverpack, ziplines along power lines), trucks on recorded routes, trains
  (rails on the grid, stations, signals), which create demand for rail beds, cuttings, tunnels and bridges.
- **Simple logic:** sensors (belt full, box full) and on/off conditions on machines.

Everything stays deterministic (section 3.4): machines change the world only through the core, never
through loaded chunks.
Open items from Milestone 3 (the user's to unblock; do them when they come up):

- **The relay (TURN):** left out for now (section 1). If friends can't connect, the user adds the TURN
  secrets (`docs/WORKFLOW.md` section 4); then test with `&relay` on both ends.
- **Instant feedback for your own edits** (was step 3.9): measured, not built. On one machine a client's
  action applies about 90 ms after it is made (the host's own, about 76 ms: `INPUT_DELAY` is 4 ticks);
  over the internet add the round trip, so roughly 110–170 ms. Build it only if placing and breaking feel
  laggy in real play: show your own block edits at once in the render cache, undo them if the core
  disagrees, never touch the core (`interaction.rs`).
- A client's every resync logs the tick and both hashes; turn any real one into a replayable test.

---

## 5. Roadmap after Milestone 5

Milestones 6–7 (fluids and depth, endgame) are in `docs/ROADMAP.md`. Read it only when planning the next
milestone; Milestone 5's cleanup step moves Milestone 6 from there into this plan.

---

## 6. Open questions for the user

Ask these when the milestone that needs the answer comes up, not before. Record the answers in section 1
and adjust the steps.

| Needed by | Question |
|---|---|
| M5 | Which part first: terraforming machines, blueprints and drones, transport (trains, trucks, personal), or simple logic? (Recommended: terraforming, the signature feature.) |
| M5 | Terraforming: is "mark an area with a tool, pick a job" right, and where does dug material go: items into boxes and belts (a lot of stone), or straight to a dump site? |
| M5 | Transport: trains first, or personal movement (hoverpack, ziplines) first? |
| Any time | Should the Miner Mk1 and the smelter stay unpowered (a burner tier) while newer machines need power? (Built that way; easy to change.) |
| M7 | Megaproject theme (rocket ship or something else) and what launching unlocks. |

---

## 7. Working in this repo

See `docs/WORKFLOW.md`: commands, browser verification, Windows gotchas, deploying, working with the user,
and the balance numbers. Read the section you need.

---

## 8. Change log of this plan

- **2026-09-25:** Plan created (`3239215`); the user made keeping the codebase small for agents a top
  priority (section 3.1; later milestones in `docs/ROADMAP.md`, how-tos in `docs/WORKFLOW.md`).
- **2026-09-25: Milestone 1 (Foundation) done** (`31bb05b` to `74fb476`): fixed 60 Hz tick, core `Sim`,
  actions and `SimEvent`s, several players, canonical bytes and state hash, saves and the world list.
  Deviations: the player id sits beside each queued action; all tracked deposits are hashed and saved;
  world names and play time live in the browser's record. Tests 56 → 74, wasm 81.5 KB gzipped.
- **2026-09-25: Milestone 2 (Make it a game) done** (`b49b766` to `3bf3f92`): items, machine registry,
  smelter, constructor and panels, splitter and filter, belt shapes, boxes, power, research, Mk2 and fast
  belts, onboarding tips. Deviations: one kind serves several blocks through extra `MACHINES` rows;
  machines hang on the nearest pole; power balances before miners each tick. Saves version 9. Tests
  74 → 113, wasm 120.3 KB gzipped. Lessons: measure wasm size every step; the golden hash catches every
  unintended core change; scenario tests with a bare `Factory` replace most browser checks.
- **2026-09-26: Milestone 3 (Co-op) done**, apart from the user's live tests (section 4): lockstep over
  bytes (`action/codec.rs`, `net/`: roles, frames, checksums every 60 ticks), join snapshots and player
  keys (save version 10), avatars and name tags, host-owned loose items, a hidden co-op tab that keeps
  ticking (`net/ticker.ts`), the signalling Worker (`signal/`, deployed by the user; STUN only so far),
  WebRTC with 16 KB message pieces, the "Play together" menu and the Tab player list, pings, silence
  timeouts (10 s) and in-place resync (`Game::resync` keeps the body and loaded chunks). Deviations: the
  build id is the bundle's URL, not the git commit; step 3.9 (instant own edits) measured and deferred.
  Measured: 10 minutes hidden at 60 ticks/s with 0 mismatches; host `update` + pump 0.015 ms with 4
  players; a client uses about 8 KB/s down and 4 KB/s up while idle, most of it per-packet overhead.
  Lessons: keep heavy sort keys `#[inline(never)]` (one inlined key cost 9 KB of wasm); key hidden-tab
  work off stalled frames, not `document.hidden`; Windows PowerShell's `Get-Content` reads UTF-8 as ANSI,
  so edit files with the file tools or Node. Tests 113 → 127; wasm 120.3 → 131.5 KB, JS 32.1 → 39.1 KB
  gzipped.
- **2026-09-26:** The user tested co-op across two machines (works) and added play-test notes P1–P5
  (Escape closes panels first, minimap by default, leaf decay, grass spread, tools) ahead of Milestone 4.
  A graphics agent now works in parallel on branch `art` (`docs/ART_HANDOVER.md`). The user answered
  the tool questions: tools wear out, come in tiers, and a better pickaxe keeps slightly more ore.
- **2026-09-26:** P1 and P2 done: Escape in a panel shows a "Click to keep playing" hint, a second one
  pauses; the minimap (`minimap.rs`: top blocks cached per chunk column, a redraw 0.07 ms; N toggles it,
  no water tint yet). Tests 127 → 130; wasm 131.5 → 134.2 KB gzipped.
- **2026-09-26:** P3 and P4 done: block timers (`sim/timers.rs`: sorted, one per position and kind,
  capped at 4096, saved and hashed; save version 11), started by `Sim::block_changed` from breaking and
  placing and from the timers themselves. Leaf decay (a `LeafDecayed` event plays a soft rustle), grass
  spreading and covered grass dying. Deviations: leaf half-life 5 s, not 8, so a felled tree is bare
  within a minute; grass needs "nothing solid above" rather than air (a belt doesn't kill it); no
  particles (the game has none yet). The golden hash was re-recorded. Tests 130 → 136; wasm 134.2 →
  136.8 KB gzipped.
- **2026-09-26:** P5 done: tools (`tools.rs`: kinds, tiers, `break_speed`, `ore_yield`; six items and hand
  recipes). Deviation: wear is the stack count (a tool's stack size is its uses), not new item state, so
  every path that moves items keeps wear, no save format change was needed (still version 11), and two
  worn tools of a kind pool their uses; dropping a tool drops it whole. A box feeding a belt would split
  a tool into one-use pieces (harmless, and they pool again). A `ToolWornOut` event plays a sound and a
  "wore out" toast (a pickup notice with count 0). Tests 136 → 141; wasm 136.8 → 138.0 KB gzipped.
- **2026-09-26:** P1–P5 pushed (`233371b`). The user answered the Milestone 4 questions (section 1: ores
  follow geology with limestone and quartz added, today's rates, the world pauses while closed, new
  rules for new worlds only), and Milestone 4 was detailed as steps 4.1–4.11.
- **2026-09-27: Milestone 4 (Reasons to explore) done** (`8d8ee22` to the cleanup commit): generator
  versions (version 1 pinned by a digest test), saplings, rocks, limestone, quartz and glass (blocks
  30–35), biomes and rock provinces (`worldgen/biome.rs`, version 2 about 10 % slower to generate), ores
  by biome (`worldgen/geology.rs`) with stained soil and sand hints (36–43), scanner and core drill
  (`prospect.rs`, queries only), minimap marks, day and night (`daytime.rs`, `render/sky.ts`), sky and
  block light (`light.rs`; meshing 0.357 → 0.488 ms per chunk, +25 % mesh memory) and the lamp (44). On
  the user's requests: the art salvage (pixel art, alternates), a build menu grid with search and
  filters (`ui/crafting.ts`, `recipes::Group`), full-height block icons. Deviations: limestone and
  quartz ore are deposit ores that drop themselves; only veins and lodes get map marks; no smelter glow.
  Save version 12. Tests 141 → 183; wasm 138.0 → 161.4 KB gzipped.
- **2026-09-27: Belt placement upgrade** (user request, between milestones): ramps are no longer items
  or research. `factory::belt_shape::derive_slopes` (run in `relink`) makes a flat belt an up ramp when
  a belt one ahead and one up continues its way, and a down ramp below a belt one up behind it; old
  ramp blocks act as belts and drop belts. `belt_line.rs`: hold right-click with belts and drag to lay
  a line (longer axis first, one turn, follows one-block steps, stops at walls and machines), with
  ghost belts, host outlines and a HUD label; it builds through ordinary `PlaceBlock`s, 3 per tick.
  `Action::Rotate` (codec tag 19) turns belts and routers: R; research moved to T. Tech 1 is now
  Belt Lifts (lifts only). The golden hash was re-recorded (the scripted lone ramp now lies flat).
  Tests 183 → 193.