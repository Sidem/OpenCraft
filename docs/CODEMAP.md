# OpenCraft code map

An index: one line per module or group, enough to pick the file. The module's own header (`//!` in Rust, the top comment in
TS) holds its invariants and how to extend it; read that next. Grep symbol names to land on a line. Keep entries to a few
words: detail belongs in the module header, never here. `node scripts/check-map.mjs` (part of `npm run check`) fails when a
path here no longer exists or a module is missing. How to add content, APIs, UI, saved state and sounds: [EXTENDING.md](EXTENDING.md).

## Engine: `crates/engine/src` (Rust → wasm, owns all game state)

Tests sit in `<module>/tests.rs`; `tests.rs` holds Game-level scenarios. Layers, outer to inner: the facade and `api/`, the
local player's hands, presentation, then the **deterministic core**, which never reads the layers above it
(`scripts/check-map.mjs` enforces the core list).

### Facade and API
- `lib.rs`: `Game`, `run_tick` at 60 Hz, `act`. `frame.rs`: `update`, one frame.
- `api/`: the JS API, one `#[wasm_bindgen] impl Game` per area; methods only forward. `engine.d.ts` (generated) lists them.
  `input`, `view`, `render`, `inventory`, `machine`, `crafting`, `research`, `creative`, `content`, `hud`, `sites`, `save`,
  `net`, `minimap`, `prospect`, `analytics`, `towns`, `orbit`, `ruins`, `signs`, `opening`, `blueprint`, `hand_waste`,
  `lessons`, `guide`, `surroundings`, `xray`, `led`, `seismic`, `codex`, `static_boxes`, `calendar`, `weather`, `debug` (give, teleport, run_ticks, scene building).
- `events.rs`: `SimEvent`s → item spawns, sounds, toasts, analytics. `authority.rs`: bodies, loose items, join/leave. `grass_sweep.rs`: seeds grass on bare dirt near bodies (`Action::GrowGrass`).

### Deterministic core (changes only through `Action`s at a tick)
- `sim.rs`: `Sim`, `step`, `state_hash`, `PlayerId`. `sim/`: `state.rs` (the core's bytes, in order), `event.rs`, `timers.rs` (leaf decay and `leaf_held_by_tree`, grass, saplings),
  `sod.rs` (turf faces a dig uncovers start bare and regrow), `water.rs`,
  `saplings.rs`, `torches.rs`, `scrap.rs`, `vault.rs` (ruin vault puzzle), `felling.rs` + `felling/tree.rs` (grown trees felled whole), `fire/` (`blaze.rs`, `front.rs`: burning trees to charcoal, grass fronts, flamethrower rays).
- `action.rs`: the `Action` enum. `action/`: `apply.rs` (one arm each), `clicks.rs`, `blocks.rs`, `docks.rs`, `multiblock.rs`,
  `codec.rs` + `codec/read.rs` (bytes; tags append only).
- `bytes.rs`: canonical encoding (a block id is one byte below 255, else 255 and a `u16`). `save.rs`: the file, `SAVE_VERSION`. `join.rs`: const-table joins. `math.rs`: vectors, hashes, `Rng`.
- `doors.rs`, `deposits.rs` (pools, tiers, `HAND_YIELD`), `daytime.rs` (clock and calendar: seasons, day length, growth by season), `weather/` (fronts, rain and snow, `climate.rs` temperature, `forecast.rs`: pure functions of place and tick), `crafting.rs`, `take_apart.rs`, `perks.rs`, `mode.rs`, `tools.rs` (tool classes, retired tools).
- `inventory.rs` + `inventory/stacks.rs` + `inventory/kinds.rs`; `equipment.rs` (worn gear, packs, upgrades); `tool_belt/` (the worn Tool Belt: speeds, crouch reach, the hand's head, migration); `offhand.rs`.
- `research.rs`; `research/`: `techs.rs` (`TECHS`), `ids.rs`, `rings.rs` (rings, wedges, thresholds, red deeds), `red_green.rs`, `blue.rs`, `violet.rs`, `gold_sky.rs`, `later.rs` (techs added since, at the open end), `migrate.rs` (save < 86 migration), `queue.rs`, `bonus.rs`.
- `codex/`: world-wide obtained-items bitset (save 83), `data.rs` (periodic table forms/facts), `achievements.rs`.
- `ghosts.rs` (planned blocks). `drones/` (construction drones, `earthworks.rs`, `cargo/`).
- `helpers/`: jetpack, hover pack, `drone/` (personal drone, `trip.rs`). `blueprint/mod.rs` (data, bytes; `plan.rs` the view from above).
- `signs/` (note signs; `view.rs` draws boards). `led/` (LED lamp colours, `SetLed`; `view.rs` the tint box). `street/` (street lamps: placing and breaking a 1×1×4 column of blocks, no state; `model.rs` the model, `view.rs` draws it from the head). `satellites.rs`, `pods.rs`, `opening.rs` (+ `opening/miners.rs`).
- `settlements/`: towns. `orders.rs`, `run.rs`, `plans.rs`, `plots.rs`, `build.rs`, `state.rs`, `perks/` (+ `harbour/`).
- `elevator/`: the space elevator, phase 1 (M14A): sections as orders, the climbers' draw (`mod.rs`); the tether and placement hint (`view.rs`).
- `shape/`: block shape modifiers (slabs ¼/½/¾, vertical slabs, auto-corner stairs, 45° pitched slopes/roofs); `boxes.rs`.
- `colour/`: per-face surface paint modifier (`FacePaint`, 16-hue palette, `PaintCharges`, legacy concrete migration).
- `hints.rs`: onboarding tips. `lessons.rs`: the bits of `Sim::taught`.

### Factory (core): `factory/`
- `mod.rs`: `Factory`. `tick.rs`, `table.rs` (`Kind`, `MACHINES`), `machine.rs` (trait), `state.rs`, `switch.rs` (on/off switches), `tiers.rs`, `upgrades.rs` (kits),
  `tier_mark.rs` (Mk studs).
- Logistics: `belt.rs` (`BELT_TIERS`), `provenance.rs`, `belt_items.rs`, `belt_shape.rs` (ramps, lifts), `belt_model.rs`,
  `belt_curve.rs`, `underpass.rs`, `belt_chain.rs`, `router.rs` (splitter, filter, merger) + `router_model.rs` + `filter_rules.rs`, `storage.rs` +
  `box_label.rs`, `buffer.rs`, `links.rs` (`relink`) + `belt_order.rs`, `ports.rs` (drone pads), `stock.rs`.
- Extraction: `miner.rs` (+ `miner_model.rs`, `miner_burner.rs`), `forecast.rs` (deposit depletion forecast), `quarry.rs` + `quarry/`, `pipes.rs` (+ `pipe_model.rs`), `pumping.rs`.
- Power: `power.rs`, `pole.rs` + `pole_mount.rs` + `pole_tower.rs` + `pole_hang.rs` (where wires hang: insulators), `wiring.rs`, `cable.rs`, `generator.rs`, `power_mix.rs`,
  `grid_colour.rs`. Data and light: `fibre.rs`, `laser.rs`. Logic: `sensor.rs`.
- Rails: `rail.rs` + `rail/curve.rs` + `rail/ground.rs` + `rail/pieces.rs` (docks as track pieces; core side `action/docks.rs`),
  `trains.rs` + `trains/` (cars, schedule, signals, docks, model, motion, view).
- `lab.rs`. `sites.rs` + `sites/` (terraforming, survey, tunnels). `footprint/`: multi-block machines; `port_marks.rs` (port stubs); `preview.rs` (`place_preview`: a scratch factory for ghost models).
- `process/`: every inputs-to-outputs machine is one `Processor` driven by a `ProcessSpec` row (`specs.rs`; `basic.rs` the first
  rows); `mod.rs`, `work.rs`, `state.rs`, `extra.rs`, `kinds.rs`, `status.rs`, `view.rs`, `model.rs` + `parts.rs`, `display.rs`
  (`Look::Product`), `assembly_view.rs` (`Look::Work`), `port_mouth.rs` (port openings), `shape.rs` (cells and ports: ask it, not the spec), `intake.rs`,
  `legacy.rs`, `facts.rs`. Machines with logic of their own: `steam.rs` (+ `steam_view.rs`), `solar.rs`, `diesel.rs`, `hydro.rs` (+ `hydro/view.rs`, `Look::WaterWheel`),
  `nuclear.rs`, `rectenna.rs`, `ground.rs`, `pump.rs` (+ `pump/view.rs`, `Look::Jack`), `refinery.rs`, `gas.rs`, `washer.rs`, `recycler.rs`,
  `center.rs`, `ailab.rs`, `datacenter.rs`, `tower.rs`, `optimizer.rs`, `hub.rs`, `fab.rs`, `launch.rs` (+ `rocket.rs`),
  `pods.rs` + `pods/channels.rs`, `hangar.rs`, `hoist.rs`, `docks.rs`, `depot.rs`, `anchor.rs` (the elevator's, a `Pick::Depot`), `ejector.rs`, `upgrader.rs`, `laser.rs`,
  `forester.rs` + `forester/` (`patch.rs`, `work.rs`, `fell.rs`, `view.rs`, `crane.rs`, `scene.rs`; `Look::Crane`), `thumper.rs` (`Look::Thumper`), `kiln.rs` (`GLASS_KILN`).
- Readouts and views: `describe.rs`, `panel.rs`, `facts.rs`, `efficiency.rs`, `render.rs` (instances; `push_round`, `push_cone`,
  `push_framed`; `plume.rs`, `capsule.rs`; `render/fake_round.rs` and `tests/face_fight.rs` are checks), `static_view.rs` (chunk index and static/dynamic split), `hitbox.rs`, `smoke.rs`,
  `placed.rs` (what blueprints copy).

### Content tables (data; append ids, never renumber)
- `block/`: `mod.rs` (ids), `defs.rs` (`DEFS` rows), `make.rs`, `tables.rs` (`TABLE_LEN`), `tex.rs` + `tex/later.rs`, `turn.rs`; id ranges in
  `chemistry.rs`, `forest.rs`, `ruins.rs`, `decor.rs`, `trees.rs`, `coal_miner.rs`, `house.rs`, `ejector.rs`, `sign.rs`,
  `forester.rs`, `woods.rs`, `ores.rs`, `merger.rs`, `led.rs` (ids 220–222, the lamp's three steps), `street.rs` (ids 223–227: head + four post ids by arm direction).
  Ground pigments: items 414–418 (`item/later.rs`), `recipes/pigment.rs` (hand grind rows), heap models in `item_models/stock.rs`.
  Pigments tech (tech 89): items 419–429, blocks 228–238 (`block/tints.rs`, the mixed-hue concretes), `recipes/dye.rs` (`HUES` palette, machine rows 155–176).
  Stained glass: blocks 239–254 (`block/glass.rs`, translucent cubes), `recipes/glass.rs` (rows 177–192, Architecture).
  Natural gas (CHEM-GAS): blocks 255–260 (`block/gas.rs`; 256–260 first wide blocks in `WIDE_BLOCK_ITEMS`), items 430–439, `recipes/gas.rs` (rows 193–197), `worldgen/gas.rs` (v13 independent fields), `item_models/gas.rs`, `textures/gas.rs`.
  Seismic prospecting (F-SEIS): block 261 (`block/thumper.rs`), items 442–444 (Mk1–Mk3), `recipes/thumper.rs`, `item_models/thumper.rs`, `textures/thumper.rs`.
  Glass Kiln & Paint Sprayer (L-COLOUR): block 262 (`block/kiln.rs`), items 445–446, `item_models/kiln.rs`, `textures/kiln.rs`.
  Space elevator (M14A): block 270 (`block/elevator.rs`), item 463, `recipes/towns.rs`, tech 78 (`research/later.rs`).
  Clay & kilns (MAT-CLAY): blocks 263–269 (`block/clay.rs`; wide, paired in `WIDE_BLOCK_ITEMS`), items 451–457, `recipes/clay.rs` (rows 232–236, `Category::Firing`), `worldgen/clay.rs` (v14 beds and pockets), `textures/clay.rs`; river towns' `WARES` in `settlements/orders.rs`.
- `item.rs` + `item/` (`later.rs` ids, `rows.rs`): `ItemId`; ids below 256 are blocks, wider blocks pair with items in `WIDE_BLOCK_ITEMS`. `wood.rs`: the ten woods.
- `recipes/`: `mod.rs` (`RECIPES`; `first.rs`), `machine.rs` (`MACHINE_RECIPES`, `FUELS`; `machine_later.rs`), `process_notes.rs` (one line of real chemistry a chemical machine row; shown on its card and in the item guide), `rows.rs`, `tiers.rs`,
  `refit.rs`, `timing.rs`, `group.rs` + `menu_order.rs`, `item_groups.rs`, `recycling.rs`, `content.rs`; one file per era or
  set: `materials`, `tooling`, `heavy`, `electronics`, `aluminium`, `transport`, `wiring`, `solar`, `chemistry`, `compute`,
  `gear`, `gold`, `orbit`, `automate`, `recycler`, `scrap`, `towns`, `decor`, `house`, `led`.
- `item_models.rs` + `item_models/`: loose items and icons built of boxes and cylinders (`shapes.rs`, `power.rs`, `belts.rs`,
  `icon_frame.rs`, `machines.rs` + `machines_plant.rs`, `placed.rs`, `plants.rs`).
- `textures.rs` + `textures/`: procedural placeholder layers (the art pack overrides them in the browser).

### World and generation
- `chunk.rs` (32³ + sparse shapes/paint/bare turf faces/waterlogged cells), `world/mod.rs` (`*_anywhere` accessors), `world/shapes.rs` (save 84), `world/colours.rs` (save 85), `world/sod.rs` (save 90), `world/waterlog.rs` (save 91), `world/streaming.rs`, `world/boxlight.rs`,
  `world/glows.rs`, `world/settle.rs` (simulation liquid edits wait 0.15 s), `world/cover_clear.rs` (sickle-cut columns; saved), `world/upgrade.rs` (in-place deposits-layer refresh).
- `worldgen/mod.rs`: heights, caves, `generate`, generator versions (pinned by `released_versions_never_change`). `layers.rs` (`Layers`: ground, water, cover, deposits), `column.rs`,
  `biome.rs`, `landform.rs`, `caves.rs`, `water.rs`, `rivers.rs` + `rivers/`, `shore.rs`, `trees.rs` (up to version 11),
  `grow/` (version 12: `kinds.rs`, `shapes.rs`, `place.rs`), `geology.rs` + `strata.rs` + `ore.rs`, `gas.rs` (v13 gas fields), `clay.rs` (v14 clay and kaolin), `bearing.rs`, `ruins.rs`, `logbook.rs`.
- `noise.rs`, `light.rs` (sky and block light while meshing; render cache only).

### Local player: hands, tools and previews (presentation; act only by queuing actions)
- `interaction.rs` + `interaction/aim.rs` (+ `interaction/chop.rs`), `player.rs`, `physics.rs`, `raycast.rs`, `hitbox.rs`, `camera.rs`,
  `strategy.rs` + `strategy/navigation.rs`, `item_select.rs`.
- `belt_line.rs` + `belt_line/` (`path.rs`, `preview.rs`, `route.rs`), `upgrade_aim.rs`, `belt_bend.rs`, `shape_hands.rs` (slab/stair/roof shape mode & ghost preview), `colour_hands.rs` (Paint Sprayer hands & eyedropper), `pipette.rs` (Middle-click/V block & ghost picker), `place_refusal.rs`.
- Previews and hands: `quarry_preview.rs`, `forester_preview.rs`, `footprint_preview.rs`, `port_view.rs` + `port_view/`,
  `town_preview.rs`, `ghost_mode.rs` + `ghost_models.rs` (ghosts and blueprint stamps as see-through models), `ghost_build.rs` (X: build the aimed ghost from the pack), `power_tools.rs` + `power_tools/`, `rail_tools.rs`, `rail_snap.rs`, `train_tools.rs`,
  `dock_tools.rs`, `cargo_tools.rs`, `site_hands.rs`, `helper_hands.rs`, `drone_hands.rs`, `blueprint/hands.rs`, `blueprint/repeat.rs`.
- Lessons: `stall_lesson.rs`, `need_lesson.rs`, `recipe_lesson.rs` (no recipe chosen), `upgrade_lesson.rs` (a bottleneck and kits in the pack), `blueprint_lesson.rs` (the walk-through after the blueprint prompt), `planner_lesson.rs` (a planner in the pack, no site yet), `map_lesson.rs` (far from spawn, the map never opened), `first_steps.rs` + `first_steps/`.
- Tools: `prospect.rs`, `seismic/` (`mod.rs`, `scan.rs`, `volume.rs`: 2D MRI slice + 48³ 3D volume query), `survey.rs` + `survey/orbit.rs`, `telescope.rs` + `telescope/` (`ground.rs`, `beacons.rs`), `sickle.rs`, `flame_hands.rs`.

### Presentation: drawing, sound, maps, analytics (never feeds the core)
- `mesher.rs` + `mesher/` (`quad.rs`, `face_layer.rs` (alternates, turf's lit grown faces), `plants.rs`, `water.rs`, `blocks/` + `blocks/slope.rs` shaped slabs/stairs/roofs, `cover.rs` + `cover/shape.rs`); trees: `shaped.rs`, `tree_grid.rs`, `logs.rs` + `logs/`
  (`node.rs`, `ring.rs`), `foliage.rs` + `foliage/style.rs`.
- `entities.rs` + `entities/merge.rs`, `avatars.rs` + `avatars/` (`grip.rs`, `hand.rs`, `tool_hold.rs`; first-person hand & character rig), `halos.rs`, `fire_view.rs` + `fire_view/torch.rs` (burning tree/ember emitters, ballistic torch plumes, flame lights & glow), `wire_props.rs`,
  `held_light.rs`, `drone_view.rs` + `companion_view.rs`, `orbit_fx.rs`, `sound.rs`.
- `minimap.rs` + `minimap/` (`atlas.rs`, `marks.rs`, `describe.rs`, `sites.rs`), `ore_guide.rs`, `hand_waste.rs`.
- `xray.rs`: the X-ray goggles' view (a local switch; the factory's boxes near the camera, drawn behind the terrain only).
- `static_boxes/`: per-chunk static machine and wire box cache (pre-lit, cube/round split, GPU-resident).
- `tier_palette.rs`: the two palettes of the tier colours Mk1–Mk5 (standard, colour-blind) and the player's local choice.
- `item_guide.rs`: every way to get an item and what uses it. `analytics/` (`series.rs`, `machines.rs`, `power.rs`; never saved).
  `surroundings/` (ambient sound floats, `roof.rs`).

### Co-op
- `net/mod.rs`: `Role`, `INPUT_DELAY`, `CHECKSUM_TICKS`. `snapshot.rs`, `players.rs`, `items.rs`.

### Tests and benches
- `tests.rs`, `tests/frame_bench.rs`; ignored `bench_*` tests (`cargo test --release -q bench_ -- --ignored --nocapture`):
  meshing, `tests/forest_bench.rs`, `tests/tree_spikes.rs`, big base, trains, towns, water, drones, orbit, worldgen,
  `factory/tests/bench_marks.rs`, `factory/tests/bench_megabase.rs`.

## Web host: `web/src` (TypeScript + WebGL2, thin platform layer)

- `main.ts`: bootstrap, frame loop. `pause.ts`, `input.ts` (keys, mouse, one-shot `Action`s), `controls/view.ts`, `base.css`,
  `patch-notes.md`.
- `render/`: `renderer.ts` (+ `frame.ts`), `mesh-events.ts`, `shaped.ts`, `falling.ts`, `geometry.ts`, `shaders.ts`, `boxes.ts` +
  `round-mesh.ts` + `box-glsl.ts`, `water.ts`, `outlines.ts`, `precip.ts`, `halos.ts`, `fire.ts` (instanced shader flames & embers), `sky.ts` + `sky/`, `art-pack.ts` (128 px pack),
  `art/` (the pack's files), `held-light.ts`, `sign-faces.ts` + `sign-layout.ts`, `tier-paint.ts` + `tier-wire.ts` (a tier palette recolours the baked layers), `frame-stats.ts`, `gpu-probe.ts`, `quality.ts`, `auto-quality.ts`, `gl.ts`, `mat4.ts`.
- `audio/`: `sound.ts` (master bus), `sound-actions.ts`, `synth.ts` + `synth-orbit.ts` + `synth-fire.ts` + `synth-seismic.ts` + `synth-chime.ts` (Codex unlock chimes) + `flame-loop.ts` (continuous torch audio), `settings.ts`, `ambience.ts` +
  `ambience-levels.ts` + `ambience-mix.ts` + `synth-ambience.ts`, `surroundings.ts`, `music.ts` + `music-director.ts` +
  `music-pieces.ts` (one row a piece) + `music/`.
- `net/`: `session.ts`, `host.ts`, `client.ts`, `protocol.ts`, `build-id.ts`, `coop.ts`, `transport.ts`, `webrtc.ts`, `broadcast.ts`, `signal.ts`, `ticker.ts`.
- `save/`: `store.ts` (IndexedDB), `session.ts`, `export.ts`, `leave-guard.ts` (drawn by `ui/leave-prompt.ts`; `ui/no-storage.ts` fills Worlds without storage); `comfort/settings.ts`. `console/`: `commands.ts`, `host.ts`.
- `ui/`: one component per `<name>.ts` + `<name>.css`; `dom.ts` builds elements.
  - HUD: `hud.ts` (+ `hud-columns.css`, `target-card.ts`, `target-card.css`, `target-controls.ts`; `itemIcon` via `icon-paint.ts`, `icon-geometry.ts`), `icon-sprite.ts`, `hints.ts`, `todo.ts` (pinned shopping list),
    `key-prompt.ts`, `ghost-panel.ts` (ghost mode's controls card), `xray-badge.ts` (the goggles' icon and rim; keeps the on/off choice), `led-panel.ts` (an LED lamp's brightness and colour), `prospect.ts`, `nametags.ts`, `scope.ts`, `vignette.ts`, `minimap.ts`, `compass.ts`, `season-chip.ts` and `calendar.ts` (Shift+N; icons and rings in `seasons.ts`; weather row `forecast.ts`), `craftqueue.ts`.
  - Inventory and building: `inventory.ts`, `pager.ts`, `slot-controls.ts`, `crafting.ts` (+ `craft-nav.css`), `craft-family.ts`,
    `craft-tiers.ts`, `craft-card.ts`, `craft-info.ts`, `item-guide.ts`, `take-apart.ts`, `offhand.ts`, `held-name.ts`,
    `drag-throw.ts`, `craft-terms.ts`, `creative.ts`, `gear-upgrade.ts`, `box-label.ts`, `drone.ts`, `drone-slots.ts`, `slot-view.ts`.
  - Machines: `machine.ts`, `machine-upgrade.ts`, `machine-recipes.ts` (+ `recipe-rates.ts`), `rate-table.ts`, `filter-panel.ts`,
    `quarry.ts`, `pod-channel.ts`, `town.ts`, `harbour.ts`, `elevator.ts`, `site.ts`, `logbook.ts`, `vault.ts`, `sign.ts` (`colour-wheel.ts`,
    `colour.ts`), `seismic.ts` + `seismic-controls.ts` + `seismic-render.ts` + `seismic-volume.ts` + `seismic-data.ts`, `train.ts`.
  - Research: `research.ts` (screen, HUD tracker), `research-frontier.ts` (pure: cells, discovery, wide rings, chains), `research-grid.ts` (rings × disciplines grid, hover lines), `research-chip.ts`, `research-now.ts` (now, ready, next-ring checklist), `research-detail.ts` (selected tech), `research-wedges.ts` (12 discipline rows), `research-kinds.ts`, `research-search.ts`, `research-queue.ts`, `codex.ts` (+ `codex-elements.ts`: periodic table & achievements; `codex-toast.ts`: unlock popups).
  - Maps: `worldmap.ts`, `pins.ts`, `map-pin-list.ts`, `map-filter.ts` + `map-filter-panel.ts`, `map-info.ts`, `map-key.ts` (+
    `map-key-rows.ts`), `map-focus.ts` + `map-focus-layer.ts`, `ore-guide.ts`. Analytics (P): `analytics.ts`, a tab each in
    `analytics-overview.ts`, `analytics-power.ts` (+ `analytics-grids.ts`), `analytics-items.ts`, `analytics-machines.ts`
    (base `analytics-tab.ts`), `analytics-chart.ts`, `analytics-format.ts`, `analytics-panels.css`.
  - Menu: `menu.ts` (+ `menu-pages.css`), `settings-tabs.ts`, `home.ts` (+ `home.css`, `home-text.ts`, `greetings.ts`, `tip-text.ts`),
    `controls.ts` (+ `controls.css`; `control-bindings.ts` mirrors `input.ts`, guarded by a test; `keyboard-view.ts`,
    `keyboard-geometry.ts`, `keyboard-layout.ts`), `worlds.ts`, `world-upgrade.ts` (+ `world-upgrade-text.ts`: deposits-layer upgrade card & notice), `coop.ts`, `players.ts`, `patch-notes.ts` (+ `inline-text.ts`),
    `comfort.ts`, `sky.ts`, `tier-palette.ts` (the colour-blind tier colours' choice and store), `desktop-only.ts`, `perf-report.ts` (+ `perf-text.ts`), `quality.ts`, `blueprints.ts` (+
    `blueprint-plan.ts`), `console.ts`.
  - Sound: `sound-panel.ts`, `master-volume.ts`, `volume-control.ts`, `knob.ts`, `sound-lab.ts` + `sound-lab-footer.ts` (key O only).
- `wasm/`: generated by `npm run build:wasm` (gitignored); `engine.d.ts` is the API reference.
- Pure logic tests: `web/test/*.test.mjs` (`node --test`).

## Outside the game

- `signal/`: the Cloudflare signalling Worker (`src/index.ts`, `src/room.ts`); deploying: WORKFLOW section 4.
- `scripts/`: `check.mjs` (`npm run check`), `check-size.mjs`, `check-css.mjs`, `check-map.mjs`, `check-lessons.mjs`,
  `check-music.mjs`, `build-wasm.mjs`, `dev.mjs`, `wasm-sizes.mjs`, `inspect-save.mjs` (read a save headlessly), `recipe-graph.mjs`,
  `bench-flight.mjs` (headless frame timing; `still`), `frame-bench.mjs` (real browser; `PROFILE=`), `prof-top.mjs`,
  `port-shots.mjs`, `bake-sky/` (not part of the build).
- `tools/art/`: the art pack's Python tool (`build.py`, `check.py`, `grade.py`, `trees.py`, `woods.py`); see `docs/ART_STYLE.md`.
- `.github/workflows/pages.yml`: CI on push to `main` (check, build, deploy).

## Where the tuning knobs live

Balance numbers are named constants atop their module; grep the name for the value.
- Ticks and co-op: `TICK_RATE` (`lib.rs`), `INPUT_DELAY`, `CHECKSUM_TICKS` (`net/mod.rs`), `MAX_PLAYERS` (`authority.rs`).
- Mining: `HAND_YIELD`, taper (`deposits.rs`), belt tiers (`tool_belt/mod.rs` `BELTS`), `MINER_TIERS` (`factory/miner.rs`).
- Machines: `MACHINES` (`factory/table.rs`), `SPECS` (`factory/process/basic.rs`, machine files), `MACHINE_RECIPES`,
  `FUELS` (`recipes/machine.rs`), `BELT_TIERS` (`factory/belt.rs`), `POLE_TIERS`, `GENERATOR_TIERS`, `LAB_TIERS`,
  `BOX_SLOTS`, `QUARRY_TIERS`, `PUMP_TIERS` (their `factory/` files).
- Research: `TECHS` (`research/techs.rs` and the era files). Hand recipes: `RECIPES` (`recipes/mod.rs`).
- Towns: `UPKEEP_PER_MINUTE`, `ORDERS` (`settlements/orders.rs`), perk rates (`settlements/perks/`). Elevator: `ORDERS`, `DRAW` (`elevator/mod.rs`).
- World: `SEA_LEVEL` (`worldgen/mod.rs`), biome thresholds (`worldgen/biome.rs`), ore tables (`worldgen/geology.rs`,
  `strata.rs`, `ore.rs`), rivers (`worldgen/rivers.rs`), `WATER_DELAY` (`sim/water.rs`), timers (`sim/timers.rs`).
