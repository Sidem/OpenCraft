# OpenCraft development plan

**Status:** 2026-09-25 · Milestone 2 in progress: steps 2.1–2.6 done · **Next up: step 2.7
(power).** Steps 2.8 and 2.9 wait for the user's answers (section 6).

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
<https://sidem.github.io/OpenCraft/>.

- **Milestone 1 (Foundation) is done** (section 8): a fixed 60 Hz tick, a deterministic core changed only
  by actions, several players in the engine, state hashes, and worlds that save in the browser.
- **The next job is Milestone 2: Make it a game** (section 4): items separate from blocks, a smelter and
  constructor, belt logistics, power, research and upgrades.

Before you change code:

1. Read sections 0–4 of this file (section 3.1 carefully), then `docs/CODEMAP.md`, then the nested
   `CLAUDE.md` of the area you work in. Skim README.md only if you need the player's view.
2. Run `npm run build:wasm` (if `web/src/wasm` is missing) and `npm run check` to confirm a green baseline
   (74 engine tests).
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
| 2026-09-25 | **Approach for co-op** (proposed by Claude, adopted with this plan): player-hosted over WebRTC; a deterministic simulation core driven by tick-stamped actions (Factorio-style); player movement and loose items replicated from the host. Revisit only if Milestone 3 prototyping shows a real problem. |

### Proposed, not yet confirmed by the user

- Upgrades raise **recovery**, not just speed (Mk2 miner ≈ 75%), so upgrading extends a deposit's life.
- Bulk materials (stone, sand, clay, gravel) stay effectively infinite for quarries.
- Research via a lab that consumes parts (Factorio) vs. milestone deliveries (Satisfactory). See section 6.
- The Miner Mk1 and the smelter stay unpowered (a burner tier); newer machines need power (step 2.7).
- Old saves keep loading across format changes where a migration is cheap (Milestone 2 rules).

---

## 2. Where the code stands (after Milestone 1)

### Architecture

Rust owns all game state and hot loops (`crates/engine`, compiled to wasm with wasm-bindgen). TypeScript
(`web/src`) is a thin platform layer: input, WebGL2 rendering, DOM UI, Web Audio. Bulk data (chunk meshes,
box instances, sound events, textures) is read zero-copy from wasm memory through `*_ptr` / `*_count`
accessors. `Game` (`lib.rs`) is a thin facade: its JS-facing API is split by area into `api/*.rs` and acts
for the local player (`Game.local`). The deterministic core is `Sim` (`sim.rs`: tick, world, factory with
deposits, each player's inventory, rng); `Sim::state_hash` fingerprints it through the canonical encoding
in `bytes.rs`, which `save.rs` also reads back for saves. The authority (`authority.rs`) owns every
player's body and the loose items. The local player's hands (mining, placing, footsteps) live in `interaction.rs`. `Game`
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
- **Factory** (`factory/`, one file per machine kind, a machine table and a `Machine` trait):
  - The Miner Mk1 draws 1 unit/s and recovers 60%.
  - Belts move 1 block/s, with per-cell item lists, side-joins, corners and back-pressure.
  - Storage boxes hold 24 slots and push into belts leading away.
  - The smelter melts ore into ingots with coal or logs as fuel (`MACHINE_RECIPES`, `FUELS` in
    `recipes.rs`); belts and miners deliver into it through `links.rs` (`Sinks`).
  - The constructor makes one input into parts (plates, rods, screws, wire) with the recipe chosen in
    its panel. Right-click on a smelter or constructor opens the machine panel (`factory/panel.rs`,
    `ui/machine.ts`): status, progress, buffers, recipe choice, put-in and take buttons.
  - Machine models are drawn as instanced boxes, and status readouts come from `describe`.
- **Inventory** (`inventory.rs`): 36 slots (hotbar 0–8), a cursor stack, click, shift-click and
  quick-move.
- **Crafting** (`recipes.rs`): hand recipes for Miner Mk1, belts ×4, Storage Box, Smelter and
  Constructor, used by the build menu
  (`web/src/ui/inventory.ts`, key E).
- **Sound:** procedural foley, 7 materials including metal, and a sound designer (key O).
- **Items** (`item.rs`): `ItemId(u16)`. Ids below 256 are the blocks with the same number (0–16: AIR,
  STONE, DIRT, GRASS, SAND, LOG, LEAVES, COAL_ORE, IRON_ORE, COPPER_ORE, BEDROCK, SPENT_ROCK, BELT, MINER,
  STORAGE, SMELTER, CONSTRUCTOR); from 256: iron ingot, copper ingot (from the smelter), iron plate, iron
  rod, screws, copper wire (from the constructor; no use yet, power and Mk2 will use them). Every item is drawn as a textured box.
- **Debug API** on `window.opencraft.game`: `give`, `teleport`, `run_ticks`, `skip_time`, `find_deposit`,
  `block_at`, `toggle_fly`, `set_look`, `add_player`, `remove_player`, `state_hash`.
- **Saving** (`save.rs`, `web/src/save/`): worlds autosave to IndexedDB; the menu lists, creates,
  exports and imports them.
- **Size:** about 131 KB gzipped in total (wasm 102.4 KB, JS 30 KB, CSS 4 KB).

### Known limitations and technical debt

1. Players can't put items
   into a box by hand (machines take them through their panel).
2. Still single-player-shaped: streaming centres on the local player (other bodies wait where the ground
   isn't loaded), only the local player has hands, nothing draws other players' bodies, and a leaving
   player's inventory is dropped. Milestone 3 handles these.
3. Two tabs on the same world overwrite each other's saves (the later save wins).
4. Veins and lodes can only be found by digging; there's no prospecting (Milestone 4).
5. The outcrop nearest spawn (about 11, 62, 2 with seed 1337) is buried under 1–2 blocks.
6. TypeScript mirrors a few engine constants: `INSTANCE_FLOATS`, the 6 floats per sound event, and the
   order of sound materials and event kinds. Replace them with getters when touching that code.
7. Item and belt instances aren't interpolated between ticks (only the camera is); optional polish.

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
  machine registry, step 2.2, as the smelter arrives).

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
  current one (steps with where, how and done-when). Future milestones stay as short bullet lists in the roadmap, which is read only when planning.
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
- **Every milestone ends with a cleanup step** (see step 2.11): size check clean, code map current, plan
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

## 4. Milestone 2: Make it a game (NEXT)

**Goal:** turn the extraction slice into a factory game. Items become separate from blocks. Ore goes through
a processing chain (smelter, then constructor). Belts get real logistics, machines get power, and research
and upgrades give progression. Everything stays deterministic, saved and hashed like Milestone 1's core,
and every machine is a registry entry, not a special case.

Rules for every step:

- New core state goes in its type's `write_state` / `read_state` (hash and saves).
- A save format change bumps `SAVE_VERSION`. Players have worlds now, so **migrate older saves when it's
  cheap** (a version-aware read). Refuse them with the existing message only when it isn't.
- A world generation change bumps `WORLDGEN_VERSION`.
- Balance numbers are named constants; record them in `docs/WORKFLOW.md` section 6.
- Browser proof: build the chain, save, reload, screenshot. Tests first.

### Steps

- [x] **2.1 Item registry** (`item.rs` new; `inventory.rs`, `recipes.rs`, `factory/`, `entities.rs`,
  `action.rs`, `api/`, `web/src/ui/`)
  - `ItemId(u16)` newtype and an `ITEMS` table (name, stack size, icon, the block it places). **Ids below
    256 are the blocks with the same number**, so today's ids and saves stay valid. Non-block items
    start at 256.
  - Stacks, recipes, belts, boxes, miners' buffers, loose items and events carry `ItemId`. Placing asks
    the table for the block. `ByteReader::block` for items becomes `item` (validated against the table).
  - Icons: procedural 16×16 icons for non-block items (texture layers or a small icon atlas), read by TS
    through the API like block textures. `hud.blockIcon` becomes `itemIcon`. No constants mirrored in TS.
  - First non-block items: iron ingot and copper ingot (no source until 2.3; `give` works).
  - Save: `SAVE_VERSION` 2 writes `u16` item ids; version-1 saves still load.
  - *Done:* icons are isometric boxes (`item_icon`: three texture layers plus box proportions), so
    ingots are small bars in the HUD, on belts and on the ground; `ByteReader::version` reads v1 saves.
  - **Done when:** all tests pass on `ItemId`; a test loads a version-1 save made before the change
    (commit the bytes as a test fixture); `give` shows ingots with icons in the inventory.

- [x] **2.2 Machine registry** (restructure only, no behaviour change; `factory/`)
  - First add a golden test: the scripted 6,300-tick run in `sim/tests.rs` ends at a recorded
    `state_hash`. The refactor must not change it.
  - Keep typed storage per kind (a `Vec` per machine struct, no trait objects). Each kind's file exposes
    the same functions (`step`, `outputs`, `describe`, `model`, `write_state` / `read_state`,
    `contents`), and `mod.rs`, `links.rs`, `describe.rs` and `render.rs` each dispatch through one
    `match` on `Slot`.
  - A machine table (block id → kind, name, buffer sizes) replaces scattered per-kind constants.
  - Shared input/output buffers (`factory/buffer.rs`) for the processing machines to come.
  - *Done:* a small `Machine` trait (static dispatch) holds each kind's bytes, contents, readout and
    model; placement moved from `action.rs` into `Factory::place`; miners and boxes hold a `Buffer`.
    Adding a machine still touches a few `match`es and loops in `mod.rs`, all found by the compiler.
  - **Done when:** same tests and golden hash. The code map's "How to add a machine" shrinks to: its
    file, a `Slot` variant, a table row, a hand recipe. `factory/mod.rs` stays under 400 lines.

- [x] **2.3 Smelter** (`factory/smelter.rs`, a machine recipe table, block, textures, model)
  - Machine recipes are data: `{ machine, inputs, outputs, seconds }` (in `recipes.rs` or a new
    `processing.rs` if it grows).
  - The smelter takes ore plus fuel (coal ore or logs, each with a burn time) and makes ingots. The
    recipe follows the ore it's given. Belts deliver into it (items sorted into the ore or fuel buffer),
    and it pushes ingots into a belt leading away, like a box. Right-click takes its output.
  - `describe` shows status (working, no fuel, no ore, output full) and rates; the model shows a lamp.
  - A hand recipe builds it (stone plus iron ore).
  - **Done when:** a scenario test runs miner → belt → smelter, with coal fed from a box, → ingots in a
    box at the computed rate. The state hash test covers a smelter. Browser screenshot of the line.

- [x] **2.4 Machine panel and constructor** (`ui/machine.ts` + `.css`, `factory/constructor.rs`,
  `api/machine.rs`)
  - Right-click on a machine with a panel opens it: recipe choice, buffers, status, and a take-output
    button. Right-click on a box keeps taking everything.
  - New action `SetRecipe { pos, recipe }`. Changing the recipe returns buffered inputs to the player.
  - The constructor makes one input into parts, with the recipe chosen in the panel: iron plate, iron
    rod, screws (from rods) and copper wire. New items join the table.
  - Hand recipes may start asking for parts (e.g. a Miner Mk1 needs plates). Tune numbers after
    playing.
  - **Done when:** a scenario test (ingots → constructor set to plates → plates in a box); the panel
    works in the browser; a save round trip keeps recipes and buffers.
  - *Added:* an `Insert` action and put-in buttons in the panel, so ore and fuel can go into a smelter by
    hand (in 2.3, fuel could only come from a miner on coal). *Done:* no hand recipe asks for parts yet;
    the parts get their first uses in 2.7 (generator, poles) and 2.9 (Mk2).

- [x] **2.5 Belt logistics: splitter and filter** (`factory/router.rs`)
  - Splitter: one input, round robin to up to three outputs, skipping blocked ones. Belts side-joining
    already merge, so no merger block unless play shows a need.
  - Filter: the chosen item goes straight on, everything else to the sides. The item is set in the
    machine panel (2.4).
  - **Done when:** tests for round robin with a blocked output and for filtering; save round trip.
  - *Done:* one `Router` machine kind serves both blocks (`MACHINES` gained a second row per kind). The
    splitter and filter recipes are the first to use parts (plates, wire).

- [x] **2.6 Belt logistics: climbing and crossing** (`factory/belt_shape.rs`, `factory/links.rs`)
  - Ramps: a belt that rises or falls one block per cell. A vertical lift for taller climbs. An
    underpass that carries items under a crossing belt for a few cells.
  - Everything stays on the grid (free-form curves fight the voxels).
  - **Done when:** tests for items going up a ramp, up a lift and under a crossing; screenshot.
  - *Done:* five belt blocks share `Kind::Belt` with a saved `shape`. Items cross an underpass
    instantly (hidden under the hoods); a lift entered from the side of a stack pops to its centre.

- [ ] **2.7 Power** (`factory/power.rs`, generator and pole blocks, wire instances)
  - A coal generator burns fuel into power. Poles link with wires to other poles and machines in range.
    Placing a pole links it to the nearest pole automatically, and wires are drawn as thin boxes.
  - The grid is a graph. Its connected components are derived data, rebuilt when poles or machines
    change. Each component's supply over its demand gives a speed factor for its consumers.
  - Consumers: the constructor, splitter and filter, and the Mk2 machines of 2.9. The Miner Mk1 and the
    smelter stay unpowered (the burner tier). This is a proposal; see section 1.
  - Readouts show supply, demand and speed.
  - **Done when:** tests for components, a brownout halving speed, and poles in unloaded chunks; save
    round trip; screenshot of a powered line.

- [ ] **2.8 Research** (`research.rs`, a station block or a delivery point, `ui/research.ts`)
  - **Needs the user's answer first** (section 6: lab consuming parts, or milestone deliveries).
  - The tech tree is data in Rust: nodes with costs and the recipes they unlock. Research state is core
    state (saved, hashed, shared by all players in a world). The build menu hides or greys locked recipes.
  - Start small: about six nodes covering the constructor, logistics, power and Mk2.
  - **Done when:** tests for unlocking and for locked recipes being refused by `Craft`; a screenshot of
    the research screen.

- [ ] **2.9 Upgrades** (`factory/miner.rs`, `factory/belt.rs`, recipes)
  - **Needs the user's confirmation** (section 6) that upgrades raise recovery, not just speed.
  - Miner Mk2: recovery about 75%, rate about 2 units/s, needs power, crafted from parts. Fast belt:
    2 blocks/s. Both unlocked by research.
  - **Done when:** tests show Mk2 extracting more ore from the same deposit than Mk1; a fast belt keeps up
    with a Mk2.

- [ ] **2.10 Onboarding hints** (`ui/hints.ts` + `.css`)
  - Short hints in plain language, shown in order, one at a time, dismissable: dig to find the outcrop,
    craft a miner, place it against ore, add belts and a box, build a smelter.
  - Progress comes from engine getters (items owned, machines placed). Which hints were dismissed is UI
    state in `localStorage`, not game state.
  - **Done when:** a new world walks through the hints in the browser; screenshot.

- [ ] **2.11 Milestone cleanup** (every milestone ends with this step)
  - `npm run check` passes with no size warnings. Split anything that grew past its soft limit.
  - Remove dead code and leftover old paths from the refactors.
  - The code map matches the tree, and the nested `CLAUDE.md` files are current.
  - Compress this plan: Milestone 2 becomes a few lines in section 8. Move Milestone 3 from
    `docs/ROADMAP.md` into section 4 and detail it to this level. Ask the user the open questions tagged
    M3 (section 6) before detailing it.

**Suggested commits:** one per step, or per clean part of a step. Every commit passes `npm run check`, and
the game still works. Steps 2.8 and 2.9 wait for the user's answers; if those haven't come, do 2.10 first.

---

## 5. Roadmap after Milestone 2

Milestones 3–7 (co-op, exploration, terrain and scale, fluids and depth, endgame) are in
`docs/ROADMAP.md`. Read it only when planning the next milestone; step 2.11 moves Milestone 3 from there
into this plan.

---

## 6. Open questions for the user

Ask these when the milestone that needs the answer comes up, not before. The M2 ones were asked when
Milestone 2 was planned (step 1.9); record the answers in section 1 and adjust the steps.

| Needed by | Question |
|---|---|
| M2 (2.8) | Research style: a lab consuming parts (Factorio) or milestone deliveries (Satisfactory)? |
| M2 (2.9) | Confirm upgrades raise recovery (proposal: Mk2 ≈ 75%) and that bulk materials stay infinite. |
| M2 (2.7) | Should the Miner Mk1 and the smelter stay unpowered (a burner tier) while newer machines need power? (Default: yes.) |
| M2 or later | Should factories keep running while the game is closed (simulate the missed time on load, capped)? |
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
  codebase from growing unwieldy is of utmost importance. Added section 3.1 (mandatory agent rules), the
  restructure-first step, the milestone cleanup step, and the rule that the plan details only the current
  milestone (milestones 2–7 moved to `docs/ROADMAP.md`, the operational reference to `docs/WORKFLOW.md`).
- **2026-09-25: Milestone 1 (Foundation) done**, commits `31bb05b` to the step 1.9 commit.
  - Built: the restructure for agents (1.0); a fixed 60 Hz tick with at most 8 ticks per frame and an
    interpolated camera (1.1); the core `Sim` separated from the view (1.2); every core change an
    `Action` applied at a tick, answered by `SimEvent`s (1.3); several players, with `Join` / `Leave`
    actions and `authority.rs` (1.4); canonical bytes (`bytes.rs`) and an FNV-1a state hash with
    determinism tests (1.5); the save format (`save.rs`, 1.6); saving in the browser with a world list
    (`web/src/save/`, `ui/worlds.ts`, 1.7); the README (1.8).
  - Deviations worth knowing: the player id sits beside each queued action, not inside it; all tracked
    deposits are hashed and saved, because tracking changes behaviour; world names and play time live in
    the browser's record, not the save; switching worlds reloads the page; each world keeps its previous
    save as a backup slot.
  - Cost: tests 56 → 74. Wasm 188,923 → 218,063 bytes raw, 81.5 KB gzipped (of which about +3.5 KB for
    actions and players, +2.4 KB for the hash, +4.5 KB for loading saves); JS +2.7 KB gzipped.
  - Lessons: new generic code shows up in the wasm size, so measure each step. Keeping the byte
    primitives out of line (`#[inline(never)]`) saved 1.6 KB. Hash tests catch determinism bugs at the
    exact tick, and the browser pane can't lock the pointer, so drive `window.opencraft.game`.
- **2026-09-25:** Step 1.9 (milestone cleanup): no size warnings; removed the unused `set_view_radius`;
  code map and `CLAUDE.md` files checked against the tree. Milestone 2 moved in from `docs/ROADMAP.md`
  and detailed as steps 2.1–2.11. Before detailing it, the M2 questions (research style, upgrades raising
  recovery, a burner tier) could not be asked mid-task, so they were put to the user with the milestone
  report; steps 2.8 and 2.9 wait for the answers, and the rest doesn't depend on them.
- **2026-09-25:** Step 2.1 (item registry) done. `ItemId(u16)` and the item table in `item.rs`; block
  items derive from `block::DEFS`. Saves are version 2 (`u16` item ids); version 1 still loads, tested
  against the committed `save/v1.ocworld`. Tests 74 → 78; wasm 81.5 → 83.5 KB gzipped.
- **2026-09-25:** Step 2.2 (machine registry) done, behaviour unchanged: the golden hash test
  (`sim/tests.rs`, recorded before the refactor) still passes. Machine table `MACHINES`, the `Machine`
  trait, `factory/buffer.rs`; `MINER_BUFFER` and `STORAGE_SLOTS` became table rows. Tests 78 → 79;
  wasm 83.5 → 84.2 KB gzipped; `factory/mod.rs` 341 lines.
- **2026-09-25:** Step 2.3 (smelter) done. `factory/smelter.rs` (three one-slot buffers, work and fire in
  whole ticks, fire burns only while smelting); `MACHINE_RECIPES` and `FUELS` in `recipes.rs`. Belts and
  miners deliver into any machine through `Link::Machine(Slot)` and `Sinks` (`Link::Storage` is gone);
  `Link`, `Slot`, `Sinks` and `deliver` moved to `links.rs` to keep `factory/mod.rs` at 352 lines.
  Deviation: a machine recipe has one `output` (item, count), not a list, until a machine needs more.
  Saves are version 3 (adds the smelter list; 1 and 2 still load). The golden hash was re-recorded with
  a smelter in the script. The browser run showed a coal miner and an iron miner feeding a smelter, 28
  ingots in a minute. Tests 79 → 84; wasm 84.2 → 88.6 KB gzipped.
- **2026-09-25:** Step 2.4 (machine panel and constructor) done. `factory/constructor.rs` (recipe chosen
  by the player; changing it hands the inputs back), `factory/panel.rs` (panel view, `set_recipe`,
  `insert`, `take_contents` moved here), `api/machine.rs`, `ui/machine.ts`. Actions `SetRecipe` and
  `Insert` (added to the step: put items in by hand). Right-click on a machine with `panel: true` sets
  `Game::panel_request`, which the host polls. Four parts join the item table. `Buffer::feed` replaces
  three copies of the round-robin push. Saves are version 4 (adds the constructor list; 1–3 still load,
  and a version-3 world opened in the browser). Golden hash re-recorded with a constructor in the script.
  Tests 84 → 90; wasm 88.6 → 97.1 KB gzipped (14 panel exports, the constructor, the panel view; no float
  formatting); `factory/mod.rs` 376 lines.
- **2026-09-25:** Step 2.5 (splitter and filter) done. `factory/router.rs`: one `Router` kind for both
  blocks (holds one item; front/left/right outputs from `relink`; splitter round robin skipping blocked
  outputs; filter sends its item straight on, the rest aside). `MACHINES` rows are Kind-ordered first,
  then extra blocks sharing a kind. Action `SetFilter`; the panel shows a filter's item choice. Recipes
  use plates and wire. Test-only factory accessors moved to `factory/tests.rs` (`factory/mod.rs` 349
  lines). Saves are version 5 (adds the router list); golden hash re-recorded with a filter in the
  script. Browser: constructor → splitter → three boxes, 6 plates each per minute. Tests 90 → 93; wasm
  97.1 → 99.7 KB gzipped.
- **2026-09-25:** Step 2.6 (climbing and crossing) done. `factory/belt_shape.rs`: ramp up, ramp down,
  lift and underpass entry/exit are belts with a `Shape` (extra `MACHINES` rows of `Kind::Belt`), so
  items, spacing and `belt_step` are shared; `relink` works out each shape's output (an `into` helper
  for "arriving at cell t going dir"), lift stacks and which belts machines may feed (not down ramps or
  exits). Models: stepped ramps, lift posts, underpass hoods. Saves are version 6 (belts gain a shape;
  a version-5 world loaded in the browser); golden hash re-recorded with a ramp the box feeds. Browser:
  plates up two lifts onto a hill, under a crossing belt and down a ramp into a box. Tests 93 → 96;
  wasm 99.7 → 102.4 KB gzipped.
