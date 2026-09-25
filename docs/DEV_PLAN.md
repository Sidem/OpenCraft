# OpenCraft development plan

**Status:** 2026-09-25 · steps 1.0 (restructure), 1.1 (fixed 60 Hz tick), 1.2 (core `Sim`), 1.3 (actions),
1.4 (several players), 1.5 (state hash), 1.6 (save format), 1.7 (saving in the browser) and 1.8 (docs)
done · **Next up: Milestone 1, step 1.9 (milestone cleanup).**

> **This project is written entirely by AI coding agents.** Every session starts cold, and every line an
> agent has to read costs tokens and time. **Keeping the codebase small, modular and cheap to read is as
> important as any feature.** A sprawling codebase makes every future feature slower and more expensive,
> and that cost compounds. The rules in **section 3.1 are mandatory** and override convenience. When a
> change would break them, restructure first.

This is the plan of record. It says where the game is heading, what exists today, and exactly what to build
next. README.md covers setup and how the game works for players; this file covers direction and work.

---

## 0. Handover: read this first

You are picking up a working browser game (Rust → wasm engine, TypeScript/WebGL2 host). The extraction
concept (finite ore deposits, lossy hand mining, efficient miners, belts, boxes) is built and live at
<https://sidem.github.io/OpenCraft/>. The next job is **Milestone 1: Foundation** (section 4).

- **Step 1.0 is done:** the codebase was restructured so it's cheap for agents to work on (small modules,
  tests in their own files, `npm run check`, a code map).
- **Next comes the foundation itself:** fixed simulation ticks, all state changes as actions, several
  players in the engine, a deterministic core with tests, and saving. It's mostly invisible to players, but
  multiplayer, saving, catch-up and everything after build on it.

Before you change code:

1. Read sections 0–4 of this file (section 3.1 carefully), then `docs/CODEMAP.md`, then the nested
   `CLAUDE.md` of the area you work in. Skim README.md only if you need the player's view.
2. Run `npm run build:wasm` (if `web/src/wasm` is missing) and `npm run check` to confirm a green baseline
   (56 engine tests).
3. Work through Milestone 1 in step order. Each step lists where, how and when it's done. Do one step, or
   one clean part of a step, per session, and stop in a green, committed state.
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
| 2026-09-25 | **Approach for co-op** (proposed by Claude, adopted with this plan): player-hosted over WebRTC; a deterministic simulation core driven by tick-stamped actions (Factorio-style); player movement and loose items replicated from the host. Revisit only if Milestone 3 prototyping shows a real problem. |

### Proposed, not yet confirmed by the user

- Upgrades raise **recovery**, not just speed (Mk2 miner ≈ 75%), so upgrading extends a deposit's life.
- Bulk materials (stone, sand, clay, gravel) stay effectively infinite for quarries.
- Research via a lab that consumes parts (Factorio) vs. milestone deliveries (Satisfactory). See section 6.

---

## 2. Where the code stands (after step 1.6)

### Architecture

Rust owns all game state and hot loops (`crates/engine`, compiled to wasm with wasm-bindgen). TypeScript
(`web/src`) is a thin platform layer: input, WebGL2 rendering, DOM UI, Web Audio. Bulk data (chunk meshes,
box instances, sound events, textures) is read zero-copy from wasm memory through `*_ptr` / `*_count`
accessors. `Game` (`lib.rs`) is a thin facade: its JS-facing API is split by area into `api/*.rs` and acts
for the local player (`Game.local`). The deterministic core is `Sim` (`sim.rs`: tick, world, factory with
deposits, each player's inventory, rng); `Sim::state_hash` fingerprints it through the canonical encoding
in `bytes.rs`, which `save.rs` also reads back for saves. The authority (`authority.rs`) owns every player's body and the
loose items. The local player's hands (mining, placing, footsteps) live in `interaction.rs`. `Game`
changes the core only by queuing `Action`s (`action.rs`), applied at the next tick; the core answers with
`SimEvent`s (`events.rs` reacts). `docs/CODEMAP.md` maps every module.

Each frame, `web/src/main.ts`:

1. forwards input (`set_move`, `look`, `set_mining`, `set_using`, actions from `input.ts`),
2. calls `game.update(dt)`. This runs streaming, then as many fixed 60 Hz ticks as the frame time adds up to
   (`Game::run_tick`: every body's physics, the local player's targeting, mining and placing, item
   entities, all queuing actions, then the core's `Sim::step`, then `handle_sim_events` for item spawns,
   sounds and toasts), then interpolates the camera between the last two ticks and writes box instances,
3. runs `begin_work()` + `work_step()` under a time budget (generate or mesh one chunk per step),
4. drains mesh and unload events to the renderer, plays sound events, renders, updates the HUD.

### What exists

- **World:** 32³ chunks, 256 tall; seeded terrain with cliffs, caves and trees; greedy mesher with AO;
  streaming nearest-first; edits kept when chunks unload (`World.saved`).
- **Deposits** (`deposits.rs`, placed by `worldgen/ore.rs`):
  - Outcrops, veins and lodes of coal, iron and copper. The tiers are outcrop 100 units per block with a
    60/min shared draw cap, vein 1,000 with 240/min, and lode 2,000 with 1,200/min.
  - A pool is shared per deposit. Output tapers over the last 20%, and the block nearest the miner
    becomes `SPENT_ROCK` for each block's worth drawn, even in unloaded chunks.
  - Ownership is deterministic: the first deposit in `DepositKey` order wins.
  - Hand mining keeps `HAND_YIELD` = 3 per block and costs the deposit one block.
- **Factory** (`factory/`, one file per machine kind):
  - The Miner Mk1 draws 1 unit/s and recovers 60%.
  - Belts move 1 block/s, with per-cell item lists, side-joins, corners and back-pressure.
  - Storage boxes hold 24 slots and push into belts leading away.
  - Machine models are drawn as instanced boxes, and status readouts come from `describe`.
- **Inventory** (`inventory.rs`): 36 slots (hotbar 0–8), a cursor stack, click, shift-click and
  quick-move.
- **Crafting** (`recipes.rs`): hand recipes for Miner Mk1, belts ×4 and Storage Box, used by the build menu
  (`web/src/ui/inventory.ts`, key E).
- **Sound:** procedural foley, 7 materials including metal, and a sound designer (key O).
- **Items are blocks:** `BlockId` (u8) doubles as the item id. Ids 0–14: AIR, STONE, DIRT, GRASS, SAND,
  LOG, LEAVES, COAL_ORE, IRON_ORE, COPPER_ORE, BEDROCK, SPENT_ROCK, BELT, MINER, STORAGE.
- **Debug API** on `window.opencraft.game`: `give`, `teleport`, `run_ticks`, `skip_time`, `find_deposit`,
  `block_at`, `toggle_fly`, `set_look`, `add_player`, `remove_player`, `state_hash`.
- **Saving** (`save.rs`, `web/src/save/`): worlds autosave to IndexedDB; the menu lists, creates,
  exports and imports them.
- **Size:** about 113 KB gzipped in total (wasm 81 KB, JS 28 KB, CSS 4 KB).

### Known limitations and technical debt (Milestone 1 fixes most of these)

1. ~~No saving.~~ Worlds save in the browser since step 1.7. Two tabs on the same world overwrite each
   other's saves (the later save wins).
2. ~~Frame-rate dependent simulation.~~ Fixed in step 1.1. The player's body still only moves while its
   chunk is loaded, but that is the authority's business, not the core's.
3. ~~Presentation mixed into simulation.~~ Fixed in step 1.2: the factory emits `SimEvent`s, and
   `target_detail` no longer tracks deposits.
4. ~~State changes are direct calls.~~ Fixed in step 1.3 (actions). Tests and `find_outcrop_block` /
   `build_mine` still set up worlds directly, which is fine for tests.
5. ~~One player is built in.~~ Fixed in step 1.4. Still single-player-shaped: streaming centres on the
   local player (other bodies wait where the ground isn't loaded), only the local player has hands, and
   nothing draws other players' bodies. Milestone 3 handles these.
6. ~~`World::set_block` silently fails in unloaded chunks.~~ Core edits use `set_block_anywhere` since
   step 1.2.
7. Belts can't climb, and there are no splitters or filters.
8. Veins and lodes can only be found by digging; there's no prospecting.
9. The outcrop nearest spawn (about 11, 62, 2 with seed 1337) is buried under 1–2 blocks.
10. Items share the block id space; a separate item registry is needed for ingots and parts (Milestone 2).
11. TypeScript mirrors a few engine constants: `INSTANCE_FLOATS`, the 6 floats per sound event, and the
    order of sound materials and event kinds. Replace them with getters when touching that code.

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
  machine registry with the smelter in Milestone 2).

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
  (a few lines) and move the next milestone in from `docs/ROADMAP.md`, detailed to the level Milestone 1
  has now. Future milestones stay as short bullet lists in the roadmap, which is read only when planning.
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
- **Every milestone ends with a cleanup step** (see step 1.9): size check clean, code map current, plan
  compressed, dead code gone.

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

The **deterministic core** (world edits, factory, deposits, inventories, crafting) must produce identical
state on every machine given the same seed and the same actions. Therefore core code must not depend on:

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

## 4. Milestone 1: Foundation (NEXT)

**Goal:**

- First, restructure the codebase so agents can work on it cheaply (step 1.0, done).
- Then:
  - the game runs on a fixed tick,
  - every change to the core is an action,
  - the engine can hold several players,
  - a test proves two engines fed the same actions stay identical,
  - worlds save and load in the browser.

No networking yet. **Behaviour must stay the same as today** apart from saving; all existing tests keep
passing (adjust them to the new API where needed).

### Target structure

```
Game (thin wasm facade: lib.rs + api/*.rs) → same JS API where practical, so the TS changes stay small
 ├─ core: Sim (new, sim.rs)     deterministic: tick, World edits, Factory, Deposits,
 │                              players' inventories/cursor/selection, core Rng
 │   └─ apply(Action) + step()  → emits SimEvents (block broken, spill items, miner working…)
 ├─ authority (host side)       player bodies (physics), loose item entities, pickups;
 │                              turns intents into Actions; turns SimEvents into item spawns
 └─ view (local only)           camera + interpolation, targeting, mining progress, streaming/meshing,
                                box instances, sounds, pickup toasts, readouts
```

In single-player the local game is the authority. In co-op (Milestone 3) only the host is, and the core runs
on every peer.

### Steps

- [x] **1.0 Restructure for agents** (done 2026-09-25, nine commits starting at `31bb05b`)
  - `rustfmt.toml` (width 120) and a one-off `cargo fmt`; every inline test module moved to
    `<module>/tests.rs`.
  - `lib.rs` split into `api/*.rs` (one `#[wasm_bindgen] impl Game` per area) and `interaction.rs`;
    `factory.rs` became `factory/` (belt, miner, storage, links, render, describe);
    `worldgen/ore.rs` split out; `style.css` became one CSS file per component; `sound-lab.ts` and
    `renderer.ts` split.
  - Every module has a header. `npm run check` (fmt, clippy `-D warnings`, tests, tsc, size budgets) runs
    in CI. `docs/CODEMAP.md`, `crates/engine/CLAUDE.md` and `web/src/CLAUDE.md` written.
  - Same 56 tests and JS API; wasm 188,960 bytes vs 188,923 before (+0.02%); browser smoke test passed.

- [x] **1.1 Fixed simulation tick** (done 2026-09-25)
  - `Game::update` accumulates frame time and calls `Game::run_tick` (`lib.rs`) for each whole tick, at most
    8 per frame (a 0.1 s host frame plus a leftover partial tick needs 7, so no time is lost at normal frame
    rates). Beyond that the excess is dropped.
  - `prev_eye` / `render_eye` give the interpolated camera (`eye_x/y/z`). Box instances are drawn relative to
    it but not interpolated themselves. `update_target` also runs per frame, for the HUD.
  - `skip_time` runs whole ticks (all systems, not only the factory) and discards their sounds. `run_ticks(n)`
    added. `teleport` resets the interpolation. `Game.time` became `tick: u64`.
  - Test `results_do_not_depend_on_frame_rate`: walking, jumping, mining and a running miner give identical
    state at 30, 60, 144 fps and irregular frames. 57 tests; wasm 188,889 bytes (−71). In the browser, the
    camera moves every frame at 144 fps while the ticks advance every 2–3 frames.
  - Per tick: player physics as 2 substeps of 1/120 s, then targeting, mining, placing and item entities,
    then the factory, all advanced by `TICK`. `look()` stays per frame (yaw and pitch aren't core state).
  - Optional polish left for later: interpolating item and belt instances.

- [x] **1.2 Separate the core from presentation** (done 2026-09-25)
  - `sim.rs`: `Sim { tick, world, factory, players: Vec<PlayerCore>, rng, events }` and `Sim::step`.
    `PlayerCore` holds the `Inventory` (slots, cursor, selection). `Game` keeps `sim` plus the body, items
    and view state; `LOCAL` (lib.rs) indexes the local player until step 1.4. `World` still holds the render
    cache too.
  - `Factory::update(world, tick, events)` takes the `Sim` tick instead of keeping its own counter. Miners
    emit `SimEvent::MinerWorking { pos }` every 54 ticks while drawing (`pulse_step`), and
    `Game::present_events` plays them within 24 blocks of the camera.
  - `deposits::owner_of` and `DepositState::survey` are read-only. `target_detail` caches one survey in
    `Game.surveyed` (a view cache, rebuilt when the target's deposit changes); `lookup` alone tracks.
  - `World::block_anywhere_or_generate` added. Breaking and placing use it and `set_block_anywhere`.
  - `world.rs` crossed 400 lines, so it became `world/mod.rs` plus `world/streaming.rs`.
  - Tests `target_detail_leaves_the_core_unchanged` and `working_miner_reports_itself_and_is_heard_nearby`.
    59 tests; wasm 189,797 bytes (+908). Browser (seed 1337 outcrop): looking at ore shows its figures
    with `deposits_tracked()` still 0; the placed miner tracks it, runs, and pulses.

- [x] **1.3 Actions** (done 2026-09-25)
  - `action.rs`: `Action` (`BreakBlock`, `PlaceBlock { pos, slot, facing, against }`, `TakeContents`,
    `Craft`, `ClickSlot`, `CloseInventory`, `SelectSlot`, `ScrollSlot { delta }`, `DropSelected`,
    `PickUp`, `Give`) and `Sim::apply`, which validates against current state and does nothing when an
    action no longer fits. The player travels beside the action: `Sim::queue(tick, player, action)`.
    `PlayerId(u8)` exists now; `LOCAL` is `PlayerId(0)`.
  - `Sim::step` applies due actions in (tick, player, sequence) order, then the factory. `Game::act`
    queues for the current tick, so single-player sees results within the same frame, usually.
  - The hands queue actions: the mining timer queues `BreakBlock`; right-click resolves `TakeContents`
    or `PlaceBlock` (`interaction::right_click_action`). Loose items predict pickups on a scratch copy of
    the inventory and queue `PickUp`; what no longer fits comes back as `Thrown`.
  - Events: `BlockBroken`, `BlockPlaced` (sounds), `Gained` (toast and pickup sound), `Crafted` (toast),
    `Dropped` (block drops, spawned by the authority), `Thrown` (thrown in front of the player).
    `events.rs` (`Game::handle_sim_events`) reacts to them.
  - JS API names unchanged. `craft` now returns how many crafts the inventory can pay for (they run at
    the next tick); `give` returns nothing. `main.ts` plays the hotbar tick sound when the selection
    changes across frames.
  - Tests: `action/tests.rs` (queue order, stale actions, place and break in unloaded chunks, pickup
    overflow); Game tests now queue actions and run a tick. 63 tests. Wasm 195,320 bytes (+5.5 KB raw,
    +1.6 KB gzipped: the queue, `apply` and event handling). Browser: DOM craft button, slot clicks,
    close, hotbar select, drop, mining and placing a miner all work through actions.

- [x] **1.4 Several players in the engine** (done 2026-09-25)
  - `Sim.players: Vec<Option<PlayerCore>>` indexed by `PlayerId`; the new `Join` and `Leave` actions add
    and remove entries, so joins happen at a tick boundary like any change. Leaving drops the inventory,
    and ids are reused.
  - `authority.rs` (new): `Game.bodies: Vec<Option<Player>>` indexed the same way; `step_bodies` (physics,
    fall reset), `step_items` (items fly to the nearest player with room: `entities::Collector`),
    `throw` from the right body, `join` / `leave`. `Game.local` replaces the `LOCAL` constant;
    `body()` and `inventory()` give the local player's. The hands stay local-only.
  - API: `add_player() -> id | undefined` and `remove_player(id)` in `api/debug.rs`; everything else still
    acts for the local player. No TS changes.
  - Tests: `two_players_build_and_craft_with_their_own_inventories` (bare `Sim`: joining, crafting,
    placing and breaking in one tick, leaving), `a_second_player_has_its_own_body_pickups_and_throws`,
    `the_nearest_player_with_room_gets_the_item`. 66 tests. Wasm 201,229 bytes (+5.9 KB raw, +1.9 KB
    gzipped; the two exports are 1.75 KB raw of it). Browser: mining, pickup, toast, drop, adding and
    removing a player all work.

- [x] **1.5 State hash and determinism tests** (done 2026-09-25)
  - `bytes.rs` (new): `ByteWriter`, the canonical little-endian encoding of core state, and `fnv1a`.
    Each core type writes itself in a `write_state` next to its fields, skipping derived data.
    `Sim::write_state` writes tick, rng, players, edited chunks (loaded or stored, sorted by coordinate,
    run-length encoded), machines in `Vec` order, then every tracked deposit sorted by key.
    `Sim::state_hash` is FNV-1a over those bytes. It is exposed to JS as `state_hash()` (a BigInt).
  - Pending actions are not hashed, since peers may already hold different actions for future ticks.
  - `World::set_block_anywhere` no longer marks an unloaded chunk as edited when the block is already
    there, matching `set_block`.
  - `math::sort_small_by_key` replaces `worldgen::sort_by_ownership`.
  - Tests in `sim/tests.rs`: same actions, same hash every tick for 6,300 ticks (miner, belts, box,
    hand mining, crafting, two players, first spent rock); loaded versus bare core with the chunks
    streamed out and back; the hash covers inventories, players, time and seed.
    `target_detail_leaves_the_core_unchanged` also checks the hash across ore and machine readouts. The
    frame-rate test compares state hashes. A mutation check (breaking read loaded chunks only) fails at
    tick 1. Test 2 (save/load continuity) comes with step 1.6.
  - 71 tests, suite still about 1.5 s. Wasm 206,788 bytes (+5.6 KB raw, +2.4 KB gzipped, almost all of it
    the encoding that saves will reuse).

- [x] **1.6 Save format** (done 2026-09-25)
  - `save.rs` (new): magic `OCW1`, `SAVE_VERSION`, `WORLDGEN_VERSION` (`worldgen/mod.rs`; **bump it
    whenever generation changes incompatibly**), seed, then exactly `Sim::write_state`, then the local
    player's id, every body (position, velocity, yaw, pitch, flying) and the loose items.
  - `bytes.rs` gained `ByteReader`, and every `write_state` a `read_state` beside it. Reads return
    `None` on anything malformed (past the end, unknown block ids, bad enums, duplicate chunks, deposits
    or machine positions, lengths the data can't hold, bytes left over). Machines are rebuilt through
    their constructors and the factory relinks on the first tick; tracked deposits are looked up again
    by key (`deposit_by_key`) and re-surveyed.
  - API (`api/save.rs`): `save() -> Uint8Array`, static `Game.load(bytes, view_radius)` which throws a
    sentence a player can read (not an OpenCraft world, newer version, older version, world generation
    changed, damaged), plus `seed()` and `play_seconds()` for the world list.
  - Tests: `a_saved_world_loads_back_to_the_same_bytes` (a miner line, a second player, flying, loose
    items: save, load, save gives identical bytes and hash), `foreign_old_and_damaged_files_are_refused`
    (every truncation, a trailing byte, every single-byte corruption: refused or loaded, never a panic),
    and test 2, `a_reloaded_core_carries_on_identically` (reload at tick 3,000, equal hashes to 6,300).
  - 74 tests. Wasm 218,177 bytes (+11.4 KB raw, about +4.5 KB gzipped: reading back and checking every
    type; the reader's primitives are kept out of line to save 0.4 KB).

- [x] **1.7 Saving in the browser** (done 2026-09-25)
  - `save/store.ts`: IndexedDB `opencraft` with two stores: `worlds` holds `{ id, name, seed, updated,
    playTime, slot }` and `saves` holds the bytes under `[id, slot]`. Each save goes into the slot that
    isn't newest, in one transaction with the record, so the previous save stays as a backup. Bytes
    are gzipped with `CompressionStream`, except saves made as the page hides or closes, which go in raw
    so the write starts at once (`unpack` accepts both).
  - `save/session.ts`: startup opens the most recently played world, falling back to the backup (with a
    notice) if the newest save won't load. `Session` autosaves every 60 s, on pause, on hide and on
    `pagehide`, skipping a save when no tick has run since the last one. Switching worlds saves, marks
    the target as latest and reloads the page (one `Game` per page; wasm memory never shrinks).
  - `ui/worlds.ts` + `.css` in the menu: the world list (Play, Export, Delete with confirmation; the
    open world is marked Playing), New world (name, seed as a number or any text, blank for random),
    Import. The play button says Continue for a saved world. `?seed=N` creates a world named
    "Seed N" and drops the parameter, so a reload continues it. First visit: "My world", seed 1337.
  - Browser, verified: built a miner line (miner, 3 belts, box); reloading with no explicit save
    brought back the same play time, position, flying and line, and the box kept filling (28 → 40 ore
    in 20 s, the miner's 36/min). New world, switching back, export (532 bytes gzipped), import,
    delete, a damaged newest save (backup used), both saves damaged (error plus world list) all work.
    No console errors. JS +2.7 KB and CSS +0.2 KB gzipped; wasm unchanged.

- [x] **1.8 Docs** (done 2026-09-25)
  - README: a "Saving and worlds" section for players, `?seed=` described as starting a new world, the
    architecture diagram redrawn as core, authority and view plus `save.rs`, a paragraph on the
    deterministic core, roadmap ticks, the size figure.
  - `docs/CODEMAP.md`: every Milestone 1 module was already listed step by step; added a recipe for
    changing what gets saved and reflowed the core-state recipe.

- [ ] **1.9 Milestone cleanup** (every milestone ends with this step)
  - `npm run check` passes with no size warnings. Split anything that grew past its soft limit.
  - Remove dead code and leftover old paths from the refactors.
  - The code map matches the tree, and the nested `CLAUDE.md` files are current.
  - Compress this plan: Milestone 1 becomes a few lines in section 8. Move Milestone 2 from
    `docs/ROADMAP.md` into section 4 and detail it to the level Milestone 1 has now (steps with where, how
    and done-when). Ask the user the open questions tagged M2 (section 6) before detailing it.

**Suggested commits:** one per step, or per clean part of a step. Every commit passes `npm run check`, and the game
still works.

---

## 5. Roadmap after Milestone 1

Milestones 2–7 (content, co-op, exploration, terrain and scale, fluids and depth, endgame) are in
`docs/ROADMAP.md`. Read it only when planning the next milestone; step 1.9 moves Milestone 2 from there
into this plan.

---

## 6. Open questions for the user

Ask these when the milestone that needs the answer comes up, not before.

| Needed by | Question |
|---|---|
| M1/M2 | Should factories keep running while the game is closed (simulate the missed time on load, capped)? |
| M2 | Research style: a lab consuming parts (Factorio) or milestone deliveries (Satisfactory)? |
| M2 | Confirm upgrades raise recovery (proposal: Mk2 ≈ 75%) and that bulk materials stay infinite. |
| M3 | Where to host the signalling service and TURN relay (needs an account, e.g. Cloudflare)? |
| M3 | Target co-op size (2–4? up to 8?). Sets bandwidth and performance budgets. |
| M7 | Megaproject theme (rocket ship or something else) and what launching unlocks. |

---

## 7. Working in this repo

See `docs/WORKFLOW.md`: commands, browser verification, Windows gotchas, deploying, working with the user,
and the balance numbers. Read the section you need.

---

## 8. Change log of this plan

- **2026-09-25:** Plan created after the deposits and factory slice (`3239215`). It records the decisions
  (peaceful, open-ended, co-op), the co-op approach, Milestone 1 in detail, and the roadmap to Milestone 7.
- **2026-09-25:** The user stated the project is developed entirely by AI agents and that keeping the
  codebase from growing unwieldy is of utmost importance. Added section 3.1 (mandatory agent rules), step
  1.0 (restructure first), step 1.9 (the milestone cleanup pattern), and the rule that the plan details only
  the current milestone. Following that rule, moved milestones 2–7 to `docs/ROADMAP.md` and the
  operational reference to `docs/WORKFLOW.md`.
- **2026-09-25:** Step 1.0 done (restructure for agents). Section 2 now describes the new layout; steps
  1.1–1.7 name the new files. Added debt item 11 (constants TypeScript still mirrors). README's project
  layout now points to `docs/CODEMAP.md`.
- **2026-09-25:** Step 1.1 done (fixed 60 Hz tick). The cap is 8 ticks per frame rather than about 6, so a
  0.1 s frame never loses time. Debt item 2 now lists what is still load-dependent.
- **2026-09-25:** Step 1.2 done (`Sim`). `Factory::update` takes the tick from `Sim` rather than the planned
  `(world, events)` signature, so there is one tick counter. Debt items 3 and 6 closed; removed a broken
  leftover table from section 2.
- **2026-09-25:** Step 1.3 done (actions). Differences from the plan text: the player id sits beside the
  action in the queue rather than in every variant; `ScrollSlot { delta }` added so several scrolls within
  one tick all count; the planned `Spill` / `BlockChanged` / `Sound` events became `Dropped`, `Thrown`,
  `BlockBroken`, `BlockPlaced`, `Gained` and `Crafted`, which the new `events.rs` maps to spawns, sounds
  and toasts. Debt item 4 closed.
- **2026-09-25:** Step 1.4 done (several players). Joining and leaving became actions (`Join`, `Leave`),
  which the plan text didn't name. Bodies, loose items and pickups moved out of `lib.rs` into the new
  `authority.rs`. The plan listed `player.rs`, but it needed no change. Open for Milestone 3: whether a
  leaving player's inventory is kept for rejoining (today it is dropped). Debt item 5 closed.
- **2026-09-25:** Step 1.5 done (state hash). The plan said to hash only deposit states that "differ from
  generation"; all tracked states are hashed instead, because tracking changes behaviour: a miner draws
  nothing from an untracked deposit. The hash covers the same bytes the save will store, so step 1.6 now
  starts from `bytes.rs` and the `write_state` methods, and its section list was updated to match.
- **2026-09-25:** Step 1.6 done (save format). Differences from the plan text: the file holds no world
  name or play time (the browser's world record keeps the name, and play time comes from the saved tick);
  bodies also save their velocity and are stored by slot (the slot is the id), and the local player's id
  is saved. `Game.load` takes a view radius, since the host picks it.
- **2026-09-25:** Step 1.7 done (saving in the browser). Differences from the plan text: save bytes live
  in their own `saves` store so listing worlds never reads them, and the backup is the other of two
  slots rather than a separate swap. Saves made while the page closes skip compression. "Load" is a
  Play button per world, and switching worlds reloads the page. The M1 question was settled by the
  default (several named worlds).
- **2026-09-25:** Step 1.8 done (docs). The code map had been kept current in each step, so this step
  was mostly the README.
