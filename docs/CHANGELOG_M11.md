# Milestone 11 (Compute and photonics): the step list

Goal: the factory learns compute. Chips come from chemistry (acid and pure water), datacenters turn megawatts and coolant
into **compute**, a second grid carries compute like power, and compute buys things: AI research, endless bonus techs, the
optimizer, drone swarms, auto-routing and AI survey. Laser links join far plants and datacenters by line of sight. Numbers
below are first guesses; tune in play (`docs/WORKFLOW.md` section 6). Order: chips, the data grid, datacenters and cooling,
what compute buys, laser links, helpers, cleanup.
- [x] **11.1 Pure water, the chip fab and AI accelerators** (built 2026-10-08: block 98 Chip Fab, items 379 pure water canister, 380
  wafer, 381 AI accelerator, machine recipes 61 pure water (chemical plant: empty canister + sand + 2 water), 62 wafer (2 silicon +
  acid canister + pure water canister, 8 s, both shells come back), 63 accelerator (2 wafers + 2 processors + plastic, 20 s), techs
  Wafers = 56 (needs Acids and Lubricants and Gold Science, so a research center) and Accelerators = 57, textures 246–250,
  `Category::Fabrication`; `factory/process/fab.rs` is a 4×4×3, 1 MW `Pick::ByInput` machine with six input slots and no water
  inlet, and `fab_text` names what a half-fed batch lacks. Wafers and accelerators share one output slot, so the player filters the
  front belt. `recipes/machine.rs` (406) and `recipes/mod.rs` (401) passed their soft limits: split the machine table before adding
  to it again. No save bump; golden hash re-recorded for the new techs. Tests 634 → 643.)
- [x] **11.2 The data grid** (built 2026-10-08: `factory/fibre.rs`, block 99 Fibre Node, `Kind::Node`, tech Data Network 58 (needs Wafers),
  recipe 2 nodes from 2 processors, 4 glass, 2 plastic, 6 copper wire, textures 257–259, `SAVE_VERSION` 40, golden hash re-recorded.
  **Differs from the first idea:** not a generalised power `Channel` but its own small `Data`, derived at every relink and balanced
  after `Power::balance`. Nodes within 12 blocks join by themselves (no hand wiring); a processor whose spec has `compute` (new
  `ProcessSpec` field, TF at full power: positive makes, negative uses) hangs on the nearest node within 5. A producer gives its TF
  times its power speed; a consumer wants its TF while it wants power and runs at power speed × the grid's satisfaction (0 with
  no node: "No data link"). No real machine uses `compute` yet (11.3's datacenter is the first producer). `factory/machine.rs`
  split out of `factory/mod.rs`; `process/specs.rs` is at 408 lines: move its inline rows out before adding a spec. Tests 647 → 653.)
- [x] **11.3 AI datacenter** (built 2026-10-08: `process/datacenter.rs`, `Energy::Compute`, block 100 Datacenter, tech Datacenters 59 (needs
  Accelerators, Data Network, Diesel Power), textures 260–261, 4×4×3 hall, 3 MW, 100 TF at full power times its power share, hung on
  a fibre node like any producer. **Heat** is the reactor's model (heat in `progress`, coolant in `steam.water` filled by `draw_water`
  from a pump pipe to its blue inlet, a unit per 10 s of full load, `Status::Overheated` with hysteresis): without coolant it sheds
  only 5% of its load, so it trips after about 21 s, draws nothing and gives no compute (`Data::balance` skips it) until the heat has
  halved. Hand recipe 40 beams, 20 concrete, 16 accelerators, 32 copper wire. No save change (`SAVE_VERSION` stays 40), golden hash
  re-recorded. **Not yet:** Mk rack tiers (+50% compute for +50% power and heat): a later step if wanted. Tests 653 → 660.)
- [x] **11.4 Cooling tower and the coolant loop** (built 2026-10-08: `process/tower.rs`, block 101, tech Cooling 60, textures 262–263,
  3×3×5, 300 kW. **Differs from the first idea:** no return pipe; a tower on a datacenter's water network (`tower::seat`, two to a
  tower) supplies its coolant from its own tank while powered and watered, losing 1 unit in 20, else the datacenter uses the pump.
  Test: an hour is ~360 pump units open, ≤ 23 looped. Hand recipe 30 concrete, 12 pipes, 4 motors, 12 steel plates. Tests 660 → 668.)
- [x] **11.5 AI lab and compute research** (built 2026-10-08: `process/ailab.rs`, block 102, tech AI Labs 61, textures 264–265, 2×2×2, 100 kW,
  a data consumer of 20 TF. **Differs from the first idea:** a spec row of the center's code, not a new `Pick`; `step_centers` scales its
  speed by its grid's satisfaction (no node or no datacenter: it idles); twice a lab's speed, every 2nd unit free. **Compute cost is a
  marker:** `research::needs_ai_lab` (`AI_ONLY` in `research/compute.rs`: tech AI Research 62, no unlocks yet, 11.6 builds on it); labs and
  centers show `LabStatus::NeedsAi`. Recipe 20 steel plates, 6 accelerators, 10 processors, 20 circuits, 8 glass. Tests 668 → 673.)
- [x] **11.6 Endless bonus techs** (built 2026-10-08: `research/bonus.rs`, `Unlock::Bonus(kind)`, techs 63 Mining Productivity, 64 Machine
  Speed, 65 Drone Speed, all needing AI Research and no packs. **Differs from the first idea:** no save bump: a level is the tech's
  `progress`, which `Research` already saves (the table just grew by three; old saves load, golden hash re-recorded). `units` is
  `LEVELS` 100, a cap no play reaches (cost ×1.25 a level) that keeps `progress <= units` and the tests that finish every tech
  finite; creative worlds get level 100 (+300%). Cost is compute through lab time: `Research::unit_seconds` (60 s × 1.25 per level),
  paid only by an AI lab (`needs_ai_lab` is true for bonus techs; small labs no longer ask for power for them). Each level adds 3%
  (linear) through `Research::rate_permille`, read once a tick in `Factory::update` into the derived `Miner::boost` and
  `Processor::boost` (work of recipes only) and by `Sim::flight_step` for construction and cargo drones. The research screen
  shows "level N" and the next unit's seconds (`tech_endless`). Tests 673 → 683.)
- [x] **11.7 Optimizer node** (built 2026-10-08: `process/optimizer.rs`, block 103, `Energy::Optimizer` (a power sink like the winch), tech Optimizer 66 in the new `research/ai.rs` (needs AI Research; append later Milestone 11 techs there, after the bonus techs), textures 266–267, 2×2×2, 150 kW, a data consumer of 20 TF that always wants it. **Differs from the first idea:** range, not grid: every processor and miner whose anchor is within 16 blocks gets +25% whatever grid it is on, scaled by the optimizer's power share times its grid's satisfaction (so a shortage weakens it smoothly, and none gives nothing); a machine takes the best optimizer in range (`bonus_at`). It multiplies into `boost` in `Factory::update` beside the research bonuses (`bonuses` reads this tick's balance, never a derived `speed`, so a loaded core matches). Hand recipe 8 steel plates, 4 accelerators, 10 processors, 10 circuits, 4 glass. No save change; golden hash re-recorded for the new tech. Tests 683 → 689.)
- [x] **11.8 Laser power links** (built 2026-10-08: `factory/laser.rs`, `process/laser.rs`, blocks 104 emitter / 105 receiver (1×1,
  `Energy::Beam`, no power of their own), tech Photonics 67 (needs AI Research, 150 units, gold packs), textures 268–272,
  hand recipes 6 aluminium plates, 1 accelerator, 4 glass, 4 circuits each. The emitter shoots out of its front up to 128
  blocks; air and glass pass, a receiver (any side) links, anything else blocks. Rays are cached (`Beams`) and recast only on
  relink or when `Sim::block_changed` hits a ray (`beam_cut_check`), so there is no per-tick ray cost. **Power:** `Power::rebuild`
  groups poles into wire "sides", joins sides through beams into grids; `balance` accumulates demand per side and charges the
  emitter's grid receiving-side demand / 9 (the 90%). **Left out for now:** Mk tiers to 512 blocks. Drawn as a thin lit box,
  flickering up to its blocker. No save change; golden hash re-recorded. Tests 689 → 696.)
- [x] **11.9 Mirrors and data beams** (built 2026-10-09: blocks 106 Laser Mirror and 107 Data Receiver (`process/laser.rs`, 1×1,
  `Energy::Beam`), textures 276–278, both unlocked by Photonics 67, recipes in `recipes/compute.rs` (mirror 4 aluminium plates + 6
  glass; data receiver the receiver's recipe + 2 processors). **Differs from the first idea:** the data mode is not a setting in the
  receiver's panel but its own block: a beam that ends in a Data Receiver is a data link, so no new state, action or save change
  (`SAVE_VERSION` stays 43; golden hash unchanged). A mirror turns a horizontal beam 90° (double-sided; its facing parity picks the
  diagonal, `reflect`; R turns a placed emitter or mirror, `factory::turns`); a path is a list of `Ray` segments, at most 8 turns and
  128 blocks in all. A data link joins the data grids of the fibre nodes within 5 blocks of its two ends, losslessly, and needs no
  power at either end (`Data::joined`); readouts say when an end has no node. **Left out:** vertical mirrors, beam tiers.
  Tests 716.)
- [x] **11.10 Swarm hub and drone swarms** (built 2026-10-09: block 108, `process/hub.rs`, 2×2×2, 100 kW + 30 TF, `Energy::Optimizer`;
  tech Drone Swarms 71 needs AI Research and Swarm Logistics; recipe `SWARM_HUB_RECIPE`). A hub within 12 blocks adds 50% to a
  port's fleet (`Hangar::bonus`, derived each tick, best hub counts, no stacking) and costs compute. Done: a hub raises a Mk4
  port's fleet from 16 to 24 (the plan said Mk3, which keeps 12) and it drops back without compute.
- [x] **11.11 Auto-routing** (built 2026-10-09: `belt_line/route.rs`, `Unlock::Feature(Feature::AutoRoute)`, tech Auto-Routing 72 needs AI Research and Swarm Logistics).
  With a belt selected, a dragged line whose pointer rests on a machine routes itself there: an A* over voxels (a query, never core
  state) round walls and over steps, the last belt feeding the aimed face; no route says "No route" and marks the target red. In
  ghost mode (B) a dragged line is planted as ghosts for drones or hands (a click on a ghost removes it). Done: routes across a
  gap, round a wall and up a step, ghosts placed without editing the world, a blocked target says so. Not done: underpasses in
  routes, starting from a machine's output port (the drag still starts at the aimed face).
- [x] **11.12 AI survey** (built 2026-10-09: `survey.rs`, `Feature::AiSurvey`, tech AI Survey 73 needs AI Research). Guesses where ore lies from the
  stained soil on the explored map within 256 blocks (stain colour = ore, squares of stained columns joined into one guess, snapped to 8
  blocks, strength by stained area, dropped near an already prospected deposit of that ore), drawn as faint rings on both maps. Queries
  only: the state hash never moves. Done: a real world's stains match the generator's ore, a planted stain is guessed once the tech is
  researched and not before. Not done: guesses from scanner bearings, ores that leave no stain (bauxite, oil sand, uranium).
- [x] **11.13 Cleanup** (2026-10-09): three tips (compute, lasers, smarter tools: `hints.rs`), `bench_compute` (a plant of 6 datacenters, 8 AI labs,
  4 optimizers, 4 swarm hubs, 6 Mk4 ports, 14 nodes, 20 generators and 4 laser links: about 2 µs a tick, worst under 0.1 ms), README and CODEMAP,
  the balance note in DEV_PLAN section 4, this file, and Milestone 12 moved in from the roadmap.

