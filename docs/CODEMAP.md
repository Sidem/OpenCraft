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
  `lessons`, `guide`, `surroundings`, `debug` (give, teleport, run_ticks, scene building).
- `events.rs`: `SimEvent`s → item spawns, sounds, toasts, analytics. `authority.rs`: bodies, loose items, join/leave.

### Deterministic core (changes only through `Action`s at a tick)
- `sim.rs`: `Sim`, `step`, `state_hash`, `PlayerId`. `sim/`: `event.rs`, `timers.rs` (leaf decay, grass, saplings), `water.rs`,
  `saplings.rs`, `torches.rs`, `scrap.rs`, `vault.rs` (ruin vault puzzle), `felling.rs` + `felling/tree.rs` (grown trees felled whole).
- `action.rs`: the `Action` enum. `action/`: `apply.rs` (one arm each), `clicks.rs`, `blocks.rs`, `docks.rs`, `multiblock.rs`,
  `codec.rs` + `codec/read.rs` (bytes; tags append only).
- `bytes.rs`: canonical encoding. `save.rs`: the file, `SAVE_VERSION`. `join.rs`: const-table joins. `math.rs`: vectors, hashes, `Rng`.
- `doors.rs`, `deposits.rs` (pools, tiers, `HAND_YIELD`), `daytime.rs`, `crafting.rs`, `take_apart.rs`, `perks.rs`, `mode.rs`, `tools.rs`.
- `inventory.rs` + `inventory/stacks.rs` + `inventory/kinds.rs`; `equipment.rs` (worn gear, packs); `offhand.rs`.
- `research.rs`; `research/`: `techs.rs` (first era), `electronics.rs`, `later.rs` (open end), `queue.rs`, `bonus.rs`.
- `ghosts.rs` (planned blocks). `drones/` (construction drones, `earthworks.rs`, `cargo/`).
- `helpers/`: jetpack, hover pack, `drone/` (personal drone, `trip.rs`). `blueprint/mod.rs` (data, bytes; `plan.rs` the view from above).
- `signs/` (note signs; `view.rs` draws boards). `satellites.rs`, `pods.rs`, `opening.rs` (+ `opening/miners.rs`).
- `settlements/`: towns. `orders.rs`, `run.rs`, `plans.rs`, `plots.rs`, `build.rs`, `state.rs`, `perks/` (+ `harbour/`).
- `hints.rs`: onboarding tips. `lessons.rs`: the bits of `Sim::taught`.

### Factory (core): `factory/`
- `mod.rs`: `Factory`. `tick.rs`, `table.rs` (`Kind`, `MACHINES`), `machine.rs` (trait), `state.rs`, `tiers.rs`, `upgrades.rs` (kits),
  `tier_mark.rs` (Mk studs).
- Logistics: `belt.rs` (`BELT_TIERS`), `provenance.rs`, `belt_items.rs`, `belt_shape.rs` (ramps, lifts), `belt_model.rs`,
  `belt_curve.rs`, `underpass.rs`, `belt_chain.rs`, `router.rs` (splitter, filter, merger) + `router_model.rs` + `filter_rules.rs`, `storage.rs` +
  `box_label.rs`, `buffer.rs`, `links.rs` (`relink`) + `belt_order.rs`, `ports.rs` (drone pads), `stock.rs`.
- Extraction: `miner.rs` (+ `miner_model.rs`, `miner_burner.rs`), `quarry.rs` + `quarry/`, `pipes.rs`, `pumping.rs`.
- Power: `power.rs`, `pole.rs` + `pole_mount.rs` + `pole_tower.rs` + `pole_hang.rs` (where wires hang: insulators), `wiring.rs`, `cable.rs`, `generator.rs`, `power_mix.rs`,
  `grid_colour.rs`. Data and light: `fibre.rs`, `laser.rs`. Logic: `sensor.rs`.
- Rails: `rail.rs` + `rail/curve.rs` + `rail/ground.rs` + `rail/pieces.rs` (docks as track pieces; core side `action/docks.rs`),
  `trains.rs` + `trains/` (cars, schedule, signals, docks, model, motion, view).
- `lab.rs`. `sites.rs` + `sites/` (terraforming, survey, tunnels). `footprint/`: multi-block machines; `port_marks.rs` (port stubs); `preview.rs` (`place_preview`: a scratch factory for ghost models).
- `process/`: every inputs-to-outputs machine is one `Processor` driven by a `ProcessSpec` row (`specs.rs`; `basic.rs` the first
  rows); `mod.rs`, `work.rs`, `state.rs`, `extra.rs`, `kinds.rs`, `status.rs`, `view.rs`, `model.rs` + `parts.rs`, `display.rs`
  (`Look::Product`), `assembly_view.rs` (`Look::Work`), `port_mouth.rs` (port openings), `shape.rs` (cells and ports: ask it, not the spec), `intake.rs`,
  `legacy.rs`, `facts.rs`. Machines with logic of their own: `steam.rs` (+ `steam_view.rs`), `solar.rs`, `diesel.rs`, `hydro.rs`,
  `nuclear.rs`, `rectenna.rs`, `ground.rs`, `pump.rs` (+ `pump/view.rs`, `Look::Jack`), `refinery.rs`, `washer.rs`, `recycler.rs`,
  `center.rs`, `ailab.rs`, `datacenter.rs`, `tower.rs`, `optimizer.rs`, `hub.rs`, `fab.rs`, `launch.rs` (+ `rocket.rs`),
  `pods.rs` + `pods/channels.rs`, `hangar.rs`, `hoist.rs`, `docks.rs`, `depot.rs`, `ejector.rs`, `upgrader.rs`, `laser.rs`,
  `forester.rs` + `forester/` (`patch.rs`, `work.rs`, `fell.rs`, `view.rs`, `crane.rs`, `scene.rs`; `Look::Crane`).
- Readouts and views: `describe.rs`, `panel.rs`, `facts.rs`, `efficiency.rs`, `render.rs` (instances; `push_round`, `push_cone`,
  `push_framed`; `plume.rs`, `capsule.rs`; `render/fake_round.rs` and `tests/face_fight.rs` are checks), `hitbox.rs`, `smoke.rs`,
  `placed.rs` (what blueprints copy).

### Content tables (data; append ids, never renumber)
- `block/`: `mod.rs` (ids), `defs.rs` (`DEFS` rows), `make.rs`, `tables.rs`, `tex.rs` + `tex/later.rs`, `turn.rs`; id ranges in
  `chemistry.rs`, `forest.rs`, `ruins.rs`, `decor.rs`, `trees.rs`, `coal_miner.rs`, `house.rs`, `ejector.rs`, `sign.rs`,
  `forester.rs`, `woods.rs`, `ores.rs`.
- `item.rs` + `item/` (`later.rs` ids, `rows.rs`): `ItemId`; ids below 256 are blocks. `wood.rs`: the ten woods.
- `recipes/`: `mod.rs` (`RECIPES`; `first.rs`), `machine.rs` (`MACHINE_RECIPES`, `FUELS`; `machine_later.rs`), `rows.rs`, `tiers.rs`,
  `refit.rs`, `timing.rs`, `group.rs` + `menu_order.rs`, `item_groups.rs`, `recycling.rs`, `content.rs`; one file per era or
  set: `materials`, `tooling`, `heavy`, `electronics`, `aluminium`, `transport`, `wiring`, `solar`, `chemistry`, `compute`,
  `gear`, `gold`, `orbit`, `automate`, `recycler`, `scrap`, `towns`, `decor`, `house`.
- `item_models.rs` + `item_models/`: loose items and icons built of boxes and cylinders (`shapes.rs`, `power.rs`, `belts.rs`,
  `icon_frame.rs`, `machines.rs` + `machines_plant.rs`, `placed.rs`, `plants.rs`).
- `textures.rs` + `textures/`: procedural placeholder layers (the art pack overrides them in the browser).

### World and generation
- `chunk.rs` (32³), `world/mod.rs` (`*_anywhere` accessors for core code), `world/streaming.rs`, `world/boxlight.rs`,
  `world/glows.rs`, `world/settle.rs` (simulation liquid edits wait 0.15 s), `world/cover_clear.rs` (sickle-cut columns; saved).
- `worldgen/mod.rs`: heights, caves, `generate`, generator versions (pinned by `released_versions_never_change`). `column.rs`,
  `biome.rs`, `landform.rs`, `caves.rs`, `water.rs`, `rivers.rs` + `rivers/`, `shore.rs`, `trees.rs` (up to version 11),
  `grow/` (version 12: `kinds.rs`, `shapes.rs`, `place.rs`), `geology.rs` + `strata.rs` + `ore.rs`, `bearing.rs`, `ruins.rs`, `logbook.rs`.
- `noise.rs`, `light.rs` (sky and block light while meshing; render cache only).

### Local player: hands, tools and previews (presentation; act only by queuing actions)
- `interaction.rs` + `interaction/aim.rs` (+ `interaction/chop.rs`), `player.rs`, `physics.rs`, `raycast.rs`, `hitbox.rs`, `camera.rs`,
  `strategy.rs` + `strategy/navigation.rs`, `item_select.rs`.
- `belt_line.rs` + `belt_line/` (`path.rs`, `preview.rs`, `route.rs`), `upgrade_aim.rs`, `belt_bend.rs`, `place_refusal.rs`.
- Previews and hands: `quarry_preview.rs`, `forester_preview.rs`, `footprint_preview.rs`, `port_view.rs` + `port_view/`,
  `town_preview.rs`, `ghost_mode.rs` + `ghost_models.rs` (ghosts and blueprint stamps as see-through models), `ghost_build.rs` (X: build the aimed ghost from the pack), `power_tools.rs` + `power_tools/`, `rail_tools.rs`, `rail_snap.rs`, `train_tools.rs`,
  `dock_tools.rs`, `cargo_tools.rs`, `site_hands.rs`, `helper_hands.rs`, `drone_hands.rs`, `blueprint/hands.rs`, `blueprint/repeat.rs`.
- Lessons: `stall_lesson.rs`, `need_lesson.rs`, `recipe_lesson.rs` (no recipe chosen), `upgrade_lesson.rs` (a bottleneck and kits in the pack), `blueprint_lesson.rs` (the walk-through after the blueprint prompt), `planner_lesson.rs` (a planner in the pack, no site yet), `map_lesson.rs` (far from spawn, the map never opened), `first_steps.rs` + `first_steps/`.
- Tools: `prospect.rs`, `survey.rs` + `survey/orbit.rs`, `telescope.rs` + `telescope/` (`ground.rs`, `beacons.rs`), `sickle.rs`.

### Presentation: drawing, sound, maps, analytics (never feeds the core)
- `mesher.rs` + `mesher/` (`quad.rs`, `water.rs`, `cover.rs`); trees: `shaped.rs`, `tree_grid.rs`, `logs.rs` + `logs/`
  (`node.rs`, `ring.rs`), `foliage.rs` + `foliage/style.rs`.
- `entities.rs` + `entities/merge.rs`, `avatars.rs` + `avatars/` (`grip.rs`, `tool_hold.rs`; first-person hand), `halos.rs`, `wire_props.rs`,
  `held_light.rs`, `drone_view.rs` + `companion_view.rs`, `orbit_fx.rs`, `sound.rs`.
- `minimap.rs` + `minimap/` (`atlas.rs`, `marks.rs`, `describe.rs`, `sites.rs`), `ore_guide.rs`, `hand_waste.rs`.
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
  `round-mesh.ts` + `box-glsl.ts`, `water.ts`, `outlines.ts`, `precip.ts`, `halos.ts`, `sky.ts` + `sky/`, `art-pack.ts` (128 px pack),
  `art/` (the pack's files), `held-light.ts`, `sign-faces.ts` + `sign-layout.ts`, `tier-paint.ts` + `tier-wire.ts` (a tier palette recolours the baked layers), `frame-stats.ts`, `gpu-probe.ts`, `quality.ts`, `auto-quality.ts`, `gl.ts`, `mat4.ts`.
- `audio/`: `sound.ts` (master bus), `sound-actions.ts`, `synth.ts` + `synth-orbit.ts`, `settings.ts`, `ambience.ts` +
  `ambience-levels.ts` + `ambience-mix.ts` + `synth-ambience.ts`, `surroundings.ts`, `music.ts` + `music-director.ts` +
  `music-pieces.ts` (one row a piece) + `music/`.
- `net/`: `session.ts`, `host.ts`, `client.ts`, `protocol.ts`, `coop.ts`, `transport.ts`, `webrtc.ts`, `broadcast.ts`, `signal.ts`, `ticker.ts`.
- `save/`: `store.ts` (IndexedDB), `session.ts`, `export.ts`, `leave-guard.ts` (drawn by `ui/leave-prompt.ts`; `ui/no-storage.ts` fills Worlds without storage); `comfort/settings.ts`. `console/`: `commands.ts`, `host.ts`.
- `ui/`: one component per `<name>.ts` + `<name>.css`; `dom.ts` builds elements.
  - HUD: `hud.ts` (+ `hud-columns.css`; `itemIcon` via `icon-paint.ts`, `icon-geometry.ts`), `icon-sprite.ts`, `hints.ts`,
    `key-prompt.ts`, `ghost-panel.ts` (ghost mode's controls card), `prospect.ts`, `nametags.ts`, `scope.ts`, `vignette.ts`, `minimap.ts`, `compass.ts`, `craftqueue.ts`.
  - Inventory and building: `inventory.ts`, `pager.ts`, `slot-controls.ts`, `crafting.ts` (+ `craft-nav.css`), `craft-family.ts`,
    `craft-tiers.ts`, `craft-card.ts`, `craft-info.ts`, `item-guide.ts`, `take-apart.ts`, `offhand.ts`, `held-name.ts`,
    `drag-throw.ts`, `craft-terms.ts`, `creative.ts`, `pack-upgrade.ts`, `box-label.ts`, `drone.ts`, `drone-slots.ts`, `slot-view.ts`.
  - Machines: `machine.ts`, `machine-upgrade.ts`, `machine-recipes.ts` (+ `recipe-rates.ts`), `rate-table.ts`, `filter-panel.ts`,
    `quarry.ts`, `pod-channel.ts`, `town.ts`, `harbour.ts`, `site.ts`, `logbook.ts`, `vault.ts`, `sign.ts` (`colour-wheel.ts`,
    `colour.ts`), `train.ts`.
  - Research: `research.ts` (+ `research-card.css`), `research-view.ts`, `tech-tree.ts`, `research-eras.ts`, `research-node.ts`,
    `research-kinds.ts`, `research-map.ts`, `research-search.ts`, `research-queue.ts`.
  - Maps: `worldmap.ts`, `pins.ts`, `map-pin-list.ts`, `map-filter.ts` + `map-filter-panel.ts`, `map-info.ts`, `map-key.ts` (+
    `map-key-rows.ts`), `map-focus.ts` + `map-focus-layer.ts`, `ore-guide.ts`. Analytics (P): `analytics.ts`, a tab each in
    `analytics-overview.ts`, `analytics-power.ts` (+ `analytics-grids.ts`), `analytics-items.ts`, `analytics-machines.ts`
    (base `analytics-tab.ts`), `analytics-chart.ts`, `analytics-format.ts`, `analytics-panels.css`.
  - Menu: `menu.ts` (+ `menu-pages.css`), `settings-tabs.ts`, `home.ts` (+ `home.css`, `home-text.ts`, `greetings.ts`, `tip-text.ts`),
    `controls.ts` (+ `controls.css`; `control-bindings.ts` mirrors `input.ts`, guarded by a test; `keyboard-view.ts`,
    `keyboard-geometry.ts`, `keyboard-layout.ts`), `worlds.ts`, `coop.ts`, `players.ts`, `patch-notes.ts` (+ `inline-text.ts`),
    `comfort.ts`, `sky.ts`, `tier-palette.ts` (the colour-blind tier colours' choice and store), `desktop-only.ts`, `perf-report.ts` (+ `perf-text.ts`), `quality.ts`, `blueprints.ts` (+
    `blueprint-plan.ts`), `console.ts`, `view.css`.
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
- Mining: `HAND_YIELD`, taper (`deposits.rs`), tool tiers (`tools.rs`), `MINER_TIERS` (`factory/miner.rs`).
- Machines: `MACHINES` (`factory/table.rs`), `SPECS` (`factory/process/basic.rs`, machine files), `MACHINE_RECIPES`,
  `FUELS` (`recipes/machine.rs`), `BELT_TIERS` (`factory/belt.rs`), `POLE_TIERS`, `GENERATOR_TIERS`, `LAB_TIERS`,
  `BOX_SLOTS`, `QUARRY_TIERS`, `PUMP_TIERS` (their `factory/` files).
- Research: `TECHS` (`research/techs.rs` and the era files). Hand recipes: `RECIPES` (`recipes/mod.rs`).
- Towns: `UPKEEP_PER_MINUTE`, `ORDERS` (`settlements/orders.rs`), perk rates (`settlements/perks/`).
- World: `SEA_LEVEL` (`worldgen/mod.rs`), biome thresholds (`worldgen/biome.rs`), ore tables (`worldgen/geology.rs`,
  `strata.rs`, `ore.rs`), rivers (`worldgen/rivers.rs`), `WATER_DELAY` (`sim/water.rs`), timers (`sim/timers.rs`).
