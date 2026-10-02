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
| `sim.rs` | The deterministic core `Sim`: tick, world, factory, `players` (`Option<PlayerCore>` per `PlayerId`: inventory, key, `crafts`), `away` (players who left with a key), rng, block `timers` (`sim/timers.rs`), flowing `water` checks (`sim/water.rs`), action queue (`queue`, `write_pending` / `read_pending`); `step`; `state_hash` / `write_state` / `read_state`; `PlayerId`, `SimEvent`. Determinism tests in `sim/tests.rs` |
| `sim/timers.rs` | Block timers (core state): `BlockTimers` (sorted pending changes, capped), `Sim::block_changed` (starts them after a block changes, then the water checks), `run_timers` (after each tick's actions): leaf decay, grass spreading and dying, sapling growth |
| `sim/water.rs` | Flowing water (core state): `WaterQueue` (checks in order, one per cell, capped), `water_changed` (schedules checks near a change), `run_water` (at most `MAX_WATER_UPDATES` a tick), the rules (`water_rule`: the sea refills at or below `SEA_LEVEL`, falling water, levels 7 to 1) |
| `sim/saplings.rs` | Saplings: the leaf drop chance, where they can be planted, growth into a tree (`worldgen::tree_blocks`) |
| `sim/torches.rs` | Torches: where one can stand (`torch_fits`: an empty cell on a solid block), dropping when its block goes (from `block_changed`) |
| `daytime.rs` | Time of day from the core tick (`DAY_TICKS`: a 20-minute day, a new world starts at 7:00): `time_of_day`, `day_number`; no state of its own; `sunlight(tick)` (thousandths, integer parabola between 6:00 and 18:00) for solar panels |
| `bytes.rs` | `ByteWriter` / `ByteReader` (little-endian canonical encoding of core state; each type has a `write_state` and a `read_state`; `item` reads the layout of the reader's save `version`), `fnv1a` |
| `save.rs` | Save file: header (magic, `SAVE_VERSION`, the world's generator version), seed, core, bodies, loose items; `save_bytes` / `from_save` with player-readable refusals; older versions back to `OLDEST_VERSION` load through `ByteReader::version`. Tests in `save/tests.rs` (with the committed `v1.ocworld` and `v9.ocworld` fixtures) |
| `action.rs` | `Action` enum and `Sim::apply`: join, leave, break, place, take contents, machine settings (recipe, filter, quarry), set research, craft (queued: `crafting.rs`), cancel craft, inventory clicks (shift-right-click moves every stack of an item: `QuickMoveAll`, `StoreAll`, `TakeAll`), sort (backpack or box), select, drop, pick up, give, rotate (R), mark and remove terraforming sites |
| `action/blocks.rs`, `codec.rs`, `multiblock.rs` | `blocks.rs`: what `BreakBlock` and `PlaceBlock` do (`break_block`, `place_block`, tool wear); `Action::write` / `read` (a tag byte, then the fields; damaged bytes give `None`), `is_peer_input`; `place_footprint` (a multi-block machine: every cell free, the rest `MACHINE_PART`) and `clear_parts` (breaking any cell breaks it, from `break_block`) |
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
| `api/crafting.rs` | Recipe queries (`recipe_locked_by`, `craftable_times` counting parts made from ingots, `recipe_tenths`, `recipe_part_crafts`, `recipe_group` and group names), `craft`, the queue (`craft_queue`: item, amount left, permille per order) and `cancel_craft`, `craft_steps(order)` (an order's remaining steps: item, amount, progress) |
| `api/research.rs` | Research screen: the tech table (`tech_*`), progress, `current_research`, `set_research` |
| `api/content.rs` | Block names and sound materials, `item_name`, `item_icon` (single box), `item_model` (manufactured item box parts), `tool_uses`, `hand_yield`, `miner_recovery`, `item_tier` and `tier_colour` (tier chips) |
| `api/hud.rs` | Player flags, target and `target_detail`, mining progress, onboarding hints (`hint_*`), stats counters, belt line outlines and label (`line_cells`, `line_label`, which also describes a quarry or multi-block machine about to be placed), `placement_box` (7 numbers a box: cells and a colour) |
| `api/sites.rs` | Terraforming sites: `mark_site`, `remove_site` (queued actions), `sites` / `site_fields` (flat list), `site_survey` |
| `api/save.rs` | `save`, `load` (static), `seed`, `play_seconds` |
| `api/net.rs` | `start_host`, `start_client`, `from_snapshot`, `resync`, `is_client`, `host_join` / `host_leave`, `snapshot`, `host_stamp`, `take_frames`, `take_checksums`, `take_outbox`, `push_frames`, `local_player`, `take_states`, `host_state`, `push_states`, `take_item_view`, `push_items`, `core_tick`, `confirmed_tick` |
| `api/minimap.rs` | `minimap_redraw` (only when needed), `minimap_ptr` / `minimap_size` (the RGBA image), `minimap_players`, `minimap_marks` / `minimap_mark_fields`, `known_deposits` / `set_known_deposits`; the world map: `world_map_draw` / `world_map_ptr` / `world_map_version` / `world_map_marks`, `explored_map` / `set_explored_map`; `ore_guide` / `ore_guide_notes` |
| `api/prospect.rs` | The latest prospecting reading (`prospect_seq`, `prospect_kind`, `prospect_records`, `prospect_fields`, `prospect_origin`), `held_device`, `scan_range`, `deposit_label` |
| `api/debug.rs` | `give`, `teleport`, `add_player` / `remove_player`, `state_hash`, `debug_desync` (breaks this core, for resync tests), `run_ticks`, `skip_time`, `find_deposit`, `block_at`, `player_x/y/z` |
| `interaction.rs` | Local player's hands and feet: targeting (a `MACHINE_PART` stands for its machine), mining timer (queues `BreakBlock`), `right_click_action`, `play`, footsteps |
| `quarry_preview.rs`, `footprint_preview.rs` | Placing a quarry (the box it would dig: `quarry_preview`, its label) or a multi-block machine (`footprint_ghost`: its cells, red where in the way; `footprint_label`); R turns either (`place_turn`, `placing_facing`, used by `right_click_action`) |
| `belt_line.rs` | Drag-to-build belt lines: `plan` (longer axis first, one turn, follows one-block steps), ghost-belt preview, builds by queuing `PlaceBlock`s a few per tick, each naming its slot (held stack first, then the other stacks: `queue_build`); with kits held, `plan_upgrade` follows belts and queues `Upgrade`s paid from every kit stack (a click on a miner upgrades it; Shift-click on a belt queues its whole line: `factory/belt_chain.rs`, same-tier belts joined by belt links). `upgrade_aim.rs`: what a held kit would upgrade, `Aim` (outline, belts, HUD card with kits needed and held, red when unaffordable or locked), read by `placement_box`, `line_cells`, `line_label` |
| `block/mod.rs` | Block ids, `DEFS` table, sound materials, flowing water ids (`flow`, `flow_level`) |
| `block/tables.rs` | Flat lookup tables over `DEFS` for hot loops (`OPAQUE`, `SOLID`, `FACE_TEX`, `ALT_TEX` alternates…) |
| `block/tex.rs` | Texture array layers (item textures too); `alternates` / `look`: three extra looks for ores, limestone, leaves |
| `item.rs` | `ItemId` (ids below 256 are the blocks, others start at 256), the item table (`def`, `name`, `stack_size`, `places`), ingots, parts, science packs |
| `item_models.rs` | Shared small box assemblies (parts, tools, scanner, core drill) for loose items and HUD icons; data only |
| `crafting.rs` + `crafting/tests.rs` | The hand-craft queue: `plan` (a recipe's steps, with the parts the inventory can't give crafted first from what it holds: at most 6 deep, 64 steps), `max_times`, `CraftQueue` (per player, at most 12 `Order`s; materials are paid when queued into `held`, the steps run one at a time by `Recipe::hand_ticks`, the last one delivers, cancel and leaving refund), `Sim::run_crafting` at the start of a tick, bytes (save version 23) |
| `hints.rs` | Onboarding hints `HINTS` (text plus a check on the player's inventory and the factory), `progress` (read-only) |
| `research.rs` | `Unlock`, `Tech`, `PACKS`, `Research` (core state the factory owns: current tech, units done; `state`, `locked_by`, `has`, `add_unit`); the tech table `TECHS` lives in `research/techs.rs` (data: prerequisites, packs per unit, units, seconds, unlocks); lint in `research/tests.rs` |
| `chunk.rs` | 32³ block storage; uniform chunks cost no heap |
| `world/mod.rs` | Loaded chunks, edits (`saved` keeps edited chunks), block accessors (`*_anywhere` for core code, with a small cache of generated chunks; `set_block_anywhere_later` remeshes in the streaming budget; `is_air_anywhere` lets column scans skip the sky), render events, `adopt_loaded` (a resync keeps the render cache) |
| `world/streaming.rs` | Streaming and meshing: re-centring on the local player and within `OTHERS_RADIUS` of the others (meshing only the local player's), generation and mesh queues, `work_step`, `remesh`, `area_ready`; `neighbourhood` and `chunks_above` (what lighting a chunk reads) |
| `world/boxlight.rs` | `World::light_at`: the light byte of any cell for things drawn as boxes, by lighting a cell's chunk on demand (a small cache dropped when a block changes nearby); render cache only |
| `worldgen/mod.rs` | Terrain heights, surface, caves, trees; per-column cache; `generate(chunk)`; generator versions (`WORLDGEN_VERSION` is the newest; a world keeps its own `WorldGen::version`; each released version is pinned by `released_versions_never_change`) |
| `worldgen/biome.rs` | Version 2: `Biome` per column (`biome_at`), its rock, surface (`surface_v2`) and tree density |
| `worldgen/geology.rs` | Version 2 deposit seeding: version 1's counts, each ore drawn from its biome's weights (`ORES_BY_BIOME`); surface hints (`stain_surface`, `hint_for`) |
| `worldgen/water.rs` | Version 3 water: sea below `SEA_LEVEL` (62), ponds (`Pond`, one per 96-block cell, a bowl and a bank shaped into `height_at`), `water_top` per column, `surface_v3` (sand under water and on shores), `WaterGuard` (no caves within 2 blocks of water) |
| `worldgen/caves.rs` | Spaghetti caves (`CaveField`), shared by every version |
| `worldgen/strata.rs` | Versions 3 and 4 deposit seeding: rare exposed outcrops on bare rock (`bare_rock`), depth bands (`ore_band`), the starter set near spawn (`starter_outcrops`, `starter_reach`) |
| `worldgen/ore.rs` | Deposit seeding (outcrops, veins, lodes), stamping, `deposit_at` ownership, `find_deposit`, `deposit_by_key` |
| `deposits.rs` | Deposit geometry, tiers, pooled reserves, draw caps, taper, spent rock, `HAND_YIELD`; `owner_of` and `DepositState::survey` (read-only queries) |
| `factory/mod.rs`, `table.rs` | Machine table (`table.rs`: `Kind`, `MACHINES`: one row per block, Kind-ordered rows first, then extra blocks sharing a kind; `machine`), the `Machine` trait every kind implements (`cells`: one, or a footprint's), `Factory`: a `Vec` per kind, index `at` (every cell), `count(kind)`, the world's `research`, `place` / add / remove, `processors_of(block)`, `update` (one tick: power balance, miners, quarries, boxes, processors (with the machine recipes research allows), powered machines and labs, belts; emits `SimEvent`s) |
| `factory/state.rs` | `Factory::write_state` / `read_state`: every machine list, kind by kind (older save versions skip later kinds; before 18 smelters and constructors were two lists, read by `process/legacy.rs`), the sites, then the deposits and the research |
| `factory/lab.rs` | Research lab: one buffer slot per science pack (three since save 20, four since 22), `LAB_TIERS` (speed, kW, every nth unit free), `step_labs` (units for the current tech, never more than it has left, at its grid's speed); bytes, readout, panel, model |
| `factory/power.rs` | Power: `Power` (derived in `relink` from the stored wires: pole grids, the pole each generator and machine is wired to, else the cable it hangs on; `balance` each tick: demand, generators give stored energy in order until supply meets it, lighting fuel as needed, speed per grid), wire drawing; power constants. Belts, splitters and filters take no power. `factory/pole.rs`: `Pole` (with a tier; tier 15 is the power cable, `CABLE_TIER`), `POLE_TIERS` (link range, reach, slots), cable joining rules; `factory/cable.rs`: how cables look |
| `factory/wiring.rs` | The player's power wires (`Hook`: pole cell to machine or pole anchor cell, saved since version 24): `hookup` (what a click would do: connect, move, cut, or why not), `connect` / `disconnect` (`Action::Connect`, `Disconnect`), `auto_hook` (a new pole wires itself to the nearest powered pole), slots used, `hook_by_reach` (old saves; tests wire by range unless `Factory::by_hand`), `resolve_hooks` for `power.rs`; who takes a wire (`powered_cells`, add new powered machines there) |
| `power_tools.rs` | The pole tool and cable drop (presentation plus ordinary actions): `plan_pole` (free placement where aimed, Shift snaps to full reach along the view), `Game::pole_ghost`, hold-to-chain in `update_power_tools`, `cable_drop`, ghost boxes, wire and HUD label. `power_tools/wire.rs`: the wiring hand: selecting a pole, the aim (`WireAim`), click, outlines, preview wire and label |
| `factory/generator.rs` | Coal generator: `GENERATOR_TIERS` (kW, fuel yield), fuel buffer and stored energy (kW·ticks, lit from `recipes::fuel_energy`), gives only what its grid draws; bytes, readout, panel, model |
| `factory/belt_shape.rs` | Belt `Shape`s (flat, ramp up/down, lift, underpass entry/exit): where items ride (`item_at`, `shows`), shape models, `UNDERPASS_RANGE`; `derive_slopes` makes flat belts ramps from their neighbours (in `relink`) |
| `factory/buffer.rs` | `Buffer`: the item stacks a machine holds (box slots, miner output, processing buffers); `feed` pushes into belts leading away |
| `factory/belt.rs` | Belt items, spacing, `accept`, `belt_step` (each belt at its own `speed`, from `BELT_TIERS`); its bytes, readout and model; belt constants |
| `factory/miner.rs` | Miners (one kind, powered; `tier` indexes `MINER_TIERS`: rate, recovery, power, look): `step` (draw, push out, `MinerWorking` events); its bytes, readout and model; miner constants |
| `factory/tiers.rs` | Tiered families (belts, miners, processors, poles, boxes, pumps, quarries, labs, generators; up to Mk4): `FAMILIES` (the block and each tier's item), `placed_by` (item → block and tier), `item_of` (what a tier drops), `family` (kits per step) |
| `factory/storage.rs` | Storage box: `BOX_SLOTS` per tier, `step` (feeds belts leading away); its bytes and readout |
| `factory/router.rs` | Splitter and filter (one `Router` kind): holds one item, passes it front/left/right (round robin; a filter sends its item front, others aside); bytes, readout, filter panel, model |
| `factory/process/mod.rs` | `Processor`: every inputs-to-outputs machine (smelter, constructor, assembler) in one struct driven by its spec, turned by `dir`: `cells`, ports (`takes_from`, `out_faces`), `room_for` / `accept` / `insert` (fuel to the fuel buffer, only what an unlocked recipe uses, a stack of each chosen input), `set_recipe`, `step` (batches in thousandths of a Mk1 tick at tier speed × power share; burners light fuel while a batch runs), `wants_power`, `Status`, bytes (block, tier, facing, recipe, batch, buffers) |
| `factory/process/specs.rs` | `ProcessSpec` rows (`SPECS`): block, recipe categories, `Pick` (chosen in the panel or by what it holds), buffer sizes and the byproduct buffer (`side`), `ProcessTier` per tier (`Energy`: burner, electric or recipe-only, so a smelter goes electric at Mk3; speed, fuel, kW), `footprint` and ports, status words, map colour, model parts; `spec(block)`, `makes(block, category)`, `recipe`, `recipe_using`. Models (`*_PARTS`) are in `process/parts.rs` |
| `factory/footprint/` | Multi-block machines: `Footprint` (size, `Port`s by `Side` and `Role` in, out or side (byproduct); `SINGLE`), `cells` (right and away from the placer, turned by `dir`), `centre`, `faces` (a port is every bottom-layer face of its side), `takes`, `of(block)`, `blocked`; `Factory::footprint_at` / `block_at` (any cell to its machine) |
| `factory/process/steam.rs` | Steam power and bulk storage: `Steam` state, boilers (`boil`: fuel to steam, water from a pump on their pipe net, `draw_water`), turbines (`run_turbine`: power sources `power.rs` asks; `TURBINE_KW`, two to a boiler), the silo's status line, `link` (a boiler's net, a turbine's boilers); tests in `steam/tests.rs` (`plant`, silo, save, `bench_plant`). `process/solar.rs` (+ tests): solar power as processors: `SOLAR_SPEC` (2×2×1) and `ACCUMULATOR_SPEC` (2×2×2), `Store` (charge in kW·ticks, saved for accumulators only), `run` (called by `Power::balance`: panels give what the grid lacks by `daytime::sunlight`, accumulators discharge before any fuel burns, spare sun charges them), `SOLAR_KW`, `ACCUMULATOR_KW`, `CHARGE_CAP` |
| `factory/process/` others | `model.rs`: models as data (`Part` boxes with a `Look`: textures, tier band, fire, status lamp, press stroke), turned with the machine, and port hatches (in, out, byproduct); `view.rs`: status line, readout, panel; `legacy.rs`: smelters and constructors from saves before version 18, as Mk1 processors; `work.rs`: the work loop (`work`, `next` by `Pick`, `blocked`, `full_output`, a burner lighting fuel) |
| `factory/panel.rs` | What a player does to a machine by hand: `panel` (view: status, progress, buffers by role, filter item), `box_slots`, `set_recipe`, `set_filter`, `insert`, `wants`, `take_contents` |
| `factory/links.rs` | Where items go: `Slot`, `Link`, `Sinks` (machines that take items), `deliver`; `relink`: belt outputs for every shape, corners, lift stacks, machine outputs (multi-blocks only at their ports), downstream-first belt order (derived data) |
| `factory/render.rs` | Box instance format (`INSTANCE_FLOATS`; the last float is the uv mode plus twice the cell's light byte), `push_box`, `light_boxes` (lights every instance from `World::light_at`, called from `Game::update`); `write_instances` asks nearby machines for models; `map_machines` (block and position for the minimap's marks) |
| `factory/pipes.rs` | Pipework (one kind, `Kind::Pipe`: `Part` pump, pipe, outlet): networks and arms (`link_pipework`, from `relink`), bytes, readouts, models |
| `factory/pumping.rs` | Moving water each tick (`step_pipework`; `PUMP_TIERS`: rate, hold, kW): pumps lift the highest, farthest source in reach; outlets pour where the water lands; block edits go to `Factory.changed` for the water rules |
| `factory/quarry.rs` | Quarry (`Kind::Quarry`; `QUARRY_TIERS`: ticks a block, kW): digs its box one block at a time (`step`: `seek` past air and non-ground, noting ore in `found`; waits when flooded, full, unpowered or paused), output buffer fed like a miner's, `set` (panel choices), bytes, readout, panel, `quarry_cracks` |
| `factory/quarry/dig_box.rs` | `DigBox` (which cells, in what order), `WIDTHS` / `DEPTHS` choices, `QUARRIABLE`, `survey` (what is left, for the panel and the preview) |
| `factory/quarry/model.rs` | The quarry's model: housing and lamp, corner posts, rails, a gantry that travels row to row, a spinning drill on the block being dug |
| `factory/sites.rs` | Terraforming sites (core state on `Factory`): `Site` (corner columns, `level`, `Job` dig, fill or flatten, the cut and fill ranges found once when marked), the cell order (`Site::cell`), `Sites::mark` / `remove` (ids from a counter, no overlaps, `MAX_SITE`, `MAX_SITES`), bytes; `survey_site` (a query over loaded chunks: cut, fill, ore, trees, water, spoil) |
| `factory/describe.rs` | `Factory::describe` (one `match` on `Slot`, then "Mk2 · next: …" for tiered machines), `fmt_int`, `fmt_duration` |
| `factory/upgrades.rs` | Upgrade kits: `TIER_COLOURS`, `KITS` (the kit per tier: green, blue), `kit_tier`, `Factory::tiered_at`, `next_upgrade` (`Step`: family, tier, kit, kits), `upgrade` (tier + 1 in place) |
| `entities.rs` | Dropped items: ids, physics (floating up through water, drifting), magnet pickup by the nearest `Collector` with room, instances (`push_item_box`) |
| `inventory.rs` | 36 slots, cursor stack, click / quick-move (`quick_move_all`, `move_all_of`: every stack of an item), `add_to_slots` and `sort_stacks` (shared with boxes; the backpack sorts, the hotbar keeps its layout) |
| `tools.rs` | Hand tools: `ToolKind`, `Tier` (uses, speed, ore kept: stone, iron, steel), `TOOLS`; `tool_for` (by the block's sound material), `break_speed` (the hands), `ore_yield` (the core), `device` (scanner, core drill). A tool's stack count is its uses left |
| `prospect.rs` | Scanner and core drill (queries, never actions): `scan`, `core_sample`, `update_prospecting` (called from `update_placing`), `Prospect` (the latest reading as flat records, timers) |
| `ore_guide.rs` | Helping find ore (queries): `guide_rows` / `guide_notes` (the ore guide from the world's own generator numbers), `stain_reading` (what lies under stained soil, how deep) |
| `recipes/mod.rs` | Hand-crafting recipes (`RECIPES`, each in a build-menu `Group`; every tier item's recipe, the previous tier plus kits, is a `pub const` in `recipes/tiers.rs`); the boiler, turbine, crusher and silo recipes are in `recipes/heavy.rs`, the arc furnace's in `recipes/electronics.rs`, the cable's in `recipes/wiring.rs` and the tool recipes in `recipes/tooling.rs`; the hand-made parts (plate, rod, screw, wire) are in `recipes/materials.rs`, solar panel and accumulator in `recipes/solar.rs`. How long a hand craft takes: `recipes/timing.rs` (the formula constants and the per-recipe `OVERRIDES` table) |
| `recipes/machine.rs` | Machine recipes (`MACHINE_RECIPES`: a `Category` (smelting, pressing, assembly, blasting, crushing, arc), inputs, outputs main first; saved by index: append only; the rows research locks: `GEAR_RECIPE`, `BRICK_RECIPE`, `QUICKLIME_RECIPE`, `ASSEMBLY_RECIPES`, `STEEL_RECIPES`, `CRUSH_RECIPES`, `ELECTRONICS_RECIPES`), `FUELS` (smelter seconds and generator kJ). Which machine takes which category: its spec (`factory/process/specs.rs`). `recipes/tests.rs`: content lint (every item has a source and a use, categories have machines, outputs fit, tiers can be made, no hand recipe uses raw ore) |
| `camera.rs` | Presentation-only camera easing: `CrouchGlide` eases the eye between standing and crouching height (called from `Game::update`) |
| `player.rs` | Character controller (walk, sprint, crouch, jump, swim, climb ladders and belt lifts, fly); `in_water`, `splash_speed` for sounds |
| `physics.rs` | Swept AABB collision against the voxel grid |
| `raycast.rs` | Voxel traversal for targeting |
| `light.rs` | Sky and block light (0–15) for a chunk being meshed (water dims it 2 per block): a field of the chunk plus a 32-block margin, sky columns shaded by the chunks above, BFS flood; `CLASS` says how each block treats light. Render cache only |
| `mesher.rs` | Greedy mesher with AO and smoothed per-vertex light (a byte per vertex after the `u32` vertices); ranges opaque, cutout, liquid (liquid AO bits mark the water line and how low it sits: `water_line`); plants as crossed quads (faces 6 and 7); `pick_layer` picks one of four looks per block |
| `mesher/quad.rs` | Per-corner AO and light, the merge key, `emit_quad` |
| `minimap.rs` | The maps' pictures (presentation only): `draw` (any window of the explored map at any scale, shaded by the height step; ore in its mark colour), `redraw` (the minimap around the player), other players' marks |
| `minimap/atlas.rs` | The explored map: top block and height of every column seen, per chunk column (`Tile`, with its ore `spots`), kept after unloading; `touch` on mesh events, `refresh_some` per frame, `refresh_in` for the minimap; `export` / `import` bytes the host stores; `MAX_TILES` |
| `minimap/marks.rs` | Map marks (presentation): `Known` (prospected veins and lodes, `remember` from `prospect.rs`, `export` / `import` for the browser's world record), `Minimap::marks` / `marks_in` (flat records with colours: ore seen at the surface, deposits (dry ones left out), machines), `ore_color` |
| `textures.rs` | Procedural 16×16 textures, one layer per `block::tex` constant; shared noise helpers and avatar patterns |
| `textures/nature.rs` | Alpine natural blocks: slate, pebbled earth, turf with blades, ragged grass edge, rippled sand, leaf clusters (4 looks), bark, bedrock; the four surface hints (`HINTS`) |
| `textures/paint.rs` | Painting helpers: palette ramps, wrapping cells, blobs kept inside the tile |
| `textures/ores.rs` | Coal lumps, iron nodules, copper crusts, quartz crystals, limestone fossils; four looks each on the slate host |
| `textures/items.rs` | Rod threads, screws, glass, science liquid, tool handle and steel for the item assemblies |
| `textures/tools.rs`, `plants.rs` | Flat tool pictures (shown on belts), the scanner screen and device casing; crossed-quad pictures: the Alpine sapling and the torch |
| `textures/stripes.rs` | Tier stripes (`tex::stripe`): a band of the tier colour on metal with a pip per Mk |
| `textures/wood.rs` | Processed wood: planks, the ladder's sides and top, the stick |
| `textures/geology.rs`, `masonry.rs`, `assembly.rs`, `steel.rs` | Granite, sandstone and basalt grains, cutout glass; stone bricks and quicklime; the assembler's housing, port hatches, concrete, the motor; the blast furnace, byproduct hatch, slag, steel items, the steel tool head and the blue pack |
| `textures/machines.rs` | Machine and item texture patterns (belts, miner, smelter, constructor, routers, generator, pole, ingots, parts); `textures/heavy.rs`: boiler, turbine, crusher, silo and crushed ore (layers 141–150); `textures/electronics.rs`: arc furnace, silicon, circuit (151–154); `textures/solar.rs`: solar panel and accumulator (layers 158–161) |
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
| `net/host.ts` | `CoopHost`: welcomes or refuses, stamps actions, takes body states; sends frames, checksums, states, each peer's items, names and pings; answers resyncs; drops silent peers (and an old connection of a key that joins again) |
| `net/client.ts` | `CoopClient`: join, outbox, own state, pongs, checksum compare; resyncs on a mismatch or 5 s of lag; ends on bye, close or silence |
| `net/ticker.ts` | `tickWhenStalled`: a tiny worker timer that runs frames without drawing while the frame loop is stopped (hidden tab, minimised window), so a co-op host keeps ticking |
| `net/signal.ts` | The deployed signalling Worker: `SIGNAL_URL`, `newRoom`, `fetchIce`, `openRoom` (WebSocket), `closeReason`, `isRoomCode` |
| `net/webrtc.ts` | Machines over WebRTC: `hostWebRtc` (room code, one data channel per joiner), `joinWebRtc`; the channel `Transport` splits messages into 16 KB pieces |
| `save/store.ts` | IndexedDB: `worlds` records (`WorldMeta`, with the prospected deposits as `marks` and the map `pins`) and `saves` bytes in two slots per world (newest + backup) plus the explored map (`MAP_SLOT`, `readMap`); gzip `pack` / `unpack` |
| `save/session.ts` | `openWorld` (latest world, backup fallback, `?seed=`), `Session` (autosave: every minute, on pause, hide and close; never for a co-op client; with the player's notes: `keepPins`, the explored map), `switchTo` (save, then reload into another world), `soloUrl` (this page without co-op parameters) |
| `base.css` | Theme variables, reset, focus rings, shared `.hidden`, `.secondary-btn`, `.close-btn` |
| `input.ts` | Keyboard/mouse, pointer lock, held state and one-shot `Action`s |
| `render/renderer.ts` | Chunk meshes (culling, opaque + cutout passes, fog), target outline, mining crack, `project` (camera-relative point to CSS pixels) |
| `render/water.ts` | The liquid range drawn last (blended, back to front, both sides) and the underwater fog (`fogFor`, from `eye_in_water`) |
| `render/outlines.ts` | Overlays: mining cracks (the player's and quarries'), a dragged belt line's cells (`line_cells`), placement boxes (amber; red cells in a machine's way); the renderer's line and crack programs |
| `render/boxes.ts` | Instanced box pipeline (items, belt items, machine parts, lit by the cell each stands in); `INSTANCE_FLOATS` |
| `render/sky.ts` | Day and night: `skyAt` (sun direction, sky and fog colours, daylight from the time of day), `clock`, `SkyPass` (full-screen gradient, sun, moon, stars) |
| `render/shaders.ts` | GLSL sources; the light curve (sky light × daylight, warm block light, `CAVE_FLOOR`); periodic world-anchored Alpine tint for terrain only (`TERRAIN_TINT_PERIOD`) |
| `render/gl.ts`, `render/mat4.ts` | Program/uniform helpers; matrix and frustum helpers |
| `ui/dom.ts` | `h()` and `button()` element helpers |
| `comfort/settings.ts` | Comfort settings against motion sickness (`ComfortStore`: field of view, mouse sensitivity, vignette level, crosshair style, size, thickness, opacity; `RANGES`, `DEFAULTS`, localStorage, `subscribe`); read by `main.ts` (sensitivity, `renderer.fovY`) and the two UI files below |
| `ui/comfort.ts` + `.css` | "Comfort settings" section of the pause menu: sliders and choices; draws the crosshair from the settings; while open the menu steps aside to preview changes |
| `ui/vignette.ts` + `.css` | Movement vignette: the screen edges darken while the camera turns or the player moves fast (`update` once a frame) |
| `ui/hud.ts` + `.css` | Crosshair (styled by comfort settings), target readout, mining bar, hotbar, toasts (a count of 0: a tool wore out), debug overlay, `itemIcon` (isometric box from `item_icon`), `showAmount` (a slot's count or a tool's wear bar) |
| `ui/inventory.ts` + `.css` | Inventory screen (E) with the build menu; opened on a box (`open([x, y, z])`), the box screen: its slots above the inventory, Sort, Take all; Sort on the backpack; Shift-right-click on a slot moves every stack of that item (to the box, from it, or between hotbar and backpack) |
| `ui/crafting.ts` + `.css` | Build menu: recipe tiles grouped by `recipe_group`, text search (output and material names), state filters (all, can craft, missing, locked), one hover info card; click queues a craft, Shift-click 5; the card shows the time and the parts crafted first. `ui/craftqueue.ts` + `.css`: the queue above the hotbar (also over the inventory screen): a group per order with a chip per step (parts first, the asked-for item last), a bar on the running step, ✕ to cancel |
| `ui/machine.ts` + `.css` | Machine panel (right-click a processor such as a smelter or constructor, a filter, generator, lab or quarry): status, progress, buffers, recipe choice, filter item, put-in and take buttons; `quarryBox` for the open quarry's outline |
| `ui/quarry.ts` | The quarry's part of the machine panel: size and depth choices, pause, layer and blocks dug and left, deposits uncovered |
| `ui/nametags.ts` + `.css` | Name tags over other players, from the engine's anchors and the session's names |
| `ui/coop.ts` + `.css` | "Play together" in the menu: name, host this world (code and link), join from a link or code, players and ping, leave, why a session ended; `playerRow` |
| `ui/players.ts` + `.css` | In game: the player list while Tab is held, join and leave notices |
| `ui/minimap.ts` + `.css` | Minimap in a round top-right frame: the engine's image (at most 4 redraws a second), marks and pins (far ones on the rim) on an overlay canvas, player arrows; N toggles (localStorage); `--map-space` moves other top-right HUD items down |
| `ui/worldmap.ts` + `.css` | The world map (M): the explored map at 7 zoom steps (drag, wheel), marks, pins, players; the pin editor and list; the ore guide beside it |
| `ui/pins.ts` | `Pins` (the player's map pins, kinds from the engine's ore guide), `drawMark` / `drawPin` shared by both maps |
| `ui/ore-guide.ts` + `.css` | "Finding ore": a depth chart of each ore's band, where it is common, how to spot it, general notes (`ore_guide`, `ore_guide_notes`) |
| `ui/hints.ts` + `.css` | Onboarding tip card in the HUD (the first hint not done or skipped); H skips, skipped tips in localStorage; "Show tips again" in the menu |
| `ui/prospect.ts` + `.css` | Prospecting card at the top left while a device is held: scan rows (arrows and distances follow the player) or a core sample's figures; calls `onReading` for the ping |
| `ui/research.ts` + `.css` + `-card.css` | Research screen (T): the tech tree, a node per tech coloured by state (done, researching, available = horizon, locked), curves to prerequisites, hover card with the details, click to choose; layout in `ui/tech-tree.ts`; HUD tracker and "research done" notice |
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
| `scripts/build-wasm.mjs`, `dev.mjs` | wasm-pack build into `web/src/wasm` (`--dev` for a debug build); Vite dev server plus a Rust watcher (honours `PORT`) |
| `scripts/check.mjs`, `check-size.mjs` | `npm run check`: fmt, clippy `-D warnings`, tests, tsc (web and `signal/`), size budgets (DEV_PLAN section 3.1); quiet output |
| `scripts/wasm-sizes.mjs` | The largest wasm functions by name, to find what made the wasm grow |
| `.github/workflows/pages.yml` | CI on push to `main`: build wasm, check, bundle, deploy to GitHub Pages |

## How to add…

**A block.** `block.rs`: append an id constant (never renumber), bump `BLOCK_COUNT`, add a `DEFS` row
(`cube`, `ore` or `machine` helper). New texture: a `tex` constant plus its arm in `textures::pixel`.
Placeable blocks work at once; worldgen use goes in `worldgen/`.

**An item or recipe.** A block is already an item. Any other item: an id constant (from 256, append only)
and a row in `item.rs` `EXTRA`, with a texture layer in `block::tex` and its pattern in `textures::pixel`
if it needs a new look; the HUD icon and the loose and belt models follow from the row. A recipe is a row
in `recipes/mod.rs` (its `group` picks the build-menu section); the build menu shows every row. To lock it
behind research, list `Unlock::Recipe(item)` in a tech's `unlocks` (`research/techs.rs`). A machine recipe is a
row in `recipes/machine.rs` with its category (lock: `Unlock::MachineRecipe(index)`). The content lint
(`recipes/tests.rs`, `research/tests.rs`) says what is missing: a source, a use, a machine.

**A tech.** A row appended to `TECHS` in `research/techs.rs` (saves store progress by index): name, blurb,
prerequisites by index, packs per unit, units, seconds, what it unlocks. The research screen and the build
menu follow. A new science pack: an item, a hand recipe and an entry in `PACKS` (labs get a slot for it).

**A processor** (anything that turns inputs into outputs: assembler, furnace, crusher): a `ProcessSpec` row
in `factory/process/specs.rs` (categories, energy, pick, buffers, tiers, footprint and ports, model parts), a `MACHINES` row of
`Kind::Process` for its block, the block (`machine(...)` in `block/mod.rs`), a hand recipe, and its family
in `factory/tiers.rs` if it has tiers. No new code unless it needs new behaviour (then in `process/`: the
boiler, turbine and silo are `Energy::Boiler`, `Energy::Turbine` and `Pick::Store` rows with their code in
`process/steam.rs`; the crusher is a plain 1×1 row with `Category::Crushing`).

**A machine** with behaviour of its own. (1) `factory/<machine>.rs`: the struct (a `Buffer` if it holds
items), `new`, `step`, `impl Machine` (bytes, contents, readout, model), its tuning constants. (2)
`factory/mod.rs`: a `Kind` and a `Slot` variant, a `MACHINES` row (block, kind, slots), a `Vec` field; then
follow the compiler through the `match`es on `Kind` and `Slot` (`place`, `remove`, `take_contents`,
`describe`), and add its list to `write_state` / `read_state` (`state.rs`), `update`, `write_instances`, and
its outputs to `links.rs`. Power: a demand in `Power::balance`, its pole in `Power::rebuild`, a `speed`
argument to `step`, its arm in `wiring.rs` (`powered_cells`, `power_slots`). Fed by belts: an arm in `Slot::is_sink` and a field in `Sinks`. A panel: `panel: true`
in its row, a `panel()` method and its arms in `panel.rs` (the host needs nothing). (3) Its block
(`machine(...)` for a model, `cube(...)` if meshed) and a hand recipe. Tests in `factory/tests.rs` (`run`,
`stocked_box`; test-only accessors live there). A second block with the same behaviour (splitter/filter) is
an extra `MACHINES` row after the Kind-ordered ones, not a new kind.
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
| `factory/table.rs` | `MACHINES` (buffer slots per machine); processors: `tiers` (speed, fuel per work, kW) and `buffers` in `factory/process/specs.rs` |
| `factory/miner.rs` | `MINER_TIERS` (rate, recovery, power per tier) |
| `recipes/machine.rs` | `MACHINE_RECIPES` (seconds per batch), `FUELS` (smelter seconds, generator kJ) |
| `factory/belt.rs` | `BELT_TIERS` (speed per tier), `ITEM_SPACING` |
| `factory/power.rs` | `NOT_WIRED` (the readout for a machine with no pole) |
| `factory/{pole,generator,lab,storage,quarry}.rs` | `POLE_TIERS`, `GENERATOR_TIERS`, `LAB_TIERS`, `BOX_SLOTS`, `QUARRY_TIERS` (one array each, indexed by tier) |
| `factory/pumping.rs`, `sites.rs` | `PUMP_TIERS`, `OUTLET_RATE`, `PUMP_RANGE`; `MAX_SITE` (columns on a side), `MAX_SITES` (per world) |
| `factory/quarry.rs` | `SCAN_PER_TICK`; `quarry/dig_box.rs`: `WIDTHS`, `DEPTHS`, defaults, `QUARRIABLE` |
| `sim/water.rs` | `WATER_DELAY`, `MAX_WATER_UPDATES`; `SEA_LEVEL` is in `worldgen/mod.rs` |
| `sim/timers.rs` | `LEAF_HALF_LIFE`, `GRASS_GROW_HALF_LIFE`, `GRASS_DIE_HALF_LIFE`, `MAX_TIMERS`, leaf check radius and support steps |
| `sim/saplings.rs` | `SAPLING_CHANCE`, `GROW_MIN`, `GROW_HALF_LIFE`, trunk heights |
| `research/techs.rs` | `TECHS` (units, seconds, packs per unit) |
| `worldgen/biome.rs` | `HIGHLAND_LEVEL`, `LOWLAND_LEVEL`, `SPAWN_CALM`, `DITHER`, thresholds in `biome_at`, `tree_factor` |
| `worldgen/geology.rs` | `ORES_BY_BIOME`, `OUTCROPS`, `VEIN_CHANCES`, `HINT_MARGIN`, `HINT_ONE_IN` (version 2) |
| `worldgen/strata.rs` | `EXPOSED_CHANCE`, `ORE_DEPTH`, `STARTERS` (version 3, released); version 4: `EXPOSED_CHANCE_METALS`, `ORE_DEPTH_V4`, `STARTERS_V4`, `STARTER_RADII_V4` |
| `worldgen/ore.rs` | `ORE_GEN`, `LODE_CHANCE`, `ORE_SPAWN_CLEARING` |
| `recipes/mod.rs` | `RECIPES` (hand) |
| `interaction.rs`, `power_tools.rs` | `REACH`, place repeat, break cooldown, footstep stride; `HOLD_REACH`, `MAX_DROP` (cables) |
| `tools.rs` | `STONE_TIER`, `IRON_TIER`, `STEEL_TIER` (uses, break speed, ore kept), `DEVICE_TIER` |
| `prospect.rs` | `SCAN_RANGE`, `SCAN_COOLDOWN`, `DRILL_SECONDS`, `DRILL_REACH`, size bands |
| `events.rs`, `player.rs` | `MINER_SOUND_RANGE`, drop pickup delay; movement speeds, jump, gravity |
