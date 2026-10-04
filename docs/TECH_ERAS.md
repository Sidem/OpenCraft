# OpenCraft tech eras: items, recipes, machines, techs

The detail behind `docs/TECH_TREE.md` (read that first). **Read only the era you plan or build.** Era 4
is detailed for building now (era 3 is built); later eras are sketches to detail when their milestone comes up. Recipes
are `inputs → outputs (machine, seconds)`; techs append to `TECHS` in the order listed ("r g b v y" are
the red, green, blue, violet and gold packs). All numbers are first guesses.

## 1. What a tier changes, per family

Each family's numbers are one array indexed by tier, atop its module.

| Family | Mk1 | Mk2 | Mk3 | Mk4 | Mk5 |
|---|---|---|---|---|---|
| Belts, ramps, lifts (blocks/s; items/s) | 1; 2.9 | 2; 5.7 | 4; 11.4 | 8; 22.9 | — |
| (measured at 60 ticks/s: a belt takes the next item on the first tick past the 0.35 gap) | 2.9 | 5.5 | 10 | | |
| Underpass range | 5 | 7 | 9 | 12 | — |
| Splitter, filter | pass as fast as a belt of their tier | | | | |
| Miner (units/s · recovery · kW) | 1 · 60 % · 5 | 2 · 75 % · 20 | 4 · 85 % · 45 | 6 · 92 % · 90 | 8 · 97 % · 150 |
| Smelter | burner ×1 | burner ×2, a quarter less fuel an ingot | electric ×3, 40 kW, no fuel | ×5, 80 kW | — |
| Other processors (speed; power grows alike) | ×1 | ×2 | ×3 | ×5 | ×8 |
| Coal generator (kW · kJ a coal) | 60 · 270 | 100 · 337 | — | — | — |
| Pole (link · reach) | 10 · 5 | 16 · 7 | 32 · 9 (pylon) | 32 · 16 (substation) | — |
| Storage box (slots) | 24 | 36 | 48 | — | — |
| Pump (sources/s · kW) | 2 · 5 | 4 · 10 | 6 · 20 | — | — |
| Lab (speed · free units) | ×1 | ×2 | ×3 · every 5th | ×4 · every 3rd | — |
| Quarry (blocks/s · kW; the box is the panel's choice) | 2 · 10 | 4 · 20 | 6 · 30 | — | — |
| Drone port (range · drones) | 32 · 4 | 48 · 8 | 64 · 12 | 96 · 16 | 128 · 24 |
| Excavator (drones · range) | 4 · 32 | 8 · 48 | 12 · 64 | 16 · 96 | — |
| Laser link (range · efficiency) | 64 · 80 % | 128 · 88 % | 256 · 94 % | 512 · 98 % | — |
| Datacenter (TF · MW) | 100 · 2 | 250 · 4 | 600 · 8 | 1,500 · 16 | 4,000 · 32 |

Kits (a craft makes 4): **green** 2 gears, 4 screws, 2 copper wire (hand or assembler) · **blue** 1 motor,
2 steel plates, 4 screws · **violet** 2 circuits, 1 motor, 2 steel plates · **gold** 1 processor, 2
aluminium plates, 1 plastic (blue and up: assembler only).

## 2. Era 3: Industry (blue, Milestone 6, built)

Steel takes three inputs and gives a byproduct; blue science pulls parts, steel and concrete together.
What was built is in the code (`recipes/machine.rs`, `factory/process/specs.rs`, `research.rs`), DEV_PLAN
section 4 and git history. The numbers worth keeping:

- **Chain:** gear, stone brick, quicklime, motor (assembler), concrete (assembler), steel ingot (blast
  furnace: 2 iron ore + coal + quicklime, slag on the side; crushed ore only goes to smelters), steel plate and beam, blue
  pack (motor, steel plate, concrete), blue kit.
- **Machines:** assembler 2×2×2 (20 kW), blast furnace 2×2×3 (burns the recipe's coal), boiler 2×2×2,
  steam turbine 3×2×2 (up to 240 kW), silo 2×2×3 (144 slots, in on three sides, out the front), crusher
  1×1 (30 kW, a plain processor with no port rules, like the smelter).
- **Steam:** 540 kJ a coal (twice a generator), 1 water source per 2,000 kJ through pipes, up to two
  touching turbines (480 kW) a boiler; a sea pump never runs dry, a pond dries.
- **Crushing:** 2 ore → 3 crushed (2 s), so a crusher line gives 1.5 ingots an ore; 1 slag → 1 sand.
- **Techs** (append order): Mechanics, Masonry, Assembly, Steelmaking, Blue Science, Mk3 Logistics, Mk3
  Machines, Steel Tools, Steam Power (r g b, 60 × 20), Ore Crushing (60 × 20), Bulk Storage (40 × 20).

## 3. Era 4: Electronics (violet, Milestone 7, now)

Precision and depth: quartz lies 25–50 down. Blueprints and drones need circuits: the factory builds its
own builders. Flight arrives: the jetpack.

| Item | Recipe | Uses |
|---|---|---|
| Silicon | 1 quartz ore, 1 coal → 1 (arc furnace 2×2×2, 120 kW, 4 s) | circuits, processors, solar |
| Circuit | 1 silicon, 3 copper wire, 1 iron plate → 2 (assembler, 4 s) | violet pack and kit, logic, drones |
| Processor | 4 circuits, 1 silicon, 1 steel plate → 1 (assembler, 10 s) | guidance modules, gold pack and kit, hover pack, chips |
| Violet pack | 2 circuits, 1 steel beam, 1 motor → 2 (assembler, 15 s) | labs |
| Drone | 2 actuators, 1 drone cell, 1 guidance module → 1 (assembler, 30 s) | drone ports (3×3 pad) hold and fly them |
| Servo | 1 motor, 2 circuits, 2 gears → 1 (assembler, 8 s) | actuators |
| Actuator | 2 servos, 2 steel beams, 4 screws → 1 (assembler, 12 s) | drones |
| Drone cell | 1 motor, 3 circuits, 2 steel plates → 1 (assembler, 14 s) | drones |
| Guidance module | 1 processor, 2 circuits, 4 copper wire → 1 (assembler, 16 s) | drones |
| Solar panel (block) | 2 silicon, 2 glass, 2 steel plates → 1 (assembler, 8 s) | 10 kW by day |
| Accumulator (2×2×2) | 8 steel plates, 4 circuits, 12 copper wire (hand) | stores 10 MJ |
| Logic blocks | sensor (box or belt fullness), switch, lamp signal: 1 circuit, 1 plate | machines on/off by condition |
| Jetpack (tool) | 4 steel plates, 2 motors, 2 circuits (hand) | burns coal from a fuel slot: 10 s of thrust a coal |
| Personal drone | 1 drone, 2 circuits (hand) | follows you; fetches a chosen item from boxes in range |

Built (7.7b to 7.8): the drone port (24 steel plates, 12 circuits, 2 processors, 4 motors; 40/60/90/140 kW only while drones fly) with Construction Drones; Swarm Logistics (200 × 40 s, after Construction Drones) unlocks the Mk2 to Mk4 kits for it (8 kits each: green, blue, violet); the Jetpack (120 × 30 s, after Robotics) and Personal Drone (160 × 35 s, after Construction Drones) techs. The Mk5 port (128 · 24) stays for era 5. Built (7.7a): the drone ladder, all r g b v: Processors (100 × 30 s, after Violet Science) → Robotics (140 × 30, after Processors and Mk4 Machines: servo, actuator) → Drone Power (160 × 35: drone cell) and Navigation (180 × 35, after Processors and Robotics: guidance module) → Construction Drones (240 × 40, after Drone Power and Navigation: the drone). The assembler takes three different inputs per recipe, which shaped the part recipes. Built (7.1): the arc furnace, silicon and circuit; Electronics costs 80 units × 20 s (r g b), after Blue Science. Built (7.2): violet pack and kit, Mk4 for belts, miners, smelters, constructors, assemblers, blast furnaces, poles and labs (boxes, pumps and quarries stay at Mk3), techs Violet Science (100 × 25 s), Mk4 Logistics (100 × 30 s), Mk4 Machines (120 × 30 s). Built (7.5): Logic (r g b v, 80 × 25 s, after Violet Science): the sensor block (1 circuit, 1 iron plate), one block with rules instead of separate sensor, switch and lamp blocks. Built (7.4): Advanced Scanning (r g b v, 60 × 25 s, after Violet Science): the Scanner Mk2, 96 blocks (the Mk1 already lists quartz). Built (7.3): one tech, Solar Power (r g b, 80 × 20 s, after Electronics), unlocks the solar panel (6 silicon, 2 glass, 4 copper wire, 2 iron plates: 10 kW peak) and the accumulator (6 steel plates, 2 circuits, 12 copper wire: 10 MJ, 60 kW).

Techs: Electronics (r g b: arc furnace, silicon, circuit) · Violet Science · then r g b v: Logic ·
Blueprints · Construction Drones · Jetpack · Personal Drone · Mk4 Logistics
· Mk4 Machines · Processors · Advanced Scanning (scanner Mk2: range 96, shows quartz).

## 4. Era 5: Earthworks (Milestone 8)

The terraforming design (`docs/ROADMAP.md`, Milestone 8): planner, excavator (work drones are this era's
construction drones), tunnels. Techs (r g b v): Earthworks (planner, excavator), Tunnelling, Earthworks
Mk2–3 via kits. Fill materials: dirt, stone, slag, later tailings. Concrete foundations: a fill job
that places concrete as the top layer.

## 5. Era 6: Distance (Milestone 9)

Bauxite (a new ore; generator version 5 with every later resource) forms only in deserts and basalt
fields, far from spawn.

| Item | Recipe | Uses |
|---|---|---|
| Aluminium ingot | 2 crushed bauxite, 1 quicklime → 1 + 1 slag (electrolytic cell 3×2×2, 300 kW, 6 s) | plates |
| Aluminium plate | 1 ingot → 1 (constructor, 3 s) | gold kit, batteries, frames, hover pack |
| Battery | 1 aluminium plate, 1 circuit, 4 copper wire → 1 (assembler, 6 s) | hover pack, cargo drones, gold pack |
| Rail | 1 steel beam, 1 concrete → 4 (assembler) | trains |
| Hover pack (tool) | 6 aluminium plates, 2 motors, 4 batteries, 2 processors (hand) | charges near a pole; hovers, faster |

Machines and techs: Bauxite Processing · Rails · Trains (locomotive, wagon) · Stations and Signals ·
Cargo Drones (port to port, battery-limited range) · Hover Pack.

## 6. Era 7: Chemistry (gold, Milestone 10)

Fluids other than water travel as **canister items** on belts (pipes stay water-only; canisters come
back empty). Rivers and hydro, hoists, and nuclear power (for what compute will need) land here.

| Item | Recipe | Uses |
|---|---|---|
| Crude canister | pumpjack (2×2×3) on an oil reservoir (lowlands, sea floor) | refinery |
| Diesel, resin, sulfur | 2 crude, 1 water → 1 diesel canister, 1 resin, 1 sulfur (refinery 3×3×4, 6 s) | power, plastic, acid |
| Plastic | 2 resin, 1 coal → 2 (chemical plant 3×2×3, 4 s) | gold pack and kit, chips, frames |
| Acid canister | 1 sulfur, 1 water → 1 (chemical plant, 3 s) | etched circuits (4 per batch), chip fab |
| Hydrogen, oxygen canisters | 2 water → 2 H + 1 O (electrolyser, 500 kW) | rocket fuel, fuel cells |
| Washed ore | 3 crushed ore, 1 water → 4 + 1 tailings (washer 2×2×2, 3 s) | 2 ingots per ore |
| Uranium fuel cell | uranium ore → centrifuge (2×2×3) → fuel cell | reactor (3×3×3, up to 2 MW; overheating stops it) |
| Gold pack | 1 plastic, 1 battery, 1 processor → 2 (assembler, 20 s) | labs |

Techs: Oil Processing · Plastics · Sulfur and Acid · Ore Washing · Diesel Power · Electrolysis · Hydro
Power (water wheels, dams) · Hoists · Gold Science · Mk5 Machines · Nuclear Power.

## 7. Era 8: Compute (Milestone 11)

| Item or machine | Recipe | Does |
|---|---|---|
| Wafer | 2 silicon, 1 acid, ultra-pure water → 1 (chip fab 4×4×3 clean room, 1 MW) | chips |
| AI accelerator | 2 wafers, 2 processors, 1 plastic → 1 (chip fab) | datacenter racks, satellites |
| AI datacenter (4×4×3) | 40 steel beams, 20 concrete, 16 AI accelerators, 32 copper wire | compute (section 1), heat to coolant |
| Cooling tower (3×3×5) | concrete, pipes, motors | closes the coolant loop (no water lost) |
| Fibre node | 1 glass, 1 circuit, 1 plate | the data grid, like poles |
| Laser emitter, receiver, mirror | aluminium, AI accelerator, glass | power or data by line of sight |
| AI lab, optimizer node, swarm hub | processors, accelerators | research from compute; +25 % speed in range; more drones per port |

Techs: Wafers · Datacenters · Data Network · Photonics (laser links, mirrors) · AI Research (compute
research, endless bonus techs) · Optimizer · Drone Swarms · Auto-Routing · AI Survey (predicts deposits
within 256 blocks from surface hints).

## 8. Era 9: Orbit (Milestone 12)

| Item or machine | Recipe | Does |
|---|---|---|
| Light frame | 2 aluminium plates, 1 steel beam, 1 plastic | rocket bodies, satellites |
| Rocket engine, fuel tank, avionics | steel, aluminium, AI accelerators | rocket parts |
| Rocket fuel | 2 hydrogen, 1 oxygen canisters → 1 (chemical plant) | launches |
| Launch site (7×7 pad and tower) | 200 concrete, 60 steel beams, 20 motors | assembles, fuels and launches |
| Satellites | light frames, solar panels, AI accelerators, plus a payload | survey · comms · power · science |
| Rectenna (5×5), ground station (3×3 dish), landing pad (3×3) | steel, aluminium, circuits | orbital power in; space data down; drop pods in |

Constellations: coverage and uptime grow with each satellite of a kind (comms full at 12, power uptime
n / 24). Survey fills the map with every deposit; comms carries data and drone ports anywhere in
coverage (building from the map); science satellites downlink space data, the last research input.

## 9. Era 10: the megaproject (Milestone 13)

Theme is the user's to choose (DEV_PLAN section 6): an orbital ring built from shipments, a space
elevator (a tether rising from a ground anchor), or an interstellar probe. Built in phases in the world.
Completing it unlocks, never ends: asteroid mining (ore arriving by drop pod: resources beyond the
finite ground), and endless bonus techs keep compute busy.
