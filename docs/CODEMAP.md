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
| `sim.rs` | The deterministic core `Sim`: tick, world, factory, `players` (`Option<PlayerCore>` per `PlayerId`: inventory, key), `away` (players who left with a key), rng, block `timers` (`sim/timers.rs`), flowing `water` checks (`sim/water.rs`), action queue (`queue`, `write_pending` / `read_pending`); `step`; `state_hash` / `write_state` / `read_state`; `PlayerId`, `SimEvent`. Determinism tests in `sim/tests.rs` |
| `sim/timers.rs` | Block timers (core state): `BlockTimers` (sorted pending changes, capped), `Sim::block_changed` (starts them after a block changes, then the water checks), `run_timers` (after each tick's actions): leaf decay, grass spreading and dying, sapling growth |
| `sim/water.rs` | Flowing water (core state): `WaterQueue` (checks in order, one per cell, capped), `water_changed` (schedules checks near a change), `run_water` (at most `MAX_WATER_UPDATES` a tick), the rules (`water_rule`: the sea refills at or below `SEA_LEVEL`, falling water, levels 7 to 1) |
| `sim/saplings.rs` | Saplings: the leaf drop chance, where they can be planted, growth into a tree (`worldgen::tree_blocks`) |
| `daytime.rs` | Time of day from the core tick (`DAY_TICKS`: a 20-minute day, a new world starts at 7:00): `time_of_day`, `day_number`; no state of its own |
| `bytes.rs` | `ByteWriter` / `ByteReader` (little-endian canonical encoding of core state; each type has a `write_state` and a `read_state`; `item` reads the layout of the reader's save `version`), `fnv1a` |
| `save.rs` | Save file: header (magic, `SAVE_VERSION`, the world's generator version), seed, core, bodies, loose items; `save_bytes` / `from_save` with player-readable refusals; older versions back to `OLDEST_VERSION` load through `ByteReader::version`. Tests in `save/tests.rs` (with the committed `v1.ocworld` and `v9.ocworld` fixtures) |
| `action.rs` | `Action` enum and `Sim::apply`: join, leave, break, place, take contents, machine settings (recipe, filter, quarry), set research, craft (refused while research locks the recipe), inventory clicks, select, drop, pick up, give, rotate (R) |
| `action/codec.rs` | `Action::write` / `read` (a tag byte, then the fields; damaged bytes give `None`), `is_peer_input` |
| `net/mod.rs` | Co-op lockstep: `Role` (`Solo`, `Host`, `Client`), `route` (where an action goes), host frames (`end_tick`, `host_stamp`, `take_frames`), client outbox and confirmed tick (`take_outbox`, `push_frames`, `catch_up`), `become_client`, checksums every `CHECKSUM_TICKS`, `INPUT_DELAY`. Tests wire a host and a client `Game` through bytes (joining mid-run, leaving and returning, a full world) |
| `net/snapshot.rs` | Join snapshots: the save bytes plus the queued actions (`snapshot_bytes`, `read_snapshot`); `resync_from` (a client replaces its core in place, keeping its body and loaded chunks: `World::adopt_loaded`) |
| `net/players.rs` | What players see of each other: `Role::drives` (which machine moves a body), `set_peer`, `net_tick` (states 20 a second, the host's item views 10), `take_states`, `host_state_of`, `push_body_states` (clients add and drop bodies to match) |
| `net/items.rs` | Host-owned loose items: `write_item_views` / `item_view_for` (items near each peer), a client's `ItemView` (`push_item_view`, glided by id), `write_item_instances` |
| `avatars.rs` | Other players drawn as body, head and visor boxes, glided towards their bodies; name-tag anchors (`labels`); hidden at the camera |
| `authority.rs` | Every player's body (`step_bodies`: physics, falling out of the world, only for bodies this machine moves), `stream_around_players`, loose items and pickups for the nearest player (`step_items`), `throw`, `spawn_item` (never on a client), `join` (by key, up to `MAX_PLAYERS`; a returning key comes back where it left) / `leave` |
| `events.rs` | `Game::handle_sim_events`: SimEvents → item spawns (drops, throws), sounds, the local player's toasts |
| `api/mod.rs` | The JS-facing API, one `#[wasm_bindgen] impl Game` block per file; methods only forward |
| `api/input.rs` | Movement, look, mining/using, hotbar selection, fly toggle, drop, `rotate_target`, `cancel_line` |
| `api/render.rs` | Streaming work (`begin_work`, `work_step`), mesh/unload events, camera, `time_of_day` / `day_number`, box instances, name-tag anchors (`label_ptr`, `label_count`), `quarry_cracks`, sound events, textures |
| `api/inventory.rs` | Inventory screen: slots, cursor stack, `close_inventory`, pickup notifications |
| `api/machine.rs` | Machine panels: `take_panel_request`, `machine_panel` (flat view), machine recipes, panel buttons (set recipe, set filter, put in, take); the quarry's part (`quarry_panel`, `quarry_found`, `quarry_box`, size and depth choices, `set_quarry`); box screens (`box_slots`, `click_box`, `store_slot`) |
| `api/crafting.rs` | Recipe queries (`recipe_locked_by`, `craftable_times`, `recipe_group` and group names) and `craft` |
| `api/research.rs` | Research screen: the tech table (`tech_*`), progress, `current_research`, `set_research` |
| `api/content.rs` | Block names and sound materials, `item_name`, `item_icon` (single box), `item_model` (manufactured item box parts), `tool_uses`, `hand_yield`, `miner_recovery` |
| `api/hud.rs` | Player flags, target and `target_detail`, mining progress, onboarding hints (`hint_*`), stats counters, belt line outlines and label (`line_cells`, `line_label`, which also describes a quarry about to be placed), `placement_box` |
| `api/save.rs` | `save`, `load` (static), `seed`, `play_seconds` |
| `api/net.rs` | `start_host`, `start_client`, `from_snapshot`, `resync`, `is_client`, `host_join` / `host_leave`, `snapshot`, `host_stamp`, `take_frames`, `take_checksums`, `take_outbox`, `push_frames`, `local_player`, `take_states`, `host_state`, `push_states`, `take_item_view`, `push_items`, `core_tick`, `confirmed_tick` |
| `api/minimap.rs` | `minimap_redraw` (only when needed), `minimap_ptr` / `minimap_size` (the RGBA image), `minimap_players`, `minimap_marks` / `minimap_mark_fields`, `known_deposits` / `set_known_deposits` |
| `api/prospect.rs` | The latest prospecting reading (`prospect_seq`, `prospect_kind`, `prospect_records`, `prospect_fields`, `prospect_origin`), `held_device`, `scan_range`, `deposit_label` |
| `api/debug.rs` | `give`, `teleport`, `add_player` / `remove_player`, `state_hash`, `debug_desync` (breaks this core, for resync tests), `run_ticks`, `skip_time`, `find_deposit`, `block_at`, `player_x/y/z` |
| `interaction.rs` | Local player's hands and feet: targeting, mining timer (queues `BreakBlock`), `right_click_action`, `play`, footsteps |
| `quarry_preview.rs` | Placing a quarry: the box a held quarry would dig (`quarry_preview`), its HUD label, R turns it (`quarry_turn`, used by `right_click_action`) |
| `belt_line.rs` | Drag-to-build belt lines: `plan` (longer axis first, one turn, follows one-block steps), ghost-belt preview, builds by queuing `PlaceBlock`s a few per tick |
| `block/mod.rs` | Block ids, `DEFS` table, sound materials, lookup tables (`FACE_TEX`, `ALT_TEX` alternates), flowing water ids (`flow`, `flow_level`) |
| `block/tex.rs` | Texture array layers (item textures too); `alternates` / `look`: three extra looks for ores, limestone, leaves |
| `item.rs` | `ItemId` (ids below 256 are the blocks, others start at 256), the item table (`def`, `name`, `stack_size`, `places`), ingots, parts, science packs |
| `item_models.rs` | Shared small box assemblies (parts, tools, scanner, core drill) for loose items and HUD icons; data only |
| `hints.rs` | Onboarding hints `HINTS` (text plus a check on the player's inventory and the factory), `progress` (read-only) |
| `research.rs` | Tech tree `TECHS` (data: prerequisites, packs per unit, units, seconds, unlocked recipes), `PACKS`, `Research` (core state the factory owns: current tech, units done; `state`, `locked_by`, `add_unit`) |
| `chunk.rs` | 32³ block storage; uniform chunks cost no heap |
| `world/mod.rs` | Loaded chunks, edits (`saved` keeps edited chunks), block accessors (`*_anywhere` for core code, with a small cache of generated chunks; `set_block_anywhere_later` remeshes in the streaming budget), render events, `adopt_loaded` (a resync keeps the render cache) |
| `world/streaming.rs` | Streaming and meshing: re-centring on the local player and within `OTHERS_RADIUS` of the others (meshing only the local player's), generation and mesh queues, `work_step`, `remesh`, `area_ready` |
| `worldgen/mod.rs` | Terrain heights, surface, caves, trees; per-column cache; `generate(chunk)`; generator versions (`WORLDGEN_VERSION` is the newest; a world keeps its own `WorldGen::version`; each released version is pinned by `released_versions_never_change`) |
| `worldgen/biome.rs` | Version 2: `Biome` per column (`biome_at`), its rock, surface (`surface_v2`) and tree density |
| `worldgen/geology.rs` | Version 2 deposit seeding: version 1's counts, each ore drawn from its biome's weights (`ORES_BY_BIOME`); surface hints (`stain_surface`, `hint_for`) |
| `worldgen/water.rs` | Version 3 water: sea below `SEA_LEVEL` (62), ponds (`Pond`, one per 96-block cell, a bowl and a bank shaped into `height_at`), `water_top` per column, `surface_v3` (sand under water and on shores), `WaterGuard` (no caves within 2 blocks of water) |
| `worldgen/caves.rs` | Spaghetti caves (`CaveField`), shared by every version |
| `worldgen/strata.rs` | Version 3 deposit seeding: rare exposed outcrops on bare rock (`bare_rock`), depth bands (`ORE_DEPTH`), the starter set near spawn (`starter_outcrops`) |
| `worldgen/ore.rs` | Deposit seeding (outcrops, veins, lodes), stamping, `deposit_at` ownership, `find_deposit`, `deposit_by_key` |
| `deposits.rs` | Deposit geometry, tiers, pooled reserves, draw caps, taper, spent rock, `HAND_YIELD`; `owner_of` and `DepositState::survey` (read-only queries) |
| `factory/mod.rs` | Machine table (`Kind`, `MACHINES`: one row per block, Kind-ordered rows first, then extra blocks sharing a kind; `machine`), the `Machine` trait every kind implements, `Factory`: a `Vec` per kind, position index `at`, `count(kind)`, the world's `research`, `place` / add / remove, `update` (one tick: miners, quarries, boxes, smelters, power balance, powered machines and labs, belts; emits `SimEvent`s) |
| `factory/state.rs` | `Factory::write_state` / `read_state`: every machine list, kind by kind (older save versions skip later kinds), then the deposits and the research |
| `factory/lab.rs` | Research lab: one buffer slot per science pack, `step_labs` (units for the current tech, never more than it has left, at its grid's speed); bytes, readout, panel, model |
| `factory/power.rs` | Power: `Pole` (a machine), `Power` (derived in `relink`: pole grids by wire range, the pole each generator and machine hangs on; `balance` each tick: demand, generators give stored energy in order until supply meets it, lighting fuel as needed, speed per grid), wire drawing; power constants |
| `factory/generator.rs` | Coal generator: fuel buffer and stored energy (kW·ticks, lit from `recipes::fuel_energy`), gives only what its grid draws; bytes, readout, panel, model |
| `factory/belt_shape.rs` | Belt `Shape`s (flat, ramp up/down, lift, underpass entry/exit): where items ride (`item_at`, `shows`), shape models, `UNDERPASS_RANGE`; `derive_slopes` makes flat belts ramps from their neighbours (in `relink`) |
| `factory/buffer.rs` | `Buffer`: the item stacks a machine holds (box slots, miner output, processing buffers); `feed` pushes into belts leading away |
| `factory/belt.rs` | Belt items, spacing, `accept`, `belt_step` (each belt at its own `speed`: fast belts double); its bytes, readout and model; belt constants |
| `factory/miner.rs` | Miners Mk1 and Mk2 (one kind, both powered; `mk2` sets rate, recovery and power): `step` (draw, push out, `MinerWorking` events); its bytes, readout and model; miner constants |
| `factory/storage.rs` | Storage box: `step` (feeds belts leading away); its bytes and readout |
| `factory/router.rs` | Splitter and filter (one `Router` kind): holds one item, passes it front/left/right (round robin; a filter sends its item front, others aside); bytes, readout, filter panel, model |
| `factory/smelter.rs` | Smelter: sorts arriving ore and fuel, batches from `MACHINE_RECIPES`, burns `FUELS`, feeds belts leading away; bytes, readout, panel, model with status lamp |
| `factory/constructor.rs` | Constructor: one input into parts with the recipe chosen in its panel; `set_recipe` hands inputs back; bytes, readout, panel, model |
| `factory/panel.rs` | What a player does to a machine by hand: `panel` (view: status, progress, buffers by role, filter item), `box_slots`, `set_recipe`, `set_filter`, `insert`, `wants`, `take_contents` |
| `factory/links.rs` | Where items go: `Slot`, `Link`, `Sinks` (machines that take items), `deliver`; `relink`: belt outputs for every shape, corners, lift stacks, machine outputs, downstream-first belt order (derived data) |
| `factory/render.rs` | Box instance format (`INSTANCE_FLOATS`, `push_box`); `write_instances` asks nearby machines for models; `map_machines` (positions for the minimap's marks) |
| `factory/pipes.rs` | Pipework (one kind, `Kind::Pipe`: `Part` pump, pipe, outlet): networks and arms (`link_pipework`, from `relink`), bytes, readouts, models |
| `factory/pumping.rs` | Moving water each tick (`step_pipework`): pumps lift the highest, farthest source in reach; outlets pour where the water lands; block edits go to `Factory.changed` for the water rules |
| `factory/quarry.rs` | Quarry (`Kind::Quarry`): digs its box one block at a time (`step`: `seek` past air and non-ground, noting ore in `found`; waits when flooded, full, unpowered or paused), output buffer fed like a miner's, `set` (panel choices), bytes, readout, panel, `quarry_cracks` |
| `factory/quarry/dig_box.rs` | `DigBox` (which cells, in what order), `WIDTHS` / `DEPTHS` choices, `QUARRIABLE`, `survey` (what is left, for the panel and the preview) |
| `factory/quarry/model.rs` | The quarry's model: housing and lamp, corner posts, rails, a gantry that travels row to row, a spinning drill on the block being dug |
| `factory/describe.rs` | `Factory::describe` (one `match` on `Slot`), `fmt_int`, `fmt_duration` |
| `entities.rs` | Dropped items: ids, physics (floating up through water, drifting), magnet pickup by the nearest `Collector` with room, instances (`push_item_box`) |
| `inventory.rs` | 36 slots, cursor stack, click / quick-move, `add_to_slots` (shared with boxes) |
| `tools.rs` | Hand tools: `ToolKind`, `Tier` (uses, speed, ore kept), `TOOLS`; `tool_for` (by the block's sound material), `break_speed` (the hands), `ore_yield` (the core), `device` (scanner, core drill). A tool's stack count is its uses left |
| `prospect.rs` | Scanner and core drill (queries, never actions): `scan`, `core_sample`, `update_prospecting` (called from `update_placing`), `Prospect` (the latest reading as flat records, timers) |
| `recipes.rs` | Hand-crafting recipes (`RECIPES`, each in a build-menu `Group`), machine recipes (`MACHINE_RECIPES`, saved by index: append only), `FUELS` (smelter seconds and generator kJ) |
| `player.rs` | Character controller (walk, sprint, crouch, jump, swim, fly); `in_water`, `splash_speed` for sounds |
| `physics.rs` | Swept AABB collision against the voxel grid |
| `raycast.rs` | Voxel traversal for targeting |
| `light.rs` | Sky and block light (0–15) for a chunk being meshed (water dims it 2 per block): a field of the chunk plus a 15-block margin, sky columns shaded by the chunks above, BFS flood; `CLASS` says how each block treats light. Render cache only |
| `mesher.rs` | Greedy mesher with AO and smoothed per-vertex light (a byte per vertex after the `u32` vertices); ranges opaque, cutout, liquid (liquid AO bits mark the water line and how low it sits: `water_line`); plants as crossed quads (faces 6 and 7); `pick_layer` picks one of four looks per block |
| `mesher/quad.rs` | Per-corner AO and light, the merge key, `emit_quad` |
| `minimap.rs` | Minimap image (presentation only): top block and height per column cached per chunk column from loaded chunks (`touch` on mesh and unload events), shaded by the height step; other players' marks |
| `minimap/marks.rs` | Deposit and machine marks (presentation): `Known` (prospected veins and lodes, `remember` from `prospect.rs`, `export` / `import` for the browser's world record), `Minimap::marks` (flat records with colours; dry deposits left out) |
| `textures.rs` | Procedural 16×16 textures, one layer per `block::tex` constant; shared noise helpers and avatar patterns |
| `textures/nature.rs` | Alpine natural blocks: slate, pebbled earth, turf with blades, ragged grass edge, rippled sand, leaf clusters (4 looks), bark, bedrock; the four surface hints (`HINTS`) |
| `textures/paint.rs` | Painting helpers: palette ramps, wrapping cells, blobs kept inside the tile |
| `textures/ores.rs` | Coal lumps, iron nodules, copper crusts, quartz crystals, limestone fossils; four looks each on the slate host |
| `textures/items.rs` | Rod threads, screws, glass, science liquid, tool handle and steel for the item assemblies |
| `textures/tools.rs` | Flat tool pictures (shown on belts); the scanner screen and device casing materials |
| `textures/plants.rs` | Alpine sapling needles and stems on transparent crossed quads |
| `textures/geology.rs` | Granite, sandstone and basalt grains, and cutout glass |
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
| `save/store.ts` | IndexedDB: `worlds` records (`WorldMeta`, with the prospected deposits as `marks`) and `saves` bytes in two slots per world (newest + backup); gzip `pack` / `unpack` |
| `save/session.ts` | `openWorld` (latest world, backup fallback, `?seed=`), `Session` (autosave: every minute, on pause, hide and close; never for a co-op client), `switchTo` (save, then reload into another world), `soloUrl` (this page without co-op parameters) |
| `base.css` | Theme variables, reset, focus rings, shared `.hidden`, `.secondary-btn`, `.close-btn` |
| `input.ts` | Keyboard/mouse, pointer lock, held state and one-shot `Action`s |
| `render/renderer.ts` | Chunk meshes (culling, opaque + cutout passes, fog), target outline, mining crack, `project` (camera-relative point to CSS pixels) |
| `render/water.ts` | The liquid range drawn last (blended, back to front, both sides) and the underwater fog (`fogFor`, from `eye_in_water`) |
| `render/outlines.ts` | Overlays: mining cracks (the player's and quarries'), a dragged belt line's cells (`line_cells`), amber quarry boxes; the renderer's line and crack programs |
| `render/boxes.ts` | Instanced box pipeline (items, belt items, machine parts); `INSTANCE_FLOATS` |
| `render/sky.ts` | Day and night: `skyAt` (sun direction, sky and fog colours, daylight from the time of day), `clock`, `SkyPass` (full-screen gradient, sun, moon, stars) |
| `render/shaders.ts` | GLSL sources; the light curve (sky light × daylight, warm block light, `CAVE_FLOOR`); periodic world-anchored Alpine tint for terrain only (`TERRAIN_TINT_PERIOD`) |
| `render/gl.ts`, `render/mat4.ts` | Program/uniform helpers; matrix and frustum helpers |
| `ui/dom.ts` | `h()` and `button()` element helpers |
| `ui/hud.ts` + `.css` | Crosshair, target readout, mining bar, hotbar, toasts (a count of 0: a tool wore out), debug overlay, `itemIcon` (isometric box from `item_icon`), `showAmount` (a slot's count or a tool's wear bar) |
| `ui/inventory.ts` + `.css` | Inventory screen (E) with the build menu; opened on a box (`open([x, y, z])`), the box screen: its slots above the inventory, Take all |
| `ui/crafting.ts` + `.css` | Build menu: recipe tiles grouped by `recipe_group`, text search (output and material names), state filters (all, can craft, missing, locked), one hover info card; click crafts, Shift-click 5 |
| `ui/machine.ts` + `.css` | Machine panel (right-click a smelter, constructor, filter, generator, lab or quarry): status, progress, buffers, recipe choice, filter item, put-in and take buttons; `quarryBox` for the open quarry's outline |
| `ui/quarry.ts` | The quarry's part of the machine panel: size and depth choices, pause, layer and blocks dug and left, deposits uncovered |
| `ui/nametags.ts` + `.css` | Name tags over other players, from the engine's anchors and the session's names |
| `ui/coop.ts` + `.css` | "Play together" in the menu: name, host this world (code and link), join from a link or code, players and ping, leave, why a session ended; `playerRow` |
| `ui/players.ts` + `.css` | In game: the player list while Tab is held, join and leave notices |
| `ui/minimap.ts` + `.css` | Minimap in a round top-right frame: the engine's image (at most 4 redraws a second), deposit and machine marks on an overlay canvas, player arrows; N toggles (localStorage); `--map-space` moves other top-right HUD items down |
| `ui/hints.ts` + `.css` | Onboarding tip card in the HUD (the first hint not done or skipped); H skips, skipped tips in localStorage; "Show tips again" in the menu |
| `ui/prospect.ts` + `.css` | Prospecting card at the top left while a device is held: scan rows (arrows and distances follow the player) or a core sample's figures; calls `onReading` for the ping |
| `ui/research.ts` + `.css` | Research screen (T): a card per tech (state, unlocks, cost, progress, choose); HUD tracker and "research done" notice |
| `ui/menu.css` | Pause/start menu and "click to keep playing" hint styles (markup in `web/index.html`) |
| `ui/worlds.ts` + `.css` | World list in the menu: play, new world (name, seed), export / import `.ocworld`, delete |
| `ui/sound-lab.ts` + `.css` | Sound designer dialog (O): material tabs, Actions tab |
| `ui/sound-lab-footer.ts` + `.css` | Designer footer: volume, copy/paste/reset settings |
| `ui/volume-control.ts` + `.css` | Mute button + volume slider (menu and designer) |
| `ui/knob.ts` + `.css` | Rotary dial widget |
| `audio/settings.ts` | Sound design data: materials, actions, dials, presets, `DEFAULT_DESIGN`, persistence |
| `audio/synth.ts` | Procedural foley synthesis (dials → samples) |
| `audio/sound.ts` | Engine sound events → Web Audio voices, buffer cache, previews |
| `wasm/` | Generated by `npm run build:wasm` (gitignored); `engine.d.ts` is the API reference |
| `../art-preview.html`, `../art/scene.js`, `../art/materials.js` | Development-only art review page; fixed seed-2024 factory fixture using public actions and the production renderer, 3×3 material samples rendered through the terrain shader, optional raw tiles and synthetic render benchmark; excluded from the production entry |

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
in `recipes.rs` (its `group` picks the build-menu section); the build menu shows every row. To lock it behind research, list its output in a tech's
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
test), else raise `OLDEST_VERSION`. A change to world generation goes into a new generator version (`worldgen/mod.rs` header); released versions never change. The browser
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
| `recipes.rs` | `MACHINE_RECIPES` (seconds per batch), `FUELS` (smelter seconds, generator kJ) |
| `factory/belt.rs` | `BELT_SPEED`, `FAST_BELT_SPEED`, `ITEM_SPACING` |
| `factory/power.rs` | `GENERATOR_POWER`, `MINER_POWER`, `MINER_MK2_POWER`, `CONSTRUCTOR_POWER`, `ROUTER_POWER`, `LAB_POWER`, `PUMP_POWER`, `QUARRY_POWER`, `WIRE_RANGE`, `POLE_REACH` |
| `factory/pumping.rs` | `PUMP_RATE`, `PUMP_HOLD`, `OUTLET_RATE`, `PUMP_RANGE` |
| `factory/quarry.rs` | `DIG_SECONDS`, `SCAN_PER_TICK`; `quarry/dig_box.rs`: `WIDTHS`, `DEPTHS`, defaults, `QUARRIABLE` |
| `sim/water.rs` | `WATER_DELAY`, `MAX_WATER_UPDATES`; `SEA_LEVEL` is in `worldgen/mod.rs` |
| `sim/timers.rs` | `LEAF_HALF_LIFE`, `GRASS_GROW_HALF_LIFE`, `GRASS_DIE_HALF_LIFE`, `MAX_TIMERS`, leaf check radius and support steps |
| `sim/saplings.rs` | `SAPLING_CHANCE`, `GROW_MIN`, `GROW_HALF_LIFE`, trunk heights |
| `research.rs` | `TECHS` (units, seconds, packs per unit) |
| `worldgen/biome.rs` | `HIGHLAND_LEVEL`, `LOWLAND_LEVEL`, `SPAWN_CALM`, `DITHER`, thresholds in `biome_at`, `tree_factor` |
| `worldgen/geology.rs` | `ORES_BY_BIOME`, `OUTCROPS`, `VEIN_CHANCES`, `HINT_MARGIN`, `HINT_ONE_IN` (version 2) |
| `worldgen/water.rs` | Version 3 water: sea below `SEA_LEVEL` (62), ponds (`Pond`, one per 96-block cell, a bowl and a bank shaped into `height_at`), `water_top` per column, `surface_v3` (sand under water and on shores), `WaterGuard` (no caves within 2 blocks of water) |
| `worldgen/caves.rs` | Spaghetti caves (`CaveField`), shared by every version |
| `worldgen/strata.rs` | Version 3 deposit seeding: rare exposed outcrops on bare rock (`bare_rock`), depth bands (`ORE_DEPTH`), the starter set near spawn (`starter_outcrops`) |
| `worldgen/ore.rs` | `ORE_GEN`, `LODE_CHANCE`, `ORE_SPAWN_CLEARING` |
| `recipes.rs` | `RECIPES` (hand) |
| `interaction.rs` | `REACH`, place repeat, break cooldown, footstep stride |
| `tools.rs` | `STONE_TIER`, `IRON_TIER` (uses, break speed, ore kept), `DEVICE_TIER` |
| `prospect.rs` | `SCAN_RANGE`, `SCAN_COOLDOWN`, `DRILL_SECONDS`, `DRILL_REACH`, size bands |
| `events.rs` | `MINER_SOUND_RANGE`, drop pickup delay |
| `player.rs` | Movement speeds, jump, gravity |
