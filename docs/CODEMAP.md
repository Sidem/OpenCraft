# OpenCraft code map

One line per module, then recipes for common changes. **Update this file in the same commit as any
structural change** (a module added, moved, split or removed). Each module's header says more: what it
owns, its invariants and how to extend it. Conventions and gotchas live in `crates/engine/CLAUDE.md` and
`web/src/CLAUDE.md`.

## Engine: `crates/engine/src` (Rust → wasm, owns all game state)

Tests live in `<module>/tests.rs` (`src/tests.rs` for the crate root). A module with submodules is a
folder with `mod.rs`.

| Module | Owns |
|---|---|
| `lib.rs` | The `Game` struct (every field), `Game::new`, the per-frame `Game::update`; the module list |
| `api/mod.rs` | The JS-facing API, one `#[wasm_bindgen] impl Game` block per file; methods only forward |
| `api/input.rs` | Movement, look, mining/using, hotbar selection, fly toggle, drop |
| `api/render.rs` | Streaming work (`begin_work`, `work_step`), mesh/unload events, camera, box instances, sound events, textures |
| `api/inventory.rs` | Inventory screen: slots, cursor stack, `close_inventory`, pickup notifications |
| `api/crafting.rs` | Recipe queries and `craft` |
| `api/content.rs` | Block names and sound materials, `hand_yield`, `miner_recovery` |
| `api/hud.rs` | Player flags, target and `target_detail`, mining progress, stats counters |
| `api/debug.rs` | `give`, `teleport`, `skip_time`, `find_deposit`, `block_at`, `player_x/y/z` |
| `interaction.rs` | Local player's hands and feet: targeting, mining, `break_block`, `try_place`, `take_from_machine`, `throw`, footsteps |
| `block.rs` | Block ids (= item ids for now), `DEFS` table, texture layers `tex`, sound materials, lookup tables |
| `chunk.rs` | 32³ block storage; uniform chunks cost no heap |
| `world.rs` | Loaded chunks, streaming queues, edits (`saved` keeps edited chunks), `*_anywhere` accessors, render events |
| `worldgen/mod.rs` | Terrain heights, surface, caves, trees; per-column cache; `generate(chunk)` |
| `worldgen/ore.rs` | Deposit seeding (outcrops, veins, lodes), stamping, `deposit_at` ownership, `find_deposit` |
| `deposits.rs` | Deposit geometry, tiers, pooled reserves, draw caps, taper, spent rock, `HAND_YIELD` |
| `factory/mod.rs` | `Factory`: machine `Vec`s, position index `at`, add/remove/take, `update` orchestration |
| `factory/belt.rs` | Belt items, spacing, `accept`, `belt_step`; belt constants |
| `factory/miner.rs` | Miner Mk1: `draw_step`, `output_step`, `sound_step`; miner constants |
| `factory/storage.rs` | Storage box slots and `output_step`; `STORAGE_SLOTS` |
| `factory/links.rs` | `relink`: belt outputs, corners, machine outputs, downstream-first belt order (derived data) |
| `factory/render.rs` | Box instance format (`INSTANCE_FLOATS`, `push_box`) and machine models |
| `factory/describe.rs` | Machine readout text, `fmt_int`, `fmt_duration` |
| `entities.rs` | Dropped items: physics, magnet pickup, instances |
| `inventory.rs` | 36 slots, cursor stack, click / quick-move, `add_to_slots` (shared with boxes) |
| `recipes.rs` | Hand-crafting recipe table |
| `player.rs` | Character controller (walk, sprint, crouch, jump, fly) |
| `physics.rs` | Swept AABB collision against the voxel grid |
| `raycast.rs` | Voxel traversal for targeting |
| `mesher.rs` | Greedy mesher with AO; packed `u32` vertex format |
| `textures.rs` | Procedural 16×16 textures, one layer per `block::tex` constant |
| `noise.rs` | Seeded Perlin noise + fBm |
| `math.rs` | `Vec3`, `IVec3`, hashes, deterministic `Rng` |
| `sound.rs` | Sound event buffer read by the host |
| `tests.rs` | Game-level scenario tests (mining, placing, sounds, miners, crafting) |

## Web host: `web/src` (TypeScript + WebGL2, thin platform layer)

| Module | Owns |
|---|---|
| `main.ts` | Bootstrap, pause menu wiring, the frame loop; imports `base.css` and `ui/menu.css` |
| `base.css` | Theme variables, reset, focus rings, shared `.hidden`, `.secondary-btn`, `.close-btn` |
| `input.ts` | Keyboard/mouse, pointer lock, held state and one-shot `Action`s |
| `render/renderer.ts` | Chunk meshes (culling, opaque + cutout passes, fog), target outline, mining crack |
| `render/boxes.ts` | Instanced box pipeline (items, belt items, machine parts); `INSTANCE_FLOATS` |
| `render/shaders.ts` | GLSL sources |
| `render/gl.ts`, `render/mat4.ts` | Program/uniform helpers; matrix and frustum helpers |
| `ui/dom.ts` | `h()` and `button()` element helpers |
| `ui/hud.ts` + `.css` | Crosshair, target readout, mining bar, hotbar, toasts, debug overlay, `blockIcon` |
| `ui/inventory.ts` + `.css` | Inventory and build screen (E) |
| `ui/menu.css` | Pause/start menu styles (markup in `web/index.html`) |
| `ui/sound-lab.ts` + `.css` | Sound designer dialog (O): material tabs, Actions tab |
| `ui/sound-lab-footer.ts` + `.css` | Designer footer: volume, copy/paste/reset settings |
| `ui/volume-control.ts` + `.css` | Mute button + volume slider (menu and designer) |
| `ui/knob.ts` + `.css` | Rotary dial widget |
| `audio/settings.ts` | Sound design data: materials, actions, dials, presets, `DEFAULT_DESIGN`, persistence |
| `audio/synth.ts` | Procedural foley synthesis (dials → samples) |
| `audio/sound.ts` | Engine sound events → Web Audio voices, buffer cache, previews |
| `wasm/` | Generated by `npm run build:wasm` (gitignored); `engine.d.ts` is the API reference |

## Scripts and CI

| File | Purpose |
|---|---|
| `scripts/build-wasm.mjs` | wasm-pack build into `web/src/wasm` (`--dev` for a debug build) |
| `scripts/dev.mjs` | Vite dev server plus a Rust watcher (honours `PORT`) |
| `scripts/check.mjs` | `npm run check`: fmt, clippy `-D warnings`, tests, tsc, size budgets; quiet output |
| `scripts/check-size.mjs` | Line budgets from DEV_PLAN section 3.1 |
| `.github/workflows/pages.yml` | CI on push to `main`: build wasm, check, bundle, deploy to GitHub Pages |

## How to add…

**A block.** `block.rs`: append an id constant (never renumber), bump `BLOCK_COUNT`, add a `DEFS` row
(`cube`, `ore` or `machine` helper). New texture: a `tex` constant plus its arm in `textures::pixel`.
Placeable blocks work at once; worldgen use goes in `worldgen/`.

**An item or recipe.** Items are blocks until Milestone 2 adds an item registry, so a new item is a new
block (non-placeable if it shouldn't exist in the world). A recipe is a row in `recipes.rs`; the build menu
shows every row.

**A machine** (today; Milestone 2 turns this into a registry):

1. The block in `block.rs` (`machine(...)` if drawn as a model, `cube(...)` if meshed).
2. `factory/<machine>.rs`: struct, constructor, `*_step` methods, and its tuning constants.
3. `factory/mod.rs`: a `Vec` field, a `Slot` variant, `add_<machine>`, arms in `remove` and
   `take_contents`, and a call in `update`.
4. Arms in `factory/links.rs` (outputs), `factory/describe.rs` (readout), `factory/render.rs` (model).
5. `interaction.rs` `try_place`: a match arm calling `add_<machine>`. A recipe in `recipes.rs`.
6. Tests in `factory/tests.rs` (see `run` and `stocked_box`).

*Intended registry (Milestone 2, with the smelter):* keep typed storage per kind (a `Vec` per machine
struct, no trait objects). Content moves into a machine table (block id, name, buffer size, recipe set,
textures), and each kind's file exposes the same set of functions (`step`, `outputs`, `describe`,
`model`), so that `mod.rs`, `links.rs`, `describe.rs` and `render.rs` each dispatch through one `match`
on `Slot`. Adding a machine then means its file, a `Slot` variant and one table row.

**A wasm API method.** Put it in the `api/*.rs` file for its area and keep it a thin forwarder; logic goes
in a module. Run `npm run build:wasm`, then call it from TS (types come from `web/src/wasm/engine.d.ts`).
Never mirror an engine constant in TS; expose a getter as `api/content.rs` does.

**A UI panel.** `web/src/ui/<panel>.ts` with a header comment, plus `<panel>.css` imported at the top of
that file. Build DOM with `h()` / `button()` from `ui/dom.ts`; reuse `.secondary-btn` and `.close-btn`.
Construct it in `main.ts`. To open it with a key: an `Action` in `input.ts` and a branch in main.ts's
action loop.

**A sound material.** Engine: a constant in `block::sound` and point blocks' `DEFS` rows at it. Web: append
the name to `MATERIALS` (same order as the engine), add a `MATERIAL_LABELS` entry and a
`DEFAULT_DESIGN.materials` entry in `audio/settings.ts`. The designer gets a tab automatically.

**A sound event kind.** A constant in `sound.rs`, its entry in `EVENT_ACTIONS` (`audio/sound.ts`) and an
action in `audio/settings.ts` (`ACTIONS`, `ACTION_INFO`, `DEFAULT_DESIGN.actions`).

## Where the tuning knobs live

| Module | Constants |
|---|---|
| `deposits.rs` | `HAND_YIELD`, `TAPER_START`, `TAPER_FLOOR`; `Tier::grade`, `Tier::draw_cap` |
| `factory/miner.rs` | `MINER_RATE`, `MINER_RECOVERY`, `MINER_BUFFER` |
| `factory/belt.rs` | `BELT_SPEED`, `ITEM_SPACING` |
| `factory/storage.rs` | `STORAGE_SLOTS` |
| `worldgen/ore.rs` | `ORE_GEN`, `LODE_CHANCE`, `ORE_SPAWN_CLEARING` |
| `recipes.rs` | `RECIPES` |
| `interaction.rs` | `REACH`, place repeat, break cooldown, footstep stride |
| `player.rs` | Movement speeds, jump, gravity |
