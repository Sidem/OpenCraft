# OpenCraft code map

An index: one line per module or group, enough to pick the file. The module's own header (`//!` in Rust, the top comment in
TS) holds its invariants and how to extend it; read that next, not this file's history. Grep symbol names to land on a line.
`node scripts/check-map.mjs` (part of `npm run check`) fails when a path here no longer exists or a module is missing.
How to add content, APIs, UI, saved state and sounds: [EXTENDING.md](EXTENDING.md).

## Engine: `crates/engine/src` (Rust → wasm, owns all game state)

Tests sit in `<module>/tests.rs`; `tests.rs` holds Game-level scenarios. Layers, outer to inner: the facade and `api/`, the
local player's hands, presentation, then the **deterministic core**, which never reads the layers above it
(`scripts/check-map.mjs` enforces the core list).

### Facade and API
- `lib.rs`: `Game` (core `sim`, local player, co-op role, bodies, view state), `run_tick` at 60 Hz, `act`. `frame.rs`: `update`,
  one frame (whole ticks, then camera, targeting, instances).
- `api/`: the JS API, one `#[wasm_bindgen] impl Game` per area; methods only forward. `engine.d.ts` (generated) lists them.
  `input`, `view`, `render`, `inventory`, `machine`, `crafting`, `research`, `creative`, `content` (names, icons, getters),
  `hud`, `sites`, `save`, `net`, `minimap`, `prospect`, `analytics`, `towns`, `orbit`, `ruins`, `opening`, `blueprint`,
  `hand_waste`, `lessons`, `guide`, `surroundings`, `debug` (give, teleport, run_ticks, scene building).
- `events.rs`: `SimEvent`s → item spawns, sounds, toasts, analytics counts. `authority.rs`: every body, loose items, join/leave.

### Deterministic core (changes only through `Action`s at a tick)
- `sim.rs`: `Sim` (tick, world, factory, players, rng, towns), `step`, `state_hash`, `PlayerId`, `SimEvent`.
- `sim/`: `timers.rs` (leaf decay, grass, saplings), `water.rs` (flowing water rules, capped per tick), `saplings.rs`,
  `torches.rs`, `scrap.rs` (salvage from ruins), `vault.rs` (the ruin vault's dial puzzle).
- `action.rs`: the `Action` enum. `action/`: `apply.rs` (one arm per action), `clicks.rs` (inventory and box clicks),
  `blocks.rs` (break and place), `multiblock.rs` (footprints), `codec.rs` + `codec/read.rs` (bytes; tags append only).
- `bytes.rs`: canonical encoding (`write_state` / `read_state`). `save.rs`: the file (core, bodies, loose items), `SAVE_VERSION`.
- `join.rs`: compile-time joins of const tables (`TECHS`, `MACHINE_RECIPES`). `math.rs`: vectors, hashes, `Rng`, small sort.
- `doors.rs`: `Action::ToggleDoor` swings a door (and the doors stacked on it) and the use button on one (`block/house.rs`).
- `deposits.rs`: deposit pools, tiers, draw caps, taper, `HAND_YIELD`. `daytime.rs`: time of day and sunlight from the tick.
- `inventory.rs` + `inventory/stacks.rs`: slots, cursor, clicks. `equipment.rs`: worn gear, packs. `offhand.rs`: the left-hand
  stack (swap, click, the tool that suits a block). `tools.rs`: tool tiers.
- `crafting.rs`: the hand-craft queue. `take_apart.rs`: taking a hand-crafted item apart (most inputs back, iron as scrap). `perks.rs`: `player_bonus` (techs plus gear). `mode.rs`: normal or creative.
- `research.rs`: `Tech`, `Unlock`, `Research`. `research/`: tech tables by era (`techs.rs` first, then `electronics.rs`;
  `later.rs` is the open end), `queue.rs`, `bonus.rs` (endless techs).
- `ghosts.rs`: planned blocks. `drones/`: construction drones, `earthworks.rs` (terraforming), `cargo/` (cargo routes).
- `helpers/`: jetpack, personal drone, hover pack. `blueprint/mod.rs`: blueprint data and bytes.
- `satellites.rs`: satellites in orbit. `pods.rs`: drop pods in flight. `opening.rs`: a new world's opening scene.
- `settlements/`: towns (Milestone 13). `orders.rs` data (orders, upkeep), `run.rs` (the tick), `plans.rs` and `plots.rs`
  (buildings), `build.rs` (the town's drones), `state.rs` (bytes), `perks/` (forge, workshop, relay, `harbour/`).
- `hints.rs`: onboarding tips (checks on the core; `Sim::step_tips` latches the opening's and the main tips' progress).
  `lessons.rs`: the bits of `Sim::taught` (what the world was taught; `Action::LessonTaught`).

### Factory (core): `factory/`
- `mod.rs`: `Factory` (a `Vec` per kind, cell index, place, remove). `tick.rs`: `update`, one tick. `table.rs`: `Kind`,
  `MACHINES`. `machine.rs`: the `Machine` trait. `state.rs`: bytes of every list. `tiers.rs`: tier families.
  `upgrades.rs`: kits, in-place upgrades. `tier_mark.rs`: the Mk studs every tiered machine wears (presentation).
- Logistics: `belt.rs` (`BELT_TIERS`, `belt_step`), `provenance.rs` (the tag each belt item carries: maker and ticks on belts),
  `belt_items.rs` (the crosshair picks a belt item, reads its tag, takes it, finds it again by `ItemKey`), `belt_shape.rs` (ramps, lifts), `underpass.rs`, `belt_chain.rs`,
  `router.rs` (splitter, filter), `storage.rs`, `buffer.rs`, `links.rs` (`relink`: where items go), `ports.rs` (drone pads).
- Extraction: `miner.rs` (+ `miner_model.rs`; `miner_burner.rs` the coal miner's fuel and fire), `quarry.rs` + `quarry/` (dig box, model), `pipes.rs`, `pumping.rs`.
- Power: `power.rs` (grids, `balance`), `pole.rs` + `pole_mount.rs` (what a pole stands or hangs on) + `pole_tower.rs` (the long-distance tower's model), `wiring.rs`, `cable.rs`, `generator.rs`, `power_mix.rs` (sources),
  `grid_colour.rs`. Data and light: `fibre.rs` (data grid), `laser.rs` (beams). Logic: `sensor.rs`.
- Rails: `rail.rs` + `rail/curve.rs`, `trains.rs` + `trains/` (cars, schedule, signals, docks, model).
- `lab.rs`: research labs. `sites.rs` + `sites/` (terraforming sites, survey, tunnels). `footprint/`: multi-block machines.
- `process/`: every inputs-to-outputs machine is one `Processor` driven by a `ProcessSpec` row (`specs.rs`; the first
  machines' rows in `basic.rs`); `mod.rs`, `work.rs` (the loop), `state.rs` (bytes), `extra.rs` (one kind's own state), `kinds.rs` (index lists for the per-kind passes), `status.rs`, `view.rs`, `model.rs` + `parts.rs` (models as data), `display.rs` (the turning product display, `Look::Product`), `shape.rs`,
  `intake.rs`, `legacy.rs`, `facts.rs`. Machines with logic of their own: `steam.rs` (+ `steam_view.rs`), `solar.rs`,
  `diesel.rs`, `hydro.rs`, `nuclear.rs`, `rectenna.rs`, `ground.rs`, `pump.rs`, `refinery.rs`, `washer.rs`, `recycler.rs`,
  `center.rs` (research center), `ailab.rs`, `datacenter.rs`, `tower.rs`, `optimizer.rs`, `hub.rs`, `fab.rs`, `launch.rs`,
  `pods.rs` + `pods/channels.rs` (launcher, pad, their colour channel), `hangar.rs` (drone port), `hoist.rs`, `docks.rs`, `depot.rs` (town depot), `ejector.rs` (belt ejector: drops items as `SimEvent::Dropped`), `upgrader.rs`, `laser.rs`.
- Readouts and views: `describe.rs`, `panel.rs`, `facts.rs`, `efficiency.rs` (analytics samples), `render.rs` (instances; `push_round` round bodies, `fake_round` the check none is faked from boxes),
  `hitbox.rs`, `smoke.rs`, `placed.rs` (what blueprints copy).

### Content tables (data; append ids, never renumber)
- `block/`: `mod.rs` (ids), `defs.rs` (`DEFS` rows), `make.rs` (row builders), `tables.rs` (hot-loop lookups), `tex.rs` +
  `tex/later.rs` (texture layers), `turn.rs` (tile turning), id ranges in `chemistry.rs`, `forest.rs`, `ruins.rs`,
  `decor.rs`, `coal_miner.rs` (id 143), `house.rs` (ids 144–156: doors, roofs, walls; `toggled` pairs the doors), `ejector.rs` (id 157); `ores.rs`.
- `item.rs` + `item/` (`later.rs` ids, `rows.rs` builders): `ItemId`; ids below 256 are blocks.
- `recipes/`: `mod.rs` (`RECIPES`, hand; the first era's rows in `first.rs`), `machine.rs` (`MACHINE_RECIPES`, `FUELS`;
  Milestones 10–11 in `machine_later.rs`), `rows.rs` (row indices), `tiers.rs`, `refit.rs`, `timing.rs`, `group.rs` +
  `menu_order.rs` (build menu), `recycling.rs` (coin values), `content.rs` (raw materials in an item); one file per era or set: `materials`, `tooling`, `heavy`, `electronics`, `aluminium`, `transport`, `wiring`, `solar`, `chemistry`, `compute`,
  `gear`, `gold`, `orbit`, `automate`, `recycler`, `scrap`, `towns`, `decor`, `house` (hand recipes).
- `item_models.rs` + `item_models/`: box and cylinder assemblies for loose items and icons (`shapes.rs` helpers; `power.rs` the poles; `belts.rs` the belt family and ejector; `icon_frame.rs` fits each icon to its box).
- `textures.rs` + `textures/`: procedural placeholder layers, one file per set (the art pack overrides them in the browser).

### World and generation
- `chunk.rs` (32³ storage), `world/mod.rs` (loaded chunks, `*_anywhere` accessors for core code), `world/streaming.rs`
  (generate and mesh queues), `world/boxlight.rs` (light for box-drawn things), `world/cover_clear.rs` (columns whose
  ground plants the sickle cut: core state, saved).
- `worldgen/mod.rs`: heights, caves, trees, `generate`, generator versions (pinned by `released_versions_never_change`).
  `column.rs` (per-column data), `biome.rs`, `landform.rs`, `caves.rs`, `water.rs` (sea, ponds), `rivers.rs` + `rivers/` (course, seg), `shore.rs`,
  `trees.rs` (species), `geology.rs` + `strata.rs` + `ore.rs` (deposit seeding), `bearing.rs`, `ruins.rs`, `logbook.rs`.
- `noise.rs`: Perlin and fBm. `light.rs`: sky and block light while meshing (render cache only).

### Local player: hands, tools and previews (presentation; act only by queuing actions)
- `interaction.rs` + `interaction/aim.rs`: targeting, mining, placing, footsteps. `player.rs`, `physics.rs`, `raycast.rs`,
  `hitbox.rs`, `camera.rs`, `strategy.rs` + `strategy/navigation.rs` (overhead camera, walking orders).
  `item_select.rs` (a clicked belt item stays selected: readout, outline, pick-up in reach).
- `belt_line.rs` + `belt_line/` (drag-built lines, underpass bridging, `route.rs` A* auto-routing), `upgrade_aim.rs`.
- `place_refusal.rs` (a placement the body or a taken cell stops: red box, "step aside", `sound::DENIED`).
- `quarry_preview.rs`, `footprint_preview.rs`, `town_preview.rs`, `ghost_mode.rs`, `power_tools.rs` + `power_tools/`,
  `rail_tools.rs`, `train_tools.rs`, `cargo_tools.rs`, `site_hands.rs`, `helper_hands.rs`, `blueprint/hands.rs`,
  `blueprint/repeat.rs` (blueprint prompt), `stall_lesson.rs` (the machine-stopped lesson: card, prompt, taught when mended), `prospect.rs` (scanner, core drill), `survey.rs` + `survey/orbit.rs`,
  `telescope.rs` + `telescope/` (`ground.rs` long ray over loaded blocks then the heightfield, `beacons.rs` pin columns),
  `sickle.rs` (right-click cuts a 3×3 patch of ground plants: `Action::ClearCover`).

### Presentation: drawing, sound, maps, analytics (never feeds the core)
- `mesher.rs` + `mesher/` (`quad.rs`, `water.rs` surface shape, `cover.rs` ground cover and indicator plants).
- `entities.rs` + `entities/merge.rs` (loose items; resting ones merge into stacks), `avatars.rs` + `avatars/`, `held_light.rs` (the light a torch in a hand gives: reach and what the frame reports), `drone_view.rs`, `orbit_fx.rs`, `sound.rs` (event buffer).
- `minimap.rs` + `minimap/` (`atlas.rs` explored map, `marks.rs` the marks as `Subject`s, `describe.rs` their words for the hover box, `sites.rs`), `ore_guide.rs`, `hand_waste.rs`.
- `item_guide.rs`: every way to get an item (found, hand, machine) and what uses it; a test makes every needed item answerable.
- `analytics/`: history and efficiency (`series.rs`, `machines.rs`, `power.rs`, `groups.rs`); never saved.
- `surroundings/`: nine floats for ambient sound, `roof.rs` (where rain stops).

### Co-op
- `net/mod.rs`: lockstep `Role`, frames, `INPUT_DELAY`, `CHECKSUM_TICKS`. `snapshot.rs` (join, resync), `players.rs`
  (bodies at 20 Hz), `items.rs` (host-owned loose items).

### Tests and benches
- `tests.rs` (scenarios), `tests/frame_bench.rs`; ignored `bench_*` tests time hot paths (`cargo test --release -q bench_
  -- --ignored --nocapture`): meshing, big base, trains, towns, water, drones, orbit, worldgen, machine marks (`factory/tests/bench_marks.rs`).

## Web host: `web/src` (TypeScript + WebGL2, thin platform layer)

- `main.ts`: bootstrap, frame loop (`advance`, `frame`). `pause.ts`: play, pause, pointer lock, Escape. `input.ts`: keys
  and mouse, one-shot `Action`s. `controls/view.ts`: perspective input. `base.css`: theme and shared classes.
  `patch-notes.md`: "What's new".
- `render/`: `renderer.ts` (chunk passes, fog), `geometry.ts`, `shaders.ts` (GLSL), `boxes.ts` (instanced boxes, and round bodies from `round-mesh.ts`),
  `water.ts`, `outlines.ts` (overlays: cracks, target outline, belt line, boxes), `precip.ts` (rain, snow), `sky.ts` +
  `sky/` (stars, weather, clouds, wet ground), `art-pack.ts` (merges the 128 px pack from `art/`), `held-light.ts` (the held lights' uniform), `gl.ts`, `mat4.ts`.
- `audio/`: `sound.ts` (events → voices, `AudioContext`), `synth.ts` + `synth-orbit.ts` (foley), `settings.ts`,
  `ambience.ts` + `ambience-levels.ts` + `ambience-mix.ts` + `synth-ambience.ts`, `surroundings.ts`,
  `music.ts` + `music-director.ts` + `music-pieces.ts` (one row a piece) + `music/`.
- `net/`: `session.ts` (start co-op from URL or menu), `host.ts`, `client.ts`, `protocol.ts`, `coop.ts`, `transport.ts`,
  `webrtc.ts`, `broadcast.ts`, `signal.ts`, `ticker.ts` (keeps a hidden host ticking).
- `save/`: `store.ts` (IndexedDB, gzip), `session.ts` (autosave, switching worlds), `export.ts` (`.ocworld` downloads), `leave-guard.ts` (the ask-before-closing setting and rule; `ui/leave-prompt.ts` draws it). `comfort/settings.ts`: comfort options.
- `console/`: `commands.ts` (pure admin commands), `host.ts` (wires them to the engine).
- `ui/`: one component per `<name>.ts` + `<name>.css`; `dom.ts` builds elements. Screens and panels:
  - HUD: `hud.ts` (+ `hud-columns.css`; `itemIcon` draws every icon through `icon-paint.ts` and `icon-geometry.ts`), `icon-sprite.ts` (flat icons), `hints.ts`, `key-prompt.ts`, `prospect.ts`, `nametags.ts`, `scope.ts` (telescope view and pins), `vignette.ts`,
    `minimap.ts`, `compass.ts`, `craftqueue.ts`.
  - Inventory and building: `inventory.ts`, `pager.ts`, `slot-controls.ts`, `crafting.ts` (+ `craft-nav.css`),
    `craft-card.ts`, `craft-info.ts` (a tile's hover card), `item-guide.ts` (an item's page: how to get it, what it
    makes), `take-apart.ts`, `offhand.ts` (the left-hand slot by the hotbar), `held-name.ts` (the selected item's name over the bar), `drag-throw.ts` (drag a stack out to throw it), `craft-terms.ts` (search synonyms, ranking), `creative.ts`, `pack-upgrade.ts`.
  - Machines: `machine.ts`, `machine-recipes.ts`, `quarry.ts`, `pod-channel.ts`, `town.ts`, `harbour.ts`, `site.ts`, `logbook.ts`, `vault.ts`.
  - Research: `research.ts` (+ `research-card.css`), `research-view.ts`, `tech-tree.ts` (pure layout), `research-eras.ts`,
    `research-node.ts`, `research-map.ts`, `research-search.ts`, `research-queue.ts`.
  - Maps: `worldmap.ts`, `pins.ts` (+ drawing), `map-filter.ts` (pure filter rules, saved state) + `map-filter-panel.ts`, `map-info.ts` (hover box), `ore-guide.ts`. Analytics (P): `analytics.ts`, a tab each in
    `analytics-overview.ts`, `analytics-power.ts`, `analytics-items.ts`, `analytics-machines.ts` (base `analytics-tab.ts`),
    `analytics-chart.ts`, `analytics-format.ts`, `analytics-panels.css`.
  - Menu: `menu.ts` (+ `menu-pages.css`), `worlds.ts`, `coop.ts`, `players.ts`, `patch-notes.ts` (+ `inline-text.ts`: bold and code spans), `comfort.ts`, `sky.ts`,
    `desktop-only.ts`, `blueprints.ts`, `console.ts`, `view.css`.
  - Sound: `sound-panel.ts`, `sound-lab.ts`, `sound-lab-footer.ts`, `volume-control.ts`, `knob.ts`.
- `wasm/`: generated by `npm run build:wasm` (gitignored); `engine.d.ts` is the API reference.
- Pure logic tests: `web/test/*.test.mjs` (`node --test`).

## Outside the game

- `signal/`: the Cloudflare signalling Worker (`src/index.ts` routes, `src/room.ts` the room); deploying: WORKFLOW section 4.
- `scripts/`: `check.mjs` (`npm run check`), `check-size.mjs` (budgets), `check-map.mjs` (this map and the core layer),
  `check-lessons.mjs` (saves must stay quiet), `check-music.mjs`, `build-wasm.mjs`, `dev.mjs`, `wasm-sizes.mjs`, `inspect-save.mjs` (read a save headlessly),
  `bench-flight.mjs` (frame timing on a real save), `bake-sky/` (star data, not part of the build).
- `tools/art/`: the art pack's Python tool (`build.py`, `check.py`); see `docs/ART_STYLE.md`.
- `.github/workflows/pages.yml`: CI on push to `main` (check, build, deploy).

## Where the tuning knobs live

Balance numbers are named constants atop their module; grep the name for the value.
- Ticks and co-op: `TICK_RATE` (`lib.rs`), `INPUT_DELAY`, `CHECKSUM_TICKS` (`net/mod.rs`), `MAX_PLAYERS` (`authority.rs`).
- Mining: `HAND_YIELD`, taper (`deposits.rs`), tool tiers (`tools.rs`), `MINER_TIERS` (`factory/miner.rs`).
- Machines: `MACHINES` (`factory/table.rs`), `SPECS` tiers and buffers (`factory/process/basic.rs`, machine files), `MACHINE_RECIPES`,
  `FUELS` (`recipes/machine.rs`), `BELT_TIERS` (`factory/belt.rs`), `POLE_TIERS`, `GENERATOR_TIERS`, `LAB_TIERS`,
  `BOX_SLOTS`, `QUARRY_TIERS`, `PUMP_TIERS` (their `factory/` files).
- Research: `TECHS` (`research/techs.rs` and the era files). Hand recipes: `RECIPES` (`recipes/mod.rs`).
- Towns: `UPKEEP_PER_MINUTE`, `ORDERS` (`settlements/orders.rs`), perk rates (`settlements/perks/`).
- World: `SEA_LEVEL` (`worldgen/mod.rs`), biome thresholds (`worldgen/biome.rs`), ore tables (`worldgen/geology.rs`,
  `strata.rs`, `ore.rs`), rivers (`worldgen/rivers.rs`), `WATER_DELAY` (`sim/water.rs`), timers (`sim/timers.rs`).
