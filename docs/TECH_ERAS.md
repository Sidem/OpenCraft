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
| Underpass reach (blocks under; made in pairs; a pair costs as many belts of its Mk as it reaches) | 4 | 6 | 8 | 10 | — |
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
- **Steam:** 540 kJ a coal (twice a generator), 1 water source per 2,000 kJ through pipes into the boiler's
  water inlets, steam piped from its two front outlets to the turbines' inlets, up to two turbines (480 kW) a
  boiler; a Mk1 pump (2 a second) covers eight boilers; a sea pump never runs dry, a pond dries.
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

Built (gear): equipment slots (back, boots, torso, tool belt: `equipment.rs`) and three techs: Hauler Gear (40 × 10 s, r g, after Steelmaking: hauler pack, 9 backpack slots), Field Gear (60 × 20 s, r g b, after Steel Tools: spring boots jump 2 blocks, mining rig +50% hand-breaking) and Exosuit (140 × 30 s, r g b v, after Robotics and Field Gear: servo boots +15%, exo frame sprint +10%, Mk2 hauler pack 18 slots). Built (7.7b to 7.8): the drone port (24 steel plates, 12 circuits, 2 processors, 4 motors; 40/60/90/140 kW only while drones fly) with Construction Drones; Swarm Logistics (200 × 40 s, after Construction Drones) unlocks the Mk2 to Mk4 kits for it (8 kits each: green, blue, violet); the Jetpack (120 × 30 s, after Robotics) and Personal Drone (160 × 35 s, after Construction Drones) techs. The Mk5 port (128 · 24) stays for era 5. Built (7.7a): the drone ladder, all r g b v: Processors (100 × 30 s, after Violet Science) → Robotics (140 × 30, after Processors and Mk4 Machines: servo, actuator) → Drone Power (160 × 35: drone cell) and Navigation (180 × 35, after Processors and Robotics: guidance module) → Construction Drones (240 × 40, after Drone Power and Navigation: the drone). The assembler takes three different inputs per recipe, which shaped the part recipes. Built (7.1): the arc furnace, silicon and circuit; Electronics costs 80 units × 20 s (r g b), after Blue Science. Built (7.2): violet pack and kit, Mk4 for belts, miners, smelters, constructors, assemblers, blast furnaces, poles and labs (boxes, pumps and quarries stay at Mk3), techs Violet Science (100 × 25 s), Mk4 Logistics (100 × 30 s), Mk4 Machines (120 × 30 s). Built (7.5): Logic (r g b v, 80 × 25 s, after Violet Science): the sensor block (1 circuit, 1 iron plate), one block with rules instead of separate sensor, switch and lamp blocks. Built (7.4): Advanced Scanning (r g b v, 60 × 25 s, after Violet Science): the Scanner Mk2, 96 blocks (the Mk1 already lists quartz). Built (7.3): one tech, Solar Power (r g b, 80 × 20 s, after Electronics), unlocks the solar panel (6 silicon, 2 glass, 4 copper wire, 2 iron plates: 10 kW peak) and the accumulator (6 steel plates, 2 circuits, 12 copper wire: 10 MJ, 60 kW).

Techs: Electronics (r g b: arc furnace, silicon, circuit) · Violet Science · then r g b v: Logic ·
Blueprints · Construction Drones · Jetpack · Personal Drone · Mk4 Logistics
· Mk4 Machines · Processors · Advanced Scanning (scanner Mk2: range 96, ore filter, reserves and a pointer).

## 4. Era 5: Earthworks (Milestone 8)

The terraforming design (`docs/ROADMAP.md`, Milestone 8): planner, earthworks by drone ports (decided 2026-10-04: no
separate excavator), tunnels. Techs (r g b v): Earthworks (planner, excavator), Tunnelling, Earthworks
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
| Cargo drone | 1 drone, 4 aluminium plates, 2 batteries → 1 (assembler, 30 s) | built (9.5b): kept in a drone port, hauls a stack (64) to a routed port; 1 battery per 150 blocks of round trip |
| Hover pack (tool) | 6 aluminium plates, 2 motors, 4 batteries, 2 processors (hand) | built (9.5a): 90 s of charge, charges within 6 blocks of a pole; holds height, 8 blocks/s |

Machines and techs: Bauxite Processing · Rails · Trains (locomotive, wagon) · Stations and Signals ·
Cargo Drones (port to port, battery-limited range) · Hover Pack.

## 6. Era 7: Chemistry (gold, Milestone 10)

Fluids other than water travel as **canister items** on belts: pipes stay water-only, so oil never exists as a
flowing block and costs the world simulation nothing (only water flows: `sim/water.rs`, capped at 256 checks a tick,
measured worst tick 0.09 ms). A canister is a steel shell: **Empty canister** (1 steel plate → 2, constructor). Every
machine that empties one gives the empty back through its output, so a plant is a loop: empties in, full canisters
out, and the player routes both. Rivers and hydro, hoists and nuclear power (for what compute will need) land here.

**The new ground (generator version 6, `worldgen/geology.rs` `EXTRAS`):** oil sand (plains and lowlands, 200+ blocks
out) and uranium (highlands and basalt fields, 400+ out) lie deep (oil 30–65 blocks down, uranium 45–85), only as
veins and lodes, never at the surface and with no stain: they are found by scanning (the Mk2 filter and its bearing
cover them). Bauxite moved nearer (300 blocks, was 600) and caves became about 13 times rarer (cave zones). **Version 7
(built, 10.8)** is the terrain overhaul: rivers flowing downhill to the sea or a lake (a staircase of pools with weirs),
lakes, drier and wetter regions (a dry region has a third of the rivers), larger continents, taller ranges and mesas.

**Oil economics** (numbers are the plan; tune in the step):
- **Reserves:** a deposit holds `blocks × grade` units (vein 1,000 a block and about 130 blocks; lode 2,000 and about
  1,500). One **crude canister is 10 units**. A pumpjack (built, step 10.2) draws up to 5 units/s (recovery 0.9:
  0.45 canisters/s, 27 a minute) but a deposit's draw cap is shared (vein 4 units/s, so about 22 canisters a minute from
  one vein; lode 20): a vein feeds one pumpjack and lasts about 9 hours, a lode feeds four for days. It drills straight
  down its own column from the surface (up to 100 blocks), so the player places it above the deposit the scanner found. The Mk2 scanner shows units and minutes to exhaustion; a core drill gives exact figures.
- **Cost of a well:** pumpjack (steel, motors, pipes, circuits; 90 kW), empties by belt, a power line, and a way
  home (belt, train dock or cargo drone: crude is a bulky haul, so a refinery near the field and products shipped is
  usually cheaper than crude shipped). Oil beyond the nearest 500 blocks is what makes trains and cargo drones pay.
- **Return:** a refinery cycle turns 3 crude canisters into 4 useful things (below), and a diesel generator returns
  many times the energy a pumpjack and refinery use, so oil is a net energy source, never a trap.

**Refining (fractional distillation):** one recipe, **Distil** (refinery 3×3×4, 150 kW, 6 s): 3 crude canisters +
1 water → 1 **naphtha** canister (light), 1 **diesel** canister (middle), 1 **heavy oil** canister and 1 sulfur. All four
come out together, so the player must use or store each stream or the refinery stops: balancing is the game.
(Built, 10.3: shells go through, so the refinery needs no empties; naphtha leaves the front and the other three the
right-hand hatches on one belt, to sort with filters; the water inlet is on the left side.)
| Stream | Used for |
|---|---|
| Naphtha | plastic (chemical plant: 2 naphtha + 1 coal → 4 plastic); cracked from heavy oil |
| Diesel | diesel generator (built, 10.5: 2×2×2, a canister is 40 MJ: 400 kW for 100 s, the empty comes back); fuel for the later drill rig and vehicles |
| Heavy oil | **Crack** (cracker 2×2×3: 2 heavy oil + 1 water → 1 naphtha + 1 diesel, rebalances), lubricant (1 heavy oil + 1 empty → 2 lubricant: Mk5 kits and gears; boilers cannot burn it, their steam cap is under a canister's energy) |
| Sulfur | acid (1 sulfur + 1 water → 1 acid canister; ore leaching, etched circuits, the chip fab) |

**Ore washing and the crusher do not conflict:** the crusher (steel, 30 kW) stays the first step for iron, copper and
bauxite (2 ore → 3 crushed, and it grinds slag to sand); the **washer** (2×2×2, 60 kW, needs a water pipe) takes only
*crushed* ore: 3 crushed + 1 water → 4 washed ore + 1 tailings. Washed ore smelts one for one, so the chain gives raw 1.0,
crushed 1.5, washed 2.0 ingots an ore. The washer refuses raw ore, the crusher is never skipped, and **tailings must
leave by belt** (they are a new fill block for the Planner and, ground in the crusher, sand), so the step costs space,
water and a haul. It is a yield step, not a replacement.

**The research center (10.7):** the lab is one cell with four pack slots and cannot take a fifth pack, so a bigger
building joins it: the **Research Center**, 2×2×2, eight pack slots (red, green, blue, violet and gold now; three spare
for later eras), belts on every face, base speed twice a lab's, the same Mk1–Mk4 tiers. A tech that needs a pack a small lab has
no slot for is researched only in a center; the small lab stays valid for the four older packs. (Built, 10.7: a processor,
`Pick::Research`; 20/40/60/80 kW for ×2/×4/×6/×8; the small lab says "needs a research center" via `needs_center`.)

| Item | Recipe | Uses |
|---|---|---|
| Oil sand, uranium ore | found by scanning (veins and lodes); hand-mined oil sand burns as a poor fuel (a third of coal) | pumpjack, centrifuge |
| Empty canister | 1 steel plate → 2 (constructor, 1 s) | every fluid |
| Crude canister | pumpjack (1×1×1 with a tall model, 90 kW) over oil sand, an empty canister in per 10 units drawn | refinery |
| Naphtha, diesel, heavy oil canisters; sulfur | Distil (above) | plastic, power, lubricant, acid |
| Plastic | 2 naphtha + 1 coal → 4 (chemical plant 3×2×3, 4 s; the empties come back) | gold pack and kit, chips, frames |
| Acid canister | 1 sulfur + 1 water → 1 (chemical plant, 3 s) | etched circuits (4 per batch), chip fab |
| Hydrogen, oxygen canisters | 3 empties + 2 water → 2 H + 1 O canisters (electrolyser 2×2×2, 500 kW, 6 s; built, 10.5) | rocket fuel, fuel cells |
| Washed ore | 3 crushed + 1 water → 4 + 1 tailings (washer 2×2×2, 60 kW, 3 s; built, 10.6: crushed ore only, tailings are a fill block the crusher grinds to sand) | smelting |
| Uranium fuel cell | 4 uranium ore + a steel plate → centrifuge (2×2×3, 200 kW, 10 s; built, 10.10: Nuclear Power, after Steam Power and Refining) → fuel cell | reactor (3×3×3, up to 2 MW, a cell lasts 150 s at full load; water-cooled, overheating stops it until cooled; built, 10.10) |
| Gold pack, gold kit | pack: 1 plastic, 1 battery, 1 processor → 2 (assembler, 20 s); kit: 1 processor, 2 aluminium plates, 1 plastic → 4 (assembler, 5 s); built, 10.11: Gold Science (after Processors, Bauxite Processing, Plastics, Research Center) | packs: centers only; kits make Mk5 (Mk5 Machines, all five packs: miner, smelter, constructor, assembler, blast furnace, drone port, research center; belts, poles and labs stop at Mk4) |

| Hoist shaft ×2, Hoist winch | shaft: 2 steel beams + 4 rods → 2; winch: 10 steel plates, 4 beams, 4 gears, 2 motors, 2 circuits (hand; built, 10.9: Hoists, after Electronics, red + green + blue) | a shaft is a 3 blocks/s climbing frame; a powered winch (20 kW) above or beside its top cell makes it carry riders at 9 blocks/s; belt lifts still carry ore |
| Water wheel | 12 planks, 8 rods, 4 gears, 6 wire (hand; built, 10.8: Hydropower, after Steam Power, red + green) | 3×3×1; 2 kW per touching water block (flowing 4), up to 48 kW, no fuel; weirs and piped pools are the dams |

Techs: Oil Processing · Plastics · Sulfur and Acid · Ore Washing · Diesel Power · Electrolysis · Hydropower · Hoists ·
Research Center · Nuclear Power · Gold Science · Mk5 Machines (all built).

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
