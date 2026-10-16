# Milestones 10 to 12 and the interlude: the step lists moved out of the plan

Moved from `docs/DEV_PLAN.md` on 2026-10-16 when the play-test response became the current work. Milestone 10 and 11
specs are in `docs/CHANGELOG_M10.md` and `docs/CHANGELOG_M11.md`; this file keeps the summaries, the performance todo,
the less-hand-crafting interlude, Milestone 12's steps and the plan's change log of 2026-10-14 and 2026-10-15.

### Milestone 10 (Fluids and chemistry), built; the user reviews and tests it before it is committed

Oil and canisters, the pumpjack, refinery, cracker, chemical plant, diesel power, electrolysis, ore washing, the research
center, a terrain overhaul with rivers and water wheels (generator versions 6 and 7), hoists, nuclear power, gold science
and Mk5. Specs: `docs/CHANGELOG_M10.md`. **Measured:** 20 dry water wheels, 8 reactors, 8 refineries, 8 diesel generators,
4 winches and 4 riders asking the hoist rate each tick cost 19 µs a tick (worst 36 µs) on one grid (`bench_chemistry`,
ignored); water's worst tick is 0.09 ms. **Balance (to tune in play):** power per source: coal generator 60 kW, steam turbine
240 kW, water wheel up to 48 kW (free, but needs a river), diesel generator 400 kW, reactor 2 MW. Oil pays: a vein's pumpjack
(90 kW) fills about 22 canisters a minute and 3 of them refine into one diesel canister (40 MJ, the energy of 148 coal) plus
naphtha and heavy oil, so one pumpjack and one refinery (150 kW) feed several diesel generators. A fuel cell (4 uranium ore
and a steel plate, 300 MJ) runs a 100 kW load for 50 minutes; the reactor needs coolant, and a centrifuge draws 200 kW.
Washing turns a raw ore's 1.0 ingots into 2.0 for 60 kW and a pipe of water per washer. Mk5 costs 4 to 8 gold kits a
machine on top of the Mk4, so it stays a late-game spend.

### Immediate todo (user request 2026-10-08; do these before step 11.2, one per session, each green and committed)

From the performance review. **Measured 2026-10-08** with `factory/tests/bench.rs` (`cargo test --release bench_big_base -- --ignored
--nocapture`, a synthetic base of belts in rows, a smelter per 10 belts and a pole per 40 cells, empty belts, native speed):

| base | steady tick | place one belt (whole relink) | hooks + power rebuild | `write_instances` |
|---|---|---|---|---|
| 3000 belts, 300 machines, 100 poles | 9 µs | 236 µs | (small) | 244 µs |
| 30000 belts, 3000 machines, 1020 poles | 141 µs | 2.9 ms | 0.25 ms | 0.7 ms (1 M floats in range) |

The review's guess was wrong where it mattered most: the power rebuild is under 10 % of a relink even at 30 000 belts, and a
relink is linear (about 90 ns a belt), so a placement hitch stays under 3 ms for any base a player will build. Rows below say
what was done or why it is parked.
- [x] **P1 Cache the unlocked machine recipes** (done: `Research::unlocked`, refreshed in `refresh()` when a tech finishes,
  on load and in `complete_all`; `machine_recipes_unlocked()` returns the `[bool; N]` by value, a per-tick snapshot, no heap
  allocation). Golden hash unchanged. Saves one allocation and about 64 tech scans per tick, per insert and per `room_for`.
- [x] **P2 Split the dirty flag: dropped.** Measured above: the power rebuild is not the cost. If a relink hitch ever shows in
  play, make `relink` incremental per touched cell instead (belt links only look at the four neighbours).
- [ ] **P3 Cheaper box instances: parked.** 0.24 ms for 61 000 boxes is cheap natively; what could matter in the browser is
  the buffer upload (`bufferSubData` of up to 4 MB a frame) and the wasm slowdown. Measure in the browser first
  (`stats` of the renderer, frame time with 60 000 boxes in range) before adding a frustum test or a spatial grid.
- [ ] **P4 Incremental co-op checksum: parked.** `Sim::state_hash` serialises the whole state every 60 ticks. Only worth it
  when a save measures slow in co-op; time it on a large save first. The hash value would change: re-record the golden hash.
- [ ] **P5 Small items (optional, cheap).** Reuse the per-frame `Float32Array` views onto wasm memory in `web/src/main.ts`
  until `memory.buffer` changes; skip `update()` of closed UI panels; only re-sort visible chunks (`renderer.ts:323`) when the
  camera crosses a chunk.
- [x] **E1 Colour-coded electric grids** (user request; built 2026-10-08, `factory/grid_colour.rs`): the biggest grid (most poles) is
  the main grid and always blue, the others take orange, pink, green, yellow, red by size (Okabe-Ito); wires, hookup wires, cable
  knots and a collar and crossarm sleeve on each pole wear it, and the pole readout names the grid. Textures 251–256, render only
  (no save bump); a grid that overtakes the main grid in poles swaps colours with it.
### Milestone 11 (Compute and photonics), built and pushed

Chips from chemistry, the data grid, AI datacenters with cooling towers, AI labs, endless bonus techs, the optimizer node, laser
power and data links with mirrors, the swarm hub, auto-routing and the AI survey. Specs: `docs/CHANGELOG_M11.md`. **Measured:** a
plant of 6 datacenters, 8 AI labs, 4 optimizers, 4 swarm hubs, 6 Mk4 drone ports, 14 fibre nodes, 20 generators and 4 laser links
costs about 2 µs a tick (worst under 0.1 ms; `bench_compute`, ignored). **Balance (to tune in play):** a datacenter draws 3 MW for
100 TF, which feeds five AI labs (20 TF each) or five optimizers; an optimizer (150 kW, 20 TF) buys +25% on everything within 16
blocks, a swarm hub (100 kW, 30 TF) +50% drones on every port within 12, so compute is spent where a plant is densest. Coolant is a
pump (a unit per 10 s) or a cooling tower (about a twentieth of that). A laser link costs 10% of what crosses it and 128 blocks
at most. The endless bonus techs cost compute only and grow a quarter dearer a level. Auto-routing and the AI survey are free to use
once researched.

### Interlude: less hand-crafting (chosen by the user on 2026-10-09 to go before Milestone 12; design in `docs/ROADMAP.md`)

- [x] **L1 + L2 `player_bonus` and hand-crafting speed techs** (built 2026-10-09): `perks.rs` (`Stat::Crafting`,
  `player_bonus` sums finished `Unlock::Perk(stat, tier)` techs, +25% each, linear), techs Handcrafting I–III 74–76
  (`research/hands.rs`: green, blue, violet packs), `crafting::craft_ticks` shortens each craft (ticks still count real
  ticks, so no save change); the build menu's "Takes N s" shows the faster time. Golden hash re-recorded (tech table grew).
- [x] **L3 Worn craft-speed gear** (built 2026-10-11): Fabricator Gloves (item 385, texture 285, tech 77 on all five packs
  after Handcrafting III and Gold Science; hand recipe 4 actuators, 4 servos, 4 gold kits) in a new sixth equipment slot,
  Gloves. `Effect::crafting` (+500 permille) is added by `perks::player_bonus(research, inventory, stat)`, which now takes the
  player's inventory; with all three techs the gloves make a craft 2.25× as fast. Save version 44 (six worn slots; 43 saves
  read five), golden hash re-recorded. Tests: `perks/tests.rs`, `equipment/tests.rs`, `crafting/tests.rs` (wear them and a
  craft takes 2/3 of the time, take them off and it is back).
- ~~L4 Craft from nearby boxes, or by request~~: dropped by the user on 2026-10-11 ("crafting from nearby boxes won't be
  a thing"). Remove it from any plan that mentions it.

### Milestone 12 steps (Orbit; moved in from the roadmap, a first draft to refine at 12.1 with the user)

Goal: the factory leaves the ground. Rockets from every line, fuel from electrolysis, satellites that give the map and the
network new reach. Numbers are first guesses. Order: the launch site and a first rocket, satellites in orbit, what each kind
gives, drop pods, cleanup. Plan the interlude chosen on 2026-10-08 for after Milestone 11 (timelapse, factory as an instrument,
follow one item, seismic prospecting, site bonuses) with the user before 12.1; it can go first.
- [x] **12.1 Rocket parts and the launch site** (built 2026-10-11: `process/launch.rs`, block 109 Launch Site, 7×7×6, 2 MW,
  `Category::Launch`; tech Rocketry 78 in `research/orbit.rs`; items 386–390 (light frame, rocket engine, fuel tank, avionics, rocket
  fuel canister); machine recipes 64–69 in `recipes/orbit.rs` (assembler parts 64–67, rocket fuel 68 in the chemical plant, the
  launch 69: 4 frames, 2 engines, 2 tanks, avionics, 8 fuel canisters, 60 s → 8 empties); hand recipe 200 concrete, 60 steel beams,
  20 motors. The part recipes take at most 3 input kinds (the assembler's slots). The rocket stands on the pad only while a batch
  runs (`Look::Rocket`). `MACHINE_RECIPES` is a compile-time join (`crate::join`) so new rows go in `recipes/orbit.rs`. Textures
  286–293. Tests 744 → 752. Not done: a launch sound or toast, a rocket-specific tip.)
- [x] **12.2 Satellites and the sky** (built 2026-10-11: `satellites.rs`, core state in `Factory::orbit`, save version 45 (a
  count and the list after the research; older saves have none), golden hash re-recorded). **Orbits:** an infinite ground cannot
  have a globe, so the sky wraps every 4096 blocks both ways; a satellite leaves straight above its pad at the launch tick and
  flies one of 8 headings (tilted off the axes, picked by launch count) at 6 blocks a second, so a position is integer maths on
  the tick alone. It covers a place within its kind's `reach` (survey 768 blocks, so about 11% of the ground per satellite) of
  the point under it; at most 24 of a kind stay up (a 25th retires the oldest). **Payload:** the rocket carries one satellite, so
  launch recipe 69 now needs a Survey Satellite (item 391, texture 294, assembler recipe 70: 2 light frames, 4 solar panels, 4
  AI accelerators, 40 s; both unlocked by Rocketry) as its last input: the pad's input buffer grew to 6 slots (a pad saved
  before 45 reads 5: `saved_input_slots`, tested) and `Processor::lifted` hands the satellite to `Factory::update`. Only the
  survey kind exists; 12.3 adds a `KINDS` row, item and recipes per kind. **Sky:** `Game::sky_satellites` gives each satellite's
  direction (hung 400 blocks up, shown within 2400 blocks) and `render/sky.ts` draws gold lights (faint by day). The launch
  site's readout says "Survey satellites: N up, M over this pad". `Game::launch_satellite_at` is a debug scene helper. Tests
  752 → 763. Not done: a map layer for coverage, a launch sound or toast.
- [x] **12.3 What satellites give** (numbers kept as built, 2026-10-14, to tune in play; split into one kind per session).
  **Survey done** (2026-10-14, `survey/orbit.rs`: veins and lodes under a survey footprint go on both maps as facts, kept for the
  session and not saved; the AI survey drops guesses near them; footprints are `MARK_COVER` marks drawn as a gold disc; no save
  change; not looked at in the browser pane). **Comms done** (2026-10-14: tech Orbital Comms 79 (needs Rocketry, Data Network), item
  392, texture 295, assembler recipe 71 and launch recipe 72, `KINDS` row `COMMS` with reach 1024. Under a comms footprint all fibre
  nodes are one data grid, rejoined each tick (`Data::uplink`, `merged`, derived, not saved), and a drone port's construction reach is
  ×4 and its cargo range ×2 (`Hangar::linked`, `reach_and_range`); both end when the satellite flies on, so a dozen are wanted for
  steady cover. No new block: no ground station yet. No save change; golden hash re-recorded; 796 tests; not seen in the browser
  pane.) **Power done** (2026-10-14: block 110 Rectenna, 5×5×1, `process/rectenna.rs`, `Energy::Rectenna`, `Source::Rectenna`;
  each power satellite over it beams 200 kW, cap 1 MW, day and night, derived every tick by `sense_sources` and given like a
  water wheel's power in `Power::balance`; tech Orbital Power 80 (needs Rocketry, Nuclear Power) unlocks it, item 393, textures
  296–298, assembler recipe 73 and launch recipe 74, hand recipe in Group::Power; reach 1024; no save change, golden hash
  re-recorded; 801 tests; not seen in the browser pane; no sky beam drawn yet). **Science done** (2026-10-14: "space data" is
  compute, not a new research mechanism: block 111 Ground Station, 3×3×2, `process/ground.rs`, a data producer (`compute 100`,
  `Energy::Optimizer` power sink, 150 kW) that offers 40 TF per science satellite over it, cap 100 TF, derived every tick by
  `ground::sense` (`Store::flow`) and read by `Data::balance` via `Processor::offered_tf`; tech Orbital Science 81 (needs Rocketry,
  Data Network), item 394, textures 299–301, assembler recipe 75 and launch recipe 76 (three inputs: the assembler's limit), `KINDS`
  row `SCIENCE`, reach 1024; the maps tint its footprint pale blue; no save change, golden hash re-recorded; 805 tests; not seen in
  the browser pane). All four kinds are in. Fold in antenna bands from the roadmap's thread "the carry" or leave them.
- [x] **12.4 Drop pods to landing pads** (built 2026-10-15, pushed; numbers to tune in play). Design choices (the plan line said
  "from a launch site"; a separate small machine fits the belts better, so the launch site stays the rocket builder): **Pod Launcher**
  (block 113, 3×3×3, 500 kW, `factory/process/pods.rs`, `Pick::Pods`): belts fill 12 cargo stacks and a **Drop Pod** (item 395, texture
  302, assembler row 77: a light frame, a fuel tank, 10 circuits, 20 s) arms it; under a comms satellite, with a landing pad on its channel
  also under cover, it loads 15 s at full power and sends up to 6 stacks. **Landing Pad** (114, 3×3×1, no power, `Pick::Land`): the pod
  comes down `FLIGHT_TICKS` (20 s) later and the cargo lands in its 24 stacks, which belts empty; what does not fit waits in the pod (in
  `Factory::pods`, saved since 46, `pods.rs`) and lands as room opens, so nothing is lost while a pad exists. **Channel** = the way the
  launcher or pad was placed facing (four), read in its status line; a pod goes to the nearest comms-covered pad on its channel (no UI to
  pick a destination was built). Tech **Drop Pods** (83, `research/later.rs`, needs Rocketry and Orbital Comms, five packs, 250 units)
  unlocks both blocks (hand recipes in `orbit.rs`) and the pod row. A capsule is drawn falling onto its pad for the last 4 s
  (`Factory::falling_pods`, `drone_view.rs`). Textures 302–305 (`ART_HANDOVER.md`). Save version 46 (the pods after the satellites),
  golden hash re-recorded; tests 805 → 822. Not done: a launch rising capsule, a touchdown puff or sound, a map mark for pads, a tip;
  seen in the browser pane (a scene with a powered launcher and pad: the pod launched, fell as a capsule and landed 200 plates; the
  launcher shows its capsule nose even when empty; `Game::launch_kind_at` is a debug helper for any satellite kind). Open question for the user: is facing-as-channel clear enough, or should a pad have a name or a
  colour chosen in its panel?
- [ ] **12.5 Cleanup:** tips, balance, worst tick, the manual (`docs/FACTORY.md`), CODEMAP; move Milestone 13 (the megaproject) in from the roadmap.
  **Done 2026-10-16:** three orbit tips at the end of `HINTS` (a launch site placed; a comms, power or science satellite up;
  a pod launcher placed), `docs/FACTORY.md` covers the four satellite kinds, drop pods, the long-distance pole, the upgrader
  and the automated buildings. **Measured** (`bench_orbit`, ignored: 24 satellites of each kind overhead, 2 launch sites,
  8 rectennas, 4 ground stations, 4 optimizers, 8 Mk4 drone ports, 8 pod launchers and 8 pads on four channels, 30 fibre
  nodes in two islands joined by the comms uplink, 40 pods in flight onto full pads): about 19 µs a tick, worst 40–63 µs.
  **Balance:** numbers kept as built, to tune in play. **Left:** Milestone 13 needs the user's megaproject theme and what
  finishing it unlocks (section 6) before it moves in from `docs/ROADMAP.md`.
- [x] **Orbit polish** (first of the order agreed on 2026-10-16, section 1; built 2026-10-16): new `SimEvent`s `Launched`
  (`Factory::update`, a launch site's lifted satellite), `PodSent` and `PodLanded` (`process/pods.rs` `run`, at a pod's landing
  tick) feed `orbit_fx.rs` (presentation): sounds `LAUNCH` (heard 12× as far), `POD_UP`, `POD_DOWN` (`audio/synth-orbit.ts`,
  `reach` in `audio/sound.ts`, three sound-designer actions), a dust burst at the machine, and an "in orbit" toast per
  satellite (`next_launch`). A pod is drawn rising out of its launcher for 3 s (`Pods::rising`, `Factory::rising_pods`). The
  maps draw pod launchers and pads with an arrow for their channel (`MARK_CHANNEL`) and launch sites as a rocket
  (`MARK_LAUNCH`). Seen in the browser pane (a pod rising and falling, the touchdown burst, the marks); the rocket launch was
  checked headless only. No save or core-state change; golden hash unchanged; 841 tests. **Not done:** a beam drawn from power
  satellites down to rectennas. **Open question for the user:** is facing-as-channel clear enough, or should a pad get a
  name or colour chosen in its panel? **Next:** the feel-and-feedback interlude (`docs/ROADMAP.md`), starting with the factory
  as an instrument.

## Change log, 2026-10-16 (the play-test response, before Milestone 13)

- **2026-10-16: R1a and R1b built.** `scripts/inspect-save.mjs` reads a player's save headlessly; the blueprint prompt
  (`blueprint/repeat.rs`, `Factory::works_at`, `blueprint_prompt` in `api/blueprint.rs`, a card in `ui/hints.ts`) offers a
  blueprint when a machine is built a third time nearby, with the box preset. Garbled "·" and "×" fixed in five files. No save or
  core change; golden hash unchanged.
- **2026-10-16: R1 talked (docs only).** The tree's pack demand and far material measured (one hauler trip of bauxite covers
  the game); the user picked settlements as growing demand and a blueprint prompt. New steps R1a (save inspector) and R1b
  (blueprint prompt); R6's talk moved up to follow them.
- **2026-10-16: Whole-game review and play-test response (user request, docs only).** Claude's review (strengths, weaknesses,
  past and planned work, new dimensions, art, plants, music) was adopted in full; with the user's 32-hour save it became steps
  R1–R8 in section 4, eight decision rows in section 1 and new open questions in section 6. Finished Milestone 10–12 detail moved
  to the new `docs/CHANGELOG_M12.md`. `docs/ROADMAP.md` gained the designs (settlements, decor, pressure without combat, a
  living world) and the trimmed carry; `docs/ART_HANDOVER.md` the art direction update. No code change.
- **2026-10-16: Long-distance pole placing preview (user request).** While one is held and aimed, `Factory::long_link` says
  what `auto_hook` would wire it to, or why not (too far, full, unpowered, alone), and `power_tools/long_pole.rs` shows it:
  the footprint outline green or red, the pole it links to boxed, the label's line ("Links to the long-distance pole 132
  blocks away · 28 to spare" or the reason), a ghost tower and its wire in the grid's colour (red and cut at 160 blocks
  when too far), a beacon over the far tower that shows through the fog (`mark_beacon`: glow flag 2, `BOXES` in the box
  shader), a power-up or power-down blip as the spot crosses the edge of reach, and the maps ring every long-distance pole's
  reach (`MARK_RANGE`, dashed). Presentation only; no save or core change. Seen in the browser pane. 844 tests.

## Change log, 2026-10-16 (before the play-test response)

- **2026-10-16: Dialog screens fixed (user bug report).** Moving the machine panel into the inventory (7bd9b78) deleted the
  shared dialog styles (`.mp-backdrop`, `.mp`, `.mp-head`, `.mp-keys`, `.mp-body`) that research, analytics, the world map,
  blueprints and the site panel use; they are back in `base.css`, and the machine panel's root is now `.mp-panel` so its
  in-inventory rules no longer reach the dialogs. Web only; seen in the browser pane.
- **2026-10-16: Launch recipes named by their satellite (user bug report).** The recipe picker showed every launch as
  "Empty Canister" (its only output); `Game::machine_recipe_payload` gives a launch row's satellite (its last input) and
  `ui/machine-recipes.ts` shows it as the product ("Into orbit"), the empties as "Also gives". 842 tests.
- **2026-10-16: Orbit polish.** Launch and landing sounds, dust bursts and the "in orbit" toast (`orbit_fx.rs`,
  `audio/synth-orbit.ts`), the capsule rising from its launcher, channel arrows and a rocket mark on the maps. The agreed order
  after Milestone 12 is recorded in section 1 and `docs/ROADMAP.md`. 841 tests.
- **2026-10-16: Step 12.5, first part.** Orbit tips (rockets, what satellites give, drop pods) in `hints.rs`; `bench_orbit`
  (`factory/tests/bench_orbit.rs`, ignored): about 19 µs a tick, worst under 0.07 ms; `docs/FACTORY.md` brought up to date
  (satellite kinds, drop pods, long-distance pole, upgrader, automated buildings). No save or core change.
- **2026-10-16: Hand-mining waste taught as it happens; machine panels beside the inventory (user request, off-plan).** `SimEvent::HandMined` (kept, lost; `Deposits::hand_mined` now returns the block's share) feeds `hand_waste.rs` (session counter, not saved): a notice under the crosshair per ore block, from 3 blocks an amber warning card ahead of every tip (`ui/hints.ts`, H snoozes 10 blocks). The machine panel is now a `Beside` column of the inventory screen (`ui/inventory.ts`); new `Action::InsertCursor` (tag 57) clicks the held stack into a machine. The recipe list scrolls within its own height. The inventory header's list of clicks became a context controls bar (`ui/slot-controls.ts`). 837 tests.
- **2026-10-16: README rewritten as a project brief (user request, off-plan).** README.md now pitches the game and explains how the repo is built for fast agent iteration; the player's manual moved verbatim to `docs/PLAYING.md` (controls, worlds, co-op, crafting, upgrades, sound) and `docs/FACTORY.md` (resources and extraction). No code change.
- **2026-10-16: The opening scene, a hook for the first minute, and readable tips (user request, off-plan).** A new normal world (`Game::new_normal`, used by `save/session.ts` for fresh worlds; `new Game` and creative worlds have none) starts facing a stopped line on a stone-brick slab: ore box, belts with ore waiting at an outlined gap, a stone furnace with coal, an ingot box; the missing belt lies on the grass in front. The player breaks it, walks over it, places it (it keeps their facing, so it points away) and turns it with R; then the line runs and its load (14 iron, 4 copper ore) is exactly a Miner Mk1. `opening.rs` (`Sim::build_opening`, `Opening`, `Step`, `marker`), `Sim::opening` saved since version 47; `api/opening.rs`; a floating key/mouse prompt over what to act on (`ui/key-prompt.ts`), the power-up sound when the line starts. Tips are bigger and skimmable: a lead-in title, `*highlights*` and `[key]` caps in hint texts (`ui/hints.ts` `richText`; all `HINTS` marked up, a test checks the pairs). Hints "Carry the ore away" and "Ingots by belt" now also need a miner and a generator (the scene's belts must not skip the miner tips); "Place the miner" says where ore shows. Golden hash re-recorded; 835 tests.

## Change log, 2026-10-14 to 2026-10-15

- **2026-10-15: Hover pack charges from long-distance poles; power-up/down sounds (user request, off-plan).** `Factory::charge_poles` (everything but cables, with its middle and whether it is long) replaces `poles()` in the hover pack's core checks; the hands tie the pack within `CHARGE_RANGE` 6 of an ordinary pole or `CHARGE_RANGE_LONG` 32 of a long-distance one, and keep a tied pole `CHARGE_HOLD` 2 further out so the edge does not flicker (`helper_hands.rs`). New sound events `POWER_UP` / `POWER_DOWN` (8, 9; `audio/synth.ts` `powerUp` / `powerDown`, actions `powerup` / `powerdown` in the sound designer) play on the local pack's tie starting and ending. No save-format or core change.
- **2026-10-15: Automated buildings and the Upgrader (user request, off-plan).** Assembler rows for pole, long-distance pole, solar panel, accumulator, lamp, box, underpass and rail, and constructor rows for planks and sticks (`recipes/automate.rs`, rows 78–87, each equal to its hand recipe; the long-distance pole (12 beams, 16 aluminium plates, 60 wire) and the solar panel (6 silicon, 2 glass, 6 wire) lost a kind so they fit an assembler's three slots, and a stack caps one kind at 64). Block 115 Upgrader (`factory/process/upgrader.rs`, 2×2×2, `Category::Refit`, five input slots, tech Assembly; hand recipe in `recipes/refit.rs`): one machine row per tier of every `FAMILIES` entry (the Mk1 item plus each step's kits, built at compile time by `ladder` and `rows!`), locked like the upgrade (`Research::locked_by`, by category). No save-format change, no new tech, golden hash unchanged; 829 tests. Next free block 116, item 396, tech 84, machine recipe 135 (new tables join after `REFIT`), texture 306 (the Upgrader reuses assembler layers: request line in `ART_HANDOVER.md`).
- **2026-10-15: Research screen redesign (user request, web only).** A pan and zoom canvas instead of scrollbars: `ui/research-view.ts` (`PanZoom`: drag, wheel at the pointer, glide, level of detail near/far/tiny), eras as tinted bands by pack count with progress and jump buttons (`ui/research-eras.ts`, `tech-tree.ts` now lays out per era and sweeps rows to cut crossings), a minimap (`research-map.ts`), search by name or unlock (`research-search.ts`), nodes show what they unlock (`research-node.ts`). No engine or save change.
- **2026-10-15: Drop pods (step 12.4).** Blocks 113 Pod Launcher and 114 Landing Pad (`factory/process/pods.rs`, `Pick::Pods` / `Pick::Land`), item 395 Drop Pod, texture 302–305, assembler row 77, tech Drop Pods 83 (`research/later.rs`), `pods.rs` (pods in flight, `Factory::pods`, save version 46), the falling capsule (`drone_view.rs`). Golden hash re-recorded; 822 tests. Next free block 115, item 396, tech 84, machine recipe 78, texture 306.
- **2026-10-14: Long-distance pole (user request, off-plan).** Block `LONG_POLE` 112, a 2×2×3 lattice tower (tech Long-Distance Power 82 in the new `research/later.rs`, the open end of `TECHS`; hand recipe `LONG_POLE_RECIPE` at the end of `RECIPES`). A `Pole` of tier 14 (`LONG_TIER`) whose cells are indexed in `Factory.at` like a multi-block machine (`footprint::of` / `layout_dir` / `footprint_at` know it; it stores no facing, so it stands the same whichever way it is placed). Links 160 blocks to other long-distance poles, at most 16 to an ordinary pole (`pole::span`), reach 10, 6 slots. No save-format change. Also: Esc on the pause screen plays on (`main.ts`; if the browser will not re-lock the mouse the click-to-play hint shows). Reuses the pole textures: art request in `ART_HANDOVER.md`. Golden hash re-recorded. Next free block 113.
- **2026-10-14: What satellites give (step 12.3, all four kinds).** Survey (`survey/orbit.rs`), comms (`Data::uplink`, `Hangar::linked`), power (Rectenna 110, `process/rectenna.rs`) and science (Ground Station 111, `process/ground.rs`: 40 TF a satellite overhead, cap 100, 150 kW). Techs 79–81, items 391–394, machine recipes 70–76, textures 294–301. Golden hash re-recorded; 805 tests. Next free block 112, item 395, tech 82, machine recipe 77, texture 302.
- **2026-10-14: Hit boxes follow the model (user request).** Poles, belts, splitters, filters, pipes, sensors and nodes are hit where their model is, not as a whole cell, so a pole no longer hides the block behind it: `hitbox.rs` (`Boxes`: oriented boxes from the model's own instances, slab test) fed by `Factory::hit_boxes` (`factory/hitbox.rs`; a belt without its items via `Belt::frame`, a cable a 0.4 cube), used by `interaction/aim.rs`; the target outline hugs the model (`Game::target_fit`). Presentation only: no save change. 779 tests.
- **2026-10-14: one slot per input item, and click-to-take.** `Processor::room_for` caps every `Pick::ByInput` machine like `Pick::Chosen` (a stack of an item at most, so one item can't fill several slots; `input_slots_for` is gone). `Action::TakeMachineSlot` (tag 56) takes a panel slot's stack (inputs and fuel too) into the inventory; the machine panel's filled slots are buttons. 771 tests.
