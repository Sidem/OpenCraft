# The factory: resources and extraction

How deposits, machines, logistics, power and research work for the player, era by era. Controls and crafting
are in [PLAYING.md](PLAYING.md); the recipes behind each era in [TECH_ERAS.md](TECH_ERAS.md).

Ore does not sit in single blocks. It comes in **deposits**, and every ore block belongs to exactly one deposit.
A deposit holds a pool of ore units shared by all of its blocks. Look at an ore block to see its deposit's
size.

| Tier    | Where                                        | Units per block | Shared draw limit |
| ------- | -------------------------------------------- | --------------- | ----------------- |
| Outcrop | small clusters; few show, most are buried    | 100             | 60 per minute     |
| Vein    | long seams 8–70 blocks down, by ore          | 1,000           | 240 per minute    |
| Lode    | rare, huge bodies deep down (y 12–25)        | 2,000           | 1,200 per minute  |

- **By hand** you keep 3 ore per block (4 with an iron pickaxe). The rest of that block's share of the pool
  is lost. Each block broken by hand shows what you kept and lost under the crosshair, and after a few blocks a
  warning tip adds it up.
- **Tools.** Hold a pickaxe (stone and ore), an axe (wood) or a shovel (dirt, grass, sand) in the hotbar to
  break those blocks faster: stone tools twice as fast, iron tools four times. Each block broken with the
  right tool uses it up a little (stone tools last 150 blocks, iron 600); the bar under its slot shows
  what's left. Stone tools cost 3 stone (1 for a shovel) and 2 sticks; iron tools 3 iron plates (1 for a
  shovel) and 2 iron rods. Bare hands still break everything, only slower.
- **Saplings.** Leaves now and then leave a sapling behind, whether you break them or they fall off a
  felled tree. Plant it on dirt or grass and it grows into a tree after a few minutes, as long as
  nothing blocks its trunk.
- **Prospecting.** A scanner (4 iron plates, 6 copper wire, 4 screws) lists the deposits within 48
  blocks when you right-click with it: which ore, vein or outcrop, how far, which way, how deep and
  roughly how big; the arrows turn as you walk. The Advanced Scanning tech (violet science) adds the
  Scanner Mk2 (a scanner, 3 circuits and 2 steel plates), which reaches 96 blocks (the way to find
  quartz in distant highlands, deserts and basalt fields) and reads more: each deposit shows about how many
  ore units are left and how long a full-speed mine takes to work it out, **R** limits the list to one ore
  (coal, iron, copper, limestone, quartz, bauxite, oil sand, uranium, then all), and a pointer at the top of the screen keeps leading to
  the nearest match after you put the scanner away. When the ore you filtered on is not within range, it tells you
  which way to travel (a compass point and a 200-block distance band) to the nearest ground whose biome holds it,
  even over land you have never seen, and the pointer leads there; scan again as you go. A core drill (6 iron plates, 2 copper wire, 8 screws)
  held on the ground for 3 seconds tells you exactly what lies beneath: how many blocks and units are
  left, and between which heights. Neither wears out.
- **A miner** placed against any block of a deposit draws from the whole pool and delivers 60% of what it
  drills, while it has power (see Power). All miners on one deposit share its draw limit. Output slows over the last 20% of the pool.
  Each time a block's worth is used up, the ore block nearest the miner turns into spent rock, even in
  chunks that aren't loaded. A worked-out deposit is gone for good. The **coal miner** (`factory/miner_burner.rs`) is the
  same machine without a wire: it burns a stack of coal ore or logs you give it (10 kW of fire, only while it digs),
  digs 0.5 units a second and keeps 40%, and cannot be upgraded. A belt ending at it (or a router or filter facing it)
  feeds it fuel; a miner or quarry beside it does not.
- **Conveyor belts** carry items one block per second. Hold right-click on the ground and drag to lay a
  whole line: outlines show where the belts go (red past the belts you have) and releasing builds it.
  A single belt runs the way you face; press R on a belt to turn it. A belt ending in another belt's side
  merges into it. A belt fed only from one side turns the corner. Stand on a belt and it carries you
  along at its speed (you can still walk on it; crouch to stop at its end).
- **Climbing and crossing.** Belts climb and drop one-block steps by themselves: a belt with a belt one
  block ahead and one up becomes a ramp up, and one below a belt one up behind it a ramp down. Stacked lifts carry items straight up, and the top one hands them on
  one ahead and one up. Underpasses are one item in Mk1 to Mk4 (the same speed as the belt of that Mk): place two facing the same
  way and the first sends items under whatever lies between to the next one, up to 4 / 6 / 8 / 10 blocks ahead, so
  two lines can cross. Drag a belt line over a belt, a machine or a wall and it dives under by itself with the
  lowest underpass that reaches, is at least as fast as your belt and that you hold a pair of. Pairs you don't have
  show red and are left out when you build; an obstacle too wide for any underpass is marked red and ends the line.
- **Belt ejector** (the Underpasses tech; 4 iron plates, 2 iron rods and 2 belts). Put it at the end of a belt, facing the way the
  belt runs: it takes up to 4 items at a time from any side, needs no power and drops one every tenth of a second
  out of the mouth of its tube, in a small arc onto the ground (`factory/process/ejector.rs`, a `Pick::Eject` processor), so the belt backs up behind it
  instead of flooding the ground. Loose items lying still and close together merge into stacks up to the item's stack size
  (`entities/merge.rs`; also in chunks nobody has loaded; tools never merge). Dropped items vanish after 5 minutes
  unless picked up.
- **Point at an item** riding a belt to read it: its name, how long it has been on belts (an item waiting at a line's
  end shows the line is backed up), which miner, quarry, box or machine put it there and where that is, and the recipe
  if a machine made it. A splitter or filter keeps that story; a machine that takes the item in starts a new one.
  Press **I** to take that item into your pack. **Click** an item to keep it selected: the readout stays up and follows
  it (an amber outline marks it); click another item to move the selection, anywhere else to let go. Selected and
  within reach, **I** still takes it. `item_select.rs`.
- **Storage boxes** accept items from belts that end in them. They push items into belts that lead away
  from them, and miners next to a box fill it directly. Right-click a box to open it like a chest (Take all empties it); right-click a miner to take its ore.
- **A smelter** melts iron or copper ore into ingots, one every 1.5 seconds while its fire burns (and sand into
  glass, one block every 2 seconds, or a quartz ore into two). Feed it ore
  and fuel (coal ore burns 8 seconds, a log 4) by belt or from a miner next to it; it sorts them itself.
  It burns fuel only while smelting, and pushes ingots into a belt leading away. Its lamp shows green when
  working, red when out of fuel and yellow when its output is full. With the Masonry tech it also fires 2
  stone into a stone brick (a building block) and limestone into quicklime.
- **A constructor** shapes ingots into parts: iron plates (2 ingots each), iron rods, screws (4 from a rod)
  and copper wire (2 from a copper ingot). Right-click it to choose what it makes; changing the choice
  gives back the ingots it held. Belts bring the input and a belt leading away takes the parts. Parts build
  splitters and filters, and the machines still to come (better miners).
- **What a machine makes and its Mk show on it.** Every machine that makes items has a small light plate on top (its roof,
  or the top of its tallest tank or column) with a model of its product standing on it, turning slowly while the machine
  works, so it reads from every side: what you chose, or for a smelter, crusher or washer what it last made; an empty
  plate means nothing chosen yet. Every machine that comes in tiers wears one pale stud per
  Mk on its sides, tinted with the tier colour (Mk3: three studs on a blue band; poles wear rings, a belt run shows them every few blocks, an underpass on its
  hood). Products show within 48 blocks, studs within 24 (`process/display.rs`, `tier_mark.rs`).
- **The assembler** (Assembly research) is 2×2×2: the ghost shows where it goes (red where something is in
  the way) and R turns it. Belts bring parts in at the cyan hatches on its back and sides, and a belt leading
  away from the amber hatch at its front takes what it makes: motors (a rod, 2 gears, 4 wire), concrete (4
  from quicklime, 2 sand and 2 stone), red and green packs and green kits. It holds a stack of each input at
  most and draws 20 kW. Break any part of it to pick it up with what it held. Buildings you would otherwise craft by
  hand have assembler recipes too, at the hand cost: poles, long-distance poles, solar panels, accumulators, lamps,
  boxes, underpasses and rails (the constructor saws planks and sticks).
- **Decor** (Architecture research, after Masonry and Assembly; red and green packs) is machine-made only, 14 blocks for
  building something that looks good. An assembler tints 4 concrete with one pinch of a mineral into 4 coloured blocks: iron
  ore red, copper ore green, coal black, quicklime white, sulfur yellow. A smelter fires basalt, granite, sandstone or slag
  into basalt tiles, granite, sandstone and slag bricks (one block each). An assembler also makes framed glass (4 glass, 2
  rods for 4), metal panels (4 plates for 4), catwalks (a see-through grating you walk on: 4 rods, a plate for 4), railings
  (non-solid, you walk through: 4 rods for 4) and panel lights (glass, plate, wire for 2; a soft light like a torch's).
  They are not in the build menu: belt them into a box, then take them from it.
- **The upgrader** (Assembly research, 2×2×2, 30 kW; 12 iron plates, 10 gears, 16 copper wire, 2 motors) raises a
  machine's Mk without you: choose the tier it makes, and belts bring the Mk1 item and the kits of each step (Mk3 takes
  green and blue kits, Mk4 adds violet). Each tier unlocks with its upgrade's research.
- **The blast furnace** (Steelmaking research) is 2×2×3 and needs no power. Belts bring iron ore, coal and
  quicklime in at the hatches on its back and left (2 ore, a coal and a quicklime make a steel ingot in 4 s);
  steel leaves by the amber hatch at its front and slag by the violet hatch on its right. Slag has to be
  taken away (or piled: 16 wait inside) or the furnace stops and says so. Constructors press steel ingots
  into plates and beams; steel pickaxes, axes and shovels last 1,500 blocks, dig six times as fast as bare
  hands and keep 5 ore per block by hand.
- **Steam power** (Steam Power research, with blue packs) is a boiler (2×2×2) and steam turbines (3×2×2)
  joined by pipes. Each of the boiler's back, left and right faces has two inlets, one per cell: a dark **coal
  chute** for a belt (a coal makes 540 kJ of steam, twice a generator's 270) and a **blue banded pipe** for water.
  Its **two steam outlets** (red banded nozzles) are on the front, below the firebox door; pipe them to the
  **steam inlet** on the right-hand end of each turbine (the end away from the generator block). Water pipes are
  blue banded and steam pipes pale with a red band; a network that carries both (turns red) works for neither.
  A boiler takes a water unit (2,000 kJ) when its water runs low (a gauge on its front shows the water held)
  and says "Out of water" when the pump has none (a pond dries; the sea never does). One Mk1 pump (2 blocks a
  second) feeds eight boilers at full burn, so several boilers can share a pump's pipe. A turbine is wired to a
  power pole like a generator and gives up to 240 kW, only what its grid draws; a boiler feeds two turbines,
  so 480 kW. Start the first pump with a coal generator on the same grid.
- **The crusher** (Ore Crushing research, 1×1, 30 kW) turns 2 iron or copper ore into 3 crushed ore in 2 s,
  so a smelter gets 1.5 ingots an ore (crushed ore smelts 1 to 1 in 1.5 s). It also grinds slag into sand.
- **The silo** (Bulk Storage research, 2×2×3) is a big box of 144 stacks: belts bring anything in on its
  back and sides, and a belt leading away from its front takes items out. Its panel lists what it holds.
- **The arc furnace** (Electronics research, 2×2×2, 120 kW) makes silicon from a quartz ore and a coal in
  4 s: choose its recipe in its panel, belt quartz and coal in at the hatches on its back and sides, and
  silicon leaves by the front. An assembler then makes 2 circuits from 1 silicon, 3 copper wire and an iron
  plate in 4 s. Quartz lies deep (25–50 blocks down) or shows on desert and highland rock as pale, pink-white crystals.
- **Aluminium** (Bauxite Processing research, after Ore Crushing and Violet Science): a crusher grinds 2 bauxite ore
  into 3 crushed bauxite in 2 s. The electrolytic cell (3×2×2, 300 kW) turns 2 crushed bauxite and a quicklime into an
  aluminium ingot in 6 s: choose its recipe in its panel, belt both in at the hatches on its back and left, the ingot
  leaves by the front and the slag by the violet hatch on the right. A constructor presses an ingot into a plate in 3 s,
  and an assembler makes a battery from a plate, a circuit and 4 copper wire in 6 s.
- **Rails** (Rails research, after Violet Science and Earthworks): 1 steel beam and 1 concrete make 6. Rails are laid
  like power poles: each one is a node on the grid, and a smooth curve of track joins it to the node before. With
  rails in hand, click the ground to place the first node (its track runs the way you face), then click on to place
  the next: the track bends in an arc between them (a node faces so the curve leaves smoothly; hold Shift to place at
  the full 32-block reach along the way you face, crouch-click the ground to start a separate line). Click a node to
  select it, click another to join them, crouch-click a joined node to cut the track. A pair joins if it is 3 to 32
  blocks apart, climbs no more than 1 in 3 and bends no tighter than 8 blocks of radius; a node holds up to 4 tracks
  (for junctions). The outline turns red and the label says why when it does not fit. Track does not clear the
  ground: use the planner to level, cut or tunnel the route.
- **Trains** (Trains research, after Rails): a locomotive is 4 motors, 4 circuits, 10 steel plates and 4 steel beams.
  Hold it and click a rail node that has track to put it on the line; crouch-click near it to pick it up again. It
  runs at 9 blocks a second along the curves, turns round at the end of the track and takes the straightest track at
  a junction. A track with a train on it cannot be cut, and removing a node under one gives the locomotive back. No
  fuel yet, and trains pass through each other.
- **Freight** (Freight research, after Trains): a wagon (8 steel plates, 2 steel beams and a storage box) holds 24 slots;
  hold one and click a rail node beside a train to couple it on behind (up to 6; a train on its first node is pulled
  forward to make room). A loading dock and an unloading dock (2×2×1: 10 steel plates, 2 motors, 2 circuits and a
  storage box) stand within 2 blocks of a rail node. Belts feed the loading dock from any side; a train with wagons
  that reaches the node takes the items on board, an unloading dock takes the cargo and belts leading away empty it.
  A train waits at a dock until nothing has moved for 5 seconds, then drives on. Crouch-click picks up the whole
  train with its wagons and cargo.
- **Schedules**: with a locomotive in hand, click a node a train is on to select it (a freshly placed one is selected),
  then click docks to add them to its schedule (up to 8) and crouch-click a dock to clear it. The train drives to its
  docks in order, choosing branches at junctions, passes the docks that are not next and starts over after the last.
  With no schedule it stops at every dock it reaches. No fuel yet.
- **Signals** (Freight research): a rail signal (1 steel plate and 1 circuit make 2) goes on a rail node when you click it
  with the signal in hand (click again to take it back). Signals cut the track into stretches that hold one train at a
  time: a train waits at a signal while the stretch ahead has another train in it, and at a junction takes a free
  branch. A passing loop between two signals lets two trains meet and pass; a plain line with a signal in the middle
  can deadlock.
- **Solar power** (Solar Power research, after Electronics) is a solar panel (2×2×1) that gives up to 10 kW at
  noon, less towards dawn and dusk and nothing at night, and the accumulator (2×2×2) that stores 10 MJ of spare
  sun and gives it back (up to 60 kW) when the sun falls short, before any generator or turbine burns fuel.
  Wire both to a power pole like any machine. Six panels and one accumulator carry a steady 15 kW round the
  clock; the world's day is 20 minutes.
- **Splitters and filters** sit in a belt line. A splitter shares items between the belts leading away in
  front, to the left and to the right, skipping any that are full. A filter sends the item you choose in
  its panel straight on and everything else to the sides.
- **Power.** Miners, constructors, labs, pumps and quarries need power; the smelter burns its own fuel, and
  belts, splitters and filters need none. A coal generator turns coal ore (270 kJ) or logs (135 kJ), brought
  by belt, by a miner beside it or by hand, into stored energy and gives up to 60 kW, only as much as its
  grid draws, so fuel lasts longer under a light load. Power poles connect only what you wire to them, and
  each wire takes one of a pole's slots (Mk1 4, Mk2 8, Mk3 12, Mk4 16): a pole wires to poles within 10
  blocks and to generators and machines within 5 (taller poles reach further; you see the wires). **Grid colours:** every grid wears one accent colour on its wires, pole collars and cables. The biggest grid (most poles) is the main grid and is always blue; any other, separate grid is orange, pink, green, yellow or red, so a pole in a different colour is not on the main grid. Wire two grids together and both turn blue; the pole readout names its grid.
  **Long-distance pole** (Long-Distance Power research; 12 steel beams, 16 aluminium plates, 60 copper wire): a 2×2×4
  lattice tower that links to another long-distance pole up to 160 blocks away (to an ordinary pole only within 16),
  wires machines within 10 blocks and holds 6 wires. While you hold one, its outline turns green when it will link and
  red when it won't, the label says how far the pole it links to is and how much reach is left (or why it won't link), a
  light pillar marks the far tower, and the maps ring every long-distance pole's 160-block reach.
  **Placing:** with a pole in hand, right-click puts it where you aim, if that is within reach of the last
  pole (further off, at the widest spacing towards the aim); hold Shift to snap to the widest spacing along
  your view, and hold Shift and right-click while walking to lay a line. A new pole wires itself to the
  nearest powered pole in range (one with a generator on its grid), so it is live at once, and it is selected.
  A pole needs the ground under it or a wall beside it (the spot turns red in open air); with no ground under
  it, it is bolted to the wall (a plate and two struts), on the side you aimed at.
  **Wiring:** right-click a pole to select it (blue box; right-click again to deselect). Aim at a machine
  with no power and it shows a green outline and a wire: click to connect. Crouch-click moves a machine to
  the selected pole, cuts a wire, or links two poles (the label says which). A power cable (4 from an iron
  plate and 2 copper wire) needs no wiring: it joins the grid of a pole within 5 blocks and links to touching
  cables: right-click drops a column of cables to the ground, down a shaft to a miner (crouch places one), and
  a machine within 2 blocks of a cable is powered. The Logic tech (violet science) adds the **sensor** (a
  circuit and an iron plate): place it between a storage box, silo or belt and a machine that takes a power
  wire, facing the machine (R turns it). It reads how full the thing behind it is and cuts the machine's
  wire by a rule; right-click steps through the rules (always on, always off, run while under half full, run
  until nearly full, run while over half full, run while it holds anything). Its lamp shows green when on.
  Drones are the long road of violet science: Processors, Robotics (after Mk4 Machines), Drone Power and Navigation,
  then Construction Drones, each costing all four packs. They unlock assembler recipes for the parts (processor, servo,
  actuator, drone cell, guidance module) and the drone itself (2 actuators, a cell and a guidance module, 30 s), and
  Construction Drones also unlocks the **drone port** (24 steel plates, 12 circuits, 2 processors, 4 motors): a 3×3
  pad on power (40 kW while drones fly) that keeps 4 drones. Put drones in by hand or belt and set storage boxes
  touching the pad with what your ghosts need: every half second a drone flies out to the nearest ghost within 32
  blocks, takes the item from a box, builds it and comes home, or breaks a block you marked (B, left-click) and
  puts what it leaves into the boxes. Swarm Logistics unlocks kits for Mk2 to Mk4 ports (8, 12, 16 drones; reach 48, 64, 96).
  Breaking a port gives its drones back. The Cargo Drones tech (after Construction Drones and Bauxite Processing)
  lets assemblers build **cargo drones** (a drone, 4 aluminium plates, 2 batteries): put them into a port (a port keeps
  construction drones or cargo drones, not both), then hold one and click a port and then another to set a route
  (crouch-click clears it; the outline is red when the second port is too far for the first one's tier: 200, 400, 800
  or 1,600 blocks). Powered, a port with a route sends a drone every 2 seconds that finds a stack in the boxes touching
  its pad (batteries and drones are never cargo), flies it up to 64 items to the other port, unloads into the boxes
  beside that pad and flies home. Each trip burns batteries from the first port's boxes, one for every 150 blocks flown
  there and back, so keep a belt of batteries into them. The Jetpack tech (4 steel plates, 2 motors, 2 circuits) and the Personal Drone
  tech (a drone and 2 circuits) make the two helpers for your pack: see the keys above. The Hover Pack tech (after the
  Jetpack and Bauxite Processing; 6 aluminium plates, 2 motors, 4 batteries, 2 processors) makes a pack that hovers:
  hold jump in the air and you stay at your height (jump rises, crouch sinks) and glide at 8 blocks a second, 13 when
  sprinting. It runs on charge, 90 seconds of hover at most, and refills (twice as fast as it drains) while you stand
  within 6 blocks of a power pole (32 of a long-distance pole); the line under the hotbar shows the charge. A hover pack wins over the jetpack. The Earthworks tech (needs
  Construction Drones) unlocks the **Planner** (3 iron plates, 4 copper wire, 2 glass, a circuit): mark an area to
  dig, fill or flatten and drone ports in reach do the work, digging into the storage boxes beside their pad and
  filling from them (dirt on top, else stone, dirt, sand or grass), so ground dug in one site fills another.
  The same planner bores **tunnels**: pick Tunnel in the panel, a section (1 × 2, 3 × 3 or 5 × 5) and the two blocks
  (the first is the middle of the floor; the tunnel runs along the longer side, up to 96 blocks, rising or falling
  1 block in 2 at most). Drones leave any block next to water until the water is gone, so a tunnel never floods.
  A drilling Miner Mk1
  draws 5 kW (one coal runs it long enough to mine about 32), a Mk2 20 kW, a working constructor 15 kW.
  Short of power, every machine on the grid slows down to match; with none, it stops. The first loop: a
  miner on coal next to a generator, a pole within reach of both, wired to each, and a log to start the
  fire; from then on the miner fuels its own power. Worlds saved before wiring by hand are wired once the
  old way, by range.
- **Research.** Splitters, filters, lifts, underpasses and green science packs start locked (greyed
  out in the build menu). A research lab uses science packs to unlock them: press T for the tech
  tree (done techs are green, the ones you can start glow blue, the one being researched is orange; hover any
  tech for its details), click one to choose it, and every powered lab with the right packs works on it, one unit at a time (5 or 10 s each, one of each of
  the tech's packs). Red packs are an iron plate and 2 copper wire; green packs, unlocked by research, are
  2 belts and 4 screws. Belt Routing (10 red) unlocks splitters and filters, then Belt Lifts (20 red),
  Green Science (30 red), and with red and green packs Underpasses (15), Mechanics (30: gears, green kits
  and Mk2 miners, smelters and constructors), Belt Mk2 (20) Assembly (40: the assembler, motors, concrete) Steelmaking (50, after Assembly and Masonry: the blast furnace, steel) and Blue Science (50: blue packs, from a motor, a steel plate and concrete in an assembler). With blue packs too: Mk3 Logistics (60: blue kits, Mk3 belts), Mk3 Machines (80) and Steel Tools (30), and after Blue Science Steam Power (60, also after Fluid Handling), Ore Crushing (60) and Bulk Storage (40). Fluid Handling (15 red, after Belt
  Routing) unlocks pumps, pipes and outlets; Masonry (15 red) stone bricks and quicklime. A lab draws 10 kW
  while it works.
- **Pumps and pipes.** A pump lifts 2 blocks of still water a second out of the water it touches, the
  highest first (5 kW). Pipes join on every side; an outlet at the other end pours the water out in front
  of it (it faces the way you did when you placed it): down to where it lands, then filling that pool from
  the bottom up. With nowhere to pour, the pumps stop. Drain a pond or a flooded pit this way; the sea
  refills whatever you take, so it can't be lowered.
- **Fluids and chemistry** (Milestone 10). Pipes carry only water; every other fluid rides belts in **canisters**
  (an empty one is 1 steel plate for 2, and empties come back). Oil Processing unlocks the **pumpjack** (90 kW):
  placed over oil sand it drills straight down its own column and fills a canister about every 2 s from the deposit's
  shared pool. Refining unlocks the **refinery** (3 crude canisters and water give naphtha, diesel and heavy oil
  canisters and sulfur: the first leaves the front, the rest the right-hand hatches, so sort the belt with filters) and the
  **cracker** (heavy oil and water into naphtha and diesel). Plastics, then Acids and Lubricants, unlock the
  **chemical plant** (plastic from naphtha and coal, acid from sulfur and water, lubricant from heavy oil). A
  **diesel generator** burns a canister for 100 s of 400 kW and gives only what the grid lacks; the
  **electrolyser** splits water into hydrogen and oxygen for later. **Ore washing** (the washer takes *crushed* ore
  only, water from a pipe) turns a raw ore's 1.0 ingots into 2.0 and leaves tailings you must belt away. A **research
  center** (2×2×2, eight pack slots, twice a lab's speed and power) takes the gold pack that a lab has no slot for.
  The **water wheel** (rivers and lakes exist in worlds made from this version on) gives 2 kW per still water block
  and 4 per flowing one it touches, up to 48. A **hoist** (a steel shaft with a powered winch on top) lifts riders at
  9 blocks a second. **Nuclear power**: the centrifuge makes fuel cells from uranium ore and a steel plate; the
  **reactor** (3×3×3, fuel cells at the back, up to 2 MW) heats as it gives, a water pipe to its blue inlet keeps it
  cool, and it shuts down when overheated until it has cooled. Gold science and Mk5 Machines close the milestone.
- **Recycling.** After Blue Science, the **recycler** (2×2×2, 90 kW, six input hatches, two output hatches) destroys
  any item belts bring it and pays **recycling coins** (1024 to a stack; one day they buy cosmetics). Every item pays a whole number of coins, from 1
  for raw things and basic parts (80 of the 227 items) up to 64 for the dearest item, the datacenter. The price follows the
  recipes: the material that went in and how deep the chain is, on a flattening curve, so a motor pays 4, an AI accelerator
  11, a drone 27 and a Mk5 drone port 55. A tool pays by its uses, so a worn one pays less. Slag and tailings pay the least,
  and so does what is crushed from them. An item is worth what its cheapest recipe makes it, so no route earns more.- **Chips.** The Wafers tech (gold packs, so in a research center) opens the **chip fab** (4×4×3 clean room, 1 MW):
  2 silicon, an acid canister and a pure water canister make a **wafer** in 8 seconds and the two shells come back out of
  the side hatches; belt wafers back in with 2 processors and a plastic and the Accelerators tech lets it build an **AI
  accelerator** (20 s). **Pure water** is a chemical plant recipe (an empty canister, a sand and 2 units of piped
  water). A half-fed fab says what it still lacks.
- **Data grid.** The Data Network tech (after Wafers) gives **fibre nodes** (2 from 2 processors, 4 glass, 2 plastic and 6 copper
  wire). Nodes within 12 blocks join into one data grid by themselves, and a machine that makes or uses compute (TF) hangs on
  the nearest node within 5 blocks; thin light lines show what is joined, and a node's readout gives the grid's supply and
  demand. A grid shares its compute among its consumers, so a shortage slows them all alike, and a consumer with no node in
  reach does not run.
- **Datacenters.** The Datacenters tech (after Accelerators, Data Network and Diesel Power) gives the **AI datacenter**: a 4×4×3
  hall (40 beams, 20 concrete, 16 accelerators, 32 copper wire) that draws 3 MW all the time and gives its data grid 100 TF at
  full power. It runs hot: pipe water from a pump to its blue inlet on the left (a unit per 10 seconds), or it shuts down after
  about 20 seconds, drawing and giving nothing, until it has cooled.
- **Cooling towers.** The Cooling tech gives the **cooling tower** (30 concrete, 12 pipes, 4 motors, 12 steel plates; 300 kW, 3×3×5).
  Pipe its blue inlet to the same pipes as a datacenter's: the datacenter then takes its coolant through the tower ("cooling loop
  closed") and only one unit in twenty is lost, so a single pump feeds it easily. One tower cools two datacenters; with no power
  or water at the tower the datacenter falls back to the pump.
- **AI labs.** The AI Labs tech gives the **AI lab** (20 steel plates, 6 accelerators, 10 processors, 20 circuits, 8 glass; 2×2×2,
  100 kW): a research center on the data grid. It researches at twice a lab's speed, takes every pack, makes every 2nd unit free
  of packs and uses 20 TF while it works, so it idles without a datacenter on its grid. Techs that cost compute (AI Research is
  the first) can be researched only in it; labs and centers say so.
- **Optimizer.** The Optimizer tech gives the **optimizer node** (8 steel plates, 4 accelerators, 10 processors, 10 circuits, 4
  glass; 2×2×2, 150 kW). It uses 20 TF of the data grid it hangs on, and every machine and miner within 16 blocks of it works
  25% faster, whatever grid it is on. Machines take the best optimizer in range, they do not add up, and a shortage of power or
  compute weakens it (it says so when you point at it).
- **Laser power links.** The Photonics tech gives the **laser emitter** and **laser receiver** (each 6 aluminium plates, 1
  accelerator, 4 glass, 4 circuits). Put both on poles and aim the emitter (R turns it) along a straight, clear line of up to
  128 blocks at the receiver, which takes the beam from any side: the two power grids become one, and the far side burns a
  tenth more power on the way. Air and glass let the beam through, any other block stops it (it flickers up to the block);
  break that block and the link returns.
- **Mirrors and data beams.** Photonics also gives the **laser mirror** (4 aluminium plates, 6 glass) and the **data receiver**.
  R turns a mirror a quarter turn; a beam bends 90° at each (up to 8, 128 blocks in all). A beam that ends in a data receiver
  joins the data grids at its two ends instead of the power grids.
- **Swarm hub.** Drone Swarms (after AI Research and Swarm Logistics) gives the **swarm hub** (2×2×2, 100 kW, 30 TF of the data
  grid): every drone port within 12 blocks keeps 50% more drones (a Mk4 port 24 instead of 16). Ports take the best hub in range,
  hubs do not add up, and without power or compute the extra drones are not replaced.
- **Auto-routing.** The Auto-Routing tech makes a belt line dragged onto a machine find its own way there, round walls and over
  steps (it says "No route" and marks the target red when it cannot). In ghost mode (B) a dragged belt line is planted as ghosts
  for your drones to build.
- **AI survey.** The AI Survey tech marks, on the minimap and the world map, faint rings where ore probably lies, guessed from the
  stained soil you have seen within 256 blocks (bigger rings for more stain). They are hints: scan or drill to be sure.
- **Rocketry.** Rocketry (after Electrolysis and Accelerators, all five science packs) gives four rocket parts the assembler
  builds (light frame, rocket engine, fuel tank, avionics), **rocket fuel** (the chemical plant mixes 2 hydrogen and 1 oxygen
  canister into a fuel canister and 2 empties), the **survey satellite** (an assembler: 2 light frames, 4 solar panels, 4 AI
  accelerators) and the **launch site**, a 7×7×6 pad and gantry (2 MW). Belt 4 light frames, 2 engines, 2 fuel tanks, an
  avionics, 8 fuel canisters and a satellite in at the back hatches: the pad builds a rocket for a minute (you see it standing
  there), launches the satellite into orbit and gives the 8 empty canisters back at the front. A pad missing a part says
  which, and says how many satellites are up and how many are over it. Satellites fly across the sky at 6 blocks a second, a
  lap of the world's repeating sky in a few minutes, and show as small gold lights (brightest at night); a satellite covers
  the ground within 768 blocks of the point under it. The first 24 of a kind stay up; a 25th retires the oldest.
- **What satellites give.** Every kind works only where its circle is (768 blocks for survey, 1,024 for the others) and
  flies on, so a dozen of a kind give steady cover. Each kind has its own tech after Rocketry, its own assembler recipe and
  its own rocket (the same parts, with that satellite as the last input). **Survey:** the veins and lodes under its circle
  go on the minimap and the world map as known ore (for this session), and the maps draw its footprint as a gold disc.
  **Comms** (Orbital Comms, after Data Network; 2 light frames, 8 processors, 8 aluminium plates): every fibre node under
  its circle is on one data grid, and a drone port under it builds 4× as far and sends cargo drones twice as far.
  **Power** (Orbital Power, after Nuclear Power; 4 light frames, 24 solar panels, 16 circuits): the **rectenna** (5×5,
  20 aluminium plates, 60 copper wire, 20 circuits, 20 concrete; hang it on a pole) gets 200 kW from each power satellite
  over it, up to 1 MW, day and night. **Science** (Orbital Science, after Data Network; 2 light frames, 8 AI accelerators,
  8 processors): the **ground station** (3×3×2, 150 kW, a fibre node in reach) puts 40 TF on its data grid for each
  science satellite over it, up to 100 TF.
- **Drop pods.** The Drop Pods tech (after Rocketry and Orbital Comms) gives the **pod launcher** (3×3×3, 500 kW; 60
  concrete, 20 steel beams, 10 motors, 20 circuits), the **landing pad** (3×3×1, no power) and the **drop pod** (an
  assembler: a light frame, a fuel tank, 10 circuits). Belts fill a launcher with cargo (12 stacks) and drop pods. Right-click
  a launcher or pad to tune it to a channel, one of six colours (blue until you change it); the way it faces does not matter.
  While a comms satellite is
  over the launcher and over a pad on its channel, the launcher loads for 15 seconds and sends one pod with up to 6 stacks
  to the nearest such pad; it comes down 20 seconds later and the cargo goes into the pad's 24 stacks, which belts empty.
  A full pad keeps the pod waiting and lands what fits, so nothing is lost. The launcher's status says what it waits for.
  On the maps, launchers (a square with an arrow) and pads (a square) are drawn in their channel's colour, and each has a
  mark in that colour on its front, so matching ones are easy to spot.
- **Endless bonus techs.** AI Research opens **Mining Productivity** (miners draw 3% faster per level), **Machine Speed**
  (smelters, assemblers and every other processing machine work 3% faster) and **Drone Speed** (drones fly 3% faster). They use
  no packs, only an AI lab's time and compute; each level takes a quarter longer than the last, and you can keep going (the
  screen shows the level, not a bar). Levels add up (10 levels: +30%), not compound.
- **Quarries.** A quarry digs the ground in front of it for real: stone, rock, soil and sand, about 2 blocks
  a second at full power (10 kW), layer by layer from its own level down. While you hold one, its box shows
  in amber with how many blocks it would yield; R turns it. Its panel picks the size (5 to 11 wide) and the
  depth (8 or 16 layers, to sea level, to bedrock), pauses it, and lists the ore veins the pit uncovered,
  which it leaves standing for miners. Its output feeds belts like a miner's. If the pit breaks into
  water, it waits until you pump the water out.
