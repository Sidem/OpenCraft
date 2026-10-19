# Milestone 13 (Settlements): the finished steps moved out of the plan

Moved from `docs/DEV_PLAN.md` on 2026-10-16 at step 13.6, when 13.1 to 13.5b were built (13.6a and the change log after R1c), to keep the plan under its
budget. Each step keeps what was built and, after "As talked", what was agreed. The player's view is `docs/TOWNS.md`;
the code map rows are under `settlements/` in `docs/CODEMAP.md`.
- [x] **13.1 Town kinds, sites and the depot.** Built 2026-10-16: `settlements/` (`Sim::settlements`, save version 48;
  `KINDS` rows: harbour = sea within 6 blocks, river town = river or lake within 6, highland workshop = Highlands, basalt
  forge = BasaltFields; water rows first), `Settlements::site` (pure function of the generator: `WorldGen::open_water`,
  `biome_at`; never within 200 blocks of spawn, at least 300 from another town), block 116 Town Depot (3×3×2,
  `factory/process/depot.rs`, `Pick::Store`, belts in on all sides; placeholder look), tech Settlements 84 (red, green,
  blue; needs Blue Science), its hand recipe (`recipes/towns.rs`), founding in `put_footprint` and removal in `dismantle`,
  the placing preview (`town_preview.rs`: outline green or red, the label names the town or why not) and a readout line on
  the depot. In the user's world (seed 1337) the nearest sites are 205–363 blocks out. Tested headlessly; seen in the
  browser pane. Perks (13.4) and orders (13.2) are not built yet; breaking a depot ends its town for now (13.3b makes it
  dormant instead).
- [x] **13.2 Orders, upkeep and stalling.** Built 2026-10-16: `settlements/orders.rs` (the tables), `settlements/run.rs`
  (the tick, after the factory), `Pick::Depot` with `Processor::wants` (each good capped: two minutes of upkeep plus what
  the order still misses, so the store can't silt up; a stalled town takes only upkeep goods), save version 49, the depot
  readout (stage, order progress, upkeep, supplied or stalled). A town already stocks the camp's upkeep before the camp
  stands; the store's last minute of upkeep is never counted toward the order. Tested headlessly (belt-fed through two
  stages, stalled and back) and in the browser pane. The plan as talked:
  Numbers settled in a talk (section 1, 2026-10-16). Data rows per kind and stage
  (5 stages): the stage's order (about 1,000, 2,000, 4,000, 8,000, 16,000 items; bricks, glass, planks and concrete first,
  steel and lamps, then plastics, circuits and processors) and its upkeep (20, 60, 150, 300, 600 items a minute, of the
  kind's goods: harbour planks and glass, river town bricks and lamps, workshop gears and steel plates, forge coal and
  quicklime, plus one better good a stage). The depot takes in only what its town wants (its order and upkeep goods); each
  minute it uses up a minute's upkeep from its store, then counts the rest toward the open order; a complete order opens the
  next stage. A minute's upkeep missing: the town is **stalled** (its perk pauses, the order stops filling), nothing is lost,
  and it resumes once the upkeep is there. Town state saved (stage, order progress, stalled). **Done when:** a scenario test
  feeds a depot by belt through two stages, then starves it into a stall and back.
- [x] **13.3a Plots, plans and the town's drones.** Built 2026-10-16 (save version 50; `Town::planned` plans the stages of
  an older save on load; buildings going up show as one faint outline each; drones carry up to 8 blocks a trip; a camp
  takes about 3 minutes; a basalt forge seen growing to a village in the browser pane). As talked (section 1):
  `settlements/plans.rs`: building plans as
  data (cells by role: wall, roof, floor, window, lamp, chimney; 2 to 4 buildings a stage) and a palette per kind mapping
  roles to existing blocks (harbour planks, river town stone bricks, workshop steel-looking blocks, forge stone with
  chimneys). Plots in fixed rings round the depot, chosen when a stage opens from the anchor alone and read through
  `block_anywhere_or_generate` (no dependence on loaded chunks): skipped when steep, over deep water or holding a non-natural
  block; stilts over shallow water; each building on a stone foundation down to the ground; never digs. The town's open
  build list is core state on `Town` (save version 50), never the shared `Ghosts`, so the player's drones never take it; it
  is drawn as faint ghosts. The depot's own fleet (2, plus 2 a stage) reuses the `drones/` flight; blocks are free; work
  pauses while stalled. Lamps in the plans light the windows at night; forge and workshop chimneys smoke (`factory/smoke.rs`).
  **Done when:** a scenario test builds a camp on uneven ground with the town's drones only, no player block touched; a town
  is seen growing from a depot to its third stage in the browser pane.
- [x] **13.3b Rebuilding and dormant towns.** Built 2026-10-16 (save version 51; `Settlements::sleep` / `broken`, the
  dormant spot marked amber and `Unfit::Dormant` naming it). A town's own blocks drop nothing when broken (from the floor
  up), so the free rebuilds can't be farmed (Claude's pick at the user's request, 2026-10-16). As talked: A built town cell that no longer holds its block goes back on the build list
  (a cell the player has filled with something else is skipped). Breaking the depot makes the town dormant (kept with its
  stage, progress and plan; no tick, no building); `Settlements::site` lets a depot on a dormant town's own anchor wake it.
  **Done when:** tests break a town block (rebuilt) and the depot (dormant, then woken with its stage).
- [x] **13.4a Perks: the depot's out side, relay, forge, workshop.** Built 2026-10-16 (save version 52; `settlements/perks/`,
  the depot's perk buffers, `Step::half` for half-price crafts, the perk line in the depot readout). The forge turns 4 iron
  ore into 3 plates and 4 copper ore into 6 ingots; the workshop mends one tool a batch (a batch is 4 items at the upkeep's
  rate). Hand-craft cards show the half price; a hint on the card naming the workshop waits for 13.5. As talked: One module per perk,
  each paused while its town is stalled or dormant. The depot's front side sends perk goods out onto belts (the other three
  stay in; the player can also take them by hand). **River town:** drone ports within 128 blocks get double reach and cargo
  range. **Basalt forge:** takes raw iron and copper ore (on top of its order and upkeep goods) and sends out plates with 50%
  more yield, at a rate growing with its stage like the upkeep (about 20 ore a minute at a camp to 600 at a city).
  **Highland workshop:** while one is supplied, hauler packs, jetpacks and gloves cost half their inputs in hand crafting
  anywhere; worn tools put into its depot come back out whole. **Done when:** each of the three perks is tested on, off and
  paused by a stall.
- [x] **13.4b Harbour contract work.** Built 2026-10-16 (save version 53). `settlements/perks/harbour/`: `Made` (a bit per
  item, noted from the tick's `Produced` events, so a pre-53 world starts empty and fills as its machines run), `price.rs`
  (raw content by the route `recipes::recycling::raw_cost` already picks; sand salvaged from slag counts as raw; tools
  are not offered, since priced per use one would come out worn), `Town::product` and `Town::paid` (saved: a dear
  product takes many batches). Choices made in the build: the harbour pays in `BATCH` raw goods at its upkeep's rate
  (20 to 600 a minute), the depot wants each raw good only up to what the next product still owes (a stack at most), and
  another pick starts the payment over. The chooser is `ui/harbour.ts` in the town section. Seen in the browser on the
  dev world (a harbour founded at (-200, 64, -470), an assembler making lamps beside it). As planned: The player picks, in the depot panel, a product their machines
  have made; the harbour takes three times its basic raw content and sends it out. New pieces: a core record of which items
  the player's machines have ever made (a bitset by item, saved) and a derived raw content per item from the recipes (with
  a flag for chains that need bauxite or uranium, which are never offered). **Done when:** the perk is tested on and off,
  and a test shows the harbour never offers a far-material product.
- [x] **13.5a The town panel and stall news.** Built 2026-10-16 (`ui/town.ts`, `api/towns.rs`; no save change). The news is
  not a `SimEvent` queue but `town_states` read once a second by the HUD, compared with its last look (`lib.rs` is over
  budget for a new `Game` field, and 13.5b's map markers need the same query); the depot's store got its own panel role
  (`ROLE_STORE`, "Town") so "Take output" leaves it alone. As talked: Right-clicking a depot opens the usual machine
  panel with a town section on top (one new UI file, `ui/town.ts`, fed by a flat engine query): kind and stage, the open
  order with a progress bar a line, each upkeep good with its stock in minutes, and the perk (its text, its in and out slots,
  taken by hand like any panel slot). A toast when a town stalls, naming what it is short of, and when it is supplied again
  (a `SimEvent`, turned into news like the launch toasts). **Done when:** an engine test covers the query and the stall and
  recovery events; the panel is seen in the browser pane on the dev world's forge village.
- [x] **13.5b Maps and tips.** Built 2026-10-16 (no save change). A town is a house mark (`MARK_TOWN` + state) in its kind's
  colour (`KindDef::colour`, shared with the shading), ringed red while stalled and faded while dormant, in place of its
  depot's machine square. The shading (`minimap/sites.rs`) votes per explored tile over a 4×4 grid of dry columns with
  `settlements::kind_at` (split out of `site`); spawn and towns are filtered per query, since they change. Measured on the
  dev world (about 555 explored tiles): 3–5 ms a call while it works out 24 new tiles, 0.1 ms once cached, so a
  4,096-tile map fills in within about 17 s at the map's 10 marks a second. The tips are a new side track
  (`SIDE_HINTS`, numbered from `SIDE_BASE` 1000): Settlements needs only blue science, so linear tips would have skipped
  or been skipped by the main order; they show ahead of it once researched, clear when a town is founded, its camp
  stands, and it reaches a village or a rail, pod launcher or cargo drone exists. As talked: Each town gets a marker of its kind on both maps (a new mark kind), coloured supplied, stalled or
  dormant. While a depot is held, the world map tints explored ground by the kind a depot there would found
  (`Settlements::site` sampled coarsely, worked out once per explored tile and cached, presentation only). Three hints after
  Settlements research: found a town (where each kind goes, its perk); feed it (belts into its back and sides, a minute's
  upkeep, what a stall does); supply it from far (cargo drones, trains or drop pods); each clears when done. **Done when:** a
  first-time player can find a site and see what a town wants without reading docs; the tint's cost is measured on a big
  explored map.
- [x] **13.6a Cleanup without the play-test** (2026-10-16): `bench_towns` (`settlements/tests.rs`, ignored: one town of
  each kind made a city at once, perks running, 20 minutes) found a stage's planning re-reading every column for each
  building and slot through the 8-chunk generated cache, about 14 ms a town; `plan_stage` now reads each column's ground
  once a plan (`plots.rs`), so a camp plans in about 4 ms a town. Four cities then cost 10 µs a tick on average, 1.9 ms
  at worst; planning all twenty stages at once (only an old save loading a grown town) about 105 ms. The manual got its
  town chapter (`docs/TOWNS.md`). Milestone 14 stays in `docs/ROADMAP.md` until the R steps before it are done (the plan
  is near its budget; it moves in when it is next).

## Change log (moved from the plan's section 8)

- **2026-10-16: Step 13.6a (cleanup without the play-test).** `bench_towns`; `plan_stage` reads each column's ground once a
  plan (a camp's planning about 14 ms a town down to about 4 ms); `docs/TOWNS.md`, the player's town chapter; Milestone 13's
  finished steps moved to `docs/CHANGELOG_M13.md`. Balance (13.6b) waits for the user's play-test.
- **2026-10-16: Step 13.4b built (harbour contract work).** `settlements/perks/harbour/` (`Made`, `price`, the perk),
  `Action::SetHarbourProduct`, the chooser `ui/harbour.ts`; `settlements/state.rs` split out of `mod.rs`; save version 53.
  866 tests.
- **2026-10-16: Step 13.5b built (maps and tips).** Town houses on both maps (`MARK_TOWN`), town site shading and a
  legend on the world map while a depot is held (`minimap/sites.rs`, `world_map_sites`), and three town tips as a side
  track (`hints::SIDE_HINTS`, `hint_side`). 864 tests.
- **2026-10-16: Step 13.5a built (the town panel and stall news).** `ui/town.ts` on top of a depot's machine panel,
  `api/towns.rs` (`town_panel`, `town_states`, `town_news`), HUD toasts when a town stalls or is supplied again, the
  store's own panel role. 861 tests.
- **2026-10-16: 13.5 talked (docs only).** A town section in the machine panel, town markers and site shading on the maps,
  stall and recovery toasts, three tips (section 1); 13.5 split into 13.5a (panel, stall news) and 13.5b (maps, tips).
- **2026-10-16: Step 13.4a built (perks: the depot's out side, relay, forge, workshop).** `settlements/perks/` (one module
  a perk, a batch every `period(built)` ticks at the upkeep's rate, no saved state); the depot keeps perk goods apart
  (`Want::perk` into `input`, out at the front from `side`; save version 52, which also saves `crafting::Step::half`);
  `Hangar::relayed`; the processor's bytes moved to `factory/process/state.rs`. 860 tests; golden hash unchanged.
- **2026-10-16: 13.4 talked (docs only).** The depot's front becomes an out side; the forge's rate grows with the stage;
  the workshop halves gear and mends tools; the harbour's product is the player's pick, as 13.4b after 13.5 (section 1).
- **2026-10-16: Town blocks can't be farmed (user request).** Breaking a block of a town's plan (from the floor up)
  drops nothing (`Settlements::broken` says so to `dismantle`); chosen over rebuilds paid from the depot's store, which
  would need new saved state. No save change.
- **2026-10-16: Step 13.3b built (rebuilding and dormant towns).** Breaking a depot makes its town dormant (`Town::dormant`,
  save version 51) and a depot on its old spot wakes it; breaking a finished town block sends the cursor back so the drones
  rebuild it, unless the player filled the gap. 855 tests.
- **2026-10-16: Step 13.3a built (plots, plans and the town's drones).** `settlements/plans.rs`, `plots.rs` and `build.rs`;
  `Town` gains `planned`, `plots`, `cursor` and `drones` (save version 50); town drones, chimney smoke and outlines in
  `town_preview.rs`; `drones::approach` and `factory::smoke` opened to the crate. 854 tests.
- **2026-10-16: 13.3 talked (docs only).** The town's own drones build it for free; buildings stand on foundations and bad
  plots are skipped; one building set in each kind's materials; broken town blocks are rebuilt and a town without its depot
  goes dormant (section 1). Step 13.3 split into 13.3a (plots, plans, drones) and 13.3b (rebuilding, dormant towns).
