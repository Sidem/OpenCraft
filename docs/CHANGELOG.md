# Development history

Earlier development entries, retained from DEV_PLAN.md so the active plan stays small.

## 8. Change log of this plan

- **2026-10-04: Scanner Mk2 reads more.** `prospect::SCANNERS` rows gained an "advanced" flag. The Mk2 filters its list by ore (R, `Game::cycle_scan_filter` through `FILTER_ORES`; `prospect_records` filters, so nothing rescans), shows each deposit's ore units (exact when tracked, else estimated from the shape: `estimated_units`, within 20% of a survey) and the minutes a full-speed mine takes (units over the tier's draw cap), and the panel (`ui/prospect.ts`) leaves a pointer to the nearest vein or lode of the match at the top of the screen whatever is in hand. `SCAN_FIELDS` 6 → 8. Queries only: no save or hash change. Tests 441 → 442.
- **2026-10-04: Terraforming by drone (steps 8.1–8.3).** The Planner (item 332, tech Earthworks, 140 × 35 s) marks two corners (reach 64) and opens the site panel (`ui/site.ts`: job, level, survey, remove). Drone ports now work sites (`drones/earthworks.rs`): cuts break by hand into the pad's boxes (they wait while full), fills bring dirt or stone from the boxes; a finished site removes itself. `Site::done` is an unsaved cursor cache. `Research` got a manual `Default` (33 techs). Tests 431 → 435, golden hash re-recorded (a tech was added), no save bump. Wasm size not re-measured.
- **2026-10-05: Milestone 9 cleanup (step 9.6).** Onboarding tips for far ground, trains and hover/cargo (`hints.rs`); `bench_trains`: 14 trains cost 4.0 µs a tick, worst 31 µs (6.4 / 11 µs with signals); balance note in section 4; the Milestone 9 step list moved to `docs/CHANGELOG_M9.md`; Milestone 10 moved in from the roadmap.
- **2026-10-05: Cargo drones (step 9.5b).** The Cargo Drones tech and item 347: drone ports become stations; a route between two ports (set by clicking them with a cargo drone in hand) sends drones with a stack from the first port's boxes to the second's, burning batteries by distance (`drones/cargo/`, `cargo_tools.rs`, `Action::SetRoute`); save version 37, golden hash re-recorded. Fixed a duplication bug in `store_in_boxes` / `port_land` (drone deposits and landings also dropped loose copies). Tests 518 → 527.
- **2026-10-05: Hover pack (step 9.5a).** The Hover Pack tech (index 40, after Jetpack and Bauxite Processing) and item 346: a pack that hovers while jump is held in the air (height held, jump rises, crouch sinks, 8 blocks a second, 13 sprinting; `Player::hover`), running on `charge` (90 s at most) that refills at twice the drain while the player stands within 6 blocks of a power pole (`Action::Hover`, `Action::Charge`; `helpers/`, `helper_hands.rs`). It spends the first use of the battery. Save version 36, golden hash re-recorded. The pack draws nothing on the grid. Tests 513 → 518.
- **2026-10-05: Signals (step 9.4c).** Rail signals on nodes cut the track into sections that hold one train at a time; trains wait at a signal or take a free branch (`factory/trains/signals.rs`); `Action::ToggleSignal`; save version 35. Tests 505 → 511.
- **2026-10-04: Milestone 8 done (steps 8.4–8.5).** Tunnels: the planner panel's Tunnel job bores between two blocks; drone ports cut it into their boxes like a dig, leaving cells next to water (`Site` got `covers` / `cuts_at` / `picks`; `sites.rs` split into `sites/survey.rs` and `sites/tunnel.rs`). Cleanup: `bench_earthworks` (worst tick 0.5 ms), sites on the maps as hollow squares, a planner tip, outline colours from the Okabe-Ito palette. Tests 451 → 458, wasm 329 KB gzipped. Milestone 9 moved in; the user asked for bearings to far biomes and for trains (section 1).
- **2026-10-04: Equipment slots.** Worn gear (equipment.rs): four slots (back, boots, torso, tool belt) in Inventory.worn, so a player who leaves keeps them. Hauler packs (+9 and +18 backpack slots: the inventory array is 54 long, capacity() of it is in use, a pack stays on while its rows hold stacks), spring boots (jump 1.32 → 2 blocks), servo boots (walk and sprint +15%), exo frame (sprint +10%), mining rig (hand-breaking +50%). Movement bonuses are a Boost the authority sets on the body each tick (bodies are not core), the rig is read by interaction.rs. New: Action::ClickGear (tag 39), shift-click on gear in the pack wears it, six items (333–338) with icons (`textures/gear.rs`), six hand recipes (`recipes/gear.rs`) and three techs (Hauler Gear after Steelmaking, Field Gear after Steel Tools, Exosuit after Robotics and Field Gear: `research/personal.rs`, which also took over Jetpack, Personal Drone and Earthworks; TECHS joins the two tables at compile time). Save 28 → 29 (pack rows and worn gear follow each inventory), golden hash re-recorded. Tests 442 → 451.
- **2026-10-05: Schedules (step 9.4b3).** A train follows a list of up to 8 docks, routed by a breadth-first search at junctions (`factory/trains/schedule.rs`), edited with the locomotive in hand (`Action::TrainStop`); save version 34. Fuel and dock filters deferred to a train panel. Tests 500 → 505.
- **2026-10-05: Wagons and docks (step 9.4b2).** The Freight tech: wagons coupled behind a locomotive (24 cargo slots each, up to 6), a loading dock and an unloading dock (spec rows, belts on every side); a train with wagons stops at a node beside a dock, trades 8 items a tick and drives on after 5 idle seconds. Save version 33, golden hash re-recorded. Schedules and fuel are 9.4b3. Tests 490 → 500.
- **2026-10-04: Locomotives (step 9.4b1).** The Trains tech and the locomotive item: a train is a stretch of the track graph moving at 9 blocks a second along the curves, turning round at dead ends and taking the straightest track at junctions (`factory/trains.rs`); `PlaceTrain` / `TakeTrain` actions; save version 32, golden hash re-recorded. Wagons, stations and schedules are 9.4b2. Tests 481 → 490.
- **2026-10-04: Rails (step 9.4a).** The Rails tech and the rail block. First built as a block per cell dragged like belts (save version 30), then **redesigned at the user's request as nodes and curves**, laid like power poles: nodes on the grid with a heading, smooth Hermite track between joined nodes (`factory/rail.rs`, `rail/curve.rs`, `rail_tools.rs`); save version 31, golden hash re-recorded. 9.4b (trains, stations, routing by search over the track graph) is specified in section 4. Tests 467 → 481.
- **2026-10-04: Bearings to far ground (step 9.3).** The Mk2 scanner, filtered on an ore with none in range, names the way and distance band to the nearest biome that holds it (`worldgen/bearing.rs`, a pure query) and the pointer leads there. Tests 460 → 467.
- **2026-10-04: Aluminium (step 9.2).** Bauxite Processing: crushed bauxite, the electrolytic cell (block 77, 300 kW, slag byproduct hatch), aluminium ingot and plate, battery; a third tech table `research/distance.rs`. Golden hash re-recorded (one more tech). Tests 459 → 460.
- **2026-10-04: Bauxite (step 9.1).** Generator version 5 for new worlds: bauxite (block 76) in far deserts and basalt fields only, at least 600 blocks from spawn, shallow and exposed on bare rock, no stain; guide, scanner filter, map colour, a placeholder texture. Tests 458 → 459.

- **2026-10-04: Steam through pipes.** A boiler's faces now have named ports (`BOILER_PORTS` in `process/specs.rs`; `Port::cell` / `Which` picks one cell of a side, `Role::Water` and `Role::Steam` are pipe roles): on each of back, left and right one cell is a coal inlet (belt) and the other a water inlet (pipe), and the two front cells are steam outlets; a turbine has a steam inlet on both cells of its right end (the end away from its generator). A pipe joins a port only on that face (`Processor::pipe_ports`); steam no longer passes between touching machines. Turbines take steam from the boilers on the same pipe network (`steam::link`, still two per boiler, lower index first). A network is `Fluid::Water`, `Steam` or `Mixed` (both: works for neither, pipes turn red, machines say so). Water pipes are blue banded and steam pipes pale and red banded (`textures/piping.rs`, `tex::PIPE_WATER/PIPE_STEAM`, `COUNT` 180). `process/boiler_view.rs` became `steam_view.rs` (coal chutes, water and steam nipples on flanges, gauge; the turbine's inlets); the boiler's firebox door moved up the front. No save or hash change, but saves with a turbine standing against a boiler lose that link and need pipes. Tests 438 → 441.

- **2026-10-04: Boiler connections drawn.** The boiler lost its generic belt hatches: coal chutes (dark funnel, coal on top) mark free belt inlets, a steel flange marks every pipe touching the tank (`process/boiler_view.rs`, from `Steam::taps`), and a water gauge on the front fills with the water held. Pipes and pumps beside a boiler now draw an arm into it (`Pipework::arms`). A boiler now draws water from every network touching it (`Steam::nets`; it used to listen to the first one only, so a second pump on its own network did nothing). Presentation plus a derived-state fix: no save or hash change. Tests 435 → 438.

- **2026-09-25: Plan created; Milestones 1 and 2 done** (`3239215` to `3bf3f92`): fixed tick, core `Sim`,
  actions, state hash, saves; items, machines, belts, power, research, Mk2, tips. Tests 56 → 113, wasm
  120.3 KB. Lessons: measure wasm every step; the golden hash catches unintended core changes.
- **2026-09-26: Milestone 3 (Co-op) done** (pushed `c845691`): lockstep over bytes, join snapshots, player
  keys, avatars, the signalling Worker (STUN only), WebRTC, pings, timeouts, in-place resync; 3.9
  deferred. About 8 KB/s down, 4 up while idle. Lessons: heavy sort keys `#[inline(never)]`; edit files
  with the file tools (PowerShell reads UTF-8 as ANSI). Tests 113 → 127; wasm 131.5 KB gzipped.
- **2026-09-26: Play-test notes P1–P5 done** (`233371b`): Escape hints before pausing, the minimap, block
  timers (`sim/timers.rs`, save version 11: leaf decay, grass), tools (wear is the stack count). The `art`
  branch started (`docs/ART_HANDOVER.md`). Tests 127 → 141; wasm 138.0 KB gzipped.
- **2026-09-27: Milestone 4 (Reasons to explore) done** (`8d8ee22` to `a4fc499`): generator versions
  (version 1 pinned), saplings, rocks, limestone, quartz, glass, biomes (version 2 about 10 % slower),
  ores by biome with soil hints, scanner and core drill, map marks, day and night, sky and block light
  (meshing 0.36 → 0.49 ms per chunk, +25 % mesh memory), lamps; the art salvage and the build menu grid.
  Save version 12. Tests 141 → 183; wasm 138.0 → 161.4 KB gzipped.
- **2026-09-27: Belt placement upgrade** (`0bc0e47`): ramps derived from placement, drag-to-build lines
  (`belt_line.rs`), R rotates, research on T. Tests 183 → 193.
- **2026-09-27: Milestone 5 (Water and world shape) done** (`becc673` to `c1c61f9`): generator version 3
  (rare surface ore, depth bands, starter set), the sea and ponds, swimming, flowing water as core state
  (only the sea is endless), pumps, pipes, outlets, the quarry. Save versions 13–15. Tests 193 → 225;
  wasm 161.4 → 192.5 KB. Lessons: test the next step's scenario early (two-source water blocked one);
  PowerShell array patches misfire on a single pair.
- **2026-09-27: Power rework and 5.9** (`a682a28`): every miner needs power; generators store fuel energy
  and give only what is drawn (coal 270 kJ); the coal-miner-feeds-its-generator loop. Save version 16.
  Tests 225 → 226. Then Milestone 6 (Terraforming) was detailed.
- **2026-09-27: Torches and finding ore** (`11eee3b`): torches (`sim/torches.rs`), light sources as rows
  of `light::SOURCES`; generator version 4 (shallower bands, more exposed metal, two starter patches),
  the explored map and world map (M) with pins, ore guide, stained-soil readout. Tests 226 → 234; wasm
  192.5 → 201.6 KB (`to_lowercase` alone cost 14 KB, avoided).
- **2026-09-28: Sites in the core, wood, glass, lights, lifts** (`b33f98c` to `2d89533`): `factory/sites.rs`
  (save version 17; tags 21, 22), planks, sticks, ladders, sand → glass, torch reach 11, lamp 31
  (`bench_meshing` 0.56 ms per chunk), climbable lifts. Tests 234 → 240; wasm 204.5 KB.
- **2026-09-28: Tech tree and Industry** (user request): `docs/TECH_TREE.md` (the concept, upgrades, the content
  architecture) and `docs/TECH_ERAS.md` (per era). Industry became Milestone 6, terraforming moved to 8 (its
  sites step, built as 6.1, stays in the core); roadmap 7–13. No code changed.
- **2026-09-28: 6.1–6.6** (`8c22a45` on): tiers as data (`factory/tiers.rs`), `research::Unlock`, recipe
  categories, the content lint, upgrade kits (`Action::Upgrade` tag 23); one processing machine
  (`factory/process/`, save 18) with Masonry; multi-block footprints and the assembler (save 19); steel (the
  blast furnace with a byproduct port). Tests 240 → 270. Ctrl also crouches, and closing the tab mid-game asks.
- **2026-09-29: 6.7–6.7b:** blue science and Mk3 everything (labs with three pack slots, save 20; tiers for the rest, save 21). Tests 270 → 286.
- **2026-09-29: Milestone 6 (Industry) done** (6.8–6.10): steam (boiler, turbines as power sources in `power.rs`), the
  crusher, the silo, techs 14–16, four tips. Tests 286 → 296; wasm 232.7 KB gzipped. Lessons: write doc comments
  with the file tools or single-quoted here-strings (double-quoted PowerShell here-strings eat backticks); a plant
  test with a 405 kW load caught two turbine bugs a unit test would not. Milestone 7 moved in from the roadmap.
- **2026-09-29: Play-test notes after Milestone 6** (`Testing 5.ocworld`; pushed 28e1bf9): sort buttons for
  the backpack and boxes (`Action::SortInventory` tag 24, `SortBox` tag 25); boxes are lit by their cell
  (`world/boxlight.rs`); saved worlds open without their old guests (`Game::release_guests`), and a returning
  key replaces its old connection at once. Tests 296 → 303; wasm 235.2 KB gzipped.
- **2026-09-29: Box screen bug, step 7.1 and 7.1b:** a small box opened after a larger one showed the larger box's
  extra empty slots (`ui/inventory.ts` hides them). Step 7.1 built (arc furnace, silicon, circuits, Electronics tech).
  7.1b: the power cable block and a pole tool (ghost at full reach, hold to chain, R to place freely). Tests 305 → 316.
- **2026-09-29: Step 7.2** (violet science, Mk4 for eight families; save 22). Tests 316 → 323; wasm 235.5 KB gzipped.
- **2026-09-29: Step 7.2b** (bootstrap without raw ore; the hand-craft queue with timed crafts and automatic part crafts; save 23). Tests 323 → 336; wasm 239.3 KB gzipped.
- **2026-09-29: Step 7.3** (solar panel and accumulator; one tech). Tests 336 → 341; wasm 241.2 KB gzipped.
- **2026-09-30: Step 7.3b, power wiring by hand** (slots per pole, manual wires, auto-wire only to the nearest powered pole, free pole placement with Shift for full reach, routers need no power; save 24). The golden hash in `sim/tests.rs` was re-recorded. The research screen is now a tech tree with hover cards (`ui/research.ts`, `ui/tech-tree.ts`).
- **2026-09-29: Craft queue by step, timing in one file:** each order shows its steps (parts, then the item) in `ui/craftqueue.ts` (`craft_steps`); hand craft times are tuned in `recipes/timing.rs` (formula constants plus a per-recipe `OVERRIDES` table). Tests 341 → 344.

- **2026-10-01: Belts are assembler-made** (user request: everything must be automatable; green packs need belts). Row 30 in `MACHINE_RECIPES` (1 plate + 1 rod -> 4 belts, assembler), unlocked by Assembly; existing saves get it. Test `belts_are_machine_made_for_green_packs`.
- **2026-10-02: Kit and inventory UX** (user suggestions): a held upgrade kit outlines the aimed machine or belt and says the kits needed (`upgrade_aim.rs`); Shift-click on a belt upgrades its whole line (`Factory::belt_chain`); belt lines and kit upgrades draw on every stack, not just the held one; Shift-right-click moves every stack of an item between box and inventory (`Action` tags 29–31). `action.rs` split (`action/blocks.rs`). Tests 344 → 364.
- **2026-10-02: Comfort settings** (user felt nauseous): a "Comfort settings" section in the pause menu (`ui/comfort.ts`, `comfort/settings.ts`, localStorage): field of view, mouse sensitivity, a movement vignette (`ui/vignette.ts`), a bolder crosshair with style, size, thickness and opacity; the eye eases between standing and crouching (`camera.rs`). Presentation only. Tests 364 → 366.
- **2026-10-02: Third-person view and Kestrel avatar** (comfort option, V toggles): eased, collision-tested right-shoulder camera; camera-ray targeting still checks hand reach and visibility. Procedural ivory/teal survey robot with articulated walking, crouching, airborne/water and working poses, held items and correct crosshair-facing head; pickaxe points and axe edges lead the mining stroke. Co-op relays pose flags and velocity; box stride comes from the engine. Save-free `/character-preview.html` for review. Tests 366 → 378; saves and core hashes unchanged.
- **2026-10-03: Drones, jetpack, personal drone (steps 7.7b–7.9, save 28).** The drone port (a 3×3 pad on power, tiers by kits) keeps drones; they fly out one at a time from powered ports to build your ghosts from the storage boxes touching the pad and to tear down blocks you mark (left-click in ghost mode), putting the drops in the boxes. A coal jetpack (hold jump in the air) and a personal drone (Y fetches the held item from a box within 32 blocks). Milestone 7 is done.
- **2026-10-03: The drone ladder (step 7.7a).** Five violet techs (Processors, Robotics, Drone Power, Navigation, Construction Drones; twelve techs deep from the start of Violet Science, 920 units of all four packs for the last five) and six assembler-only parts that make a drone: processor, servo, actuator, drone cell, guidance module, drone (about 14 motors' worth of circuits and steel each). No save bump; golden hash re-recorded. The port and the flying come next.
- **2026-10-03: Blueprints (step 7.6b).** Ghost mode gains area copy: Z marks two corners, Enter copies the machines in the box (`blueprint/`: block, facing, tier, offset; turned in quarter turns, multi-block machines keep their shape), the use button stamps them as ghosts through the new `PlantGhost` action, and the L panel (`ui/blueprints.ts`) lists, renames, holds and deletes blueprints. They live in the browser per world like pins, not in the save. Settings and plain blocks are not copied yet.
- **2026-10-03: Ghosts (step 7.6a, save 26).** Planned blocks and machines are core state (`ghosts.rs`: a cell, block, facing, tier; `PlaceGhost` / `RemoveGhost`, tags 33/34), so co-op peers and later drones share them. B toggles ghost mode (`ghost_mode.rs`, presentation): right-click plants the held block's ghost for free or removes the aimed one, R turns it, reach 16, no mining, cyan outlines and a needs list. Placing the real block on a ghost uses its facing and clears it. Golden hash re-recorded.
- **2026-10-03: Logic (step 7.5, save 25).** The sensor block (Logic tech, violet science): reads the box, silo or belt behind it and cuts the power wire of the machine in front by a rule (switch, or fullness thresholds with hysteresis); right-click steps rules, R turns, a lamp shows the state. New `Kind::Sensor` with its list saved after the quarries, `Action::SetSensor`, `Hooked::off` / `Power::off` (readouts say "Switched off by a sensor"). Tests: 405 pass (5 new), golden hash re-recorded. Not tried in the browser yet.
- **2026-10-03: Scanner Mk2 (step 7.4).** Advanced Scanning (r g b v) unlocks the Scanner Mk2 (item 320: a scanner, 3 circuits and 2 steel plates by hand): it lists deposits within 96 blocks instead of 48. Range is data (`prospect::SCANNERS`); the panel shows the range of the scan taken. Tests: 400 pass (one new), golden hash re-recorded (a tech was added), no save bump.
- **2026-10-03: Belts carry bodies; frame-rate fix** (user report: 60 fps in some places, about 16 in others, with a save). Cause, found by loading the save in a headless release test: the light cache for boxes (`world/boxlight.rs`) held 24 chunks, evicted oldest first, and the save's machines and belt items in view span 27, so every frame lit all 27 chunks again (about 1.6 ms each, 45 ms a frame; places with 24 or fewer chunks ran fine). The cache now holds 128 chunks (4 MB at most) and evicts the least recently read; that frame costs 0.3 ms. A body on the ground in a belt's cell is carried along at the belt's speed (`Factory::conveyor_at`, `Player::conveyor`, set by `authority.rs`; collision still applies, a crouched body stops at an edge, footsteps ignore the carry, lifts are still climbed). Presentation/authority only: no save or core change. Tests 393 → 399.
- **2026-10-02: Strategy controls** (user request, approved after local review): V cycles first/third/overhead follow; G frees or follows, Home follows; WASD pans, wheel zooms, right-click walks, X stops, Ctrl-right-click uses/places within body reach. Smoothed ground height, zoom and view transitions; bounded routes around obstacles with one-block jumps and safe refusal. Only bodies discover columns; the camera reloads known 3D terrain and covers unknown ground with fog. Save-free development `/?preview`; no save/core format changes. Tests 378 → 392.
- **2026-10-05: Steps 10.1–10.7** (new ground, canisters and pumpjack, refinery, chemical plant, diesel and electrolysis, ore washing, research center): see section 4, tests 527 → 558.
- **2026-10-05: Terrain overhaul and hydro (step 10.8).** Generator version 7 (rivers, lakes, mesas, bigger ranges: `worldgen/rivers.rs`, `landform.rs`), block 92 and the Hydropower tech (50): the water wheel. Golden hash re-recorded, no save bump. Tests 558 → 570.
- **2026-10-05: Hoists (step 10.9).** Blocks 93–94, the Hoists tech (51): shafts that a powered winch turns into a 9 blocks/s lift. No save bump. Tests 570 → 574. Next free block 95.
- **2026-10-05: Nuclear power (step 10.10).** Blocks 95–96, item 365, the Nuclear Power tech (52): centrifuge and a water-cooled reactor that shuts down when it overheats. No save bump. Tests 574 → 581. Next free block 97, item 366, texture 241, machine recipe 57.
- **2026-10-05: Gold science and Mk5 (step 10.11).** Gold pack and kit (366–367), Mk5 items (368–374), techs Gold Science (53) and Mk5 Machines (54), a fifth pack in `PACKS`, three hints. No save bump. Tests 581 → 583. Next free block 97, item 375, texture 243, machine recipe 59, tech 55.
- **2026-10-05: Milestone 10 cleanup (step 10.12).** `bench_chemistry` (19 µs a tick, worst 36 µs), the balance note, README section on fluids and chemistry, the Milestone 10 step list moved to `docs/CHANGELOG_M10.md`, Milestone 11 moved in from the roadmap with a step list. No code change besides the ignored bench. Milestone 10 awaits the user's review and commit.
- **2026-10-06: Creative mode.** `mode.rs` (`Mode`, `Sim::mode`, `creative_items`), `api/creative.rs`, `ui/creative.ts`; a world's mode is saved (save version 38) and hashed, so co-op peers agree; the new-world form picks it. Golden hash re-recorded on purpose (the mode byte). Tests 583 → 588.
- **2026-10-06: Stack controls** (user request). Right-click takes half a stack (rounded up), again adds half of what is left to the held stack; Shift-right-click with a stack held puts one item down, outside the screen it throws one; with an empty hand it still moves every stack of that item. Works on box slots too; tools move whole. New actions `RightClickSlot`, `RightClickBox`, `ThrowCursor` (tags 48–50, no save bump). `Sim::apply` moved to `action/apply.rs` (`action.rs` was over budget). Tests 588 → 590.
- **2026-10-06: One underpass in tiers** (user request). `factory/underpass.rs` (`UNDERPASS_SPAN`, `derive_passes` rederives entry and exit at every relink; block 23 is a non-placeable legacy block), a `Family` row and items `UNDERPASS_MK2..MK4` (375–377), belt-priced recipes, tech unlocks beside the belt tiers; `belt_line/pass.rs` makes dragged lines dive under obstacles (red when unaffordable or too wide). No save bump. Tests 596 → 609.
- **2026-10-06: Analytics and efficiency** (user request). `analytics/` (history rings, per-machine averages), `factory/efficiency.rs` (read-only samples and power totals), `SimEvent::Produced` (miners, quarries, processors, pumpjacks), `api/analytics.rs`, `ui/analytics.ts` (P) and `ui/analytics-chart.ts`; machine panels and `target_detail` show an efficiency line. No save bump, the core is untouched. `main.ts` now opens its screens from one table. Tests 590 → 596.
- **2026-10-06: Leaf despawn lag** (user report). Measured in the user's save: each decaying leaf remeshed up to 8 chunks at once, marked about 36 chunks dirty for light (the 32-block margin, everything below) and dropped their box-light cache entries, so about 70–80 ms a decay. Now `light::reach` limits the dirty chunks to what an edit can change (none for equal light behaviour, the sky column for sky-only blocks like leaves), leaf timers use `set_block_anywhere_later` (remesh waits for the streaming budget), and the box-light cache goes stale and is relit in `work_step` instead of in `Game::update`. About 28 ms of work a decay left, spread by the 6 ms browser budget; `update` 0.2 ms. No save bump (render cache only).
- **2026-10-06: The recycler** (user request). Block 97, a 2×2×2 `Pick::Recycle` processor (`factory/process/recycler.rs`, six in-hatches, two out, 90 kW), the coin (item 378, stack 1024, `item::COIN`; not offered in creative), the Recycling tech (55, after Blue Science), `Processor::owed` (millicoins, saved for recyclers only, so no save bump: only new worlds hold one). What an item pays is `recipes/recycling.rs`: a const-built table over `RECIPES` and `MACHINE_RECIPES`, no per-item list. Raw 1 coin; a batch is worth `STEP` (2) × its inputs, shared over what it makes (millicoin precision, so a plank is ½); the cheapest recipe wins, so the route never changes the worth; `WASTE` (slag, tailings) and what is salvaged from only waste pay 1; tools go in whole (their count is their uses). `process/intake.rs` split out of `process/mod.rs`. Golden hash re-recorded (a new tech row). Tests 611 → 628. Next free block 98, item 379, texture 246, tech 56.
- **2026-10-07: Blast furnace on crushed iron** (user request). `BLAST_CRUSHED_RECIPE` (machine recipe 59, 2 crushed iron + coal + quicklime, same steel and slag), unlocked with Ore Crushing; the furnace is now `Pick::ByInput` (no recipe to choose, it takes ore or crushed iron and makes whichever it holds a full batch of; `recipe_using_if` and `Processor::next`). First version kept `Pick::Chosen` and the user could not get crushed iron in: a furnace's chosen recipe is on the old iron-ore recipe. A saved furnace's chosen recipe is simply ignored. The same for washed iron: `BLAST_WASHED_RECIPE` (60, unlocked with Ore Washing). No save bump. Tests 628 → 630. Next machine recipe 61.
- **2026-10-07: Tools move whole** (user bug report: a worn shovel in a box rode a belt as 87 shovels). A tool's stack count is its uses, so every one-at-a-time mover split it. Now `tools::lot(item, count)` is the piece a mover takes: a tool's whole count, 1 of anything else. Belt items and a router's held item carry `n` (`BeltItem::n`, `Router::held_n`); `Buffer::feed` takes a lot, `deliver` / `Sinks::accept` hand `n` over all-or-nothing (a sink with no room for every use makes the belt wait); the recycler takes a lot too. Save version 39 (belt and router counts; older saves read 1); golden hash re-recorded. Boxes and inventories still pool worn tools of one kind into stacks, so a belt carries one tool per slot.
- **2026-10-08: Chip fab (step 11.1).** `factory/process/fab.rs` (block 98, 4×4×3, 1 MW, `Category::Fabrication`), pure water (chemical plant recipe 61, 2 units of water), wafer and AI accelerator (recipes 62–63), items 379–381, techs Wafers (56) and Accelerators (57) in `research/compute.rs`, hand recipe in `recipes/compute.rs`, textures 246–250 (`textures/compute.rs`). Checked in the browser (placed, panel, model). Golden hash re-recorded (two tech rows). Tests 634 → 643. Next free block 99, item 382, texture 251, tech 58, machine recipe 64.

## Milestone 7 steps 7.1 to 7.6 (specs as built, moved from DEV_PLAN.md on 2026-10-03)

- [x] **7.1 Arc furnace and silicon** (built): the arc furnace (block 70, 2×2×2, `Category::Arc`, 120 kW: 1 quartz
  ore + 1 coal → 1 silicon, 4 s; hatches like the assembler) is a spec row; silicon (item 308) and the circuit
  (309: assembler, 1 silicon + 3 copper wire + 1 iron plate → 2) are machine recipes 26–27; the Electronics tech
  (r g b, 80 × 20 s) unlocks all three; textures 151–154. The tech table moved to `research/techs.rs`. Golden
  hash re-recorded (no save bump). Quartz checked: v4 band 25–50 blocks down; a geology test mines it.
- [x] **7.1b Power cable and the pole tool** (built, user request): the cable (block 71, hand recipe ×4 from 1
  iron plate + 2 copper wire, `recipes/wiring.rs`) is a `Kind::Pole` of stored tier 3 (`CABLE_TIER`, no save
  bump): cables link by touching (diagonals too) and to a pole within that pole's machine reach; machines with no
  wire hang on a cable within 2. `power_tools.rs` (presentation plus ordinary actions): the pole ghost and its
  rules moved on in 7.3b; with cables, a click hangs up to 64 down the aimed cell's column to the ground
  (crouch: one). Tests in `power_tools/tests.rs`, `factory/pole/tests.rs`. The cable wears the copper wire texture.
- [x] **7.2 Violet science and Mk4** (built): violet pack (310) and kit (311), assembler recipes 28–29; `PACKS` has four
  entries, labs a fourth slot (save 22); the cable's stored tier moved to 15 so tier 3 is the Mk4 pole. Mk4 items
  312–319 (hand recipe: Mk3 plus violet kits): belts 8/s, miners 6/s · 92 % · 90 kW, processors ×5, substation 32 · 16,
  lab ×4 · every 3rd unit free; boxes, pumps, quarries stop at Mk3. Techs 18–20; numbers pinned in `factory/tiers/tests.rs`.
- [x] **7.2b Bootstrap without raw ore, and timed hand crafting** (built): no hand recipe uses metal ore: stone makes the
  furnace, ore and fuel go in by hand, hand recipes (`recipes/materials.rs`) turn ingots into plates, rods, screws and
  wire, and the first machines are made of those. `crafting.rs`: `Action::Craft` queues an order (`CraftQueue` per
  player, max 12, save 23); `plan` adds the part crafts the inventory can't cover, materials are paid at once, a craft
  takes `hand_ticks` (90 + 30 per material, max 20 s); cancel or leave refunds (`Action::CancelCraft`, tag 26).- [x] **7.3 Solar power and accumulators** (built): the solar panel (block 72, 2×2×1, 10 kW at noon by
  `daytime::sunlight`, an integer parabola 6:00–18:00) and accumulator (73, 2×2×2, 10 MJ, 60 kW) are processor rows
  (`process/solar.rs`, `Energy::Solar` / `Accumulator`); `Power::balance` runs panels, then accumulators, before
  generators and turbines, and spare sun charges accumulators; charge saved for accumulators only. One tech, Solar
  Power (r g b after Electronics). Sized test: six panels and one accumulator carry 15 kW through a night.
- [x] **7.3b Power wiring by hand** (built, user request; save 24): wires are core state (`factory/wiring.rs`,
  `Action::Connect` / `Disconnect`, tags 27–28), each taking a slot of every pole it touches (`POLE_TIERS.slots`
  4/8/12/16); a new pole wires itself only to the nearest powered pole (`auto_hook`); cables still join by range;
  splitters and filters need no power. Hands (`power_tools/wire.rs`): poles place where aimed (Shift: full reach),
  right-click selects a pole, a click wires the machine aimed at, crouch-click moves, cuts, links. Old saves are wired
  by range once (`hook_by_reach`; tests too, unless `Factory::by_hand`).
- [x] **7.4 Scanner Mk2** (built): item 320 (`tex::SCANNER_MK2`, 166), hand recipe at the end of `RECIPES` (a Mk1 scanner,
  3 circuits, 2 steel plates), tech Advanced Scanning (r g b v, 60 × 25 s, after Violet Science). `prospect::SCANNERS` is
  the range table (48, 96); a reading remembers its range (`Prospect::range`, `scan_range()`). The Mk1 scanner already
  lists quartz (Electronics needs it, and it lies in highlands, deserts and basalt fields, not the plains spawn), so the
  Mk2's gain is reach alone. Golden hash re-recorded (one more tech), no save bump.
- [x] **7.5 Logic** (built; save 25): one block, the sensor (74, `factory/sensor.rs`, `Kind::Sensor`; 1 circuit + 1 iron
  plate, tech Logic: r g b v, 80 × 25 s, after Violet Science). It reads the box, silo or belt cell behind it and
  switches the machine in front of it by a rule (`RULES`: always on, always off, and four fullness rules with
  hysteresis; right-click steps through them with `Action::SetSensor`, tag 32; R turns it). "Off" cuts the machine's
  power wire: `switched_off` feeds `Hooked::off`, `Power::rebuild` leaves those machines out, a flip marks the factory
  dirty so the next `relink` runs. Only wire-taking machines (miners, quarries, labs, pumps, generators, electric
  processors) obey; burners ignore it. Its lamp is green on, red off. Scope cut from the plan: the separate switch
  and lamp signal are rules and the lamp on the sensor, not blocks. Golden hash re-recorded.
- [x] **7.6a Ghosts** (answers 2026-10-03: blueprints are both copied and planned): ghosts are core state (`Sim`,
  saved: a cell, a block, facing, tier; `Action::PlaceGhost` / `RemoveGhost`), so drones and co-op can use them.
  Placing a ghost from the build menu (a key toggles ghost mode, R rotates, extended reach); a real block placed on
  its ghost clears it; the HUD lists what a ghost set still needs; drawn as translucent boxes. No drones yet.
  Built (save 26): `ghosts.rs` (core: `Ghosts` sorted list, `Sim::place_ghost`, facing adopted and ghost cleared in
  `place_block`), `ghost_mode.rs` (B toggles; right-click plants or, on a ghost, removes it; R turns; 16-block reach;
  mining off; cyan outlines within 64 blocks via `placement_box`; HUD label lists what the ghosts need). `aim::target`'s
  reach is now a parameter. Golden hash re-recorded.
- [x] **7.6b Blueprints**: in ghost mode (B) Z marks a box's two corners, Enter copies the machines in it as a new blueprint
  (taken in hand), the use button stamps its ghosts, R turns it, Z puts it away; the L panel lists, renames, holds and
  deletes them. Built: `blueprint/` (`Blueprint`/`Entry`, `copy` from `Factory::placed_as`, `turned`, bytes;
  `hands.rs` the Game side), `Action::PlantGhost` (tag 35), `api/blueprint.rs`, `ui/blueprints.ts`; the bytes are kept
  per world in the browser record like pins (`WorldMeta.blueprints`), not in the save, so no save bump. Scope cuts:
  only machines are copied (plain building blocks cannot be told from terrain) and machine settings (recipe, filter,
  rule) are not, since a ghost carries none: a later step can add a setting byte to `Ghost` (save bump) and
  `Entry`. The label lists what is missing; the HUD ghost outlines are unchanged.

## Milestones 6 to 8: the step lists (moved from DEV_PLAN section 4 at the end of Milestone 8)

#### Milestone 6 (Industry), built

- [x] **6.1 Tiers as data** (`factory/tiers.rs`): `tier: u8`, numbers in `*_TIERS` arrays, `FAMILIES` names each
  tier's item. **6.2 Unlocks, categories, content lint** (`research::Unlock`, `recipes/machine.rs`
  `Category`, `recipes/tests.rs`). **6.3 Upgrade kits** (`factory/upgrades.rs`, `Action::Upgrade` tag 23,
  tier stripes; one step at a time, never refunded).
- [x] **6.4 One processing machine** (`factory/process/`: a `ProcessSpec` row per machine; smelter and
  constructor with tiers; save 18). **6.5 Footprints and the assembler** (`factory/footprint/`,
  `action/multiblock.rs`; the anchor holds the block, other cells `MACHINE_PART` 62; ports by side; save
  19). **6.6 Steel** (blast furnace with a byproduct port, slag, steel items and tools).
- [x] **6.7 Blue science and Mk3** (blue pack and kit, three-slot labs, save 20, Mk3 belts, miners,
  smelters (electric), processors) and **6.7b Mk3 for the rest** (poles `factory/pole.rs`, boxes, pumps,
  quarries, labs; generator Mk2; save 21; tier recipes in `recipes/tiers.rs`).
- [x] **6.8 Steam power** (`factory/process/steam.rs`): boiler (block 66, 2×2×2) and steam turbine (67, 3×2×2) are
  spec rows (`Energy::Boiler` / `Turbine`). A boiler burns 540 kJ a coal into steam and takes water (2,000 kJ a
  unit) from a pump on its pipe net; a turbine touching a boiler is a power source `power.rs` asks for what its
  grid lacks (up to 240 kW, two a boiler). Tests: a sea-pump plant holds a 405 kW load for a full research unit;
  a pond plant runs dry; the two-turbine limit; a save round trip.
- [x] **6.9 Crushing and bulk storage:** the crusher (block 68, 30 kW, machine recipes 21–25, items 306–307: 2 ore
  → 3 crushed, slag → sand) and the silo (69, `Pick::Store`, 144 slots, in on three sides, out the front). Techs
  14–16; textures 141–150 (`textures/heavy.rs`), recipes in `recipes/heavy.rs` and `tooling.rs`.
- [x] **6.10 Feel, balance, cleanup:** tips for kits, the assembler, steel and steam (`hints.rs`); golden hash
  re-recorded. Tests 286 → 296, wasm 232.7 KB gzipped, `bench_plant` (20 machines): 1.3 µs a tick, worst 2.15 ms.
#### Milestone 7 steps

Goal: the player's reach grows from building by hand to machines doing it. Circuits and violet science
first, then logic, then the builders (blueprints, drones, flight). Steps 7.6–7.8 follow the user's answers
of 2026-10-03 (section 1).

- [x] **7.1–7.6 built** (full specs moved to `docs/CHANGELOG.md`, "Milestone 7 steps 7.1 to 7.6"): 7.1 arc furnace, silicon,
  circuit; 7.1b power cable; 7.2 violet science and Mk4; 7.2b bootstrap without raw ore and the timed hand-craft queue
  (save 23); 7.3 solar panel and accumulator; 7.3b wiring by hand (save 24); 7.4 Scanner Mk2; 7.5 the sensor (save 25);
  7.6a ghosts (save 26) and 7.6b blueprints.- [x] **7.7 Construction drones: the long way** (user, 2026-10-03: drones must be hard to reach and very rewarding; more
  research and parts). Gate: a ladder of techs on violet packs, none cheap: Robotics (actuators and servo parts) →
  Drone Power (a drone cell, made of a battery-like part from circuits, copper, steel and a motor) → Navigation (a
  processor-based guidance module, so after Processors) → Construction Drones → Swarm Logistics (more drones per port,
  a longer range). The drone is built in an assembler from several intermediates, none raw ore; the drone port is a big
  multi-block (3×3 pad) on power. Drones take materials from boxes beside the port (a later tech may extend this to a
  network). Steps: **7.7a** the part chain and techs, **7.7b** the drone port and its supply boxes, **7.7c** drones
  flying out and building ghosts (one at a time, visible), **7.7d** tear-down marks, **7.7e** swarm upgrades. Numbers
  go in `docs/TECH_ERAS.md` when 7.7a starts.
  **7.7a built (no save bump; golden hash re-recorded):** five techs on all four packs after Violet Science: Processors
  (100 units × 30 s) → Robotics (140 × 30, also needs Mk4 Machines, so Mk4 Logistics first) → Drone Power (160 × 35) and
  Navigation (180 × 35, needs Processors and Robotics) → Construction Drones (240 × 40): twelve techs in front of the
  drone. Six assembler-only parts (`item.rs` 321–326, `MACHINE_RECIPES` 31–36, `DRONE_RECIPES`): processor, servo,
  actuator, drone cell, guidance module, drone. Numbers in `docs/TECH_ERAS.md` section 3; icons `textures/robotics.rs`.
  **7.7b–e built (save 27, merged into 28):** the drone port (block 75, `factory/process/hangar.rs`: a 3×3×1 processor spec,
  `Pick::Hangar`, 40/60/90/140 kW only while drones are out) keeps drones as its input buffer, so belts, panel, saves and
  breaking needed nothing new; tiers Mk1–Mk4 by kits (`Swarm Logistics`, tech 29): reach 32/48/64/96 blocks, fleet 4/8/12/16.
  The core `drones/` (`Drones` in `Sim`, saved) launches a drone every 30 ticks per powered port at the nearest ghost in
  reach that no drone is on: a build takes its item from a storage box touching the pad (`factory/ports.rs`), flies at
  8 blocks a second (f64, `+ - * / sqrt` only), works 1 s and places it through `put_block` (the same code a player runs);
  a tear-down mark (`Action::MarkRemoval`, tag 36: a ghost of air) works as long as bare hands take (1–4 s) and breaks it
  through `dismantle`, the drops going into the boxes. Breaking a port brings its drones back as items; a drone whose port
  is gone drops as an item. `break_block`/`place_block` were split into `dismantle`/`put_block` for this. Presentation:
  `drone_view.rs` (drones as boxes), ghost mode left-click marks or unmarks (`update_marking`, red outlines, label).
  Port recipe: 24 steel plates, 12 circuits, 2 processors, 4 motors.
- [x] **7.8 Flight and helpers** (answer: coal jetpack now, hover pack in M9): the jetpack (4 steel plates, 2 motors,
  2 circuits; 10 s of thrust a coal) and the personal drone (which, like the construction drone, needs the long ladder).
  **Built (save 28):** `helpers/` (core, per player in `PlayerCore.helpers`, saved): `Action::Jetpack { on }` (tag 37)
  burns a coal from the pack per 600 ticks of thrust (the pack stands in for a fuel slot) and the body lifts at 12 m/s²
  net up to 6 m/s while `Player::thrust` (set from the core by `authority.rs`); the hands send it when jump is held in the
  air (`helper_hands.rs`). `Action::Fetch { item, at }` (tag 38): with the personal drone in the pack, Y sends it to the
  nearest box within 32 blocks holding the held item; a round trip at 12 blocks a second (at least 1 s) brings a stack.
  Items 330/331 (tools of `DEVICE_TIER`), techs Jetpack (needs Robotics, 120 × 30 s) and Personal Drone (needs Construction
  Drones, 160 × 35 s); recipes: jetpack 4 steel plates, 2 motors, 2 circuits; personal drone 1 drone and 2 circuits.
- [x] **7.9 Feel, balance, cleanup.** Tips, research times played through, worst tick and wasm measured,
  README, CODEMAP; then move Milestone 8 in from the roadmap and ask its questions.
  **Done:** three new tips (ghosts, drones, fly and fetch), `bench_drones` (2,000 ghosts and a port: 3.6 µs a tick, worst
  61 µs), wasm 312.6 KB gzipped (`vite build`: wasm 832.6 KB raw; 241 KB at step 7.3: most of the growth came with the
  textures, blueprints, strategy camera and avatars; the drones add icons and about 10 KB of code), 431 tests (4 ignored benchmarks).
  Research times follow `docs/TECH_ERAS.md` section 3. Milestone 8 questions are open (see the status line).

#### Milestone 8 (Terraforming, era 5): in progress

Design from `docs/TECH_ERAS.md` section 4. **The user's answers (2026-10-04):** the earthworks are done by the
**drone ports** of Milestone 7 (no separate excavator machine); spoil goes into **belts and boxes**, and fill sites in
range are served from the same boxes first. **Built:** sites in the core (`factory/sites.rs`: `Site`, `Job` dig, fill or
flatten, `MarkSite` / `RemoveSite`, `survey_site`, `api/sites.rs`; saved); the drone ports work them
(`drones/earthworks.rs`).

- [x] **8.1 The planner** (`site_hands.rs`, `ui/site.ts`): item 332 (3 iron plates, 4 copper wire, 2 glass, a circuit),
  tech Earthworks (needs Construction Drones, 140 × 35 s, all four packs). Right-click a corner block, then the opposite
  one (reach 64); the site panel picks the job and the level and shows the survey (blocks to dig and fill, ore, trees,
  water, unseen columns, spoil); right-click a marked site to look at it or remove it. Outlines (job colour for the
  extent, white for the level, cyan for the box being marked) go into `placement_box`.
- [x] **8.2 Drones work sites:** a port picks the first cell in its reach that needs work (cut layers top down, then fill
  layers bottom up) after ghosts of equal distance; a cut breaks the block by hand and puts the drops into the pad's
  boxes (it waits while they are full); a fill brings dirt (top layer) or stone, dirt, sand, grass from the boxes and
  places it where the cell is free, the cells above are free and the ground below is solid; so dug ground fills other
  sites from the same boxes. A finished site removes itself.
- [x] **8.3 Fill and flatten:** the same cell scheme (a flatten is a dig and a fill in one site).
- [x] **8.4 Tunnels** (`factory/sites/tunnel.rs`; `Job::Tunnel`, `Action::MarkTunnel` tag 40): two blocks, a path along the
  longer horizontal axis (96 blocks, 1 in 2 slope), section 1×2, 3×3 or 5×5 standing on it. Drones cut it like a dig but
  leave cells touching water. A tunnel clashes with a site only where they share heights. No new tech (Earthworks gates
  it), no save bump (job byte 3), golden hash unchanged.
- [x] **8.5 Cleanup:** `bench_earthworks` (a 64 × 64 flatten, nine layers, boxes full, so every launch scans all 36,864 cells: 19 µs a tick, worst 0.5 ms); sites show on both maps as hollow squares (`MARK_SITE`); a planner tip; outline colours from the Okabe-Ito palette (dig vermillion, fill sky blue, flatten yellow, tunnel reddish purple, corner bluish green); README. Tests 457 → 458, wasm 329 KB gzipped (`vite build`: wasm 881 KB raw, JS 61 KB, CSS 8 KB).
