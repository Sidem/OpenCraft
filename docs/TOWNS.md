# Towns

The player's manual for towns (Milestone 13): founding one, feeding it, what it gives back and how to watch it.
Controls and crafting are in [PLAYING.md](PLAYING.md), the factory in [FACTORY.md](FACTORY.md). The numbers come
from `crates/engine/src/settlements/`; keep this file in sync when they change.

## Founding a town

- **Research Settlements** (blue science: red, green and blue packs, 100 units). It unlocks the **town depot**, a
  3×3 block 2 high, crafted from 40 stone bricks, 40 planks, 8 glass and 10 steel plates.
- **Where you place the depot decides the town.** A **harbour** on a sea shore, a **river town** on a river or lake
  bank (water within 6 blocks), a **highland workshop** in the granite highlands, a **basalt forge** in the basalt
  fields. Where a spot fits two kinds, water wins: a basalt shore is a harbour.
- Towns stand **at least 300 blocks apart** and never within 200 blocks of spawn.
- While you hold a depot, its outline turns green where a town fits and red where none does, and the label says
  which town it would found or why not. The map (M) shades the explored ground each kind fits, with a legend.

## Growing it

- A town grows through five stages: **camp, hamlet, village, town, city**. Each stage has an order; delivering all
  of it builds the next stage. Every kind asks for the same orders:

| For the    | Order                                                                          |
| ---------- | ------------------------------------------------------------------------------ |
| Camp       | 400 planks, 400 stone bricks, 200 glass                                        |
| Hamlet     | 600 planks, 800 concrete, 400 glass, 200 lamps                                 |
| Village    | 2,000 concrete, 1,000 steel beams, 800 glass, 200 lamps                        |
| Town       | 4,000 concrete, 2,000 steel plates, 1,000 plastic, 1,000 circuits              |
| City       | 8,000 concrete, 4,000 steel beams, 2,000 plastic, 1,000 circuits, 1,000 processors |

- **Feed the depot** with belts into its back and sides, or right-click it and put goods in. It takes only what
  its town wants: two minutes of upkeep, plus what the open order still misses. Its store holds 24 stacks.
- Each stage adds a few buildings round the depot, built block by block by the **town's own drones** from what you
  delivered. They stand on foundations (on stilts over shallow water), skip steep ground and never touch your
  blocks. A broken town block is rebuilt and drops nothing; fill the gap with your own block to keep it.
- Breaking the depot puts the town **to sleep**, keeping everything; put a depot back on its marked spot to wake it.

## Upkeep and stalls

- From the camp on, a town **uses up goods every minute**: 20 at a camp, 60 at a hamlet, 150 at a village, 300 at
  a town and 600 at a city, shared over its kind's goods (two basics first, one better good added a stage):

| Kind              | Goods, in the order they join                                       |
| ----------------- | ------------------------------------------------------------------- |
| Harbour           | planks, glass, lamps, concrete, plastic, circuits                   |
| River town        | stone bricks, lamps, glass, steel plates, motors, circuits          |
| Highland workshop | gears, steel plates, motors, steel beams, circuits, processors      |
| Basalt forge      | coal ore, quicklime, stone bricks, steel plates, motors, circuits   |

- If a minute's upkeep is missing, the town **stalls**: its order stops filling and its perk pauses until the
  goods are there. A message tells you when a town stalls (and what it is short of) and when it is supplied again.

## What towns give back (perks)

Each perk works from the camp on, while the town is supplied. Goods a town gives come out of the depot's
**front** (onto a belt, or take them in its panel); the other three sides take goods in.

- **Basalt forge:** iron and copper ore in, half again a smelter's yield out: 4 iron ore make 3 iron plates, 4
  copper ore 6 copper ingots. It works through its upkeep's number of ore a minute (20 at a camp, 600 at a city).
- **Highland workshop:** hauler packs, jetpacks and fabricator gloves cost **half their inputs** to craft by
  hand anywhere; worn pickaxes, axes and shovels put into the depot come back whole, a quarter of its upkeep rate
  a minute (5 at a camp).
- **River town:** drone ports within 128 blocks reach and fly **twice as far** (construction and cargo drones).
- **Harbour (contract work):** pick a product your machines have made in the depot's panel; the harbour takes
  **three times its raw materials** (ore, stone, sand, logs and the like) at its upkeep's rate and sends the product
  out of the front each time it is paid for. It offers only products made from at least two raw materials, never
  tools, and never anything whose chain needs bauxite or uranium. Picking another product starts the payment over.

## Watching your towns

- **Right-click a depot** for its town: stage, the open order with a bar a line, how many minutes of each upkeep
  good it holds (red when short), its perk, and a harbour's contract. "Take output" takes only what the town gives.
- **The maps** show each town as a little house in its kind's colour, ringed red while stalled and faded while
  asleep.
- Once Settlements is researched, three tips walk through founding, feeding and supplying a town from far.
