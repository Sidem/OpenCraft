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
