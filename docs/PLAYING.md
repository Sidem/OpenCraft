# Playing OpenCraft

The player's manual: controls, worlds, co-op, crafting, upgrades and sound. The factory itself (deposits,
machines, power, research and every later era) is in [FACTORY.md](FACTORY.md), towns in [TOWNS.md](TOWNS.md). New players get the basics from
the in-game tips (`crates/engine/src/hints.rs`). Keep this file in sync for anything player-visible.

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
| I                   | Pick up the item on the belt you point at, or the one you clicked to select if it is within reach; it goes into your pack, or stays if there is no room |
| Left-click an item  | Select it on its belt: its readout stays up and follows it; click elsewhere to let go |
| B                   | Ghost mode: right-click plants a free ghost of the held block or machine (again on it removes it), left-click marks the aimed block for tear-down (red outline; again clears it), R turns it, reach 16, no mining; a ghost's outline is cyan and the label lists what the ghosts need. Placing the real block on a ghost builds it facing the same way |
| Z, Enter (ghost mode) | Z marks the two corners of a box (a third press clears it); Enter copies the machines in it as a blueprint and takes it in hand. With one in hand, right-click stamps its ghosts, R turns it and Z puts it away |
| L | Blueprint library: rename, hold or delete your blueprints (kept per world in this browser) |
| Right-click (Planner in hand) | Marks a site: a corner block, then the opposite one (reach 64); a panel picks dig, fill, flatten (with the level) or a tunnel (with its section). Right-click a marked site to look at it or remove it |
| Y | With a Personal Drone in your pack: it fetches more of the item in your hand from the nearest storage box within 32 blocks |
| Jump (in the air) | With a Coal Jetpack in your pack: keep holding it to climb on a plume of fire, burning a coal from your pack every 10 seconds; with a charged Hover Pack, hold it to hover until you land (jump rises, crouch sinks) |
| Shift-click gear (inventory) | Wears it: back (hauler pack: 27 more backpack slots, 18 more for each Mk up to Mk5 with 99; four kits of the tier raise the worn pack in place from a button in the inventory, no need to take it off), boots (spring: jump 2 blocks; servo: walk 15% faster), torso (exo frame: sprint 10% faster), tool belt (mining rig: break blocks by hand 50% faster). Researched under Hauler Gear, Field Gear and Exosuit |
| Right mouse on a box | Open it like a chest: click moves stacks with the cursor, Shift-click moves a whole stack between box and inventory. Hold C to place against it instead |
| Right mouse on a miner | Take everything it holds |
| Right mouse on a smelter, constructor, assembler, filter, generator, lab or quarry | Open its panel. Hold C to place against it instead |
| E                   | Inventory and build menu                  |
| T                   | Research: choose what labs work on        |
| P                   | Analytics hub: power by source, materials made and used, machine health |
| H                   | Skip the tip on screen (the menu can show tips again) |
| M                   | The map: everywhere you've been, your pins, filter chips and the ore guide (drag to move, wheel to zoom, click to pin, rest on a mark to see what it is) |
| Telescope           | Hold it, **right-click** to look through it (first person), **wheel** zooms 2× to 8×, **left-click** pins the ground you look at (an empty note, editable on the map); while you look, every pin is a coloured column of light, even from far beyond the fog |
| Sickle              | Hold it, **right-click** natural ground (grass, dirt, sand, basalt, granite) to cut the small plants of a 3×3 patch; hold to sweep. They drop nothing and never grow back (a new floor on that ground stays bare too); trees are not touched. Two iron ingots and two sticks |
| N                   | Show or hide the minimap until you next start a game or reload (top right, north up; diamonds mark ore you've seen, rings veins and lodes you've prospected, squares your machines, pins your pins) |
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
hotbar and the backpack. **Right-click** a stack to take half of it (an odd stack gives the larger half);
right-click it again to take half of what is left and add it to the stack in your hand. With a stack in
hand, **Shift-right-click** puts one item into a slot (an empty one, or the same item with room), and
Shift-right-click outside the window throws one item out. With an empty hand, Shift-right-click moves every
stack of that item (between hotbar and backpack, or box and inventory). Tools always move whole. These work
on a box's slots too. **Sort** (beside "Backpack", and beside a box's slots when you have one open)
merges partial stacks and orders everything by item; the hotbar keeps the layout you made, and tools never
merge, so each keeps its own wear.

**Analytics (P)** is a hub with four tabs (keys 1 to 4). *Overview* shows power, machines and materials at a glance and lists what needs attention. *Power* shows how much power the grids could make, how much is used and how much the machines asked for (when that is above what is used, the grid is short and everything on it slows down), stacked by source (generators, turbines, solar, water wheels...) and by the kind of machine using it. *Materials* graphs what is made and what is used up of each item, a minute at a time: filter by group (raw materials, ingots, parts, electronics...), search, switch between Made, Used, Both and Balance, click a row to plot it, hover a row to light its line. *Machines* shows what holds machines back, per kind of machine, and the slowest ones with where they stand. The buttons at the top pick the timescale, from the last minute to the last three hours. Every machine also says how close to full speed it has run over the last ten seconds, for example "Efficiency 62%: lost 25% to waiting for input, 13% to low power": it is in the machine's panel and on the readout when you look at a machine. The graphs start when you open the world; they are not saved.

New worlds have a sea and ponds. Water flows: dig beside the sea and it pours in and refills any hole or
trench at or below its level, however far. Elsewhere water runs up to 7 blocks from its source, getting
shallower, and falls over edges; wall it off and the flow dries up. Blocks and machines can be placed
straight into water. In water you swim (Space rises), and dropped items float.

## Saving and worlds

Your world saves itself: every minute, whenever you pause (Esc), and when you switch tabs or close the page.
Opening the game again brings back the world you played last. Press **Continue** and everything is where
you left it, machines still running.

The **Worlds** list in the pause menu manages several worlds:

- **New world** takes a name, an optional seed and a mode. A number gives that exact world, any other text
  works as a seed too, and leaving it blank picks one at random. **Normal** is the game as designed.
  **Creative** is for testing: every technology is already researched and the inventory (E) gets an
  **All items** tab (click for a full stack, Shift-click for one). The mode is chosen when the world is made
  and can't be changed afterwards, and a creative world is marked "creative" in the list.
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
blocks from where you start. **Bauxite**, the ore of aluminium, lies only in deserts and basalt fields at least 300
blocks from where you start (600 in worlds made before the newest generator), shallow and showing on bare rock as
brick-red balls in red clay: it takes a trip (the ore guide, key M, says so). It leaves no stain. In new worlds
**oil sand** (plains and lowlands, 200 or more blocks out) and **uranium** (highlands and basalt fields, 400 or more
out) lie deep underground, never at the surface and without a stain: find them with the scanner (its filter and
bearing cover them), and note they have no use yet. Caves are rare in new worlds: they gather in a few zones.
Worlds made before this keep the terrain and ore they were made with (the
oldest ones, before biomes, are "classic terrain"), so nothing you built there changes.

**The map** (M) shows everywhere you have been, north up; the browser remembers it with the world.
Ore you have seen at the surface is marked with a diamond, so a patch spotted in passing is easy to find
again. Click the map to drop a **pin** (an ore, home or a note, with a few words); pins show on the
minimap too, at its edge when they are far away. Rest the mouse on a mark or a pin, on the map or on the
minimap (while the cursor is free, in a menu or panel), and a small box says what it is: an ore and its kind
of deposit (never how much is left), a machine, a town and how it is doing, a terraforming site, a
satellite's cover, or your pin's kind and note. **Show on the map**, beside the map, has chips that hide
machines, towns, sites, satellite cover, any one ore or any kind of pin (the name above a row of chips
hides the row); they apply to the map and the minimap, the pin list follows, and the browser remembers
them. Beside the map, **Finding ore** charts how deep each ore
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

## Crafting and upgrades

Craft in the build menu (E). It groups recipes by kind; type in its search box to find a recipe
or everything made from a material, and filter by what is unlocked (the default), what you can craft now, what misses materials and what
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
arc furnace 10 steel plates, 16 stone bricks and 24 copper wire, an electrolytic cell 12 steel plates, 16 stone bricks,
24 copper wire and 4 circuits. Two lifts cost 2 iron rods and 2 belts, and an
pair of underpasses 4 belts (6 Mk2 belts for a Mk2 pair, 8 Mk3 and 10 Mk4: as many belts as the blocks they reach; underpasses are only made in pairs). A research lab costs 6 iron plates, 8 copper wire and 4 belts;
Mk2 machines their Mk1 and green kits (see Upgrades); four torches a stick and a coal ore; two lamps a glass block,
an iron plate and 2 copper wire; a pump 6 iron plates, 4 iron rods and 6 copper wire; four pipes 2 iron plates; an
outlet 3 iron plates and 2 iron rods; a quarry 12 iron plates, 8 iron rods, 16 screws and 8 copper wire. Hand-mining
an outcrop or two and a furnace cover your first miner, generator and poles. After that, let them do the work.

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
  straight from your inventory (ore, fuel, ingots, packs), and take what it made. The panel opens next to your
  inventory: click a held stack into one of its slots, Shift-click a stack in your inventory to put it in, or click
  a filled slot to take it.

## Ruins

New worlds hold the ruins of an older factory: a cracked concrete slab with rusted machines, girder poles, a broken run of plating
and a collapsed shaft. The first lies 150 to 300 blocks from where you start (walk out in any direction; flat dry land only), and
about one more turns up in every 500 by 500 blocks. Break the rusted blocks for scrap metal (machines may hold a circuit); a smelter
melts 3 scrap into an iron ingot. The shaft drops about ten blocks to a chamber of spent rock with one more machine at the bottom.
Each slab holds a logbook (a green book on the concrete, it cannot be broken): right-click it to read. A page points you to ground
that holds an ore (with a button to pin it on your map), or gives a recipe tip; ruins far from the start also tell the story of the
people who left them.

## Music and sound

The menu's **Sound** page has a volume slider and a mute button each for sound effects, ambience (wind, water, birds) and
music, and a choice of how the music is picked: **By place** (the default: night, caves, deserts and highlands each have a
piece of their own, the two Open Ground pieces play by day and at the coast; a piece plays now and then with a few quiet
minutes between, and the music changes only after you have stayed somewhere about 20 seconds), **Always calm** (soft, slow
pieces one after another) or **Always adventurous** (fuller pieces). Switching crossfades. **K** mutes or unmutes everything.
Settings are kept in the browser.

## Sound designer

Press **O** in game (or **Sound designer** on the menu's Sound page) to tune every sound with dials. You don't need
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
