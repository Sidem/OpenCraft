# Patch notes

Shown in the menu under "What's new" (ui/patch-notes.ts). Newest first. Each update is a `## date · title`
line followed by `- ` lines, written for players in plain words. Anything else is ignored.

## 2026-11-14 · Smoother trees
- **No more freeze when a sapling grows into a tree**, and a shorter hitch when a tree falls. The game now redraws a new or fallen tree over the next few frames instead of all at once.

## 2026-11-14 · Felling whole trees, and wood in every colour
- **Trees come down whole.** You no longer dig a grown tree out a block at a time. Swing at its trunk near the ground and keep swinging: a red mark shows on the bark, and hitting it counts twice. When enough is done the tree tips over away from you, and its logs land along where the trunk falls. With a bare hand it takes about as long as digging out every block of the trunk used to; an axe is quicker, as before.
- **Each tree gives its own wood.** Birch, pine, palm, cherry, baobab, acacia, redwood and eucalyptus trees drop their own logs, and bamboo drops bamboo. Branches come down as sticks, and the crown leaves a sapling or two.
- **Planks, floors and doors in every wood's colour.** Saw any log into planks of its own wood, and make floors and doors from them. Every wood works in every recipe that asks for logs or planks, and what you make keeps the colour of the wood you used.
- **Tree fixes.** No more dark patches round the feet of bamboo and baobabs or on pine trunks, palm trunks bend smoothly with their fronds arching from the top, and branch ends taper instead of looking sawn off.
- **Logs you placed, and trees in worlds you already have, still break a block at a time.**
## 2026-11-14 · Eleven kinds of tree, and bamboo
- **New worlds grow real-looking trees.** Leaves are drawn as leaves, not blobs, and every tree grows its own branches: wide oaks, slim birches, pine cones of needles, pink cherry blossom in groves, palms leaning over beaches, flat-topped acacias and fat baobabs on warm dry plains, giant redwoods on wet highlands, pale tall eucalyptus and stands of bamboo in warm lowlands.
- **Forests have their own places.** New worlds have savannas, boreal forests, eucalyptus woodlands, bamboo forests and redwood forests, each with its own trees and ground cover.
- **Trunks stay clear.** No leaves hang around a trunk below its crown, so you can see the wood and reach it from the ground.
- **Each new tree drops its own sapling.** Plant one and it grows into a young tree of its kind. A forester plants and fells them like oaks; it takes the trunk and chips the branches.
- **Worlds you already have keep their trees.** Only new worlds (and new saplings) grow the new ones.
## 2026-11-14 · Lifts that go down, and hover packs that charge from further away
- **Belt lifts can carry items down.** Feed the top of a stack of lifts from a belt one block behind and one up, leave its foot unfed, and the stack lowers items and hands them on straight ahead at the bottom.
- **Dragging a belt line uses lifts at cliffs.** With lifts in your inventory, a line that meets a cliff more than one block high climbs it on a stack of lifts, and a line you point down at lower ground goes down the cliff on lifts. Point level with the top and the line stays level over the drop, as before. The line's card says how many lifts it uses.
- **Hover packs charge as far from a pole as the pole reaches another of its kind:** 10 blocks from a Mk1 pole, 16 from Mk2, 32 from Mk3 and Mk4, and 160 from a long-distance pole (was 6, and 32 for long-distance poles).

## 2026-11-13 · Three-way filters, glowing lamps and your hand in view
- **Filters now sort three ways.** Right-click a filter: its panel shows left, straight on and right as the filter sees them. Add items to each way, remove them with ×, and pick which way gets **everything else**. While the item list is open, Shift-click a stack in your inventory to add it. New filters send everything straight on until you pick something; filters you already built work as before.
- **Lamps glow.** Lamps, torches and panel lights now cast a soft glow around them, strong at night and faint by day.
- **You can see what you hold.** In first person your hand shows low on the right with the tool or block in it, and it swings as you dig. Everything you hold is the same size, a little larger than a block used to be, and stays still on screen as you look up and down. Tools are held by the handle and swing like the real thing: a pick or axe winds up and strikes edge first, a shovel stabs in and levers up, a sickle sweeps across. Other players now hold their tool in the right hand. Don't want it? **Settings → Comfort → Hand in first person.**
- **Power lines stay off the ground.** When a wire between two poles would cut through a hill, a free wooden prop appears to hold it up. It costs nothing and moves if you dig the hill away.
- **Canisters are now called barrels.** Searching "canister" still finds them.
- **Belt lifts look like one belt.** A lift's belt now comes in level with the belt that feeds it, climbs its front in one unbroken run however many lifts you stack, and turns over the top level with the belt it hands to. The plates between stacked lifts are gone.
- **Belt lines look like one belt.** The cross bars between belts that hand items to each other are gone; a bar shows only where a line starts or ends.

## 2026-11-13 · A personal drone that runs your errands
- **Press Y to open your personal drone's panel.** Search anything in the storage boxes and machine outputs within 64 blocks (twice as far as before) and send the drone for a stack or for all of it. It takes from your storage boxes before boxes on a belt line, and those before what machines have made, and makes as many trips as it takes.
- **Need something you have none of?** Search it anyway: if you can craft it by hand, **Fetch parts & craft** sends the drone for the materials and crafts it for you when they arrive. The item's page in the build menu has the same button.
- **Put away** takes every stack in your backpack whose item already has a box nearby back to that box, like quick-stacking into chests. Your hotbar stays as it is. **Store** sends a single stack to its box, or to an empty one. **It never puts anything into a box on a belt line** (one a belt or miner fills, or one that feeds a belt), so your production lines never clog with the wrong items, and it never fills an empty box you labelled. **For everything else, label a box "Misc (drone fills)"** in its box screen: when nothing has a box of its own and no empty box is left, the drone puts it there, several kinds together.
- **It keeps the stack in your hand topped up**: when you have built or used your way down to a quarter of a stack, it fetches the rest by itself. Turn it off in the panel.
- **Orders queue up** and the panel shows what it is doing and when it will be back. The line by your hotbar says so too, and tells you when something is not in reach or there is no room.
- **Order it from your inventory:** press **Drone** above the backpack (or Alt-click a slot) and click the slots you mean. Send the full ones to storage, or fill the empty ones with a stack of anything in reach.
- **See at a glance what it is doing for each slot:** a slot it is filling shows a faint picture of what is coming, a ring that fills up as the drone flies, and the seconds until it lands. A slot it is emptying shows an arrow up. Your hotbar at the bottom of the screen shows the same, so you can watch it without opening anything.
- **Shift+Y** still sends it for a stack more of what you hold, and it lands in your hand. So does a top-up.
- **A new look:** a white quadcopter with carbon arms, spinning props, a camera underneath and red, green and white lights. It flies smoothly, leans into its turns, dips at each box and carries what it picked up on a line under it. Your co-op friends' drones fly too.

## 2026-11-12 · Saving on itch.io
- **If your browser won't let the game save while it's shown inside the itch.io page**, the Worlds page now says so and offers to **open the game in its own tab**, where saving and importing work. This happens when a browser blocks third-party cookies or storage, as many private windows do.

## 2026-11-12 · A real pumpjack
- **The pumpjack is now a full-size oilfield pump**, 2 blocks wide, 3 long and 3 tall. A motor turns a crank with heavy counterweights, two arms rock the walking beam on its A-frame, and the curved horsehead at the front lifts the rod in and out of the wellhead. It moves while it pumps and comes gently to a stop where it is when it waits.
- **Empty canisters go in at the back** (the motor end), and **full ones come out at the front** (the wellhead end).
- **It finds oil under any part of its base**, so you no longer have to line up one exact block over the reservoir.
- **Pumpjacks you already built keep working as they are**: they stay one block, as a small model of the new one. Pick one up and place it again to get the full-size pumpjack.

## 2026-11-12 · See where items go, and what's in each box
- **A see-through ghost shows a big machine before you place it**, right where it will stand. It turns red when something is in the way.
- **Every side that takes or gives items now shows it plainly.** A short belt sticks out of each such side with arrows on it, and an item rides along it: into the machine where it takes items, out where its product leaves, and a single arrow where a byproduct like slag leaves. The ports on the far side show through the ghost, and the hint line names the sides as you see them ("in: far, left · out: near"). Press R and it all turns with the machine.
- **Machines you already built show a pad on the floor at every open side that takes or gives items**: arrows pointing in or out, and the item that goes there. Hold a belt to see the full arrows and belt stubs instead.
- **Storage boxes show what's inside on their lid**: the item they hold most of, standing on a small plate and turned to face you, from any side and as far as you can see the box. Or pick a label from the menu in the box's screen (type to search: Ores, Metals, Parts, Electronics, Tools, Wood, Stone, Fuel, Science) and the box shows that icon instead, full or empty. Your co-op friends see the same labels.
- **Crushed and washed ore, sulfur and quicklime ride belts half again as big**, so the little heaps are easy to tell apart.

## 2026-11-09 · Bigger items on belts
- **Items riding a belt are bigger**, so you can tell plates from rods from ingots at a glance. Fluid canisters ride bigger still. Belts carry exactly as much as before.

## 2026-11-08 · If the game runs slowly
- **The game now tells you when your browser isn't using your graphics card**, and how to turn it on. Without it everything is very slow, so this is the first thing to check.
- **Settings has a new Performance section** with a Copy report button. Play for a minute, copy the report and paste it wherever you tell us about the problem. It holds no names or saves, and goes nowhere unless you paste it.
- **New Picture quality settings:** sharpness, render distance and smooth edges. Render distance now changes while you play, no address tricks needed.
- **The game lowers its own sharpness if it runs slowly**, and tells you when it does. If that isn't enough it suggests something else on the home page, and never changes how the world looks without asking. Turn Automatic off in Settings if you would rather choose yourself.
- The frame counter on F3 now reads true on slow computers, and shows how slow the slowest frames are.

## 2026-11-07 · Round trees
- **Trees are no longer made of cubes.** Trunks are round, with bark all the way round, a wider foot where they meet the ground and rings of end grain where they are cut. Branches running sideways join their trunk.
- **Leaves grow together into one soft, lumpy crown**, shaped by all the leaves around them, with a leafy fringe at the edge and shade inside. A row of trees becomes one hedge-like canopy.
- **Logs you carry, drop or send down a belt are round logs too**, with end grain on their cut ends, in your hand and in the inventory.
- **The forester's crane keeps the tree looking like a tree**: the crown it lifts stays round, and when it cuts the trunk into logs, the trunk slides through the head and a log drops off at each cut, instead of the whole trunk shrinking.
- Only the look changed: you still chop, mine and place logs and leaves block by block, and you bump into them exactly as before.

## 2026-11-07 · The forester
- **A new machine, the forester, keeps a wood lot for you.** Research Forestry (red packs only, right at the start) and place it facing flat grass or dirt: it works an 11×11 patch in front of it, with 9 spots for trees on a grid, and a fence shows its edge. While you hold one, a green outline shows the patch; if stones, holes, water or buildings are in the way, they light up red and it won't place.
- **A harvester crane does the work.** Its knuckle boom swings out, the head grips a trunk, saws it through at the foot, lays the tree down and feeds it through its rollers into logs, then carries them back to the bunk and plants a sapling. Wild trees on its patch are felled too, and its neighbours keep their crowns.
- **A slow but steady supply**: about 6 logs a minute from a full patch. Put a few saplings in it to start; after that it keeps itself in saplings. Logs leave on a belt leading away from it, or take them from its panel. It needs 20 kW while it works.
- **Upgrade it with kits** (unlocked by the same research as the quarry's upgrades): Mk2 works twice as fast on a 14×17 patch, Mk3 three times as fast on 17×23, as far as the ground in front is flat. Its card says how much more ground to flatten.

## 2026-11-07 · Ghosts when you need them, stone when you run out
- **No more stray gold boxes.** Building the same machine a third time used to leave a gold outline standing in the world until you pressed B twice or reloaded. That box now only shows in ghost mode, where it belongs.
- **Ghost mode and blueprints now open with the Construction Drones research**, the drones that build ghosts for you. Before that, B just tells you so, and the blueprint list (L) says what it waits for. Worlds that already researched it keep it.
- **The tips now tell you where more stone is**: under the grass and dirt, so dig straight down. Right after you get power, a new tip shows how a quarry digs stone for you.

## 2026-11-07 · A guided first hour
- **The first coal miners are walked through step by step.** Pressing E makes the Production tab and the Coal Miner glow, so you can see what to click. Once the first miner burns, the tips show you how to put a storage box beside it so the coal it digs piles up. Then they ask for another coal miner, this one for iron.
- **The tips point to the map.** The patch they send you to has a flag on the minimap and on the world map (M).
- **Going electric starts with copper.** The tip asks for one more coal miner and points to the nearest copper patch with a floating marker, a flag on the map and its distance and direction.

## 2026-11-06 · Machines that say what they need
- **A machine waiting for something now names only what is missing**, and how many more: "Waiting for 2 Sand" instead of every ingredient. Point at it and it also says where that comes from: dig sand on a beach or set a quarry there, a crusher makes it from slag, quicklime comes from limestone a scanner finds.
- **A machine stuck with a full output says what its product is good for**: a blast furnace full of slag tells you a crusher turns slag into sand.
- **Short of power?** When a machine near you has worked slowly for want of power for a while, a card says how much it gets and that [P] shows what each grid makes and uses. Open it once and the card is gone for good.
- **Researched Ore Crushing?** A tip now shows how to belt your blast furnaces' slag into a crusher for a steady supply of sand.
- Worlds you already play keep quiet: they count the new cards as already learned.

## 2026-11-05 · Gentle ground, quieter water
- **New worlds no longer start you on a mountain.** Whatever the seed, the land around where you start is low and gently rolling; hills and ranges rise again a few hundred blocks out. Worlds you already have keep their ground.
- **The sea and rivers are much quieter**, a soft wash under the wind and birds instead of the loudest thing around, and the surf now swells as you reach the shore rather than filling the land behind it.
- **Ambience starts quieter**: its slider now begins at 15%. If you have moved it yourself, your setting stays.

## 2026-11-05 · A smoother first five minutes
- **New worlds now start you on two coal miners.** The little stopped line by the start smelts the iron for two coal miners, and its end box also holds the stone and coal they need, so you no longer have to dig for stone first.
- **The tips walk you all the way to them**: craft the coal miners, then follow a floating "Coal" marker to the nearest coal patch, set one on it and light it, then an "Iron" marker to the iron patch for the second. The card says how far and which way, and counts down as you walk.
- **No more Miner Mk1 by mistake at the start**: the opening no longer hands you copper, and the tips only suggest the electric miner once you are ready to build power for it.

## 2026-11-05 · A sunnier world
- **The world now looks more like the game's cover picture.** The daytime sky is a deeper blue, the sun has a wider warm glow, and a clear day has a few white clouds drifting over it with crisper edges.
- **Grass, leaves and plants are brighter and greener**, a fresh meadow green instead of the old dark olive. Bark, soil and stone keep their natural colours.
- **Meadows are fuller:** grass tufts are taller and about twice as common on the plains, and there are more flowers. Poppies and daisies have bigger heads.

## 2026-11-01 · Which way am I facing?
- **The coordinates under the minimap now say which way you face.** The little 3D arrow, which could point towards you or away from you, is gone. Instead each of X, Y and Z ends in a "+" or "−": a solid coloured badge on the axis you face most (so "+" on X means you look towards growing X, east), and a faint sign on any other axis you are also partly facing, such as when looking diagonally or up and down.

## 2026-11-01 · Machines that look the part
- **The assembler is now a robot cell you can watch work**: two orange robot arms pick each part off the conveyor and set it on the fixture, a little work cloud puffs up while the hammer pounds, and the finished item rides out of the front. An overhead gantry carries the turning display of what it makes, at the top centre.
- **The arc furnace** is a round, water-cooled furnace with a domed roof, three graphite electrodes on copper arms from a mast at the back, and a glowing roof, slag door and tap spout while it runs.
- **The blast furnace** is a tall tapering stack with a hot-blast stove, a skip bridge climbing to the top and a glowing tap hole.
- **The electrolytic cell** is a long reduction pot with sloped hood covers and a copper anode beam; its bath glows at the seams while it works.
- **The crusher** is a jaw crusher: a hopper on top, a jaw that pounds, and big flywheels driven by a belt from its motor.

## 2026-10-31 · Straight rails, docks anywhere
- **Rails snap straight.** Aim within about 13 degrees of north, south, east or west (or straight on from the last node) and the node goes onto that line, so straight track is the usual result. The first node of a line faces an axis too. Aim wider and the track still bends as before.
- **Docks go anywhere along straight track.** Hold a dock and aim at the ground along north-south or east-west track: it goes down right there and cuts the track in two. It needs 3 blocks of track on each side, no train on it, and ground under both halves.

## 2026-10-31 · Quicker freight lines
- **Lay a dock in line, like a rail.** Place a rail node, then hold a loading or unloading dock and aim along the track: the dock goes down where you aim, joined to the node, and your rails carry on from it. Where the ground is in the way or missing, the blocks to fill or clear are outlined, as for rails.
- **Shift-click with wagons to couple them all at once** (up to six).
- A dock at the end of a line now joins on to new rails; before, joining track to it did nothing.

## 2026-10-31 · Trains you can read
- **Look at a locomotive or wagon to see how full it is**: how many of its slots are used and what it carries, and what the train is doing ("Loading at a dock", "Running to stop 2 of 3"…).
- **Right-click a train to open it.** Each wagon opens like a chest: take goods out or put them in. The panel shows a bar per wagon, what the train is doing, and its stops; take a stop off with ×, or clear them all.
- **Open wagons show their load**: a heap of what they carry, rising as they fill.

## 2026-10-31 · A modern train
- **The locomotive is now a modern diesel**: a long blue hood with an amber band, a cab with a raked windscreen, headlamps, roof fans and two three-axle bogies. Wagons wear the same colours.
- **Wheels turn as the train moves**, and trains **speed up and brake smoothly**, so they glide into a dock and stop with the first wagon right under the chute or over the grate.

## 2026-10-31 · Docks are part of the track
- **A loading or unloading dock is now a piece of track.** Hold one and the rail nodes it fits are outlined: click one on straight north-south or east-west track and the dock takes its place, keeping the track on both sides. Breaking it gives the rail node back.
- **Trains stop right on the dock.** The loading dock is a gantry with a chute that pours the goods into the wagon below; the unloading dock is a shed with a grate between the rails. Belts feed and empty them from the sides.
- **You can see the trade:** goods pour from the chute, or down through the grate, while a train loads or unloads, and the dock's lamp turns green.
- Docks in older worlds keep their place and what they hold; join track to them to use them again.

## 2026-10-31 · A playtester's notes
- **A machine you upgraded is called by its new Mk everywhere**: on the map, when you look at it, in its panel, on the belts that feed it and in Analytics (P), which now lists each Mk on its own row. Before, an upgraded miner still said "Miner Mk1" on the map.
- **A worked-out deposit stays on the map, crossed out in red**, and a miner standing on one shows as a red-crossed square, so a dead outpost is easy to spot. Before, the deposit's mark simply vanished. Deposits your miners stand on are marked even if you never scanned them. A "Worked out" chip hides them.
- **Power statistics per grid**: with two or more separate power grids, the Power tab of Analytics (P) has a button for each, in the colour its poles wear. Pick one to see only that grid: what it makes, from which sources, which machines use it, and whether it runs short.
- **The world map has a key**: open "What the marks mean" beside the filters to see every mark and what it stands for.
- **Belts can bend where you want them to** (Shift+R on a belt): a bent belt takes items in from its left or right instead of its back, so it can start right beside a constructor, assembler or splitter, pick up its output and carry it along the machine, or turn just before a ramp. A belt simply running past a machine still takes nothing from it. With nothing pointed at, Shift+R bends the first belt of the next line you lay. Corners are now drawn curved, and items ride round them.
- **Signs for notes** (Building section: 4 planks, 2 sticks): a board two blocks wide on a wall, or on two posts in the open. Right-click it to write up to four lines and pick the board's and the letters' colours on a colour wheel. Everyone in a co-op world sees what you write.
- **Sort groups your backpack by kind**: buildings first (in build-menu order, each Mk next to its Mk1), then tools, building blocks, ingredients, raw materials and natural blocks. Storage boxes sort the same way. The hotbar still stays as you set it.

## 2026-10-30 · Trains on solid ground
- **Rails lie on the ground now.** A rail node stands on a solid block: aim at a wall or into the air and it drops to the ground below. The track between two nodes may sink a little into a step or lie up to a block over the ground, where a gravel bed fills the gap, and needs two blocks of room above it.
- **Where a track would float, the blocks to fill are outlined amber; where the ground is in the way, the blocks to clear are red**, and the label counts them. Track you already laid stays as it is.

## 2026-10-30 · Rates where you build
- **Every machine says what it takes and makes a minute at full speed**, in its panel and when you look at it, whatever it is doing: "Full speed: takes 40 Iron Ore, makes 40 Iron Ingot a minute". The numbers are the same ones Analytics (P) measures.
- **A belt says how many items it moved** over the last ten seconds and how much of its most that is: "Moves 36 items a minute, 21% of the 171 it can carry". The build menu lists each belt's most too.
- **The item guide says how many machines feed how many**: "3 Smelter to 2 Constructor (Iron Ingot)".

## 2026-10-30 · Blueprints you can recognise
- **Each blueprint in the library (L) now shows a picture of it from above**: machines as their icons on their footprint, belts as tracks with an arrow the way items go.
- **The card lists what the blueprint holds** ("3 Smelter", "12 Conveyor Belt"), marks in red what you carry too little of, and says what is missing to build it all.
- **Blueprints count pieces, not machines**, since belts and poles are copied too.

## 2026-10-30 · Blueprint keys that do what they say
- **Z always marks a corner.** Two presses aimed at blocks make a box, even when a box was already marked for you: pressing Z used to clear that box first, so the second press only marked one corner. Z aimed at the sky clears a box.
- **Leaving ghost mode forgets a half-marked box**, so the next time you press Z it starts fresh instead of finishing an old corner.
- **The label always says what Z, Enter, R and right-click do next**, also right after copying or stamping.

## 2026-10-30 · Machine icons you can tell apart
- **Sixteen machine icons now look like the real thing** instead of a cube: a cooling tower's waisted chimney, a steam turbine's rotor, a pumpjack nodding over its well, a hoist's headframe, a winch with its hook, a robot arm for the constructor, a coiled power cable with its plug, and more.
- **Shift-upgrading a belt line now upgrades its underpasses too**, both ends of each pair, and the label counts them (an underpass takes two kits).
- **An upgrade kit only raises a machine to its own tier.** Holding the button on an underpass with green kits used to spend your other kits and climb all the way to Mk4.
- **Higher tiers share their machine's icon**, so a Constructor Mk3 looks like a constructor too.
- **Some round parts reappear in icons** (gears, motors, servos, satellites): they had gone missing when drawn after a flat part.

## 2026-10-30 · Round pipes and fixes
- **Pipes are round**, with a steel flange at every joint and a hub where they branch. Joined pipes no longer flicker as you move: they used to draw the same surface twice where they met.
- **Copper wire is a round spool** now, with an axle hole and a loose end.
- **The map's filter chips are back in the side panel.** They had piled up over the close button in the top right corner.
- **Choosing an item in a filter now shows at once** which one is picked, and its line updates.

## 2026-10-29 · Show on map
- **Analytics can now show you where your machines are.** In the Machines tab (P), every kind of machine and every one of the slowest machines has a **Show on map** button. It opens the map with each machine of that kind circled: green at full speed, amber when slowed, red when badly held back, a grey dashed circle when it has nothing to do. A strip at the top counts what holds them back, and resting the mouse on a circle says how fast that machine runs, what stops it and where it stands. From a slowest machine, the map centres on that one. Thanks to a playtester for the idea.

## 2026-10-29 · Round things are round
- **Canisters, wafers, silicon, fuel cells, gears and other round things are now real cylinders**, in the world and in their icons, instead of crossed boxes. Canisters look like oil drums with ridges, a wafer is one disc with its chips on top, and silicon is a crystal lying on its side. Round tanks, columns, boilers and the pump's casing on machines, and the steam engine's boiler and wheels, are round too.
- **The long-distance pole is now a lattice transmission tower 2×2 and 4 high**: legs that lean in, cross bracing, two levels of arms with insulators hanging from them, and the wire on its peak. Towers you already built grow to the new height.
## 2026-10-29 · Turning product displays
- **A machine's product now stands on a small plate on top of it** and turns slowly while the machine works, so you can see what it makes from every side, not just the front. Tall machines (the refinery, cracker, centrifuge, chemical plant and others) carry it on top of their tallest tank or column, where it shows from far away. The plate stops turning when the machine stops.
- The smelter's chimney is a little shorter so it doesn't hide the product beside it, and the crusher has a bar across its hopper for its display.

## 2026-10-28 · Bigger icons
- **Every item icon now fills its box.** Tools, parts, canisters, science packs and other small items used to sit tiny in the middle of their slot; each icon is now sized to its own shape, so a pickaxe or a screw is as large as a block and much easier to recognise. Torches and saplings show as their picture instead of a see-through cube.

## 2026-10-28 · See what a machine makes
- **Every machine that makes items now shows its product** on a small light sign: on the roof of smelters, constructors, crushers, assemblers and the other machines up to the chip fab, or on the front of the tall ones. It shows the recipe you chose, or for a smelter, crusher or washer what it made last; an empty sign means no recipe yet. No need to click a machine to know what it makes.
- **The Mk is easy to see:** every machine that comes in tiers wears one pale stud per Mk on its sides, on a band in the Mk's colour (one stud for Mk1 up to five for Mk5). Storage boxes, labs and generators show their Mk for the first time. Poles wear pale rings, underpasses studs on their hood, and a belt run shows its Mk every six blocks.

## 2026-10-26 · Belt ejector
- **Belt icons look like belts.** Belts, belt lifts, underpasses, splitters, filters and the ejector now show their real shape in the hotbar and menus (a flat belt, a caged lift, a belt running into a hood) instead of a cube, so they are easy to tell apart. Belts and underpasses lying on the ground are flat too.
- **New: the Belt Ejector**, unlocked with Underpasses. Put it at the end of a belt, facing the way the belt runs, and it drops what arrives onto the ground in front of it (up to ten items a second; the belt backs up behind it). It needs no power. It looks like a funnel feeding a brass tube, and items fly out of the tube's mouth in a small arc.
- **Items lying on the ground now merge into stacks** when they sit still close together, so a busy ejector (or a long day of mining) leaves a few piles instead of hundreds of single items. Tools never merge.

## 2026-10-26 · Poles on walls
- **A power pole placed against a wall is bolted to it** (a plate and two struts) instead of standing in the air. Aim at the side of a wall block and place the pole in front of it.
- **A pole now needs the ground under it or a wall beside it.** In open air the spot turns red and the card says why; nothing is placed. Poles you already placed stay as they are.
- **"What's new" shows bold text properly** instead of the stray `**` marks.

## 2026-10-26 · Find out how to get anything
- **The build menu's search now finds every item, not only what you craft by hand.** Type "concrete" and it shows up with the items made in machines or found in the world, on a last page.
- **Click one, or any recipe you cannot craft yet, for its page in the new item guide:** every way to get it (by hand, in which machine from what and the research it needs, or where in the world to dig: how deep, in which lands, how to spot it) and what it is used to make. Every item on a page opens its own page, so concrete leads to quicklime, quicklime to limestone, and limestone to where to dig; Back retraces your steps.
- **A missing material on a recipe's card now says where it comes from** ("made in a Smelter", "found in the world").
- **Search results come best first:** what you typed by name leads (Concrete before the Blast Furnace made from it), in one list that fills the page, even if research still locks it. The best match is lit: press Enter to open its page, or the down arrow to step into the results.

## 2026-10-25 · Ask before closing the tab
- **Settings > Saving has a new option, off by default: ask me to export my worlds before the tab closes.** For browsers that delete site data when you quit. Closing the tab then shows your browser's "leave site?" prompt; choose to stay and the game offers to export this world or all worlds as `.ocworld` files. Such a browser also forgets the option, so turn it on again each visit.

## 2026-10-25 · Steadier panels, shorter craft list
- **The storage box screen keeps its size** while you move the mouse over slots: the controls tips along the bottom wrap instead of widening the panel.
- **A long hand-craft queue shows its first three orders** and a "+6 more queued" count instead of covering the screen; the rest run in turn.

## 2026-10-25 · Playtester fixes
- **The selected item's name shows above the hotbar** when you pick a slot or the item in it changes, then fades.
- **Power poles stand two blocks tall**, with their wires hanging from the top, and have a proper icon: a post with a crossarm, wearing its tier's stripe (the long-distance pole shows its lattice tower). Aim at any part of the pole.
- **A placement that cannot happen now says so.** When you are standing where a block or machine would go, the spot turns red, the card reads "step aside", and a low double knock plays (large machines already showed their red cells; now they knock too, and single blocks like smelters tell you as well).
- **Drag a stack out of the inventory window and let go outside it to throw the whole stack.**

## 2026-10-25 · A truer "machine stopped" prompt
- The floating mouse prompt over a stopped machine now shows only when a right-click really helps (feeding, emptying or fuelling something that opens). A miner without power no longer says "right-click": its card tells you to wire it to a pole.

## 2026-10-25 · Doors, roofs and more for houses
- **Wooden and iron doors** you open and close with right-click (crouch to place something against one instead). Stack two for a doorway: they swing together, and an open door can be walked through. A door will not close on you.
- **Roof tiles** in red, slate and green (stone bricks plus a little iron, coal or copper), **thatch**, **plaster**, **timber frame**, **wood floor**, **windows** and **chimney bricks**, all in the build menu's Building section.
- They are whole blocks for now: sloped roofs, stairs and slabs need a different kind of block shape and come later.

## 2026-10-25 · A torch in your hand, and a second hand
- **A torch or lamp in your hand lights the way**: the ground, machines and other players around you brighten in the lamps' warm colour, and your avatar holds it (look at yourself in third person with V). It shines through walls, like a candle through a thin sheet.
- **A left hand.** Press **J** to swap the selected slot with your left hand (the dashed slot beside the hotbar, also in the inventory screen). The torch keeps glowing from there while you work with a tool in your right hand.
- A tool in your left hand breaks the blocks it is made for when your right hand's tool is not: a pickaxe in one and an axe in the other, and each wears down as it is used.

## 2026-10-25 · Flying is for creative worlds
- **The F key flies only in creative worlds.** In a normal world you get off the ground with the jetpack or the hover pack (worn, and the hover pack charged). A world saved in the middle of a flight comes back on foot.

## 2026-10-25 · Coal miners take coal from belts
- A belt that ends at a **Coal Miner** now feeds it coal ore or logs, so one coal line can keep a whole row of them running. It still takes only fuel, and only up to one full stack.

## 2026-10-25 · Take apart what you built by mistake
- **Right-click a recipe in the build menu** to take apart one of that item you hold: you get back about four in five of each material, and some of the lost iron as scrap metal. The recipe's card says exactly what you get.
- Made a Miner when you meant a Coal Miner? Taking it apart gives back enough plates and rods for the Coal Miner.
- Tools cannot be taken apart, nor the few things a machine makes more cheaply than you can by hand.

## 2026-10-25 · Coal miner: a drill before electricity
- A new **Coal Miner** (4 iron plates, 3 iron rods, 8 stone: no copper) drills ore without any power. Right-click it to give it coal ore or logs; it burns them only while it digs.
- It is the bridge: slower than a Mk1 miner (a third of the ore) and wasteful (it keeps 40% of what it draws), and it cannot be upgraded. Put it on coal and it keeps a few coal back to fuel itself.

## 2026-10-24 · Told when a machine stops
- If a machine near you has stood stopped for half a minute, a **Machine stopped** card says why and what to do (no power, nothing to work on, output full, out of fuel, no water, no ore, or overheated), and a **Check** sign hangs over the machine.
- Each reason is explained **once**: when the machine works again, or when you press **H** on the card, that reason is counted as learned in that world and never shown again. A full machine just waiting is normal in a big base, so it is never nagged about twice.
- There is a quiet minute after each card. The opening's own furnace is left out, and worlds you saved earlier show none of these cards.

## 2026-10-24 · The first tips stay done
- The opening tips ("put the belt back", "turn it", "take the ingots") no longer come back when you later take those belts away to use somewhere else. Once you have mended the line, they are done for good in that world.
- The same goes for the numbered tips: once a tip is done, removing a machine or belt to reuse it never brings it, or an earlier one, back. In a shared game the world remembers the furthest anyone got.
- Worlds you saved earlier load as before; if you had already taken the opening's belts away, those tips stay away. The numbered tips start counting from what you have built when you next load.

## 2026-10-24 · The minimap always comes back
- Hiding the minimap with **N** now lasts only until you start a game or reload; it no longer stays hidden in every world you open afterwards. If anything else ever hides the map while it should be showing, it is put back straight away.

## 2026-10-24 · Point at an item on a belt

- Aim at an item riding a belt and the box names it, with an outline round that one item. Below the name it says **how long the item has been on belts**, **which machine put it there** (a miner, a quarry, a storage box or a machine that made it) and **where that is** from the belt you are looking at, in blocks east or west, south or north, up or down. For a machine's product it also says what the machine makes it from.
- Press **I** to pick that item up into your pack. The readout shows the key whenever you are aiming at an item; if your pack is full the item stays on the belt.
- **Click an item to keep it selected**, which makes a moving item easy to read: its box stays up (with an amber outline) and follows the item along the belts, even when you look away. Click another item to select that one, or click anywhere else to let go. A selected item you are within reach of can still be picked up with **I**; out of reach it says so. The box tells you if the item has been picked up or taken into a machine.
- An item that has waited a long time at the end of a line tells you the line is backed up. A splitter or filter keeps an item's story; a machine that takes it in and makes something new starts a new one.
- Worlds you saved before this update load as before; the items already on their belts say their origin is unknown, but still count their time on belts from now on.

## 2026-10-22 · A sickle to clear the plants

- New **sickle** (two iron ingots and two sticks). Hold it and right-click the ground to cut the grass, flowers, ferns, reeds, dry brush, lichen and ore-hint plants of a 3 by 3 patch; hold the button to sweep across a meadow. A green outline shows the patch before you cut.
- Cut plants leave nothing behind and do not grow back, so a cleared spot stays clear, even if you dig the ground away and put a new floor there. Trees, leaves and logs are not touched: chop those as before.
- In a shared game everyone sees the same cleared ground, and a world you saved keeps it. Worlds you already have load as before.

## 2026-10-22 · A telescope for your pins

- New **telescope** (a few glass lenses, iron plates and rods). Hold it and right-click to look through it: the view zooms in gently, the wheel changes the zoom, and left-click drops a pin on the ground you are looking at (an empty note, ready to name on the map).
- While you look through it, every pin you have is a tall coloured column of light, even through fog and from thousands of blocks away. Lower the telescope and they are gone.

## 2026-10-22 · Hover the map, filter the map

- Rest the mouse on anything on the map or the minimap and a small box tells you what it is: the ore and kind of a deposit you know of (never how much is left), a machine, a town and whether it is growing, short of upkeep or dormant, a terraforming site, a satellite's cover, or your pin's kind and note. On the minimap it works while the cursor is free, in a menu or a panel.
- New "Show on the map" chips beside the world map hide or show machines, towns, sites, satellite cover, each ore and each kind of pin. They apply to the minimap too, the pin list follows them, and they are remembered.

## 2026-10-20 · Pod channels you can name

- Right-click a pod launcher or a landing pad to pick its channel from six colours. A pod goes to the nearest pad on the same colour, whichever way either one faces. The map shows each in its colour, and so does the mark on the machine.
- Worlds you already have keep their routes: the four directions become blue, orange, pink and green.

## 2026-10-18 · Items you can hold

- About 70 items that were a box with a picture on each side are now real 3D models, on the ground, on belts, in your hand and in the inventory: canisters are painted drums, drones have arms and rotors, packs have straps and pockets, the locomotive and wagon are small copies of the real trains, satellites spread their solar wings, and heaps of crushed ore are heaps.
- Plastic is clean white sheets instead of something that looked like styrofoam.

## 2026-10-18 · A new look, and quieter water

- Every block, machine and item is repainted in hand-made 128 × 128 art: glowing windows, lamps and belts, granite with real crystals and mica, rock stains that look like stained ground. Add ?art=off to the address to see the old look.
- Stone, dirt, sand, grass and rock no longer show a repeating grid on large floors and walls: each block's pattern is turned its own way.
- Rivers and the sea are much quieter, and the sound of water together never passes a low limit, so walking along a river no longer hisses over everything else.
- Water has a smooth surface now: a spreading flow thins out in even steps and slopes down to a thin film instead of stepping in three coarse levels, and the gaps where the sides of water were missing at the edge of lakes are gone.
- You can tell still, flowing and falling water apart: still water is calm and glassy, a flow streaks along the way it runs, rapids churn with white foam, and a waterfall pours down in pale streaks. Thin water is clearer, so the ground shows through the end of a flow.

## 2026-10-17 · Sharper plants, a command line, livelier rain

- New research, Architecture (after Masonry and Assembly): 14 blocks for building something that looks good, made in machines. Assemblers tint concrete red, green, black, white or yellow with a pinch of iron ore, copper ore, coal, quicklime or sulfur; smelters fire basalt, granite, sandstone and slag into tiles and bricks; assemblers make framed glass, metal panels, catwalks, railings you walk through, and panel lights.
- Grasses, flowers, ferns, reeds, saplings and the plants that mark ore are drawn in four times the detail: single blades, petals and leaves.
- Press / (or the backquote key) for a command line: weather rain, weather snow 0.5, weather auto, time noon, time dusk, tp 0 100 0, give iron ingot 20, and help for the rest. Press Enter on an empty line to go back to playing. Time and teleport are for worlds of your own.
- In the "By place" music mode the quiet between pieces is now 10 to 30 seconds instead of 2 to 5 minutes.
- The rain no longer sounds like a loop: it gusts and eases, and single drops land at random, higher in the open and duller under a roof.

## 2026-10-17 · Rain and snow

- Rain and snow now fall. Rain streaks down and darkens the ground, and you hear it patter; in snow the flakes drift and snow settles on the ground in patches. Nothing falls indoors, under a roof or in a cave, and the rain on a roof sounds muffled. Birds and crickets go quiet in the rain.
- Settings > Sky has a new switch: Rain and snow (Off, Low or Full). It needs Weather on.

## 2026-10-17 · Clouds and weather

- The sky has weather now: clear most of the time, then clouds roll in, the day turns gloomy and dim, and rain or snow is on its way. It is the same for everyone in a shared game, and it follows the seasons: highlands and cold winters bring snow, deserts stay dry. Stars go out behind the clouds.
- Dusk and dawn glow red and orange, and the clouds catch the colour.
- Settings > Sky has new switches: Weather (Off, Low or Full) and Red dusk and dawn. With both off the sky looks exactly as before. Rain and snow you can see and hear come next.

## 2026-10-17 · The real night sky

- The night sky is now the real one: about 9,000 true stars, the band of the Milky Way with its dark dust lanes, and a moon that goes through its phases (a full moon every eight days). The stars drift through the seasons, so each stretch of nights shows different constellations. Look up from a hilltop.
- Settings has a new Sky section: pick Classic (the old sky), Stars, or Stars and Milky Way, draw faint lines for the constellations, and switch the twinkling off.

## 2026-10-17 · A quieter world

- Wind, water and surf no longer pile up into a roar: the ambient sounds are quieter and share a ceiling, so wind on a hilltop above a river is no louder than the wind alone. Running water now fades out much sooner as you walk away from it, and the sea is heard only near the shore. Crickets and birds call a little less often.

## 2026-10-17 · A door of rings

- Some of the far ruins hide a sealed vault at the foot of their shaft: a heavy iron door you cannot break, with six rings to turn. Right-click it. The far logbooks hint at how the rings work. Behind it waits a small store of parts. New worlds only.

## 2026-10-17 · Ruins of the last factory

- New worlds hold ruins: a cracked concrete slab with rusted machines, girder poles, a broken run of plating where a belt once ran, and a collapsed shaft that drops about ten blocks into a chamber of spent rock. The first one lies 150 to 300 blocks from where you start, and roughly one more in every 500 by 500 blocks of land. Break the rusted blocks to salvage scrap metal (machines may hold a circuit too); a smelter melts three scrap into an iron ingot. Worlds you already have keep their ground.
- Every ruin has a logbook on its slab (a green book you cannot break). Right-click to read it: a page points you to the nearest ground that holds an ore, with a button that pins it on your map, or gives a recipe tip; the far ruins also tell the story of the people who built them.

## 2026-10-17 · Music for every place

- The music now follows where you are. Night, caves, deserts and highlands each have a piece of their own, composed for them (Lantern Hours, Deep Seam, Salt Road, High Pasture); the two Open Ground pieces play by day and at the coast. A piece plays now and then with quiet minutes between, and the music waits until you have stayed somewhere for a few seconds before it changes, so a walk along a shore never makes it jump. On the Sound page you can pick By place (the default), Always calm or Always adventurous.

## 2026-10-17 · The world has a sound

- The world now has sounds of its own, with their own volume slider and mute button on the Sound page (Ambience). Wind picks up as you climb and in the open, and blows dry over deserts and basalt fields; the sea has surf and rivers rush; caves have a low hum and dripping water; birds sing by day near trees and crickets chirp at night. Working machines nearby quiet it all.

## 2026-10-16 · A better first minute

- Plants now show what lies below. Over rusty soil grow rust-orange blooms (iron), over dark soil dark shrubs (coal), over green soil tall teal bells (copper) and over pale soil pale stars (limestone and quartz). The ore guide says so too; point at the soil under one to see how deep the ore is.
- New worlds have forests that differ by place: birches among the oaks on the plains, low broad trees in the lowlands, dark pines on the highlands, cacti in the deserts and bare dead trees on the basalt fields. Birch and pine leaves leave their own saplings, which grow into their own trees. Worlds you already have keep their trees.
- The ground now has plants. Meadows grow grass tufts and clusters of daisies, poppies and bluebells; lowlands are full of ferns; highlands have tufts and bluebells, and lichen on the granite; deserts have dry brush on the sand; basalt fields have lichen and dry brush; reeds stand wherever the bank meets water. They are only for looking at: you walk through them, nothing drops, and they follow the ground when you dig or build. In worlds made before biomes existed there are none.
- New worlds have more natural water: rivers wind through the land, and where the ground drops they spill down as rapids and small waterfalls instead of standing behind dams. Lakes, ponds and the stained soil over ore have irregular shapes instead of circles. Worlds you already have keep the rivers they were made with.
- New research on blue science, Settlements: found your own towns with a town depot. Where you place it decides the town: a harbour on a sea shore, a river town on a river or lake bank, a highland workshop in the granite highlands, a basalt forge in the basalt fields. Towns stand at least 300 blocks apart and away from the spawn plains; while you hold a depot, its outline turns green or red and the label says which town it would found or why none fits.
- Towns grow: each asks for an order of goods (planks, stone bricks and glass first; concrete, steel, lamps, then plastic, circuits and processors later), and a full order builds its next stage, from camp to city. A grown town uses up some of its kind's goods every minute (a harbour planks and glass, a river town stone bricks and lamps, a highland workshop gears and steel plates, a basalt forge coal and quicklime); without them it stalls and its order waits until they arrive. The depot only takes what its town wants, and looking at it shows the order, the upkeep and whether the town is supplied. What towns give back comes in a later update.
- Towns now build themselves: each stage adds a few buildings round the depot, built block by block by the town's own drones from what you delivered. They stand on foundations (on stilts over shallow water), skip steep ground and never touch your own blocks. Each kind builds in its own materials; forges and workshops smoke from every chimney, and windows light up at night. Break a town block and its drones put it back (it drops nothing: it's the town's), unless you fill the gap with something of your own; break a depot and its town falls asleep, keeping everything, until you put a depot back on its marked spot.
- Towns now give back while supplied. A basalt forge takes iron and copper ore and sends out half again the plates and ingots a smelter would; a highland workshop makes hauler packs, jetpacks and gloves half price to craft anywhere and sends worn tools back whole; a river town doubles the reach of drone ports within 128 blocks. A town's goods come out of its depot's front side (onto a belt, or take them by hand); the other three sides take goods in. A stalled town pauses its gift until its upkeep is back.
- Right-click a town depot to see its town: its stage, the open order with a bar for each good, how many minutes of each upkeep good it holds, and its perk. "Take output" takes only what the town gives, never its store. A message tells you when a town stalls (and what it is short of) and when it is supplied again, wherever you are.
- Towns show on the minimap and the map as a little house in their kind's colour, ringed red while stalled and faded while asleep. While you hold a town depot, the map shades the explored ground each kind of town fits, with a legend. Three new tips, once you research Settlements, walk you through founding a town, feeding it and supplying it from far.
- Harbours now take on contract work. Right-click a harbour's depot and pick a product your machines have made (from at least two raw materials, never bauxite or uranium): the harbour takes three times its raw materials, as ore, stone, sand and logs, and sends the product out of its front. It works at the pace of its upkeep, faster as it grows; harbours offer only products your machines make from now on.
- Building the same machine a third time close by now offers a shortcut: a tip shows how to copy a working group as a blueprint and stamp it again, with a box already marked around your previous copy (press B, then Enter).
- Some labels showed garbled characters instead of a dot or a times sign (the blueprint label, the quarry and a few research texts); fixed.
- Placing a long-distance pole now shows whether it will connect before you build it: the outline turns green or red, a wire runs to the pole it will join, the label says how far it is and how many blocks of reach you have left, a light pillar marks the far tower even through the haze, and you hear a soft blip when you step past the edge of its reach. While you hold one, the minimap and the map ring every long-distance pole's reach.
- A new world starts in front of a small production line that has stopped: one belt came loose. Put it back, turn it the right way with R, and the line runs again; its last load of ore becomes the ingots for your first miner. Floating key prompts show what to press where.
- Tips are bigger and easier to skim: a title, the important words highlighted and keys drawn as keys.
- The tip for placing your first miner now also says where to look for ore.
- Mining ore by hand now shows what you kept and how much was lost with each block, and after a few blocks a warning sums it up and points you to a miner.
- A machine's panel now opens next to your inventory: click a held stack into a slot, Shift-click a stack to put it in, click a slot to take it.
- New tips for the space age: building a launch site, what each kind of satellite does, and sending drop pods.
- Rockets roar as they lift off (you hear them from far away) and a toast says when a satellite is in orbit.
- Drop pods now rise out of their launcher, thud down on their pad in a cloud of dust, and each has its own sound.
- On the maps, pod launchers and landing pads show an arrow for their channel, and launch sites show as a rocket.
- Fixed: the research, analytics, map, blueprint and site screens lost their window frame and showed up broken.
- The launch site's recipe list names each rocket by the satellite it sends up (it used to show only the empty canisters).
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
