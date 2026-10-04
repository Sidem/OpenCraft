# OpenCraft

A browser game in the space between **Minecraft** and **Satisfactory**: a voxel world you mine and build in,
growing towards resource extraction, automation and factories.

**Play it: <https://sidem.github.io/OpenCraft/>** (desktop browser with keyboard and mouse).

The simulation core is **Rust compiled to WebAssembly** (with SIMD). Rendering is **raw WebGL2**. There is no
game engine and no runtime dependencies. Every texture and sound effect is generated procedurally at startup,
so there are no asset files, and the whole production build is about 250 KB gzipped.

## Quick start

Prerequisites:

- Rust (stable) with the `wasm32-unknown-unknown` target. `rust-toolchain.toml` installs the target automatically.
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) (`cargo install wasm-pack`)
- Node.js 20.19 or newer

```bash
npm install
npm run dev
```

Open <http://localhost:5173> and click **Click to play**. `npm run dev` builds the engine, starts Vite and
watches `crates/`. Any Rust change triggers a wasm rebuild and a page reload. TypeScript changes hot-reload
as usual.

| Command             | What it does                                                |
| ------------------- | ----------------------------------------------------------- |
| `npm run dev`       | Dev server with Rust watch + rebuild (release wasm)         |
| `npm run build`     | Production build into `dist/` (wasm + typecheck + bundle)   |
| `npm run preview`   | Serve the production build                                  |
| `npm test`          | Engine unit tests (`cargo test`)                            |
| `npm run typecheck` | TypeScript check                                            |
| `npm run check`     | Pre-commit checks: format, lints, tests, types, sizes       |

URL parameters: `?seed=1234` starts a new world with that seed (saved like any other), and `?rd=12` sets
the render distance in chunks (2 to 24, default 8).

## Controls

| Input               | Action                                    |
| ------------------- | ----------------------------------------- |
| WASD                | Move                                      |
| Space               | Jump; in water, hold to swim up (at a bank, it climbs you out) |
| Shift               | Sprint                                    |
| C or Ctrl           | Crouch (you won't walk off edges)         |
| Hold left mouse     | Mine the targeted block                   |
| Right mouse (hold)  | Place the selected block                  |
| Right mouse drag with belts | Lay a line of belts: press on the ground, drag to where it should end (it turns once and climbs or drops one-block steps by itself), release to build. Left-click cancels |
| R                   | Turn the belt or splitter/filter you point at; while holding a quarry or an assembler, turn it before placing |
| B                   | Ghost mode: right-click plants a free ghost of the held block or machine (again on it removes it), left-click marks the aimed block for tear-down (red outline; again clears it), R turns it, reach 16, no mining; a ghost's outline is cyan and the label lists what the ghosts need. Placing the real block on a ghost builds it facing the same way |
| Z, Enter (ghost mode) | Z marks the two corners of a box (a third press clears it); Enter copies the machines in it as a blueprint and takes it in hand. With one in hand, right-click stamps its ghosts, R turns it and Z puts it away |
| L | Blueprint library: rename, hold or delete your blueprints (kept per world in this browser) |
| Right-click (Planner in hand) | Marks a site: a corner block, then the opposite one (reach 64); a panel picks dig, fill, flatten (with the level) or a tunnel (with its section). Right-click a marked site to look at it or remove it |
| Y | With a Personal Drone in your pack: it fetches more of the item in your hand from the nearest storage box within 32 blocks |
| Jump (in the air) | With a Coal Jetpack in your pack: keep holding it to climb on a plume of fire, burning a coal from your pack every 10 seconds |
| Shift-click gear (inventory) | Wears it: back (hauler pack, 9 or 18 more backpack slots), boots (spring: jump 2 blocks; servo: walk 15% faster), torso (exo frame: sprint 10% faster), tool belt (mining rig: break blocks by hand 50% faster). Researched under Hauler Gear, Field Gear and Exosuit |
| Right mouse on a box | Open it like a chest: click moves stacks with the cursor, Shift-click moves a whole stack between box and inventory. Hold C to place against it instead |
| Right mouse on a miner | Take everything it holds |
| Right mouse on a smelter, constructor, assembler, filter, generator, lab or quarry | Open its panel. Hold C to place against it instead |
| E                   | Inventory and build menu                  |
| T                   | Research: choose what labs work on        |
| H                   | Skip the tip on screen (the menu can show tips again) |
| M                   | The map: everywhere you've been, your pins, and the ore guide (drag to move, wheel to zoom, click to pin) |
| N                   | Show or hide the minimap (top right, north up; diamonds mark ore you've seen, rings veins and lodes you've prospected, squares your machines, pins your pins) |
| 1–9 / mouse wheel   | Select hotbar slot                        |
| Q                   | Drop one item                             |
| F                   | Toggle fly mode (Space / C: up / down)    |
| Hold Tab            | Who is playing with you (in a shared game) |
| K                   | Mute / unmute sound                       |
| O                   | Open the sound designer                   |
| F3                  | Debug stats                               |
| Esc                 | Release the mouse; in a panel, close it   |

Mined blocks drop as items, which are pulled into your inventory when you get close. In the inventory screen
(E), click a slot to pick up a stack and click again to put it down. Shift-click moves a stack between the
hotbar and the backpack. **Sort** (beside "Backpack", and beside a box's slots when you have one open)
merges partial stacks and orders everything by item; the hotbar keeps the layout you made, and tools never
merge, so each keeps its own wear.

New worlds have a sea and ponds. Water flows: dig beside the sea and it pours in and refills any hole or
trench at or below its level, however far. Elsewhere water runs up to 7 blocks from its source, getting
shallower, and falls over edges; wall it off and the flow dries up. Blocks and machines can be placed
straight into water. In water you swim (Space rises), and dropped items float.

## Saving and worlds

Your world saves itself: every minute, whenever you pause (Esc), and when you switch tabs or close the page.
Opening the game again brings back the world you played last. Press **Continue** and everything is where
you left it, machines still running.

The **Worlds** list in the pause menu manages several worlds:

- **New world** takes a name and an optional seed. A number gives that exact world, any other text works
  as a seed too, and leaving it blank picks one at random.
- **Play** switches to another world (the current one is saved first).
- **Export** downloads a world as an `.ocworld` file, and **Import** adds one, so you can keep a backup or
  move a world to another browser.
- **Delete** removes a world for good, after asking.

New worlds have **biomes**: plains and forest around where you start, sandy deserts over sandstone,
highlands over granite, wet lowlands thick with trees, and bare black basalt fields. The rock under
each biome decides which ores lie in it: coal and limestone in the lowlands, copper and quartz in the
highlands, quartz and limestone in the deserts, rich iron in the basalt, a bit of everything on the plains.
Deep veins and lodes give themselves away at the surface: rusty soil above iron, dark soil above coal,
blue-green soil above copper and pale soil above limestone and quartz. Point at a stained block and it
tells you what lies below and how many blocks down. Ore rarely reaches the surface:
look for it where the rock is bare, on cliffs, bare mountain tops, deserts and basalt fields. Each ore keeps to
its own depth below the ground: limestone and coal lie shallow, iron a little deeper, copper deeper still,
quartz deepest. Every new world has two small patches each of coal, iron and copper showing 28 to 110
blocks from where you start. Worlds made before this keep the terrain and ore they were made with (the
oldest ones, before biomes, are "classic terrain"), so nothing you built there changes.

**The map** (M) shows everywhere you have been, north up; the browser remembers it with the world.
Ore you have seen at the surface is marked with a diamond, so a patch spotted in passing is easy to find
again. Click the map to drop a **pin** (an ore, home or a note, with a few words); pins show on the
minimap too, at its edge when they are far away. Beside the map, **Finding ore** charts how deep each ore
lies, which biomes hold it and how to spot it, with this world's own numbers.

A day lasts 20 minutes, from sunrise through a golden dusk to a dark blue night with stars; the pause menu
shows the day and the time. Light is simulated per block: forest floors lie in shade, caves are dark even
at noon, and a shaft you dig lets the sky in. **Torches** (a stick and a coal ore make 4)
stand on top of any block and light about 11 blocks around them, full light within 4; they drop when the block
under them goes. **Lamps** (glass, an iron plate and 2 copper wire make two) are far stronger: full light within
17 blocks, fading out about 31 blocks away, in a warm glow, day and night. Machines, belts, the items on
them and dropped items are lit by the same light as the ground they stand on: a lamp lights a factory at
night, and a machine under a roof or in a cave is dark.

Worlds are stored in this browser only. Clearing the site's data deletes them, so export any world you
want to keep. The game also keeps each world's previous save as a backup and uses it automatically if the
latest save can't be read.

## Playing together

Up to four players can build in one world. **Play together** in the pause menu:

- Type your name; others see it above your head.
- **Host this world** opens the world you're playing to others and shows a code and a link. Send the link
  (or the code) to your friends. Your browser runs the world and keeps it saved, so it stays open only
  while your tab does.
- To join, paste a friend's link or code into the box and press **Join**. Your own world is saved first.
  Friends who come back later get their things back where they left them.
- Hold **Tab** in game to see who is playing and their ping. **Leave** (or closing the tab) takes you out;
  when the host leaves, the game ends for everyone and **Back to my worlds** returns to yours. A player
  whose connection drops is removed after 10 seconds of silence (their things wait for them), and a saved
  world opens without the players who were connected when it was saved.

Players connect directly (WebRTC). A small Cloudflare Worker (`signal/`) only introduces them; no game data
passes through it. Some networks (strict company or mobile networks) block direct connections, and the
game then says it couldn't connect.

## Resources and extraction

Ore does not sit in single blocks. It comes in **deposits**, and every ore block belongs to exactly one deposit.
A deposit holds a pool of ore units shared by all of its blocks. Look at an ore block to see its deposit's
size.

| Tier    | Where                                        | Units per block | Shared draw limit |
| ------- | -------------------------------------------- | --------------- | ----------------- |
| Outcrop | small clusters; few show, most are buried    | 100             | 60 per minute     |
| Vein    | long seams 8–70 blocks down, by ore          | 1,000           | 240 per minute    |
| Lode    | rare, huge bodies deep down (y 12–25)        | 2,000           | 1,200 per minute  |

- **By hand** you keep 3 ore per block (4 with an iron pickaxe). The rest of that block's share of the pool
  is lost.
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
  (coal, iron, copper, limestone, quartz, then all), and a pointer at the top of the screen keeps leading to
  the nearest match after you put the scanner away. A core drill (6 iron plates, 2 copper wire, 8 screws)
  held on the ground for 3 seconds tells you exactly what lies beneath: how many blocks and units are
  left, and between which heights. Neither wears out.
- **A miner** placed against any block of a deposit draws from the whole pool and delivers 60% of what it
  drills, while it has power (see Power). All miners on one deposit share its draw limit. Output slows over the last 20% of the pool.
  Each time a block's worth is used up, the ore block nearest the miner turns into spent rock, even in
  chunks that aren't loaded. A worked-out deposit is gone for good.
- **Conveyor belts** carry items one block per second. Hold right-click on the ground and drag to lay a
  whole line: outlines show where the belts go (red past the belts you have) and releasing builds it.
  A single belt runs the way you face; press R on a belt to turn it. A belt ending in another belt's side
  merges into it. A belt fed only from one side turns the corner. Stand on a belt and it carries you
  along at its speed (you can still walk on it; crouch to stop at its end).
- **Climbing and crossing.** Belts climb and drop one-block steps by themselves: a belt with a belt one
  block ahead and one up becomes a ramp up, and one below a belt one up behind it a ramp down. Stacked lifts carry items straight up, and the top one hands them on
  one ahead and one up. An underpass entry sends items under whatever is in front of it to the nearest
  exit facing the same way, up to 5 blocks ahead, so two lines can cross.
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
- **The assembler** (Assembly research) is 2×2×2: the ghost shows where it goes (red where something is in
  the way) and R turns it. Belts bring parts in at the cyan hatches on its back and sides, and a belt leading
  away from the amber hatch at its front takes what it makes: motors (a rod, 2 gears, 4 wire), concrete (4
  from quicklime, 2 sand and 2 stone), red and green packs and green kits. It holds a stack of each input at
  most and draws 20 kW. Break any part of it to pick it up with what it held.
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
  blocks and to generators and machines within 5 (taller poles reach further; you see the wires).
  **Placing:** with a pole in hand, right-click puts it where you aim, if that is within reach of the last
  pole (further off, at the widest spacing towards the aim); hold Shift to snap to the widest spacing along
  your view, and hold Shift and right-click while walking to lay a line. A new pole wires itself to the
  nearest powered pole in range (one with a generator on its grid), so it is live at once, and it is selected.
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
  Breaking a port gives its drones back. The Jetpack tech (4 steel plates, 2 motors, 2 circuits) and the Personal Drone
  tech (a drone and 2 circuits) make the two helpers for your pack: see the keys above. The Earthworks tech (needs
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
- **Quarries.** A quarry digs the ground in front of it for real: stone, rock, soil and sand, about 2 blocks
  a second at full power (10 kW), layer by layer from its own level down. While you hold one, its box shows
  in amber with how many blocks it would yield; R turns it. Its panel picks the size (5 to 11 wide) and the
  depth (8 or 16 layers, to sea level, to bedrock), pauses it, and lists the ore veins the pit uncovered,
  which it leaves standing for miners. Its output feeds belts like a miner's. If the pit breaks into
  water, it waits until you pump the water out.
- **Upgrades.** Machines come in tiers, shown by the coloured stripe on them (red Mk1, green Mk2, with one
  dot per Mk). Green kits (2 gears, 4 screws and 2 copper wire make 4) upgrade a placed machine in place,
  keeping everything it holds: hold kits and right-click a miner, smelter or constructor (4 kits), or hold
  the button and drag along a belt line (1 kit a belt). A Miner Mk2 draws twice as fast (where the deposit
  allows) and keeps 75% of what it draws instead of 60% (20 kW); a Smelter Mk2 works twice as fast on a
  quarter less fuel an ingot; a Constructor Mk2 twice as fast at 30 kW; a Belt Mk2 carries items at 2
  blocks a second and mixes freely with Mk1 belts. The Mk2 items are also crafted: the Mk1 plus its kits.
  Assemblers and blast furnaces upgrade too, at 8 kits a step. Blue kits (a motor, 2 steel plates and 4 screws
  make 4, in an assembler) take Mk2 to Mk3 once Mk3 Logistics (belts) or Mk3 Machines is done: a Belt Mk3
  moves 4 blocks a second (about 10 items), a Miner Mk3 draws four times as fast and keeps 85% (45 kW), and
  smelters, constructors, assemblers and blast furnaces work three times as fast; the Smelter Mk3 is
  electric (40 kW) and burns no fuel. Power poles, storage boxes, pumps, quarries, labs and the coal generator
  upgrade too: a Mk3 pole (a pylon) links 32 blocks and reaches 9, boxes hold 24, 36 and 48 stacks, pumps and
  quarries do 2, 4 and 6 a second, a Mk3 lab works three times as fast and skips the packs of every fifth unit,
  and a Mk2 generator gives 100 kW and a quarter more from the same fuel. Violet kits (2 circuits, a motor and
  2 steel plates make 4, in an assembler; violet science packs need circuits too) take Mk3 to Mk4 once Mk4
  Logistics or Mk4 Machines is done: a Belt Mk4 moves 8 blocks a second (about 20 items), a Miner Mk4 draws six
  times as fast and keeps 92% (90 kW), smelters, constructors, assemblers and blast furnaces work five times as
  fast (a smelter 80 kW), a Mk4 pole (a substation) links 32 blocks and reaches 16, and a Mk4 lab works four
  times as fast and skips the packs of every third unit. Boxes, pumps and quarries stop at Mk3.
- **Machine panels.** Right-click a smelter, constructor, assembler, arc furnace, boiler, crusher, silo, filter, generator, lab or quarry to see what it's doing, put items in
  straight from your inventory (ore, fuel, ingots, packs), and take what it made.

Craft in the build menu (E). It groups recipes by kind; type in its search box to find a recipe
or everything made from a material, and filter by what you can craft now, what misses materials and what
research still locks. Hover a recipe for its description, materials and how long it takes; click it to queue one,
Shift-click for up to five.

**Crafting takes time and queues.** Each hand craft runs for a few seconds (90 ticks plus 30 per material, at most
20 s; a plate 2.5 s, a miner 9 s), one after another, shown above the hotbar; click ✕ to cancel that
order and get everything back, including the parts it had already made. The materials are paid when you queue.
If you hold the raw materials for something that needs parts, asking for it queues the parts too, in the right
order: a Miner Mk1 from 14 iron ingots and 3 copper ingots queues 5 plates, 4 rods and 3 wire crafts and then the
miner. The queue shows each step in that order, parts first and what you asked for last (a storage box from 2 logs and 4 iron ingots: planks, plates, then the box); a part you already hold is used, not made again. Only what the inventory can pay for is queued. The time each craft takes is in `crates/engine/src/recipes/timing.rs`. Leaving a co-op world returns the queue to your inventory.

**Bootstrapping, without raw ore in a recipe.** Nothing is built from raw ore. Wood comes first (Materials): a log
saws into 4 planks (a building block that also burns briefly), and 2 planks make 4 sticks, for tools, torches, poles
and **ladders** (4 sticks make 3). A ladder is a see-through frame you climb: stand in it and hold jump to go up,
crouch to go down, let go to stay put. Belt lifts climb the same way. Then: 16 stone make a **stone furnace** (the
Smelter), which you feed by hand (right-click it) with iron or copper ore and coal ore or logs; it melts ingots and you
take them from its panel. From ingots you craft, by hand and slowly, iron plates (2 ingots), iron rods (1),
screws (a rod makes 4) and copper wire (an ingot makes 2). Those make the first machines: a Miner Mk1 costs 5 plates,
4 rods and 6 wire, four belts a plate and a rod, a box 8 planks and 2 plates, a generator 4 plates, 8 wire and 12
stone, two poles 2 sticks and 2 wire. Once a constructor (10 iron ingots, 4 copper ingots, 8 stone) makes the
parts for you, hand crafting is for one-offs. Also by recipe: a constructor is as above; a splitter 2 iron plates and
2 belts, a filter 2 iron plates, 2 copper wire and 2 belts, an assembler 12 iron plates, 6 gears, 12 copper wire and
4 iron rods; a boiler 24 stone bricks, 8 steel plates and 6 pipes, a steam turbine 12 steel plates, 4 motors and 16
copper wire, a crusher 6 steel plates, 2 motors and 4 gears, a silo 24 steel plates, 8 concrete and 4 steel beams, an
arc furnace 10 steel plates, 16 stone bricks and 24 copper wire. Two lifts cost 2 iron rods and 2 belts, and an
underpass entry or exit 2 iron plates and 2 belts. A research lab costs 6 iron plates, 8 copper wire and 4 belts;
Mk2 machines their Mk1 and green kits (see Upgrades); four torches a stick and a coal ore; two lamps a glass block,
an iron plate and 2 copper wire; a pump 6 iron plates, 4 iron rods and 6 copper wire; four pipes 2 iron plates; an
outlet 3 iron plates and 2 iron rods; a quarry 12 iron plates, 8 iron rods, 16 screws and 8 copper wire. Hand-mining
an outcrop or two and a furnace cover your first miner, generator and poles. After that, let them do the work.

## Sound designer

Press **O** in game (or **Sound designer** in the pause menu) to tune every sound with dials. You don't need
any sound-design experience to use it.

- **Materials** (Stone, Dirt, Grass, Sand, Wood, Leaves, Metal): each has ten dials with plain meanings, such as
  Pitch (deep ↔ high), Tone (muffled ↔ bright), Crunch (smooth ↔ gritty), Scatter (one hit ↔ many bits),
  Snap (soft ↔ sharp), Thud, Ring, Length, Volume and Variation. Hover over a dial for an explanation.
- **Listen** buttons play mining, breaking, placing, footsteps and landing on the material. Walk and mining
  loops keep playing while you turn dials. Pressing O while looking at a block opens that block's material.
- **Actions** adjusts footsteps, landing, pickup and so on for every material at once.
- Changes apply in the game immediately and are saved in the browser.
- **Copy settings** exports the whole design as JSON. To make a design the built-in default, paste it over
  DEFAULT_DESIGN in web/src/audio/settings.ts, since it uses the same shape.

## Architecture

```
┌──────────────────────── Rust → wasm (crates/engine) ─────────────────────────┐
│ Game (lib.rs, api/): what the host calls; acts for the local player          │
│  ├─ core: Sim        deterministic: tick, world edits, factory, deposits,    │
│  │                   inventories, rng. Changes only by applying Actions at   │
│  │                   a tick, and reports back with SimEvents                 │
│  ├─ authority        every player's body (physics), loose items, pickups     │
│  └─ view (local)     camera, targeting, mining, streaming, greedy meshing,   │
│                      box instances, sounds, toasts                           │
│ save.rs: the core's bytes (also its state hash) + bodies + items ⇄ save file │
└───────────────▲──────────────────────────────────────────────┬───────────────┘
      input, dt │                                              │ pointers into wasm memory
┌───────────────┴───────── TypeScript platform (web/src) ──────▼───────────────┐
│ input.ts · render/ (WebGL2) · ui/ (DOM) · audio/ · save/ (IndexedDB)         │
└──────────────────────────────────────────────────────────────────────────────┘
```

Rust owns all game state and every hot loop. TypeScript is a thin platform layer: it forwards input, uploads
data to the GPU, draws the HUD and stores saves.

Inside the engine, the core only changes when a queued action is applied at a tick, so the same actions
always produce the same world. Tests compare state hashes tick by tick, and a world reloaded from a save
carries on identically. Co-op builds on this (lockstep): every player's machine runs the core, only actions
travel, the host orders them into per-tick frames, and machines compare state hashes every second, resyncing
from a snapshot if they ever differ. The view is local presentation and never feeds back into the core.

Each frame:

1. `game.update(dt)` runs the simulation in fixed 60 Hz ticks (physics at 120 Hz, targeting, mining and
   placing, items, the factory), so it behaves the same at any frame rate. The camera is interpolated
   between the last two ticks. It then writes one box instance (12 floats) per dropped item, belt item and
   machine part.
2. `game.work_step()` is called in a loop until a time budget runs out (6 ms, or 28 ms while loading).
   Each step generates or meshes one chunk, nearest first.
3. Mesh and unload events are drained. Mesh data is read with a `Uint32Array` view directly on wasm memory
   and uploaded, with no copy and no serialization. Sound events (dig, break, place, footstep, landing,
   pickup, drop) are read the same way. Each carries a sound material and a camera-relative position.
4. Chunks are frustum-culled and drawn front to back, opaque pass first, then cutout. All box instances go
   out in a single instanced draw call, read from wasm memory like the meshes.

### Performance choices

- **32³ chunks, 256 blocks tall.** Uniform chunks (all air, all stone) take no heap memory.
- **Greedy meshing with per-vertex AO and smooth light.** Faces merge only where AO and light are constant along
  the merge axis, so merging never changes the shading. Quads are split along the brighter diagonal to avoid seams.
- **Light in the mesher.** Sky and block light (0–15) are flooded over the chunk plus a 20-block margin while it
  meshes, so they are never stored and chunk borders always match; an edit relights the chunks it can reach.
- **5 bytes per vertex.** Position, face, AO and texture layer are packed into one `u32`, plus a light byte. UVs are derived in
  the shader from position, and texture wrapping tiles merged quads. One shared index buffer serves every chunk.
- **Texture array** (`TEXTURE_2D_ARRAY`) instead of an atlas. There is no bleeding, and mipmaps and
  anisotropic filtering work correctly.
- **Separate opaque and cutout passes.** Only leaves and plants use `discard`, so opaque geometry keeps early-z.
- **Camera-relative rendering.** Chunk offsets are computed in float64 on the CPU, so precision holds at
  any distance from the origin.
- **Order-independent world generation.** Trees and ore deposits crossing chunk borders are derived from
  hashes of their source cell. Chunks can therefore be generated in any order, which prepares for moving
  generation to worker threads. Where deposits overlap, a fixed priority decides which one owns each block.
  The same rule answers "which deposit is this block in?", so ownership needs no stored map.
- **Factory state is small and lazy.** Belts hold a short list of items with their progress. Links between
  machines are rebuilt only when something is placed or removed. A deposit's pool is tracked only once
  something mines, drills or inspects it, and its draw budget is refilled on demand, not every tick.
- **Wasm SIMD128, fat LTO, `wasm-opt -O3`.**
- **Procedural audio.** Each material sound is layered from noise bursts (grainy or smooth, single or scattered),
  a tone filter, a low thud and tuned resonators. The designer's dials map directly onto these layers.
  Buffers are synthesized in 4 ms background slices and rebuilt only when a dial affecting them changes.
  Fixed seeds keep a sound's randomness stable while you tune it. Playback adds pitch jitter, distance
  falloff and stereo panning, and a compressor sits on the master bus.

Measured in Chrome with an RTX 3060: generating or meshing one chunk takes a median of about 0.1 ms, and
99% finish within about 1.1 ms. The full 8-chunk radius (about 2,150 chunks) streams in about 350 ms of CPU
time.

## Project layout

`crates/engine` is the Rust engine, which owns all game state. `web/` is the TypeScript host (rendering, input,
UI, audio), and `scripts/` holds the build, dev and check scripts. [docs/CODEMAP.md](docs/CODEMAP.md) lists
every module and explains how to add blocks, recipes, machines, API methods, UI panels and sound materials.

## Deployment

Every push to `main` runs `.github/workflows/pages.yml`. It builds the wasm, runs `npm run check`, builds the Vite
bundle, and publishes `dist/` to GitHub Pages. The build uses relative asset URLs (`base: './'`), so it
works from any sub-path.

For debugging, the running game is exposed as `window.opencraft.game` in the devtools console. For example:

- `opencraft.game.give(8, 64)` gives a stack of iron ore. Block ids: 7 coal ore, 8 iron ore, 9 copper ore,
  12 belt, 13 miner, 14 box, 15 smelter, 16 constructor, 17 splitter, 18 filter, 19 ramp up, 20 ramp down, 21 lift, 22 underpass entry,
  23 underpass exit, 24 generator, 25 power pole, 26 research lab, 27 Miner Mk2,
  28 fast belt, 29 sapling, 30 granite, 31 sandstone, 32 basalt, 33 limestone, 34 quartz ore, 35 glass, 36–39 stained soils (grass a shade off), 40–43 stained sand, 44 lamp, 45 water, 46–52 flowing water, 53 pump, 54 pipe, 55 outlet, 56 quarry, 57 torch, 58 planks, 59 ladder, 60 stone bricks, 61 assembler, 62 machine part (a multi-block machine's other cells), 63 concrete, 64 blast furnace, 65 slag, 66 boiler, 67 steam turbine, 68 crusher, 69 silo, 70 arc furnace; items: 256 iron ingot, 257 copper ingot,
  258 iron plate, 259 iron rod, 260 screws, 261 copper wire, 262 red science pack, 263 green science pack,
  264–269 stone and iron tools, 270 scanner, 271 core drill, 272 stick, 273 gear, 274 green kit, 275 quicklime,
  276 smelter Mk2, 277 constructor Mk2, 278 motor, 279–281 steel ingot, plate and beam, 282–284 steel tools, 285 blue science pack, 286 blue kit, 287–294 Mk3 belt, miner, smelter and constructor, Mk2 and Mk3 assembler and blast furnace, 295–305 Mk2 and Mk3 pole, box, pump, quarry and lab, and the Mk2 generator, 306 crushed iron, 307 crushed copper, 308 silicon, 309 circuit, 310 violet science pack, 311 violet kit, 312–319 Mk4 belt, miner, smelter, constructor, assembler, blast furnace, pole and lab.
- `opencraft.game.teleport(0, 120, 0)` moves you.
- `opencraft.game.find_deposit(1)` returns `[x, y, z, ore]` for the nearest deposit of a tier
  (0 lode, 1 vein, 2 outcrop).
- `opencraft.game.skip_time(600)` runs the game ten minutes ahead, silently (dropped items older than five
  minutes despawn).
- `opencraft.game.block_at(x, y, z)` reads a block.

## Roadmap

The plan of record, with decisions, rules and the current milestone's steps, is
[docs/DEV_PLAN.md](docs/DEV_PLAN.md). Later milestones are in [docs/ROADMAP.md](docs/ROADMAP.md). In short:

Sandbox foundation:

- [x] Fixed simulation tick, deterministic core driven by actions (groundwork for co-op)
- [x] Save and load worlds: autosave, several worlds, export and import
- [x] Co-op multiplayer: player-hosted over WebRTC, 2–4 players
- [ ] Simulation and worldgen in Web Workers
- [x] Water: a sea and ponds, flowing water, swimming
- [x] Full inventory screen
- [x] Day/night cycle, sky and voxel lighting (sky light, torches, lamps)
- [x] Minimap, and a world map (M) of everywhere you've been with pins and an ore guide
- [x] A living surface: leaves of felled trees decay, grass grows back over bare dirt, saplings grow into trees
- [x] Tools that wear out: pickaxe, axe and shovel in stone and iron
- [x] Biomes: plains, desert, highlands, lowlands, basalt fields, each with its own rock
- [x] Ores that follow the rock: limestone and quartz join coal, iron and copper
- [x] Surface hints: stained soil above deep deposits
- [x] Prospecting: a scanner and a core drill

Factory layer (the Satisfactory half):

- [x] Ore deposits (outcrop, vein, lode) with shared pools, draw limits and depletion
- [x] Miner Mk1, conveyor belts (merging, corners) and storage boxes, crafted by hand
- [x] Item registry separate from blocks (ingots so far) and machine recipes
- [x] Smelter: ore and fuel into ingots
- [x] Constructor and parts (plates, rods, screws, wire), with a machine panel
- [x] Machines: assembler (the first multi-block machine), blast furnace (steel and slag), Mk3 tiers of the main machines
- [x] Splitters and filters
- [x] Belts that climb (ramps by placement, lifts) and cross (underpasses)
- [x] Drag-to-build belt lines, R to rotate
- [x] Miner and belt tiers (Miner Mk2, fast belts)
- [x] Prospecting: find veins and lodes without digging blind
- [x] Power grid: coal generators that burn only what is used, poles, brownouts; every miner runs on it
- [x] Pumps, pipes and outlets: drain ponds and flooded pits
- [x] Quarry: automated digging that leaves a real pit
- [x] Research: labs, science packs and a small tech tree
- [x] Industry: colour-coded upgrade kits, multi-block machines, steel, blue science, steam power, crushing, silos
- [ ] Electronics, blueprints and construction drones, the jetpack
- [ ] Terraforming: a planner and excavators with work drones
- [ ] Aluminium from far away, trains
- [ ] Oil, plastics and nuclear power
- [ ] AI datacenters and laser links for power and data
- [ ] Rockets and satellite constellations
- [ ] An optional endgame megaproject that doesn't end the game
