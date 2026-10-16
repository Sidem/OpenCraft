# Patch notes

Shown in the menu under "What's new" (ui/patch-notes.ts). Newest first. Each update is a `## date · title`
line followed by `- ` lines, written for players in plain words. Anything else is ignored.

## 2026-10-16 · A better first minute

- A new world starts in front of a small production line that has stopped: one belt came loose. Put it back, turn it the right way with R, and the line runs again; its last load of ore becomes the ingots for your first miner. Floating key prompts show what to press where.
- Tips are bigger and easier to skim: a title, the important words highlighted and keys drawn as keys.
- The tip for placing your first miner now also says where to look for ore.
- Mining ore by hand now shows what you kept and how much was lost with each block, and after a few blocks a warning sums it up and points you to a miner.
- A machine's panel now opens next to your inventory: click a held stack into a slot, Shift-click a stack to put it in, click a slot to take it.
- New tips for the space age: building a launch site, what each kind of satellite does, and sending drop pods.
- Rockets roar as they lift off (you hear them from far away) and a toast says when a satellite is in orbit.
- Drop pods now rise out of their launcher, thud down on their pad in a cloud of dust, and each has its own sound.
- On the maps, pod launchers and landing pads show an arrow for their channel, and launch sites show as a rocket.
- The inventory's long line of instructions is gone: a bar along the bottom now shows just the clicks that work on the slot under your pointer.
- Long recipe lists (the Upgrader's) scroll inside their own box instead of stretching the whole panel.

## 2026-10-15 · Hover pack charging from far away

- A long-distance pole charges your hover pack from up to 32 blocks away, so a tower can keep a whole base topped up. Ordinary poles still need you within 6 blocks.
- The hover pack now makes a power-up sound when it starts charging and a power-down sound when the connection drops (you walked out of range, or the pole was removed). They are in the sound designer as Power up and Power down.

## 2026-10-15 · Automating buildings and the Upgrader

- Assemblers now build power poles, long-distance poles, solar panels, accumulators, lamps, storage boxes, underpasses and rails, so a base can build its own parts. Constructors saw logs into planks and planks into sticks.
- The long-distance pole now costs 12 steel beams, 16 aluminium plates and 60 copper wire, and the solar panel 6 silicon, 2 glass and 6 copper wire, so an assembler can hold everything.
- "Choose what it makes" in constructors, assemblers and other machines is redesigned: each recipe is a card with the product and how many a batch makes, the batch time, and an icon chip for every ingredient with its count (byproducts show as "Also gives"). Recipes you cannot use yet sink to the bottom and say what to research, and long lists have a search box.
- New building, the Upgrader (researched with Assembly): belts bring in a Mk1 item and the kits of each step, you choose the tier, and it makes it in one machine, using the kits up. A Mk4 pole is a pole and a green, a blue and a violet kit.

## 2026-10-15 · A research map

- The research screen is a map you can drag and zoom with the wheel: no more scrolling in two directions. It opens on what you are researching now.
- The tree is split into eras by the science packs they need (Red, Green, Blue, Violet, Gold), each with how many of its techs are done. Click an era at the top to fly there.
- Zoomed out you see names, then only coloured tiles for the whole tree at once. A small map in the corner shows where you are; click or drag it to move.
- Search (press /) finds techs by name or by what they unlock, such as "steel". Enter jumps to the next match.
- Each tech shows icons of what it unlocks and how many units it costs; Where I am and Fit all buttons bring you back.

## 2026-10-15 · Drop pods

- New research, Drop Pods (needs Rocketry and Orbital Comms): ship cargo through the sky. An assembler builds drop pods (a light frame, a fuel tank and 10 circuits each).
- The Pod Launcher (3×3×3, 500 kW) takes cargo from belts and a drop pod. Under a comms satellite it loads for 15 seconds, then sends up to 6 stacks. 20 seconds later the pod comes down on a Landing Pad (3×3, no power), which belts empty. Watch it fall.
- A launcher and a pad pair up by the way they face (four channels, shown in their readouts); a pod goes to the nearest pad on its channel that a comms satellite covers. If the pad is full the rest waits in the sky.

## 2026-10-14 · Long-distance power

- New research, Long-Distance Power: the long-distance pole, a lattice tower 2×2 and 3 high, links to other long-distance poles up to 160 blocks away (an ordinary pole only 16), to carry power out to far ore mines and oil wells. It powers machines within 10 blocks, takes 6 connections, and wires like any pole.
- Esc on the pause menu now plays on (if your browser won't give the mouse back at once, click the game).

## 2026-10-14 · Science from orbit

- New research, Orbital Science: an assembler builds science satellites, and the new Ground Station (a 3×3 dish you hang on a power pole and a fibre node) downlinks their data as compute.
- Each science satellite over a ground station gives its data grid 40 TF, up to 100 TF, with no datacenter. The station uses 150 kW. The satellite flies on, so build a constellation.
- A science satellite's circle on the maps is pale blue.

## 2026-10-14 · Power from orbit

- New research, Orbital Power: an assembler builds power satellites, and the new Rectenna (a flat 5×5 field you hang on a power pole) turns their beam into electricity.
- Each power satellite over a rectenna gives it 200 kW, up to 1 MW, day and night with no fuel. The satellite flies on, so a few of them and some accumulators keep a grid steady.
- Orbital power has its own colour on the electricity graph, and a satellite's circle on the maps is pink.

## 2026-10-14 · Comms satellites

- New research, Orbital Comms: an assembler builds comms satellites and the launch site sends them up. Each one covers a circle 1024 blocks wide (drawn in blue on the maps).
- Under a comms satellite, all fibre nodes join into one data grid however far apart they are, so a datacenter can serve labs on the other side of the map.
- A drone port under one reaches four times as far, and its cargo drones fly twice as far. The cover moves on with the satellite, so a dozen up keep it steady.

## 2026-10-14 · Survey satellites show the ground

- A survey satellite now maps what is under it: every iron, coal, copper and other vein or lode inside its circle appears on the minimap and the world map as it passes, and stays there. The circle itself is drawn as a faint gold disc.
- The AI survey stops guessing where a satellite has already found the ore.

## 2026-10-14 · A clearer build list

- The Hover Pack now goes in the Flight pack equipment slot, the same slot as the Jetpack, so you wear one or the other. Carrying it in your backpack no longer does anything.
- The build list has a tab for each category (new: Computing and Space), every Mk of a building sits next to each other, and Diesel Generator and the other power buildings are under Power.
- A bright button at the top of each page tells you what the next page holds, so it is hard to miss that the list continues.
- Hovering a building shows its size, electricity use, fuel and power produced in separate fields, before the description.

## 2026-10-14 · Machine inputs and quiet tabs

- The music now fades out when you switch to another tab or window, and comes back when you return. Two new options on the Sound page let you turn this off for the music, or turn it on for sound effects too.
- A machine now takes at most one stack of each input item, so a belt full of one item can no longer fill the machine and keep the others out.
- Click any filled slot in a machine's panel, inputs and fuel included, to take its items back into your inventory.
- Poles, belts, splitters, pipes and the like are now only hit where you actually see them, not across their whole block, so you can mine the block behind a pole without taking the pole. The highlight outline hugs the model too.

## 2026-10-13 · Coordinates under the minimap

- Your position (X, Y, Z) now shows under the minimap: X in red, Y (height) in green, Z in blue.
- A small 3D gizmo next to it shows the same three colours as axes: red points east, green up, blue south, just like the map. A white arrow shows which way you are looking, including up and down.
- N hides or shows the minimap and this readout together.

## 2026-10-13 · Gears and power lines

- Gears look like gears now: a toothed cog with a hub, in your backpack, on the ground and as the winding wheel on top of a miner (which turns while it works).
- Miners look like drill rigs now: a motor housing with a fan and status lamp on top of a cage with a screw auger inside. While it works the auger turns and plunges, the fan spins and the housing shivers. Miner icons show the same rig in their tier colour.
- Power lines between poles are one smooth, sagging string, even between poles at very different heights, instead of a chain of short pieces.

## 2026-10-12 · Background music

- The game has music now: two versions of one daytime piece. Adventurous is a full orchestra with a fanfare; Ambient is soft and slow, for building.
- There is a new Sound page in the menu with separate volume sliders and mute buttons for sound effects and for music, and a choice between the two tracks. Switching fades smoothly from one to the other.
- K still mutes or unmutes everything.

## 2026-10-12 · A proper analytics screen

- Analytics (P) is now a hub with four tabs: Overview (what needs attention), Power, Materials and Machines. Press 1 to 4 to switch.
- Power shows where the electricity comes from (generators, steam, solar, water wheels...) and which machines use it.
- Materials has groups (raw materials, ingots, parts, electronics...), a search box, and shows what is made and what is used up separately, with a balance. Click an item to draw its line, hover it to make it stand out.
- Machines shows what holds each kind of machine back and lists the slowest ones with where they are.

## 2026-10-11 · A tidier inventory

- The build menu and the backpack are paged now, so nothing scrolls: every category (logistics, power and so on) starts a new page, and a page ends when no more recipes fit. Flip pages with the numbered buttons or the mouse wheel.
- The equipment and the hotbar always stay in view; a bigger hauler pack just adds backpack pages. A storage box now opens beside your inventory.
- Searching knows related words: water finds pumps, pipes and boilers; belt finds splitters and filters; power finds generators and poles.
- Ctrl-click a slot takes one item onto your cursor (again for the next one), and Ctrl-right-click puts one down, so you can build or split a stack a single item at a time.
- Belt Mk3 and Mk4 now look like Mk1 and Mk2 in menus.

## 2026-10-11 · Rockets

- New tech Rocketry (after Electrolysis and Accelerators): build light frames, rocket engines, fuel tanks and avionics in assemblers, and mix rocket fuel in the chemical plant from hydrogen and oxygen.
- The launch site is a big 7×7 pad with a gantry. Feed it 4 frames, 2 engines, 2 tanks, an avionics, 8 fuel canisters and a satellite and it builds a rocket in a minute, then launches the satellite into orbit. The empty canisters come back out.
- Survey satellites (built in an assembler from light frames, solar panels and AI accelerators) circle the sky: look up and you see them as small gold lights, and the launch site says how many are up and how many are over it.
- A satellite does nothing for you yet: what each kind gives comes next.

## 2026-10-11 · Fabricator Gloves

- Research Fabricator Gloves (after Handcrafting III and Gold Science) and craft them: worn in the new Gloves slot of the inventory screen, they make crafting by hand another 50% faster, on top of Handcrafting.
- Old saves keep working; their characters get an empty Gloves slot.

## 2026-10-10 · Better light, new machine looks and smoke

- Sunlight and moonlight now light the sides of blocks that face them, so the world has more depth.
- Water glints in the sun.
- Lamps, machine status lights and laser beams glow brightly at night.
- A working jetpack shows its flames.
- Many machines have new models that show what they do: round tanks and columns for the oil and chemistry machines, a domed reactor, a lying boiler drum, a tube cascade for the centrifuge, a round silo, a winding wheel on the miner, a hopper and jaws on the crusher, and distinct tops for the AI lab, optimizer and swarm hub.
- Loading docks now have a raised hopper and unloading docks a sunken grate, and the two laser receivers look different.
- The locomotive is now a green steam engine with big driving wheels, and wagons have ribs, a dark hold and bogies.
- Furnaces, boilers, coal and diesel generators, the refinery and the cracker puff smoke while they work, and a moving locomotive smokes too.

## 2026-10-10 · A new front door

- The menu is now a launcher: a big Play button with your world beside it, and one page at a time for your
  worlds, playing together, settings, controls and these notes.
- First time here? The home page walks you through your first minutes.
- Settings show live on the game while you change them.
- A link to the game's source code on GitHub, where you can also report a problem.

## 2026-10-10 · Quicker hands

- Research Handcrafting I, II and III: each makes crafting by hand 25% faster.
- The inventory screen is wider and no longer scrolls sideways.
- Phones and tablets get a notice instead of the game, which needs a keyboard and mouse.

## 2026-10-09 · Lasers, swarms and smarter tools

- Laser mirrors turn a beam round corners, and a Data Receiver sends compute down a beam.
- The swarm hub makes nearby drone ports fly half again as many drones.
- Auto-routing: drag a belt line onto a machine and it finds its own way there.
- AI survey: faint rings on the maps guess where ore lies from the stained soil you have seen.
- The jetpack has its own equipment slot, and you can choose which deposit the scanner points to.
- Research can be queued, underpasses are crafted in pairs, and hauler packs go up to 99 slots.

## 2026-10-08 · The age of compute

- Chip fab, wafers and AI accelerators.
- Fibre nodes form a data grid; AI datacenters fill it with compute and need cooling towers.
- AI labs research endless bonus techs: mining productivity, machine speed and drone speed.
- The optimizer speeds up every machine near it.
- Laser links carry power across up to 128 blocks of open air.
- The recycler takes items you no longer need and pays recycling coins for them.
