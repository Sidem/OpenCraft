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
| `lib.rs` | The `Game` struct (core `sim`, `local` id, co-op `role`, `bodies`, items, `avatars`, the local player's hands and view state), `Game::new`, the per-frame `update` (streaming, ticks, a client's catch-up, interpolated camera, instances), the fixed tick `run_tick` (`TICK_RATE`: bodies, hands, items, `step_core`, `net_tick`), `act` / `act_as` (route an action through the role), `body()` / `inventory()` (the local player's); the module list |
| `sim.rs` | The deterministic core `Sim`: tick, world, factory, `players` (`Option<PlayerCore>` per `PlayerId`: inventory, key), `away` (players who left with a key), rng, action queue (`queue`, `write_pending` / `read_pending`); `step`; `state_hash` / `write_state` / `read_state`; `PlayerId`, `SimEvent`. Determinism tests in `sim/tests.rs` |
| `bytes.rs` | `ByteWriter` / `ByteReader` (little-endian canonical encoding of core state; each type has a `write_state` and a `read_state`; `item` reads the layout of the reader's save `version`), `fnv1a` |
| `save.rs` | Save file: header (magic, `SAVE_VERSION`, `WORLDGEN_VERSION`), seed, core, bodies, loose items; `save_bytes` / `from_save` with player-readable refusals; older versions back to `OLDEST_VERSION` load through `ByteReader::version`. Tests in `save/tests.rs` (with the committed `v1.ocworld` and `v9.ocworld` fixtures) |
| `action.rs` | `Action` enum and `Sim::apply`: join, leave, break, place, take contents, machine settings, set research, craft (refused while research locks the recipe), inventory clicks, select, drop, pick up, give |
| `action/codec.rs` | `Action::write` / `read` (a tag byte, then the fields; damaged bytes give `None`), `is_peer_input` |
| `net/mod.rs` | Co-op lockstep: `Role` (`Solo`, `Host`, `Client`), `route` (where an action goes), host frames (`end_tick`, `host_stamp`, `take_frames`), client outbox and confirmed tick (`take_outbox`, `push_frames`, `catch_up`), `become_client`, checksums every `CHECKSUM_TICKS`, `INPUT_DELAY`. Tests wire a host and a client `Game` through bytes (joining mid-run, leaving and returning, a full world) |
| `net/snapshot.rs` | Join snapshots: the save bytes plus the queued actions (`snapshot_bytes`, `read_snapshot`); `resync_from` (a client replaces its core in place, keeping its body and loaded chunks: `World::adopt_loaded`) |
| `net/players.rs` | What players see of each other: `Role::drives` (which machine moves a body), `set_peer`, `net_tick` (states 20 a second, the host's item views 10), `take_states`, `host_state_of`, `push_body_states` (clients add and drop bodies to match) |
| `net/items.rs` | Host-owned loose items: `write_item_views` / `item_view_for` (items near each peer), a client's `ItemView` (`push_item_view`, glided by id), `write_item_instances` |
| `avatars.rs` | Other players drawn as body, head and visor boxes, glided towards their bodies; name-tag anchors (`labels`); hidden at the camera |
| `authority.rs` | Every player's body (`step_bodies`: physics, falling out of the world, only for bodies this machine moves), `stream_around_players`, loose items and pickups for the nearest player (`step_items`), `throw`, `spawn_item` (never on a client), `join` (by key, up to `MAX_PLAYERS`; a returning key comes back where it left) / `leave` |
| `events.rs` | `Game::handle_sim_events`: SimEvents → item spawns (drops, throws), sounds, the local player's toasts |
| `api/mod.rs` | The JS-facing API, one `#[wasm_bindgen] impl Game` block per file; methods only forward |
| `api/input.rs` | Movement, look, mining/using, hotbar selection, fly toggle, drop |
| `api/render.rs` | Streaming work (`begin_work`, `work_step`), mesh/unload events, camera, box instances, name-tag anchors (`label_ptr`, `label_count`), sound events, textures |
| `api/inventory.rs` | Inventory screen: slots, cursor stack, `close_inventory`, pickup notifications |
| `api/machine.rs` | Machine panels: `take_panel_request`, `machine_panel` (flat view), machine recipes, panel buttons (set recipe, set filter, put in, take); box screens (`box_slots`, `click_box`, `store_slot`) |
| `api/crafting.rs` | Recipe queries (`recipe_locked_by`) and `craft` |
| `api/research.rs` | Research screen: the tech table (`tech_*`), progress, `current_research`, `set_research` |
| `api/content.rs` | Block names and sound materials, `item_name`, `item_icon` (texture layers and box proportions), `hand_yield`, `miner_recovery` |
| `api/hud.rs` | Player flags, target and `target_detail`, mining progress, onboarding hints (`hint_*`), stats counters |
| `api/save.rs` | `save`, `load` (static), `seed`, `play_seconds` |
| `api/net.rs` | `start_host`, `start_client`, `from_snapshot`, `resync`, `is_client`, `host_join` / `host_leave`, `snapshot`, `host_stamp`, `take_frames`, `take_checksums`, `take_outbox`, `push_frames`, `local_player`, `take_states`, `host_state`, `push_states`, `take_item_view`, `push_items`, `core_tick`, `confirmed_tick` |
| `api/debug.rs` | `give`, `teleport`, `add_player` / `remove_player`, `state_hash`, `debug_desync` (breaks this core, for resync tests), `run_ticks`, `skip_time`, `find_deposit`, `block_at`, `player_x/y/z` |
| `interaction.rs` | Local player's hands and feet: targeting, mining timer (queues `BreakBlock`), `right_click_action`, `play`, footsteps |
| `block.rs` | Block ids, `DEFS` table, texture layers `tex` (item textures too), sound materials, lookup tables |
| `item.rs` | `ItemId` (ids below 256 are the blocks, others start at 256), the item table (`def`, `name`, `stack_size`, `places`), ingots, parts, science packs |
| `hints.rs` | Onboarding hints `HINTS` (text plus a check on the player's inventory and the factory), `progress` (read-only) |
| `research.rs` | Tech tree `TECHS` (data: prerequisites, packs per unit, units, seconds, unlocked recipes), `PACKS`, `Research` (core state the factory owns: current tech, units done; `state`, `locked_by`, `add_unit`) |
| `chunk.rs` | 32³ block storage; uniform chunks cost no heap |
| `world/mod.rs` | Loaded chunks, edits (`saved` keeps edited chunks), block accessors (`*_anywhere` for core code), render events, `adopt_loaded` (a resync keeps the render cache) |
| `world/streaming.rs` | Streaming and meshing: re-centring on the local player and within `OTHERS_RADIUS` of the others (meshing only the local player's), generation and mesh queues, `work_step`, `remesh`, `area_ready` |
| `worldgen/mod.rs` | Terrain heights, surface, caves, trees; per-column cache; `generate(chunk)`; `WORLDGEN_VERSION` |
| `worldgen/ore.rs` | Deposit seeding (outcrops, veins, lodes), stamping, `deposit_at` ownership, `find_deposit`, `deposit_by_key` |
| `deposits.rs` | Deposit geometry, tiers, pooled reserves, draw caps, taper, spent rock, `HAND_YIELD`; `owner_of` and `DepositState::survey` (read-only queries) |
| `factory/mod.rs` | Machine table (`Kind`, `MACHINES`: one row per block, Kind-ordered rows first, then extra blocks sharing a kind; `machine`), the `Machine` trait every kind implements, `Factory`: a `Vec` per kind, position index `at`, `count(kind)`, the world's `research`, `place` / add / remove, `update` (one tick: miners, boxes, smelters, power balance, powered machines and labs, belts; emits `SimEvent`s) |
| `factory/state.rs` | `Factory::write_state` / `read_state`: every machine list, kind by kind (older save versions skip later kinds), then the deposits and the research |
| `factory/lab.rs` | Research lab: one buffer slot per science pack, `step_labs` (units for the current tech, never more than it has left, at its grid's speed); bytes, readout, panel, model |
| `factory/power.rs` | Power: `Pole` (a machine), `Power` (derived in `relink`: pole grids by wire range, the pole each generator and machine hangs on; `balance` each tick: demand, generators burn in order until supply meets it, speed per grid), wire drawing; power constants |
| `factory/generator.rs` | Coal generator: fuel buffer, burns `FUELS` only while its grid needs power; bytes, readout, panel, model |
| `factory/belt_shape.rs` | Belt `Shape`s (flat, ramp up/down, lift, underpass entry/exit): where items ride (`item_at`, `shows`), shape models, `UNDERPASS_RANGE` |
| `factory/buffer.rs` | `Buffer`: the item stacks a machine holds (box slots, miner output, processing buffers); `feed` pushes into belts leading away |
| `factory/belt.rs` | Belt items, spacing, `accept`, `belt_step` (each belt at its own `speed`: fast belts double); its bytes, readout and model; belt constants |
| `factory/miner.rs` | Miners Mk1 and Mk2 (one kind; `mk2` sets rate, recovery and power): `step` (draw, push out, `MinerWorking` events); its bytes, readout and model; miner constants |
| `factory/storage.rs` | Storage box: `step` (feeds belts leading away); its bytes and readout |
| `factory/router.rs` | Splitter and filter (one `Router` kind): holds one item, passes it front/left/right (round robin; a filter sends its item front, others aside); bytes, readout, filter panel, model |
| `factory/smelter.rs` | Smelter: sorts arriving ore and fuel, batches from `MACHINE_RECIPES`, burns `FUELS`, feeds belts leading away; bytes, readout, panel, model with status lamp |
| `factory/constructor.rs` | Constructor: one input into parts with the recipe chosen in its panel; `set_recipe` hands inputs back; bytes, readout, panel, model |
| `factory/panel.rs` | What a player does to a machine by hand: `panel` (view: status, progress, buffers by role, filter item), `box_slots`, `set_recipe`, `set_filter`, `insert`, `wants`, `take_contents` |
| `factory/links.rs` | Where items go: `Slot`, `Link`, `Sinks` (machines that take items), `deliver`; `relink`: belt outputs for every shape, corners, lift stacks, machine outputs, downstream-first belt order (derived data) |
| `factory/render.rs` | Box instance format (`INSTANCE_FLOATS`, `push_box`); `write_instances` asks nearby machines for models |
| `factory/describe.rs` | `Factory::describe` (one `match` on `Slot`), `fmt_int`, `fmt_duration` |
| `entities.rs` | Dropped items: ids, physics, magnet pickup by the nearest `Collector` with room, instances (`push_item_box`) |
| `inventory.rs` | 36 slots, cursor stack, click / quick-move, `add_to_slots` (shared with boxes) |
| `recipes.rs` | Hand-crafting recipes (`RECIPES`), machine recipes (`MACHINE_RECIPES`, saved by index: append only), `FUELS` burn times |
| `player.rs` | Character controller (walk, sprint, crouch, jump, fly) |
| `physics.rs` | Swept AABB collision against the voxel grid |
| `raycast.rs` | Voxel traversal for targeting |
| `mesher.rs` | Greedy mesher with AO; packed `u32` vertex format |
| `textures.rs` | Procedural 16×16 textures, one layer per `block::tex` constant; nature, ore and avatar patterns |
| `textures/machines.rs` | Machine and item texture patterns (belts, miner, smelter, constructor, routers, generator, pole, ingots, parts) |
| `noise.rs` | Seeded Perlin noise + fBm |
| `math.rs` | `Vec3`, `IVec3`, hashes, deterministic `Rng`, `sort_small_by_key` |
| `sound.rs` | Sound event buffer read by the host |
| `tests.rs` | Game-level scenario tests (mining, placing, sounds, miners, a smelter line, crafting, a second player, frame-rate independence); helpers shared with `save/tests.rs` |

## Web host: `web/src` (TypeScript + WebGL2, thin platform layer)

| Module | Owns |
|---|---|
| `main.ts` | Bootstrap (opens the latest world, or co-op through `startCoop`), pause menu wiring, the frame loop (`advance`: everything but drawing, which the co-op ticker also calls; `frame`: advance, then draw and HUD); imports `base.css` and `ui/menu.css` |
| `net/transport.ts` | `Transport`: send, `onMessage`, `onClose`, close |
| `net/broadcast.ts` | Tabs on one machine over a `BroadcastChannel` per room: `listenBroadcast` (host), `connectBroadcast` (client) |
| `net/protocol.ts` | Co-op messages (hello, welcome, refuse, actions, frames, checksum, bye, state, states, items, names, ping, pong, resync, snapshot): `encode` / `decode`; `BUILD_ID` |
| `net/coop.ts` | The `Coop` interface the page sees (`pump` after every `update`, `close`, `name`, `players`, `onNotice`, `onEnd`), `PlayerInfo`, `PING_MS`, `SILENT_MS` |
| `net/session.ts` | Starting sessions: `startCoop` (URL: `?host`, `?join=`, `?relay`; a room code goes over WebRTC, a room name over BroadcastChannel), `hostWorld` (the menu), `inviteLink`, `roomOf`; player key and name in localStorage |
| `net/host.ts` | `CoopHost`: welcomes or refuses, stamps actions, takes body states; sends frames, checksums, states, each peer's items, names and pings; answers resyncs; drops silent peers |
| `net/client.ts` | `CoopClient`: join, outbox, own state, pongs, checksum compare; resyncs on a mismatch or 5 s of lag; ends on bye, close or silence |
| `net/ticker.ts` | `tickWhenStalled`: a tiny worker timer that runs frames without drawing while the frame loop is stopped (hidden tab, minimised window), so a co-op host keeps ticking |
| `net/signal.ts` | The deployed signalling Worker: `SIGNAL_URL`, `newRoom`, `fetchIce`, `openRoom` (WebSocket), `closeReason`, `isRoomCode` |
| `net/webrtc.ts` | Machines over WebRTC: `hostWebRtc` (room code, one data channel per joiner), `joinWebRtc`; the channel `Transport` splits messages into 16 KB pieces |
| `save/store.ts` | IndexedDB: `worlds` records (`WorldMeta`) and `saves` bytes in two slots per world (newest + backup); gzip `pack` / `unpack` |
| `save/session.ts` | `openWorld` (latest world, backup fallback, `?seed=`), `Session` (autosave: every minute, on pause, hide and close; never for a co-op client), `switchTo` (save, then reload into another world), `soloUrl` (this page without co-op parameters) |
| `base.css` | Theme variables, reset, focus rings, shared `.hidden`, `.secondary-btn`, `.close-btn` |
| `input.ts` | Keyboard/mouse, pointer lock, held state and one-shot `Action`s |
| `render/renderer.ts` | Chunk meshes (culling, opaque + cutout passes, fog), target outline, mining crack, `project` (camera-relative point to CSS pixels) |
| `render/boxes.ts` | Instanced box pipeline (items, belt items, machine parts); `INSTANCE_FLOATS` |
| `render/shaders.ts` | GLSL sources |
| `render/gl.ts`, `render/mat4.ts` | Program/uniform helpers; matrix and frustum helpers |
| `ui/dom.ts` | `h()` and `button()` element helpers |
| `ui/hud.ts` + `.css` | Crosshair, target readout, mining bar, hotbar, toasts, debug overlay, `itemIcon` (isometric box from `item_icon`) |
| `ui/inventory.ts` + `.css` | Inventory and build screen (E); opened on a box (`open([x, y, z])`), the box screen: its slots above the inventory, Take all |
| `ui/machine.ts` + `.css` | Machine panel (right-click a smelter, constructor, filter, generator or lab): status, progress, buffers, recipe choice, filter item, put-in and take buttons |
| `ui/nametags.ts` + `.css` | Name tags over other players, from the engine's anchors and the session's names |
| `ui/coop.ts` + `.css` | "Play together" in the menu: name, host this world (code and link), join from a link or code, players and ping, leave, why a session ended; `playerRow` |
| `ui/players.ts` + `.css` | In game: the player list while Tab is held, join and leave notices |
| `ui/hints.ts` + `.css` | Onboarding tip card in the HUD (the first hint not done or skipped); H skips, skipped tips in localStorage; "Show tips again" in the menu |
| `ui/research.ts` + `.css` | Research screen (R): a card per tech (state, unlocks, cost, progress, choose); HUD tracker and "research done" notice |
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

## Signalling Worker: `signal/` (Cloudflare, co-op)

| File | Owns |
|---|---|
| `src/index.ts` | Routes: `POST /room` (a room code), `GET /room/<code>` (WebSocket into the room), `GET /ice` (STUN plus short-lived TURN credentials); origin check |
| `src/room.ts` | `Room` Durable Object: relays offers, answers and candidates between the host and joiners (hibernating WebSockets; JSON protocol and close codes in its header) |
| `wrangler.jsonc`, `package.json` | Worker config (bindings, `ALLOWED_ORIGINS`); wrangler lives only here. Deploying: `docs/WORKFLOW.md` section 4 |
| `smoke.mjs` | End-to-end check against a local or deployed Worker |

## Scripts and CI

| File | Purpose |
|---|---|
| `scripts/build-wasm.mjs` | wasm-pack build into `web/src/wasm` (`--dev` for a debug build) |
| `scripts/dev.mjs` | Vite dev server plus a Rust watcher (honours `PORT`) |
| `scripts/check.mjs` | `npm run check`: fmt, clippy `-D warnings`, tests, tsc (web and `signal/`), size budgets; quiet output |
| `scripts/check-size.mjs` | Line budgets from DEV_PLAN section 3.1 |
| `scripts/wasm-sizes.mjs` | The largest wasm functions by name, to find what made the wasm grow |
| `.github/workflows/pages.yml` | CI on push to `main`: build wasm, check, bundle, deploy to GitHub Pages |

## How to add…

**A block.** `block.rs`: append an id constant (never renumber), bump `BLOCK_COUNT`, add a `DEFS` row
(`cube`, `ore` or `machine` helper). New texture: a `tex` constant plus its arm in `textures::pixel`.
Placeable blocks work at once; worldgen use goes in `worldgen/`.

**An item or recipe.** A block is already an item. Any other item: an id constant (from 256, append only)
and a row in `item.rs` `EXTRA`, with a texture layer in `block::tex` and its pattern in `textures::pixel`
if it needs a new look; the HUD icon and the loose and belt models follow from the row. A recipe is a row
in `recipes.rs`; the build menu shows every row. To lock it behind research, list its output in a tech's
`unlocks` (`research.rs`).

**A tech.** A row appended to `TECHS` in `research.rs` (saves store progress by index): name, blurb,
prerequisites by index, packs per unit, units, seconds, unlocked items. The research screen and the build
menu follow. A new science pack: an item, a hand recipe and an entry in `PACKS` (labs get a slot for it).

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
`Action` variant, its arm in `Sim::apply_to_player` (`action.rs`) and its bytes in `action/codec.rs`; `Game` queues it with `act` (local
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
| `net/mod.rs` | `INPUT_DELAY` (ticks from stamping an action to applying it), `CHECKSUM_TICKS` |
| `net/players.rs`, `net/items.rs` | `STATE_TICKS` (body states), `ITEM_TICKS`, item `VIEW_RADIUS`, glide rates |
| `avatars.rs` | Avatar sizes, `LABEL_RANGE`, `GLIDE_RATE`; `world/streaming.rs`: `OTHERS_RADIUS` |
| `authority.rs` | `PHYSICS_SUBSTEPS` (per tick), `FALL_LIMIT`, `MAX_PLAYERS` |
| `deposits.rs` | `HAND_YIELD`, `TAPER_START`, `TAPER_FLOOR`; `Tier::grade`, `Tier::draw_cap` |
| `factory/mod.rs` | `MACHINES` (buffer slots per machine) |
| `factory/miner.rs` | `MINER_RATE`, `MINER_RECOVERY`, `MK2_RATE`, `MK2_RECOVERY` |
| `recipes.rs` | `MACHINE_RECIPES` (seconds per batch), `FUELS` (burn seconds) |
| `factory/belt.rs` | `BELT_SPEED`, `FAST_BELT_SPEED`, `ITEM_SPACING` |
| `factory/power.rs` | `GENERATOR_POWER`, `MINER_MK2_POWER`, `CONSTRUCTOR_POWER`, `ROUTER_POWER`, `LAB_POWER`, `WIRE_RANGE`, `POLE_REACH` |
| `hints.rs` | Onboarding hints `HINTS` (text plus a check on the player's inventory and the factory), `progress` (read-only) |
| `research.rs` | `TECHS` (units, seconds, packs per unit) |
| `worldgen/ore.rs` | `ORE_GEN`, `LODE_CHANCE`, `ORE_SPAWN_CLEARING` |
| `recipes.rs` | `RECIPES` (hand) |
| `interaction.rs` | `REACH`, place repeat, break cooldown, footstep stride |
| `events.rs` | `MINER_SOUND_RANGE`, drop pickup delay |
| `player.rs` | Movement speeds, jump, gravity |
