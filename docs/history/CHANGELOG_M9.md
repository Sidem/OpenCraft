# Milestone 9 (Distance): the step list

Moved from DEV_PLAN section 4 at the end of Milestone 9 (2026-10-05).


Goal: the world is bigger than the factory. A new ore sits only in far biomes, so the player must find it, reach it and
haul it home. Order: the ore, aluminium, finding far ground, rails, then flight and cargo.

- [x] **9.1 Bauxite and generator version 5** (`worldgen/geology.rs`, new worlds only; versions 1–4 never make it): block
  76 `BAUXITE_ORE`, in deposits only in deserts and basalt fields at least 600 blocks from spawn (weights 4 and 3 on top of
  those biomes' ores: 27% and 23% of their deposits), a shallow band of 4–16 blocks, exposed on bare rock like the metals.
  No stain (it shows itself). Ore guide row and note (version 5 worlds only), scanner filter, map colour, texture layer 186.
  Used by nothing until 9.2 (`NO_USE_YET`). Tests 458 → 459; v5 is not pinned in `released_versions_never_change` yet.
- [x] **9.2 Aluminium** (`research/distance.rs`, `recipes/aluminium.rs`, `textures/aluminium.rs`): tech Bauxite Processing
  (index 36, after Ore Crushing and Violet Science; 120 units of all four packs) unlocks the electrolytic cell (block 77, a
  `ProcessSpec` row of 3×2×2 with a slag `Role::Side` hatch on the right, 300 kW, new `Category::Electrolysis`; hand recipe 12 steel
  plates, 16 stone bricks, 24 copper wire, 4 circuits) and machine recipes 37–40 (`BAUXITE_RECIPES`: crusher 2 ore → 3
  crushed bauxite; cell 2 crushed + 1 quicklime → ingot + slag, 6 s; constructor ingot → plate, 3 s; assembler plate + circuit
  + 4 wire → battery, 6 s). Items 339–342, layers 187–192. The battery waits for 9.5 (`NO_USE_YET`). The scenario test runs
  bauxite → crushed → aluminium → plate → battery. The tech count grew, so the golden hash was re-recorded (research bytes
  list every tech); no save version bump (older saves read fewer techs). Tests 459 → 460.
- [x] **9.3 Finding far ground** (user request, 2026-10-04: the player must roughly know which way to travel to reach the
  right biome). Built on the plan's proposal while the two open questions (section 6) stay open: the **Scanner Mk2 gives it
  free** and it is a **bearing only**. `WorldGen::bearing_to(ore, x, z)` (`worldgen/bearing.rs`) samples the height and biome
  fields on a 48-block grid in square rings (to 3,072 blocks) and returns the nearest column whose biome holds the ore (a
  query: no state, never touches chunks, so the hash cannot move; `None` before version 2, bauxite before 5). With an ore
  filter on and none of it in range, the scan panel says "south-west, 600 to 800 blocks" and the top pointer leads to that
  ground (`ui/prospect.ts` `farGround`); scanning again on the way re-aims it. Tests: nearest-sampled, here, old worlds,
  order-independent, and that a Mk2 scan round the target finds bauxite on three seeds. Verified in the browser on a
  version 5 world. No map pin yet (add if the user wants one). Tests 460 → 467.
- [ ] **9.4 Rails and trains** (the user wants trains for hauling far resources). Trucks are not planned.
  - [x] **9.4a Rails** (`factory/rail.rs`, `rail/curve.rs`, `rail_tools.rs`, `Kind::Rail`, block 78, tech Rails index 37: needs
    Violet Science and Earthworks; hand recipe 1 steel beam + 1 concrete → 6 rails). **Redesigned 2026-10-04 at the user's
    request** (the first version laid a block per cell like belts): rails are built like power poles. A rail is a *node* on
    the grid with a heading byte (the facing of its `PlaceBlock`); `Track {a, b}` joins two nodes (saved; `Action::Connect` /
    `Disconnect` between nodes, quietly nothing unless `Factory::track_fit` says `Fit::Ok`) and is a smooth curve free of the
    voxel grid: a cubic Hermite spline in x and z leaving each node along its heading (flipped to the way of travel,
    tangents as long as the chord), height linear along it (`Curve`). A pair fits when 3 to 32 blocks apart (`MAX_SPAN`),
    at most 1 in 3 steep, each heading within 60° of the chord and the bend radius `chord / (2 sin a)` at least 8; a node
    holds up to 4 tracks (junctions). The hand (`rail_tools.rs`): click ground to place a node joined to the selected one
    (a first node faces the view, a later one the mirror of the previous heading in the chord, so the track is an arc and
    a straight chord stays straight; Shift = full reach along the view; crouch starts a new line), click a node to select
    or join, crouch-click to cut. Ghost node and curve, outlines (green fits, red why-not, amber cut, blue selected) and
    HUD label. Track pieces are boxes (sleeper + two rails per 0.75 block, tilted to the grade), culled by range. Track
    does not clear terrain (planner jobs). Save version 31 (headings, then the track list after the rails; v30 rails
    load facing north); golden hash re-recorded. Tests 467 → 481. Not done: warning when a curve passes through solid
    blocks; rail removal refunds nothing special (a node is an ordinary block).
  - [x] **9.4b1 Locomotives** (`factory/trains.rs`, `trains/model.rs`, `train_tools.rs`; tech Trains, index 38; hand recipe 4 motors +
    4 circuits + 10 steel plates + 4 steel beams → item 343): a `Train` is a stretch of the track graph, `path` = directed edges
    tail first, `head` = mm along the last edge (arc length, so speed is even on bends: 150 mm a tick, 9 blocks a second). At a
    node it takes the track most straight ahead (within 60°, `next_from`), with none it reverses, so a plain line shuttles.
    `Action::PlaceTrain` / `TakeTrain` (click a node, crouch-click to pick up); a track with a train on it cannot be cut. Save 32.
  - [x] **9.4b2 Wagons and docks** (`trains/cars.rs`, `trains/docks.rs`, `process/docks.rs`; tech Freight, index 39; wagon 8 steel plates +
    2 steel beams + 1 storage box → item 344; docks 10 steel plates + 2 motors + 2 circuits + 1 storage box → blocks 79 Loading, 80
    Unloading): a train has up to 6 `cars` (2.5 m, 24 cargo slots each, one `Buffer`) and `idle`; `couple` puts track behind the train
    under the wagon or pulls the train forward. Docks are two spec rows (2×2×1, 48 slots like a silo, belts on every side). A train
    with wagons stops at a node with a dock cell within 2 blocks, trades 8 items a tick and drives on after 5 idle seconds. Save 33.
    Tests include ore through a loading dock and a train into an unloading dock (40 of 40). Placeholder looks: `ART_HANDOVER.md`.
  - [x] **9.4b3 Schedules** (fuel and dock filters deferred): a train has a list of up to 8 dock anchors
    (`factory/trains/schedule.rs`) and a `next` index. At junctions `steer` runs a breadth-first search toward the next
    dock (60° turn limit); the train passes other docks, stops at the next, then moves on (round again). A gone dock is
    skipped; no schedule = stop at every dock. Edited with the locomotive in hand: click a train's node to select it,
    click docks to append, crouch-click a dock to clear (`Action::TrainStop`, tag 43). Save version 34. Tests 500 → 505
    (order across a fork, passing docks, edit limits, gone dock and save, the hand). **Deferred** to a train panel
    (9.4c or later): fuel and dock item filters.
  - [x] **9.4c Signals** (`factory/trains/signals.rs`; the Rail Signal item 345, recipe 1 steel plate + 1 circuit → 2, unlocked
    by Freight): a signal is a flag on a rail node (`Rail::signal`, an item put there and taken back by clicking the node
    with it in hand: `Action::ToggleSignal`, tag 44). Signals cut the track into *sections* (tracks joined through nodes
    without one). A train reaching a signal node goes on only into a section no other train is in (a train occupies every
    edge of its `path`); at a junction it takes the straightest branch whose section is free, else it waits at the node
    (stateless, asked again each tick). No signals, no change. `step_trains` steps one train at a time with its slot empty
    so it sees only the others. Drawn as a post with an amber lamp beside the node; breaking the node returns the item.
    Save version 35. Tests 505 → 511: a passing loop between two signals lets a train from each end pass without ever
    sharing a track (and without signals they meet), a signal on a plain line holds both trains (a layout without a
    loop can deadlock) until one is removed, save and refund, the hand. **Known:** scheduled trains detour onto any free
    branch when theirs is blocked; no lamp colours yet.
- [x] **9.5 Cargo drones and the hover pack** (`docs/TECH_ERAS.md` section 5): port-to-port cargo drones limited by
  battery range; the hover pack as flight's second tier.
  - [x] **9.5a Hover pack** (`helpers/mod.rs`, `helper_hands.rs`, `player.rs`; tech Hover Pack, index 40, needs Jetpack and Bauxite
    Processing, 160 units of all four packs; hand recipe 6 aluminium plates + 2 motors + 4 batteries + 2 processors → item 346,
    icon layer 200): hold jump in the air with the pack in the pack and the body hovers (`Action::Hover`, tag 45) until it lands:
    its height is held (jump rises and crouch sinks at 4 blocks a second), it moves at 8 blocks a second (13 sprinting) and it
    wins over the jetpack. `Helpers.charge` (ticks of hover, `HOVER_CAP` 90 s) drains 1 a tick while hovering and fills
    `CHARGE_RATE` (2) a tick while the pack is tied to a pole (`Action::Charge`, tag 46: the hands send it for the nearest pole
    within `CHARGE_RANGE` 6 blocks of the feet, the core only checks that the pole stands; it unties itself when the pole goes).
    The pack draws no power from the grid. HUD line "Hover pack · N s of charge". Save version 36 (older saves read no
    charge); golden hash re-recorded. The first use of the battery.
  - [x] **9.5b Cargo drones** (`drones/cargo/`, `cargo_tools.rs`; tech Cargo Drones, index 41, needs Construction Drones and Bauxite
    Processing, 200 units of all four packs; assembler recipe 41: 1 drone + 4 aluminium plates + 2 batteries → item 347, 30 s;
    icon layer 201). Design (the user asked for my own): **the existing drone ports are the stations** and the boxes touching
    a pad are the interface, so belts feed and empty them as they already do. A port keeps one kind of drone (its one slot:
    construction or cargo; `Hangar::kind` tracks which is out). A **route** joins a source port to a destination port
    (`Action::SetRoute`, tag 47; one route per source, several sources may share a destination; set with a cargo drone in
    hand: click the first port, then the second, crouch-click clears). It must lie within the source tier's `cargo_range`
    (200 / 400 / 800 / 1,600 blocks). Every 2 s a powered source port with a cargo drone home launches one if its boxes hold
    a stack (the first that is not a battery or a drone) and the destination's boxes have room: it carries up to 64 items at 12
    blocks a second, unloads into the far boxes and flies home; what did not fit comes back. **Battery-limited:** a trip burns
    `ceil(round trip / 150)` batteries (at least one) from the source's boxes at launch, so distance is paid for in
    aluminium (80 blocks = 2 a trip, 600 = 8): trains need track but no fuel, cargo drones need no track but a battery
    belt. Core state `Sim.cargo` (routes, couriers), save version 37; golden hash re-recorded. Fleet conserved (home or in
    the air), breaking a port returns its cargo drones and drops its routes. Outlines (green / red out of range, blue first
    port, amber destination) and HUD label from the hand. **Also fixed:** `Factory::store_in_boxes` and `port_land` read
    `Buffer::add`'s return (what did *not* fit) as what did, so construction drones' deposits and landings also spilled
    loose duplicates; now counted properly (`storing_and_landing_report_what_really_happened`). Tests 518 → 527. **Not
    done:** a line drawn between routed ports on the maps, a port panel showing the route, drones needing their own battery
    in flight (batteries are spent at launch), several routes per source, item filters per route (the first stack goes).
- [x] **9.6 Cleanup:** onboarding tips for far ground, trains and hover/cargo; `bench_trains` (14 trains: 4.0 µs a tick, worst 31 µs; with a signal on every node 6.4 µs, worst 11 µs); README and CODEMAP; balance note in DEV_PLAN section 4. No new tests besides the ignored bench.
