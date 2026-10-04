# Development history

Earlier development entries, retained from DEV_PLAN.md so the active plan stays small.

## 8. Change log of this plan

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
