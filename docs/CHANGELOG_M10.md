# Milestone 10 (Fluids and chemistry): the step list

Moved from DEV_PLAN section 4 at the end of Milestone 10 (2026-10-05).


Goal: the factory learns fluids and chemistry. **The user's requests of 2026-10-05, which every step honours:** the world
generator changes drastically (two new versions); bauxite is nearer; caves are much rarer; water and oil never lag; the
research building grows to take a fifth pack; ore washing must not conflict with the crusher; oil must make economic
sense and refining must give several distillates. Numbers and reasoning: `docs/TECH_ERAS.md` section 6.
- [x] **10.1 The new ground (generator version 6)** (`worldgen/geology.rs` `EXTRAS`, `strata.rs`, `caves.rs`; blocks 81 Oil
  Sand and 82 Uranium Ore, layers 202–203, `block/ores.rs` split out). New worlds only; versions 1–5 are pinned by
  `released_versions_never_change`. (a) **Bauxite from 300 blocks out** (was 600; `Extra` rows carry a distance per
  version). (b) **About 13 times fewer caves:** *cave zones* (slow noise, 192-block scale, `ZONE_MIN`, about a quarter of
  the ground) and thinner tunnels (`TUNNEL_V6` 0.0018, was 0.0045); a chunk outside a zone skips both tunnel fields
  (`caves/tests.rs`: 545k cave cells in version 5, 41k in version 6 over the same 400 columns). (c) **Oil sand** (plains and
  lowlands, 200+ out, 30–65 down) and **uranium** (highlands and basalt, 400+ out, 45–85 down): veins and lodes only,
  thinned by `RARITY` (0.35 and 0.4), no surface hint: found with the scanner (`FILTER_ORES` has 8); about 120 oil and 26
  uranium deposits in 2,048 × 2,048 blocks. Ore guide rows follow the world's version (`ore_shares(ore, version)`).
  **Measured:** water's worst tick 0.09 ms (`bench_water`); oil is never a world block. Tests 527 → 532.
- [x] **10.2 Canisters and the pumpjack** (`factory/process/pump.rs`, `recipes/chemistry.rs`, `research/chemistry.rs`,
  `textures/chemistry.rs`; block 83, items 348 Empty and 349 Crude Oil Canister, layers 204–207, tech Oil Processing =
  42, machine recipe 42). The pumpjack is a one-block processor (`Pick::Pump`, 90 kW, tall model) that **drills straight
  down its own column** (up to 100 blocks) to oil sand and tracks that deposit like a miner (`sink_well`; saved as deposit
  key, drill bit and the oil kept). **Pace:** 5 units/s, 0.9 kept, 10 units to a canister, through the deposit's shared
  draw cap: a vein gives about 22 canisters a minute for hours, a lode far more. **Power:** 90 kW is more than a coal
  generator's 60 kW: oil wants a turbine or solar, the intended step up. Empties: 1 steel plate → 2 (hand or constructor,
  1.5 s). Oil sand burns as a weak fuel (90 kJ; coal 270). Split: `process/status.rs`, `research/join.rs`,
  `recipes/group.rs`. Tests 532 → 537 (`pump/tests.rs`).
- [x] **10.3 Refinery and cracker** (`factory/process/refinery.rs`; blocks 84–85, items 350 naphtha, 351 diesel, 352 heavy oil
  canisters and 353 sulfur, layers 208–215, `Category::Distilling` / `Cracking`, machine recipes 43–44, tech Refining = 43).
  **Refinery** 3×3×4, 150 kW: **Distil** 3 crude canisters + 1 water → naphtha, diesel, heavy oil canisters + 1 sulfur, 6 s.
  **Cracker** 2×2×3, 90 kW: **Crack** 2 heavy oil + 1 water → naphtha + diesel, 4 s. Both `Pick::ByInput`; **shells go
  through** (no empties needed). The first output leaves the front, the rest the right-hand hatches (one mixed belt: sort
  with filters); a full stream stops the machine and the status names it (`held_recipe`). **Water** is generalised from
  the boiler: any footprint with a `Role::Water` port is linked by `steam.rs`, `draw_water` fills a 4-unit tank from a
  pump, `recipes::water_use` is spent when a batch starts, `Status::NoWater` waits. Tests 537 → 542.
- [x] **10.4 Chemical plant** (`PLANT_SPEC` in `refinery.rs`; block 86, items 354 plastic, 355 acid and 356 lubricant
  canisters, layers 216–220, `Category::Chemistry`, machine recipes 45–47, techs Plastics = 44, Acids and Lubricants = 45).
  3×2×3, 120 kW, `Pick::Chosen`, water inlet on the left, one side hatch. **Plastic** 2 naphtha + 1 coal → 4 plastic + the 2
  empties back, 4 s. **Acid** 1 sulfur + 1 empty + 1 water → 1 acid canister, 3 s. **Lubricant** 1 heavy oil + 1 empty → 2
  lubricant, 3 s. Hand recipe: 14 plates, 6 beams, 8 glass, 2 motors, 6 circuits. **Dropped:** heavy oil as boiler fuel (a
  boiler's steam cap is under a canister's energy; the diesel generator burns canisters). Tests 542 → 544.
- [x] **10.5 Diesel power and electrolysis** (`process/diesel.rs`, `ELECTROLYSER_SPEC` in `refinery.rs`; blocks 87–88, items
  357 hydrogen and 358 oxygen canisters, layers 221–226, `Category::Splitting`, machine recipe 48, techs Diesel Power = 46,
  Electrolysis = 47; `block/chemistry.rs` split out). **Diesel generator** 2×2×2, `Energy::Diesel`: a power source filling
  what its grid still lacks (after sun, accumulators, coal and turbines) up to 400 kW; a diesel canister is 40 MJ (100 s at
  full load, 148 coal), lit only when asked, the empty goes to the output at once (a full output stops it); charge saved
  like an accumulator's. **Electrolyser** 2×2×2, 500 kW, water inlet: 3 empties + 2 water → 2 hydrogen (front) + 1 oxygen
  (side), 6 s; both wait for fuel cells and rockets. Tests 544 → 549.
- [x] **10.6 Ore washing** (`process/washer.rs`, `textures/washing.rs`; blocks 89 Ore Washer and 90 Tailings (a plain cube, a
  Planner fill block), items 359–361 washed iron, copper and bauxite, layers 227–232, `Category::Washing`, machine recipes
  49–55, tech Ore Washing = 48 needing Fluid Handling, Ore Crushing and Bauxite Processing). **Washer** 2×2×2, 60 kW,
  water inlet on the left, `Pick::ByInput` over *crushed* ore only (raw ore does not fit, so the crusher is never skipped):
  3 crushed + 1 water → 4 washed (front) + 1 tailings (right hatch), 3 s. Washed ore smelts 1 for 1 (the cell has a washed
  bauxite row), so a raw ore gives **1.0 ingots, crushed 1.5, washed 2.0** (`ingots_per_ore_by_route`). **Tailings** must
  be belted away (a full hatch stops it); the crusher grinds them to sand (row 55, as it does slag). Tests 549 → 553 (`washer/tests.rs`).
- [x] **10.7 The research center** (`process/center.rs`; block 91, items 362–364 Research Center Mk2–Mk4, tech Research Center
  = 49 needing Electronics; no new texture layers: it wears the lab's). A **processor** (`Pick::Research`, so footprint,
  ports, power, wiring and tiers are inherited): 2×2×2, belts into every side, eight pack slots (`CENTER_SLOTS`; a stack
  of each kind), Mk1–Mk4 at **twice a lab's speed and power** (×2/×4/×6/×8, 20/40/60/80 kW; free every 5th and 3rd unit at
  Mk3/Mk4). It researches exactly like a lab (`Study` holds the unit in progress, saved for centers only; `step_labs`
  counts both so the two never overshoot a tech). The small lab keeps `LAB_PACK_SLOTS` = 4 slots and now says "needs a
  research center" for a tech whose pack lies past them (`needs_center`; none does yet, gold packs in 10.11 will).
  Hand recipes: lab + 8 plates + 6 circuits + 6 glass; Mk2–Mk4 are the previous tier + 8 kits. No save bump (a new
  block's record only; labs unchanged), so the plan's lab-buffer migration was not needed. Golden hash re-recorded.
  `process/shape.rs` split out of `process/mod.rs`. Tests 553 → 558.
- [x] **10.8 Terrain overhaul and hydro (generator version 7, new worlds only).** `worldgen/landform.rs`: continents 1.4× larger,
  taller sharper ranges (soft cap 196), stepped mesas in hot dry country, spawn lifted to dry land. `worldgen/rivers.rs`: one
  jittered node per 256-block cell flows to its lowest lower neighbour, so every chain ends in the sea or a lake; a node is wet
  by the moisture field (dry country 29% of nodes, wet 81%); lakes where a river ends inland and by chance in wet country.
  Water is a staircase of 12-block pools with weirs between, so still water stays put (`river_channels_hold_their_water`:
  under 1% leaking faces); channel carved, banks raised, a valley ramp grows with the height difference (gorges); no water
  within 100 blocks of spawn, ponds keep off. Bearings, deposits, the starter set and cave guards read `height_at` and
  `water_top`, so they agree. Version 6 is now pinned. **Water wheel** (`process/hydro.rs`, block 92, tech Hydropower = 50 after
  Steam Power, red + green): 3×3×1, each tick counts the water touching it (still 2 kW, flowing 4 kW, at most 48 kW) and
  gives what its grid lacks before any fuel burns; weirs and piped pools are the dams (no separate block). Recipe: 12 planks,
  8 rods, 4 gears, 6 wire. `worldgen/trees.rs` split out. Drone tests pin version 6 (their ground lies under the new sea).
  Golden hash re-recorded. Tests 558 → 570.
- [x] **10.9 Hoists.** Blocks 93 (Hoist Shaft, a steel climbing frame: a ladder variant, 3 blocks/s, `Player::climbable`) and 94
  (Hoist Winch, `process/hoist.rs`, spec `Energy::Hoist`, 20 kW, a pole like any consumer); tech Hoists = 51 (Electronics, red +
  green + blue). A shaft whose top cell has a powered winch above or beside it carries riders at up to 9 blocks/s, scaled by the
  grid's power share (`Factory::hoist_rate`, set each tick by the authority as `Player::hoist`; the body's climb rate is
  `CLIMB_SPEED.max(hoist)`); without a winch or power it stays a ladder. Not built: boxes that ride the shaft (belt lifts already
  carry items). Textures 233–235; `block/make.rs` split out of `block/mod.rs`. Golden hash re-recorded, no save bump. Tests 570 → 574.
- [x] **10.10 Nuclear power.** Blocks 95 (Centrifuge, 2×2×3, 200 kW, `Category::Enrichment`: 4 uranium ore + a steel plate → a fuel
  cell, item 365, 10 s) and 96 (Nuclear Reactor, 3×3×3, `process/nuclear.rs`, `Energy::Reactor`); tech Nuclear Power = 52 (Steam Power +
  Refining, red–violet). The reactor is a power source like the diesel generator: fuel cells in at the back, 2 MW at most, a cell lasts
  150 s at full load, it gives only what the grid lacks (order: sun, hydro, reactor, coal, steam, diesel). Whatever it gives heats it;
  water from a pipe to its blue inlet cools it (a unit per 10 s of full output; the tank is `steam.water` in kW·ticks) and with water
  it never heats, without it only sheds 100 kW. At 20 s of full output of heat it shuts down (`Status::Overheated`) until the heat has
  halved. Heat is kept in `progress`, so no save bump. Textures 236–240 (`textures/nuclear.rs`). Golden hash re-recorded. Tests 574 → 581.
- [x] **10.11 Gold science and Mk5.** Items 366 (Gold Science Pack, 1 plastic + 1 battery + 1 processor → 2, assembler 20 s) and 367 (Gold
  Kit, 1 processor + 2 aluminium plates + 1 plastic → 4, 5 s), machine recipes 57–58; `PACKS` has a fifth entry, so only a research
  center holds gold packs (`needs_center` is now true for Mk5 Machines). Techs Gold Science = 53 (r g b v; Processors, Bauxite Processing,
  Plastics, Research Center) and Mk5 Machines = 54 (all five packs; unlocks the kit and Mk5 of miners, smelters, constructors, assemblers,
  blast furnaces, drone ports, research centers). Mk5 numbers: miner 8/s, 97%, 150 kW; processors ×8 (smelter 130 kW, constructor
  120, assembler 160); drone port 128 blocks, 24 drones, 200 kW; center ×10, 100 kW, every 2nd unit free. Belts, poles and labs stop at
  Mk4. Items 368–374 (Mk5), textures 241–242, three hints (oil, nuclear, gold). `item/later.rs` and `recipes/rows.rs` split out.
  Golden hash re-recorded, no save bump. Tests 581 → 583.
- [x] **10.12 Cleanup.** Tips (three hints in 10.11), the worst tick (`bench_chemistry`, ignored: 20 dry water wheels, 8 reactors, 8 refineries, 8 diesel generators, 4 winches and 4 riders asking the hoist rate every tick on one grid cost 19 µs a tick on average, worst 36 µs), the balance note (DEV_PLAN section 4), README, CODEMAP; Milestone 11 moved in from the roadmap. Tests stay at 583 (the bench is ignored).
