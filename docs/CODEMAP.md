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
| `lib.rs` | The `Game` struct (core `sim`, `local` id, `bodies`, items, the local player's hands and view state), `Game::new`, the per-frame `update` (ticks, interpolated camera, instances), the fixed tick `run_tick` (`TICK_RATE`), `act` / `act_as` (queue an action), `body()` / `inventory()` (the local player's); the module list |
| `sim.rs` | The deterministic core `Sim`: tick, world, factory, `players` (`Option<PlayerCore>` per `PlayerId`: inventory), rng, action queue (`queue`); `step`; `state_hash` / `write_state` / `read_state`; `PlayerId`, `SimEvent`. Determinism tests in `sim/tests.rs` |
| `bytes.rs` | `ByteWriter` / `ByteReader` (little-endian canonical encoding of core state; each type has a `write_state` and a `read_state`; `item` reads the layout of the reader's save `version`), `fnv1a` |
| `save.rs` | Save file: header (magic, `SAVE_VERSION`, `WORLDGEN_VERSION`), seed, core, bodies, loose items; `save_bytes` / `from_save` with player-readable refusals; older versions back to `OLDEST_VERSION` load through `ByteReader::version`. Tests in `save/tests.rs` (with the committed `v1.ocworld` fixture) |
| `action.rs` | `Action` enum and `Sim::apply`: join, leave, break, place, take contents, craft, inventory clicks, select, drop, pick up, give |
| `authority.rs` | Every player's body (`step_bodies`: physics, falling out of the world), loose items and pickups for the nearest player (`step_items`), `throw`, `join` / `leave` |
| `events.rs` | `Game::handle_sim_events`: SimEvents → item spawns (drops, throws), sounds, the local player's toasts |
| `api/mod.rs` | The JS-facing API, one `#[wasm_bindgen] impl Game` block per file; methods only forward |
| `api/input.rs` | Movement, look, mining/using, hotbar selection, fly toggle, drop |
| `api/render.rs` | Streaming work (`begin_work`, `work_step`), mesh/unload events, camera, box instances, sound events, textures |
| `api/inventory.rs` | Inventory screen: slots, cursor stack, `close_inventory`, pickup notifications |
| `api/machine.rs` | Machine panels: `take_panel_request`, `machine_panel` (flat view), machine recipes, panel buttons (set recipe, set filter, put in, take); box screens (`box_slots`, `click_box`, `store_slot`) |
| `api/crafting.rs` | Recipe queries and `craft` |
| `api/content.rs` | Block names and sound materials, `item_name`, `item_icon` (texture layers and box proportions), `hand_yield`, `miner_recovery` |
| `api/hud.rs` | Player flags, target and `target_detail`, mining progress, stats counters |
| `api/save.rs` | `save`, `load` (static), `seed`, `play_seconds` |
| `api/debug.rs` | `give`, `teleport`, `add_player` / `remove_player`, `state_hash`, `run_ticks`, `skip_time`, `find_deposit`, `block_at`, `player_x/y/z` |
| `interaction.rs` | Local player's hands and feet: targeting, mining timer (queues `BreakBlock`), `right_click_action`, `play`, footsteps |
| `block.rs` | Block ids, `DEFS` table, texture layers `tex` (item textures too), sound materials, lookup tables |
| `item.rs` | `ItemId` (ids below 256 are the blocks, others start at 256), the item table (`def`, `name`, `stack_size`, `places`), ingots |
| `chunk.rs` | 32³ block storage; uniform chunks cost no heap |
| `world/mod.rs` | Loaded chunks, edits (`saved` keeps edited chunks), block accessors (`*_anywhere` for core code), render events |
| `world/streaming.rs` | Streaming and meshing: re-centring, generation and mesh queues, `work_step`, `remesh`, `area_ready` |
| `worldgen/mod.rs` | Terrain heights, surface, caves, trees; per-column cache; `generate(chunk)`; `WORLDGEN_VERSION` |
| `worldgen/ore.rs` | Deposit seeding (outcrops, veins, lodes), stamping, `deposit_at` ownership, `find_deposit`, `deposit_by_key` |
| `deposits.rs` | Deposit geometry, tiers, pooled reserves, draw caps, taper, spent rock, `HAND_YIELD`; `owner_of` and `DepositState::survey` (read-only queries) |
| `factory/mod.rs` | Machine table (`Kind`, `MACHINES`: one row per block, Kind-ordered rows first, then extra blocks sharing a kind; `machine`), the `Machine` trait every kind implements, `Factory`: a `Vec` per kind, position index `at`, `place` / add / remove, `update` (one tick: miners, boxes, smelters, power balance, powered machines, belts; emits `SimEvent`s) |
| `factory/state.rs` | `Factory::write_state` / `read_state`: every machine list, kind by kind (older save versions skip later kinds), then the deposits |
| `factory/power.rs` | Power: `Pole` (a machine), `Power` (derived in `relink`: pole grids by wire range, the pole each generator and machine hangs on; `balance` each tick: demand, generators burn in order until supply meets it, speed per grid), wire drawing; power constants |
| `factory/generator.rs` | Coal generator: fuel buffer, burns `FUELS` only while its grid needs power; bytes, readout, panel, model |
| `factory/belt_shape.rs` | Belt `Shape`s (flat, ramp up/down, lift, underpass entry/exit): where items ride (`item_at`, `shows`), shape models, `UNDERPASS_RANGE` |
| `factory/buffer.rs` | `Buffer`: the item stacks a machine holds (box slots, miner output, processing buffers); `feed` pushes into belts leading away |
| `factory/belt.rs` | Belt items, spacing, `accept`, `belt_step`; its bytes, readout and model; belt constants |
| `factory/miner.rs` | Miner Mk1: `step` (draw, push out, `MinerWorking` events); its bytes, readout and model; miner constants |
| `factory/storage.rs` | Storage box: `step` (feeds belts leading away); its bytes and readout |
| `factory/router.rs` | Splitter and filter (one `Router` kind): holds one item, passes it front/left/right (round robin; a filter sends its item front, others aside); bytes, readout, filter panel, model |
| `factory/smelter.rs` | Smelter: sorts arriving ore and fuel, batches from `MACHINE_RECIPES`, burns `FUELS`, feeds belts leading away; bytes, readout, panel, model with status lamp |
| `factory/constructor.rs` | Constructor: one input into parts with the recipe chosen in its panel; `set_recipe` hands inputs back; bytes, readout, panel, model |
| `factory/panel.rs` | What a player does to a machine by hand: `panel` (view: status, progress, buffers by role, filter item), `box_slots`, `set_recipe`, `set_filter`, `insert`, `wants`, `take_contents` |
| `factory/links.rs` | Where items go: `Slot`, `Link`, `Sinks` (machines that take items), `deliver`; `relink`: belt outputs for every shape, corners, lift stacks, machine outputs, downstream-first belt order (derived data) |
| `factory/render.rs` | Box instance format (`INSTANCE_FLOATS`, `push_box`); `write_instances` asks nearby machines for models |
| `factory/describe.rs` | `Factory::describe` (one `match` on `Slot`), `fmt_int`, `fmt_duration` |
| `entities.rs` | Dropped items: physics, magnet pickup by the nearest `Collector` with room, instances |
| `inventory.rs` | 36 slots, cursor stack, click / quick-move, `add_to_slots` (shared with boxes) |
| `recipes.rs` | Hand-crafting recipes (`RECIPES`), machine recipes (`MACHINE_RECIPES`, saved by index: append only), `FUELS` burn times |
| `player.rs` | Character controller (walk, sprint, crouch, jump, fly) |
| `physics.rs` | Swept AABB collision against the voxel grid |
| `raycast.rs` | Voxel traversal for targeting |
| `mesher.rs` | Greedy mesher with AO; packed `u32` vertex format |
| `textures.rs` | Procedural 16×16 textures, one layer per `block::tex` constant; nature and ore patterns |
| `textures/machines.rs` | Machine and item texture patterns (belts, miner, smelter, constructor, routers, generator, pole, ingots, parts) |
| `noise.rs` | Seeded Perlin noise + fBm |
| `math.rs` | `Vec3`, `IVec3`, hashes, deterministic `Rng`, `sort_small_by_key` |
| `sound.rs` | Sound event buffer read by the host |
| `tests.rs` | Game-level scenario tests (mining, placing, sounds, miners, a smelter line, crafting, a second player, frame-rate independence); helpers shared with `save/tests.rs` |

## Web host: `web/src` (TypeScript + WebGL2, thin platform layer)

| Module | Owns |
|---|---|
| `main.ts` | Bootstrap (opens the latest world), pause menu wiring, the frame loop; imports `base.css` and `ui/menu.css` |
| `save/store.ts` | IndexedDB: `worlds` records (`WorldMeta`) and `saves` bytes in two slots per world (newest + backup); gzip `pack` / `unpack` |
| `save/session.ts` | `openWorld` (latest world, backup fallback, `?seed=`), `Session` (autosave: every minute, on pause, hide and close), `switchTo` (save, then reload into another world) |
| `base.css` | Theme variables, reset, focus rings, shared `.hidden`, `.secondary-btn`, `.close-btn` |
| `input.ts` | Keyboard/mouse, pointer lock, held state and one-shot `Action`s |
| `render/renderer.ts` | Chunk meshes (culling, opaque + cutout passes, fog), target outline, mining crack |
| `render/boxes.ts` | Instanced box pipeline (items, belt items, machine parts); `INSTANCE_FLOATS` |
| `render/shaders.ts` | GLSL sources |
| `render/gl.ts`, `render/mat4.ts` | Program/uniform helpers; matrix and frustum helpers |
| `ui/dom.ts` | `h()` and `button()` element helpers |
| `ui/hud.ts` + `.css` | Crosshair, target readout, mining bar, hotbar, toasts, debug overlay, `itemIcon` (isometric box from `item_icon`) |
| `ui/inventory.ts` + `.css` | Inventory and build screen (E); opened on a box (`open([x, y, z])`), the box screen: its slots above the inventory, Take all |
| `ui/machine.ts` + `.css` | Machine panel (right-click a smelter, constructor or filter): status, progress, buffers, recipe choice, filter item, put-in and take buttons |
| `ui/menu.css` | Pause/start menu styles (markup in `web/index.html`) |
| `ui/worlds.ts` + `.css` | World list in the menu: play, new world (name, seed), export / import `.ocworld`, delete |
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

**An item or recipe.** A block is already an item. Any other item: an id constant (from 256, append only)
and a row in `item.rs` `EXTRA`, with a texture layer in `block::tex` and its pattern in `textures::pixel`
if it needs a new look; the HUD icon and the loose and belt models follow from the row. A recipe is a row
in `recipes.rs`; the build menu shows every row.

**A machine.**

1. Its file `factory/<machine>.rs`: the struct (holding a `Buffer` if it holds items), `new`, `step`,
   `impl Machine` (bytes, contents, readout, model) and its tuning constants.
2. `factory/mod.rs`: a `Kind` and a `Slot` variant, a `MACHINES` row (block, kind, slots), a `Vec` field.
   Then follow the compiler through the `match`es on `Kind` and `Slot` (`place`, `remove`,
   `take_contents`, `describe`) and add its list to `write_state` / `read_state` (`state.rs`), `update`,
   `write_instances`, and its outputs to `links.rs`. If it uses power: a demand in `Power::balance`
   and its pole in `Power::rebuild` (`power.rs`), and a `speed` argument to its `step`. If belts and miners deliver into it, an arm in
   `Slot::is_sink` and a field in `Sinks` (`links.rs`); what it makes is a `MACHINE_RECIPES` row. A panel:
   `panel: true` in its row, a `panel()` method and its arms in `panel.rs`; the host panel needs nothing.
3. Its block in `block.rs` (`machine(...)` if drawn as a model, `cube(...)` if meshed); a hand recipe
   in `recipes.rs`. Tests in `factory/tests.rs` (see `run` and `stocked_box`; test-only accessors such as `stock` live
   there too). A second block with the same behaviour (splitter/filter) is an extra `MACHINES` row after
   the Kind-ordered ones, not a new kind.

**A wasm API method.** Put it in the `api/*.rs` file for its area and keep it a thin forwarder; logic goes
in a module. Run `npm run build:wasm`, then call it from TS (types come from `web/src/wasm/engine.d.ts`).
Never mirror an engine constant in TS; expose a getter as `api/content.rs` does.

**A UI panel.** `web/src/ui/<panel>.ts` with a header comment, plus `<panel>.css` imported at the top of
that file. Build DOM with `h()` / `button()` from `ui/dom.ts`; reuse `.secondary-btn` and `.close-btn`.
Construct it in `main.ts`. To open it with a key: an `Action` in `input.ts` and a branch in main.ts's
action loop.

**Core state, a way to change it, or a reaction to it.** State: a field on `Sim` (`sim.rs`) or the type
that owns it (per-player state in `PlayerCore`), plus its bytes in that type's `write_state` and
`read_state`, which the state hash and saves use (a save test fails if the two disagree). A change: an
`Action` variant and its arm in `Sim::apply_to_player` (`action.rs`); `Game` queues it with `act` (local
player) or `act_as`. A reaction (sound, toast, item spawn): a `SimEvent` variant, pushed by the core, and
its arm in `Game::handle_sim_events` (`events.rs`). Per-player state the authority keeps (body-related)
goes in `authority.rs`, indexed by `PlayerId` like `Sim.players`.

**Something saved.** Core state: as above. Bodies and loose items: `Player` / `Items` `write_state` and
`read_state`, called from `save.rs`. Any change to the bytes bumps `SAVE_VERSION`; keep older saves
loading when a read can follow the old layout cheaply (branch on `ByteReader::version`, add a fixture
test), else raise `OLDEST_VERSION`. A change to world generation bumps `WORLDGEN_VERSION`. The browser
side (`web/src/save/`) only stores bytes and never needs to change.

**A sound material.** Engine: a constant in `block::sound` and point blocks' `DEFS` rows at it. Web: append
the name to `MATERIALS` (same order as the engine), add a `MATERIAL_LABELS` entry and a
`DEFAULT_DESIGN.materials` entry in `audio/settings.ts`. The designer gets a tab automatically.

**A sound event kind.** A constant in `sound.rs`, its entry in `EVENT_ACTIONS` (`audio/sound.ts`) and an
action in `audio/settings.ts` (`ACTIONS`, `ACTION_INFO`, `DEFAULT_DESIGN.actions`).

## Where the tuning knobs live

| Module | Constants |
|---|---|
| `lib.rs` | `TICK_RATE` (60), `MAX_TICKS_PER_FRAME` |
| `authority.rs` | `PHYSICS_SUBSTEPS` (per tick), `FALL_LIMIT` |
| `deposits.rs` | `HAND_YIELD`, `TAPER_START`, `TAPER_FLOOR`; `Tier::grade`, `Tier::draw_cap` |
| `factory/mod.rs` | `MACHINES` (buffer slots per machine) |
| `factory/miner.rs` | `MINER_RATE`, `MINER_RECOVERY` |
| `recipes.rs` | `MACHINE_RECIPES` (seconds per batch), `FUELS` (burn seconds) |
| `factory/belt.rs` | `BELT_SPEED`, `ITEM_SPACING` |
| `factory/power.rs` | `GENERATOR_POWER`, `CONSTRUCTOR_POWER`, `ROUTER_POWER`, `WIRE_RANGE`, `POLE_REACH` |
| `worldgen/ore.rs` | `ORE_GEN`, `LODE_CHANCE`, `ORE_SPAWN_CLEARING` |
| `recipes.rs` | `RECIPES` (hand) |
| `interaction.rs` | `REACH`, place repeat, break cooldown, footstep stride |
| `events.rs` | `MINER_SOUND_RANGE`, drop pickup delay |
| `player.rs` | Movement speeds, jump, gravity |
