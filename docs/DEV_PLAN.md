# OpenCraft development plan

**Status:** 2026-10-16 · Milestones 1–11, the less-hand-crafting interlude and Milestone 12 (Orbit: 12.1–12.4 and the orbit
polish) are built and pushed · **The play-test response (section 4)**, from Claude's whole-game review (adopted in full) and
the user's 32-hour play-test (drones, trains and blueprints researched but never needed): R1 talked, R1a and R1b built; R6
talked: the player founds towns, a perk per town kind, stalled towns pause their perk, the megaproject is a space elevator ·
**Milestone 13 (Settlements) built and pushed** except 13.6b (town kinds and sites, the depot, orders, upkeep and stalls,
buildings by the town's own drones, the four perks with harbour contract work, the town panel, maps and tips; save
version 53) · **R1c built and pushed** (generator version 8: winding rivers that spill down as rapids and waterfalls,
irregular lakes, ponds and stains) · **R2 built** (R2a ground cover for every biome, R2b tree species on generator version 9 (not pinned until released), R2c indicator plants over stained ground; 883 tests) ·
**R3 built** (ambient sound; 889 tests) · **R4 built** (music by context: a tested director, streamed Opus pieces, four new pieces composed for night, caves, deserts and highlands; 889 engine tests, 13 web tests) · **R5a and R5b built** (ruins on generator version 9 with rusted blocks, scrap metal and logbooks); **next: R5c** (the vault, if wanted), then R7; 13.6b (balance) once the user has played Milestone 13 · Then R5c, R7, R8, the feel-and-feedback interlude,
the trimmed carry, Milestone 14 (the space elevator)
· Art work continues from `main` (`docs/ART_HANDOVER.md`).

> **This project is written entirely by AI coding agents.** Every session starts cold, and every line an
> agent has to read costs tokens and time. **Keeping the codebase small, modular and cheap to read is as
> important as any feature.** A sprawling codebase makes every future feature slower and more expensive,
> and that cost compounds. The rules in **section 3.1 are mandatory** and override convenience. When a
> change would break them, restructure first.

This is the plan of record. It says where the game is heading, what exists today, and exactly what to build
next. README.md is the project brief and setup; `docs/PLAYING.md`, `docs/FACTORY.md` and `docs/TOWNS.md` are the player's manual;
this file covers direction and work.

---

## 0. Handover: read this first

You are picking up a working browser factory game (Rust → wasm engine, TypeScript/WebGL2 host), live at
<https://sidem.github.io/OpenCraft/>.

- **Milestone 1 (Foundation) is done** (section 8): a fixed 60 Hz tick, a deterministic core changed only
  by actions, several players in the engine, state hashes, and worlds that save in the browser.
- **Milestone 2 (Make it a game) is done** (section 8): items, smelter, constructor, belt logistics, power,
  research with science packs, Mk2 upgrades and onboarding tips.
- **Milestone 3 (Co-op) is done** (section 8): 2–4 players in one world over WebRTC, the host's browser as
  the authority, a Cloudflare Worker to connect them, hosting and joining from the menu.
- **Milestone 4 (Reasons to explore) is done** (section 8): biomes and rock provinces, ores that follow
  geology, surface hints, prospecting, map marks, day and night, sky light and lamps, a new build menu.
- **Milestone 5 (Water and world shape) is done** (section 8): sea and ponds, swimming, flowing water,
  pumps, a quarry, rarer surface ore (generator version 3), and powered miners.
- **Milestone 6 (Industry) is done** (section 8): tiers as data, one processing machine, multi-block
  footprints, upgrade kits, the assembler, steel, blue science, Mk3 everything, steam power, the crusher
  and the silo.
- **Milestones 7 and 8 are done** (`docs/CHANGELOG.md`): circuits, violet science, solar, logic, blueprints,
  construction drones, jetpack and gear; then terraforming by drone (dig, fill, flatten, tunnels).
- **Milestone 9 is done** (`docs/CHANGELOG_M9.md`): bauxite in far biomes, aluminium, bearings to far ground, trains,
  cargo drones and the hover pack.
- **Milestone 10 is done** (`docs/CHANGELOG_M10.md`): oil and canisters, plastics,
  diesel, electrolysis, ore washing, rivers and hydro, hoists, nuclear power, gold science, Mk5.
- **Milestone 11 is done** (`docs/CHANGELOG_M11.md`): chip fab, the data grid, AI datacenters and cooling, AI labs and endless bonus
  techs, the optimizer, laser links with mirrors, the swarm hub, auto-routing, the AI survey.
- **The less-hand-crafting interlude and Milestone 12 (Orbit) are done** (`docs/CHANGELOG_M12.md`).
  **Now: the play-test response (section 4):** R1, R6, Milestone 13 (Settlements, all but its balance step 13.6b), R1c
  (natural rivers), R2 (plants: ground cover, tree species, indicator plants), R3 (ambient sound) and R4 (music by context) are done, R5a and R5b (ruins, scrap, logbooks) too; **next is R5c (the vault)**, then R7, R8.
- **Tech tree:** `docs/TECH_TREE.md` is the concept (lines, links, the far end, upgrades, and the content
  architecture every step follows: its section 8); `docs/TECH_ERAS.md` has each era's items, recipes,
  machines and techs. Read the concept once and the era you build.
- **Art** (`docs/ART_HANDOVER.md`) says who owns which looks. When gameplay needs a new look, append a
  `tex` layer with a plain placeholder pattern and add a line to that file's request list.

Before you change code:

1. Read sections 0–4 of this file (section 3.1 carefully), then `docs/CODEMAP.md`, then the nested
   `CLAUDE.md` of the area you work in. Skim `docs/PLAYING.md` / `docs/FACTORY.md` only if you need the player's view.
2. Run `npm run build:wasm` (if `web/src/wasm` is missing) and `npm run check` to confirm a green baseline
   (about 870 engine tests).
3. Work through the current milestone in step order. Each step lists where, how and when it's done. Do
   one step, or one clean part of a step, per session, and stop in a green, committed state.
4. When a step is done, tick its checkbox here, update the **Status** line at the top, and add a line to
   the change log (section 8). If players will notice it, add a plain-words line to the newest update in
   `web/src/patch-notes.md` (shown in the menu under "What's new"; start a new `## date · title` for a new day). Update `docs/CODEMAP.md` in the same commit as any structural change. Keep
   this file truthful; the next session relies on it.

Ground rules (details in `docs/WORKFLOW.md`):

- **Never push to `main` without asking the user.** Every push deploys the live site.
- Commit only when the user asks. So far the user has had finished work committed and pushed straight
  to `main` when they asked for a deploy.
- Windows machine: use PowerShell and the file tools; the Bash tool fails here.
- **Keep the codebase agent-friendly (section 3.1).** File budgets, tests in separate files, docs as maps.
- Performance comes first, and dependencies must be justified. Explain things to the user in plain language.

---

## 1. Where the project is heading

A browser game between **Minecraft** and **Satisfactory**: a fully editable voxel world that you mine,
shape and industrialise. Not a Minecraft clone; its conventions can be broken freely.

### Pillars

1. **Everything is physical.** Items travel on belts, machines are blocks in the world, the world is the
   material. No magic teleporting storage.
2. **Finite ground pushes you outward.** Deposits run dry after a long time. Local depletion drives
   expansion, and distance, depth and scale become the problems each tier solves.
3. **Terrain is something you engineer.** Cut, fill, flatten, tunnel, flood. Terraforming machines are the
   game's signature feature; no other game in the genre has them.
4. **Automation replaces your hands at every scale:** mining first, then logistics, then construction.
5. **Every tier adds a new kind of problem, not just recipes:** quantity → power → distance → depth and
   terrain → fluids → scale.

### Decisions (settled with the user)

| Date | Decision |
|---|---|
| 2026-09-25 | **Finite but long-lived deposits.** A factory lasts a long time but not forever. |
| 2026-09-25 | **Efficiency model.** Hand mining is lossy, machines are efficient; recovery rate is a progression axis. Numbers will need tuning. |
| 2026-09-25 | **Peaceful** for the foreseeable future. No enemies or combat. A tower-defence-style mode may come much later, as a world setting. |
| 2026-09-25 | **Open-ended.** An optional final construction (e.g. a rocket ship) needing enormous resources and advanced research. Completing it must **not** end the game; it should unlock something. |
| 2026-09-25 | **Built entirely by AI coding agents.** Keeping the codebase from growing unwieldy is of utmost importance; token efficiency and fast iteration drive architecture choices (section 3.1). |
| 2026-09-25 | **Co-op multiplayer must be possible**: other people can join a world. |
| 2026-09-25 | **Research is Factorio-style**: labs use science packs crafted from increasingly complex parts (<https://wiki.factorio.com/Science_pack>). |
| 2026-09-25 | **Upgrades raise recovery**, not just speed (Mk2 miner ≈ 75%), so upgrading extends a deposit's life. |
| 2026-09-25 | **Bulk materials** (stone, sand, clay, gravel) stay effectively infinite for quarries. |
| 2026-09-25 | **Approach for co-op** (proposed by Claude, adopted with this plan): player-hosted over WebRTC; a deterministic simulation core driven by tick-stamped actions (Factorio-style); player movement and loose items replicated from the host. Revisit only if Milestone 3 prototyping shows a real problem. |
| 2026-09-25 | **Co-op hosting on Cloudflare**: a Cloudflare Worker for signalling and Cloudflare TURN as the relay. The user owns the account. |
| 2026-09-25 | **Co-op size: 2–4 players.** This sets the bandwidth and performance budgets. |
| 2026-09-26 | **No TURN relay for now**: STUN only. Players whose networks block direct connections can't join; adding the two TURN secrets later (`docs/WORKFLOW.md` section 4) needs no code change. |
| 2026-09-26 | **Tools wear out and come in tiers.** A better pickaxe keeps slightly more ore by hand, but nowhere near what machines recover (step P5). |
| 2026-09-26 | **Ores follow geology.** Coal, iron and copper are placed by rock type and biome (for example copper in mountains, coal in lowlands); limestone and quartz are added; visible rock types (granite, sandstone, basalt) say what lies beneath. |
| 2026-09-26 | **Deposit rates stay as today** (how many outcrops, veins and lodes); tune after play. Surface outcrops superseded on 2026-09-27. |
| 2026-09-26 | **The world pauses while the game is closed.** No simulating missed time. |
| 2026-09-26 | **New generation rules for new worlds only.** Worlds made before Milestone 4 keep generator version 1 and play as before. |
| 2026-09-27 | **Belts:** ramps come from placement (no research), lines are dragged out, R rotates, research moved to T. |
| 2026-09-27 | **Water and world shape before terraforming** (Milestone 5): sea level, lakes, swimming, pumps and pipes. Rivers later (now Milestone 10). |
| 2026-09-27 | **Water flow is limited and deterministic:** a Minecraft-like spread, event-driven and capped per tick; no volume simulation. |
| 2026-09-27 | **Surface ore is rare and meaningful:** about 10× fewer exposed outcrops, only on bare rock; a starter set near spawn; ores in depth bands. Vein and lode counts stay. |
| 2026-09-27 | **A quarry in Milestone 5:** rock and soil get automated extraction that really digs a pit (the first excavator); it must be intuitive and satisfying to watch. |
| 2026-09-27 | **Order after Milestone 5: terraforming, then blueprints and construction drones, then transport** (personal flight first: jetpacks or hovering, and building from further away). The player should feel a steady, intuitive and fun growth of capability: building by hand → planning → automatic assembly. *Order superseded on 2026-09-28 (Industry first).* |
| 2026-09-27 | **Finding starter ore made easier** (play-test: iron under rusty soil was too deep to find by hand). Generator version 4 for new worlds: shallower bands, coal, iron and copper exposed 1.5× as often on bare rock, two starter patches of each; a world map (M) of explored ground with pins; an ore guide with depth bands; stained soil tells what lies how deep. Mute moved to K. |
| 2026-09-28 | **Glass from sand; brighter lights.** The smelter makes glass from sand (1 → 1); quartz gives 2. Torches reach 11 (full within 4), lamps 31 (full within 17): the light margin is 32, the most the 3×3×3 field allows. |
| 2026-09-28 | **Processed wood and ladders.** Logs saw into planks, planks into sticks; sticks make tools, torches, poles and ladders (a see-through frame: jump climbs, crouch descends, idle holds). |
| 2026-09-27 | **Miners need power; generators burn only what is used.** The first loop is a miner on coal feeding the generator that powers it. Energy is stored per generator (kJ); one coal (270 kJ) runs a Mk1 (5 kW) long enough to mine about 32 coal. The smelter stays a burner. |
| 2026-09-28 | **A deeper tech tree** (user request): each tier is built from the one before and makes every aspect more efficient (extraction, yield, logistics, power, processing, research, reach); machines several blocks big with ports for more inputs and outputs; an easy upgrade system: tier stripes coloured like the science packs (red, green, blue, violet, gold), upgrade kits applied in place, belt lines upgraded by dragging. Design: `docs/TECH_TREE.md`. |
| 2026-09-28 | **Industry is the next milestone (6); terraforming moves later** "where it makes more sense" (supersedes the 2026-09-27 order). Every technology opens a possibility, improves efficiency or helps the player. Targets include AI datacenters, building with drones, satellite constellations, and energy and data sent by laser; the only limit is the stack (Rust, wasm, WebGL2 in a browser). |
| 2026-09-28 | **Modular, open content architecture** (`docs/TECH_TREE.md` section 8): variants such as tiers are data (`tier: u8` into a table), never a `bool` like today's `fast` belt or `mk2` miner, nor a block per variant; processing machines are rows of one generic machine; recipes belong to categories. |
| 2026-09-28 | **The order after Industry** (proposed by Claude, confirmed): 7 Electronics, blueprints and drones (with the jetpack) · 8 Terraforming (its excavator flies Milestone 7's drones) · 9 Distance (aluminium, trains) · 10 Fluids and chemistry · 11 Compute and photonics · 12 Orbit · 13 The megaproject. **Blue packs and upgrade kits from blue on are machine-made only.** |
| 2026-10-03 | **Blueprints are both** copied from built areas and planned as ghosts; ghosts are core state. **Drones take materials from boxes beside their port.** **Drones must be hard and rewarding:** a long research ladder on violet packs and several new parts, not three cheap techs. **Coal jetpack in M7**, hover pack in M9. |
| 2026-10-06 | **Creative mode for new worlds** (user request, for testing everything): a world is normal or creative, chosen in the new-world form and never changed. Creative has every tech done and an "All items" tab in the inventory (click a stack, Shift-click one). Flying (F) already works in every world. |
| 2026-10-06 | **Analytics and machine efficiency** (user request): an Electricity and productivity screen (P) graphs the power the grids could give, the power used and asked for, and each item made a minute, over 1m to 3h, with coloured lines; every machine says how close to full speed it runs and what costs it the rest (low power, waiting for input, full output, no fuel...). **Presentation only:** history is not saved and starts when a world is opened. |
| 2026-10-06 | **One underpass, in tiers** (user request): entry and exit are the same item (pieces in a line facing the same way pair up by themselves, `factory/underpass.rs`); Mk1 to Mk4 reach 4 / 6 / 8 / 10 blocks at the speed of the belt of that Mk; a piece costs half its reach in belts of its Mk (a Mk1 pair is 4 belts). Dragging a belt line over a belt, machine or wall dives under it with the lowest underpass that reaches and is at least the belt's Mk; pairs the inventory lacks show red and are left out. The old exit block (23) still loads and is the same piece. No save-format change. |
| 2026-10-09 | **Hand-crafting speed comes from research and gear, not a skill tree** (Claude's proposal, user agreed): a "hand-crafting speed" bonus tech, a modifier layer `player_bonus(stat)` that sums techs, gear and any later perks (mining speed, reach and carry capacity read it too), and crafting that takes parts from nearby boxes or is fulfilled by drones, so the player stops hand-crafting instead of doing it faster. No XP or skill points. Built as steps L1–L3 (section 4). **2026-10-11: the user dropped "craft from nearby boxes" (old step L4); it will not be built.** |
| 2026-10-09 | **Hauler packs: 27 slots, then 18 more per Mk up to Mk5 (99), raised with 4 kits of the tier** (user request: inventory space was the pain at violet). The worn pack is upgraded in place from the inventory screen. |
| 2026-10-09 | **Underpasses are crafted in pairs; research can be queued** (user requests). A craft makes an entry and an exit; the research screen holds a queue (up to 32, with missing prerequisites added), the HUD shows only the current tech. |
| 2026-10-08 | **Player-experience features** (user's pick from Claude's proposals): a world timelapse, the factory as an instrument, follow one item, seismic prospecting, site bonuses (where you build matters), ruins of the last factory. Placed as two interludes in `docs/ROADMAP.md`; datacenter siting may join steps 11.3–11.4. |
| 2026-10-11 | **The carry: p-adic numbers and fractals by discovery** (user's pick from Claude's brainstorm): mechanisms that behave like the math (splitter addresses, a tally, fractal radiators and antennas, Hensel vaults, a Cantor sieve, a Hanoi crane, a layer printer). Mainly in the ruins, partly in existing eras, no tech line of its own; pure discovery with names only, never explained; primes 2, 3 and 5 each lead where they work best. Design in `docs/ROADMAP.md`. |
| 2026-10-16 | **Order after Milestone 12** (Claude's proposal, user agreed): 1 orbit polish (12.x's "not done" items), 2 the feel-and-feedback interlude (instrument, follow one item, timelapse; then seismic prospecting and site bonuses), 3 the carry's pieces in the existing eras (discoveries, splitter addresses, tally, radiators and analyzer), 4 pick the megaproject theme (talk only), 5 the ruins interlude with the carry's ruin pieces, 6 Milestone 13. Parked items fit in where a session has room. |
| 2026-10-16 | **Claude's whole-game review adopted in full** (user: "I agree with all your assessments"); with the user's 32-hour play-test (section 4) it **supersedes the order of the row above**: next are R1–R8 (section 4), then the feel-and-feedback interlude, the carry's era pieces and Milestone 13. |
| 2026-10-16 | **Needs before techs.** Research outran need in play (drones, trains, blueprints researched, never used). New work must give the player a problem that its tech solves; **no more multiplier layers** (compute, orbit) **and no more perk layers** (like Handcrafting I–III). Depth in the systems that exist beats new systems. |
| 2026-10-16 | **A living world:** plants per biome (ground cover on every world; tree species, desert and basalt plants on new worlds), indicator plants over ore, ambient sound, music chosen by context with silences between pieces. The user supplies new music. |
| 2026-10-16 | **Still peaceful: no combat or PvE.** Pressure comes without enemies (subsidence, mine flooding, storms, pollution; pick in R8). The tower-defence world setting stays a "much later, maybe". |
| 2026-10-16 | **Settlements as customers** (planned with the megaproject, R6): towns order goods and grow in stages, built by the existing blueprint and drone systems; no walking townspeople, no diplomacy or war. **House building** through a machine-made decor set (R7). |
| 2026-10-16 | **Art moves away from Minecraft, shapes before textures:** plants, machine silhouettes, calmer textures, atmosphere (`docs/ART_HANDOVER.md` 2b). Not realism. |
| 2026-10-16 | **The carry is trimmed:** splitter addresses and the tally stay (era pieces); in the ruins at most one vault as an easter egg; the 3-door, the 10-door, the Cantor sieve, the Hanoi crane and the layer printer are dropped; radiators, the analyzer and antenna bands are parked. **Ruins move up** (R5). |
| 2026-10-16 | **Pod channels are chosen by name and colour**, not facing. **Add the TURN relay** (the user adds the secrets; supersedes 2026-09-26). The factory instrument gets few voices, a distance cap, and music quiets under it. |
| 2026-10-16 | **R1: what makes drones, trains and blueprints needed** (user's pick from Claude's levers): **settlements as growing demand** (the main lever; R6's talk comes next) and **a blueprint prompt** when a machine group is repeated (R1b). Not now: smaller ore stacks, far bulk inputs in late recipes (fallbacks if settlements are not enough). |
| 2026-10-16 | **R6: settlements and the megaproject** (user's picks, all Claude's recommendations): **the player founds towns** with a depot at a site that fits the town's kind, far from other towns (works in existing worlds; no worldgen towns); **each town kind gives a perk** while supplied; **an unsupplied town stalls and its perk pauses**, but it never shrinks or loses what was built; **the megaproject is a space elevator** grown from the biggest town, built in phases, unlocking asteroid mining. Settlements become **Milestone 13**, the elevator **Milestone 14**; 12.5 closes. |
| 2026-10-16 | **13.1: town kinds and perks** (talked with the user). Four kinds, one per kind of place: **harbour** (sea shore): **contract work**, taking large lots of basic goods for complex products the player has already made in a machine, at about 3× their raw content, never raw or rare materials and never products whose chain needs bauxite or uranium (user: barter must not make far ores unnecessary); **river town** (river bank): **drone relay**, double range for cargo and construction drones within 128 blocks; **highland workshop** (granite hills): **gear works**, hauler packs, jetpacks and gloves cost less there and worn tools come back repaired; **basalt forge** (basalt fields): **better smelting**, ore comes back as plates with 50% more yield. Towns at least 300 blocks apart, none on the spawn plains; founding unlocks on blue packs after Steelmaking. |
| 2026-10-16 | **13.2: orders and upkeep** (user's picks, all Claude's recommendations). The player's pack (135 stacks) carries any one-off order in a trip, so the steady pull is the **upkeep**, drawn from the depot's 24-stack store. **5 stages** (camp, hamlet, village, town, city; the city is where Milestone 14 can start). **Stage orders double from about 1,000 items** (1,000, 2,000, 4,000, 8,000, 16,000: basics first, then concrete, steel and lamps, later plastic, circuits and processors). **Upkeep grows to a Mk3 belt:** 20, 60, 150, 300 and 600 items a minute by stage; a town stalls once a minute's upkeep is missing and resumes when it is there, and growth pauses while stalled. **Upkeep goods per town kind:** harbour planks and glass, river town bricks and lamps, highland workshop gears and steel plates, basalt forge coal and quicklime; later stages add one better good each. |
| 2026-10-16 | **13.3: how a town grows** (user's picks, all Claude's recommendations). **The town's own drones build it:** the depot keeps a small fleet of its own (more each stage) that builds the town's plan block by block; the blocks are free (the delivered order paid for them), no player box is touched, and the player sees drones at work before researching them. **Foundations, bad plots skipped:** plots in fixed rings round the depot, each building on a stone foundation down to the ground; plots on steep slopes, over deep water or holding the player's blocks are skipped, plots over shallow water get piers on stilts; the terrain is never dug. **One set of buildings in each kind's materials** (houses, a hall, a workshop, a tower; 2 to 4 more a stage); R7's decor joins later. **Breaking:** town blocks break normally and the town's drones put them back; breaking the depot leaves the town **dormant** (stage and progress kept), and a depot placed on the same spot wakes it. |
| 2026-10-16 | **13.4: perks** (user's picks, all Claude's recommendations). **The depot's front becomes an out side** for perk goods (belts or by hand; the other three sides stay in). **The forge's rate grows with its stage** like the upkeep (about 20 ore a minute at a camp to 600 at a city). **Workshop:** while supplied, hauler packs, jetpacks and gloves cost half their inputs in hand crafting anywhere, and worn tools put into its depot come back whole. **Harbour:** the player picks a product their machines have made in the depot panel; built as its own step (13.4b) after the panel. |
| 2026-10-16 | **13.5: the town panel, maps and tips** (user's picks, all Claude's recommendations). **A town section in the usual machine panel** (not a screen of its own). **Town markers on both maps**, coloured by state, and **site shading** on the world map while a depot is held. **A toast when a town stalls and when it recovers.** **Three tips:** found, feed, supply from far. The harbour's product chooser is built with 13.4b (Claude's default). || 2026-10-04 | **Terraforming is done by the drone ports** (no excavator machine); spoil goes into belts and boxes. **Far ground must be findable and reachable** (user): the player gets a rough bearing to the biome that holds an ore (step 9.3), and **trains** carry long-distance cargo (step 9.4; trucks are not planned). |

### Proposed, not yet confirmed by the user

- Old saves keep loading across format changes where a migration is cheap. Every save since version 1
  still loads.
- Terraforming: one excavator machine with work drones rather than separate grader, filler and borer
  machines; dug ground fills other sites first and goes onto belts after (a dump is a fill site); ore in
  a site is cut like hand mining (lossy), with a warning when planning.

---

## 2. Where the code stands (after Milestone 6)

**Milestone 6 added** (the bullets below still describe Milestones 1–5; `docs/CODEMAP.md` is current):
tiers as data up to Mk3 for every family, upgrade kits, one processing machine driven by `ProcessSpec`
rows (smelter, constructor, assembler, blast furnace, boiler, steam turbine, crusher, silo), multi-block
footprints, steel and blue science, steam power as a second kind of power source, 16 techs, 14 tips,
save version 21.

### Architecture

Rust owns all game state and hot loops (`crates/engine`, compiled to wasm with wasm-bindgen). TypeScript
(`web/src`) is a thin platform layer: input, WebGL2 rendering, DOM UI, Web Audio. Bulk data (chunk meshes,
box instances, sound events, textures) is read zero-copy from wasm memory through `*_ptr` / `*_count`
accessors. `Game` (`lib.rs`) is a thin facade: its JS-facing API is split by area into `api/*.rs` and acts
for the local player (`Game.local`). The deterministic core is `Sim` (`sim.rs`: tick, world, factory with
deposits and research, each player's inventory, rng); `Sim::state_hash` fingerprints it through the
canonical encoding in `bytes.rs`, which `save.rs` also reads back for saves. The authority
(`authority.rs`) owns every player's body and the loose items. The local player's hands (mining, placing,
footsteps) live in `interaction.rs`. `Game` changes the core only by queuing `Action`s (`action.rs`),
applied at the next tick; the core answers with `SimEvent`s (`events.rs` reacts). `docs/CODEMAP.md` maps
every module.

Each frame, `web/src/main.ts`:

1. forwards input (`set_move`, `look`, `set_mining`, `set_using`, actions from `input.ts`),
2. calls `game.update(dt)`. This runs streaming, then as many fixed 60 Hz ticks as the frame time adds up to
   (`Game::run_tick`: every body's physics, the local player's targeting, mining and placing, item
   entities, all queuing actions, then the core's `Sim::step`, then `handle_sim_events` for item spawns,
   sounds and toasts), then interpolates the camera between the last two ticks and writes box instances,
3. runs `begin_work()` + `work_step()` under a time budget (generate or mesh one chunk per step),
4. drains mesh and unload events to the renderer, plays sound events, renders, updates the HUD.

**Co-op** is lockstep: only actions travel. The host stamps every action (its own too) for
`tick + INPUT_DELAY` (4) and sends one frame per tick; clients step their core only through the ticks
they have frames for, and compare state hashes every 60 ticks (a mismatch resyncs from a snapshot).
Bodies are each machine's own (states at 20 Hz); loose items run on the host only (views at 10 Hz). The
core never learns about the network: `net/` (Rust) moves bytes, `web/src/net/` runs the session over a
`Transport` (WebRTC between machines, BroadcastChannel between tabs). Budgets for 4 players: under
~10 KB/s per client outside snapshots, and the host's frame under 2 ms more than solo.

### What exists

- **World:** 32³ chunks, 256 tall; seeded terrain with cliffs, caves and trees; greedy mesher with AO;
  streaming nearest-first; edits kept when chunks unload (`World.saved`). Generator versions: a world
  keeps its own (`WorldGen::version`); version 2 adds biomes (`worldgen/biome.rs`: plains, desert over
  sandstone, highlands over granite, lowlands, basalt fields; spawn always plains) and ores by biome
  (`worldgen/geology.rs`, including limestone and quartz) with stained soil hints above veins and lodes.
  Version 3 (frozen) makes surface ore rare (only on bare rock, a starter set 40–80 blocks out),
  puts ores in depth bands (`worldgen/strata.rs`), and adds the sea (level 62) and ponds
  (`worldgen/water.rs`). Version 4 (current) only retunes that: shallower bands, more exposed metal, two
  starter patches each 28–110 blocks out. `worldgen/tests.rs` pins versions 1–3.
- **Water** (block 45 still, 46–52 flowing): drawn blended with underwater fog (`render/water.ts`);
  flow is core state, event-driven and capped per tick (`sim/water.rs`); the sea is the only endless
  water. Bodies swim and items float (`player`, `entities`).
- **Day and light:** a 20-minute day from the core tick (`daytime.rs`), sky, sun, moon and stars
  (`render/sky.ts`); sky and block light 0–15 computed while a chunk meshes (`light.rs`: the chunk plus a
  32-block margin), smoothed per vertex (a byte after the vertices); light sources (`light::SOURCES`): lamps
  (block 44, full within 17, reach 31) and torches (block 57, full within 4, reach 11).
- **Deposits** (`deposits.rs`, placed by `worldgen/ore.rs`): outcrops, veins and lodes of coal, iron and
  copper (100 / 1,000 / 2,000 units per block, shared draw caps 60 / 240 / 1,200 per minute). A pool is
  shared per deposit, output tapers over the last 20%, and blocks turn to `SPENT_ROCK` as it drains, even
  in unloaded chunks. Hand mining keeps `HAND_YIELD` = 3 per block and costs the deposit one block.
- **Factory** (`factory/`, one file per machine kind, the `MACHINES` table and a `Machine` trait; one kind
  can serve several blocks through extra table rows):
  - Miners: Mk1 (1 unit/s, 60% recovery, 5 kW) and Mk2 (2 units/s, 75%, 20 kW). The quarry
    (`factory/quarry/`, 10 kW) digs a box of ground into a real pit; pumps, pipes and outlets
    (`factory/pipes.rs`, `pumping.rs`) move water.
  - Belts (1 block/s; fast belts 2) with side-joins, corners, back-pressure, ramps, lifts and underpasses
    (`belt_shape.rs`); splitter and filter (`router.rs`); storage boxes (24 slots, open like a chest).
  - Smelter (ore plus coal or logs → ingots) and constructor (ingots → plates, rods, screws, wire), with
    machine recipes and fuels as data (`recipes.rs`). Right-click opens a machine panel (`panel.rs`,
    `ui/machine.ts`): status, buffers, recipe or filter choice, put-in and take buttons.
  - Power (`power.rs`): coal generators that store fuel energy and give only what is drawn (up to 60 kW),
    poles that link within 10 blocks into grids, machines on the nearest pole within 5, brownouts as a
    speed factor.
  - Research labs (`lab.rs`) working through the tech tree (`research.rs`, key T) with red and green
    science packs; seven techs unlock routing, lifts, underpasses, green packs, Mk2, fast belts and fluid
    handling.
  - Models are instanced boxes; status readouts come from `describe`.
- **Inventory** (`inventory.rs`): 36 slots (hotbar 0–8), a cursor stack, click, shift-click and
  quick-move. **Crafting** (`recipes.rs`): hand recipes in the build menu (key E, `ui/crafting.ts`): a grid by `recipes::Group` with search, filters (can craft, missing, locked) and a hover card.
- **Onboarding tips** (`hints.rs`, `ui/hints.ts`, key H skips): ten tips from finding ore (pointing at the
  map) to the quarry.
- **Maps** (presentation only): the explored map (`minimap/atlas.rs`: every column seen, kept after
  unloading, stored in the browser beside the save) feeds the minimap (key N) and the world map (key M,
  `ui/worldmap.ts`: pan, zoom, pins). Marks (`minimap/marks.rs`): diamonds for ore seen at the surface,
  rings for prospected veins and lodes, squares for machines; pins are the player's notes (`ui/pins.ts`,
  in the world's record). Beside the map, the ore guide (`ore_guide.rs`, `ui/ore-guide.ts`) charts each
  ore's depth band from the world's own generator; pointing at stained soil tells the ore below and its
  depth.
- **Block timers** (`sim/timers.rs`, core state): leaves of a felled tree decay (half-life 5 s), grass
  spreads onto bare dirt beside it (30 s) and turns to dirt under a solid block (15 s), saplings grow.
  Timers start only from block changes, never from scanning chunks.
- **Saplings** (`sim/saplings.rs`): leaves drop one 1 time in 25 (broken or decayed); planted on dirt or
  grass, it grows after 60 s plus a 90 s half-life into a tree shaped like generated ones
  (`worldgen::tree_blocks`); drawn as crossed quads (`Render::Plant`, mesher faces 6 and 7).
- **Items** (`item.rs`): `ItemId(u16)`. Ids below 256 are the blocks with the same number (0–59, see
  `block/mod.rs`; 29 is the sapling, 30–43 Milestone 4's rocks, ores, glass, stained soils and sand, 44 the lamp, 45–52 water, 53–55 pump, pipe and outlet, 56 the quarry, 57 the torch, 58 planks, 59 the ladder); from 256: iron and copper ingots, iron plate, iron rod, screws, copper wire, red and green
  science packs, the six tools (264–269), scanner (270), core drill (271) and stick (272). Every item is drawn as a textured box.
- **Sound:** procedural foley, 7 materials including metal, and a sound designer (key O).
- **Debug API** on `window.opencraft.game`: `give`, `teleport`, `run_ticks`, `skip_time`, `find_deposit`,
  `block_at`, `toggle_fly`, `set_look`, `add_player`, `remove_player`, `state_hash`, `debug_desync`.
- **Saving** (`save.rs`, `web/src/save/`): worlds autosave to IndexedDB; the menu lists, creates,
  exports and imports them. Save version 17; every version since 1 loads.
- **Co-op** (`net/`, `web/src/net/`, `signal/`, `ui/coop.ts`, `ui/players.ts`): "Play together" in the
  menu hosts the open world (a room code and link from the signalling Worker) or joins from a pasted link
  or code; up to 4 players; a returning player gets their things back (player keys, `Sim.away`). Avatars
  with name tags, a Tab player list with ping, join and leave notices, readable refusals (full, another
  version, no such game, no connection), a hidden co-op tab keeps ticking, a silent peer times out after
  10 s, and a client that differs from the host or falls 5 s behind resyncs in place. URL shortcuts for
  testing: `?host`, `?host=<room>` / `?join=<room>` (tabs of one browser), `?join=<code>`, `?relay`.
- **Tools** (`tools.rs`, recipes at the end of `RECIPES`): pickaxe, axe and shovel in stone (150 uses, 2×)
  and iron (600, 4×, an iron pickaxe keeps 4 ore); a tool's stack count is its uses left, shown as a wear
  bar (`showAmount` in `ui/hud.ts`). Placeholder looks in `textures/tools.rs`.
- **Prospecting** (`prospect.rs`, `ui/prospect.ts`): the scanner lists deposits within 48 blocks (Mk2: 96) (ore,
  tier, live bearing and distance, depth, size band); the core drill (3 s hold) gives a column's exact
  figures. Queries only: no actions, the state hash never moves. Devices are tools that never wear.
- **Size:** about 261 KB gzipped in total (wasm 201.6 KB, JS 51.1 KB, CSS 6.4 KB; `vite build`).

### Known limitations and technical debt

1. Only the local player runs hand interaction; co-op relays mining/using gestures for avatar animation
   alongside body states, and applies the resulting actions through lockstep.
2. Two tabs on the same world overwrite each other's saves (the later save wins).
3. ~~Veins and lodes can only be found by digging~~: fixed by prospecting (4.6) and map marks (4.7).
4. In version 1 and 2 worlds the outcrop nearest spawn may be buried (version 3 has a starter set).
5. TypeScript mirrors a few engine constants: the 6 floats per sound event, and the
   order of sound materials and event kinds. Replace them with getters when touching that code.
6. Item and belt instances aren't interpolated between ticks (only the camera is); optional polish.
7. Balance is untested by real play: pack costs, research times and Mk2 costs will need tuning
   (`docs/WORKFLOW.md` section 6 lists the numbers).
8. In co-op every action applies about 70 ms plus the round trip after it's made; your own block edits
   aren't shown early (section 4, "instant feedback"). A name change applies the next time you host or
   join. Hosting can't be stopped without leaving the page.
9. Co-op over the internet has been tested by the user on two machines (STUN only); the relay is untested.
10. ~~A block timer next to an unloaded chunk generates it for each read~~: fixed in 5.6 (`World` keeps the
    last 8 generated chunks).
11. Light is recomputed per meshed chunk (0.43 ms per chunk in `bench_meshing` after the torch change); an
    edit relights the chunks within 20 blocks, and all below, over the following frames. A working smelter
    doesn't glow (its light would depend on factory state).
12. The explored map and pins live in one browser, per world: a co-op client keeps neither (it never
    saves), and pins aren't shared with other players. World export doesn't carry them.

---

## 3. Engineering rules for all new work

### 3.1 Keep the codebase small and cheap to work on (TOP PRIORITY, mandatory)

**Why this matters more than usual:** only AI agents develop this project, and each session starts with no
memory of the code. Tokens go to four things:

1. reading code to understand it (the biggest cost),
2. exploring to find where things live,
3. failed build and test iterations, and the output they print,
4. re-learning the project every session.

Structure decides all four. A 400-line module behind a clear interface is cheap to change forever; a
1,000-line god object taxes every feature that touches it. **Treat context cost like frame time: a budget
you measure and defend.** If a change would break these rules, restructure first, in its own commit.

**Architecture: changes stay local**

- **One feature, one module or folder.** Adding a machine, item, recipe or UI panel means new files plus at
  most one registration line in a central place. If you find yourself editing five files for one feature,
  the structure is wrong; fix it.
- **No god objects.** The wasm facade (`Game`) is thin: its API is split by area into `api/*.rs` files
  (several `#[wasm_bindgen] impl Game` blocks are allowed), and each method only forwards to a module.
- **Content is data.** Blocks, items, recipes, machines, tech tree and ore settings are tables. Code is
  written per *kind* of behaviour, never per individual item.
- **Simulation and presentation are separate modules** (section 3.4). A UI change never requires reading
  simulation code, and the reverse.
- **Boring, explicit code.** Shallow call chains, no macro tricks, no deep generic towers, no ECS framework.
  Plain data-oriented modules: typed storage in `Vec`s plus one function per system. Don't build
  abstractions ahead of need. Introduce a registry when the second instance of a kind arrives (e.g. the
  machine registry arrived with the smelter).
- **Variants are data, never booleans.** A tier, material or mode is a field indexing a table; a new
  tier or machine that only differs in numbers is a row. Content lint tests keep the tables consistent.
  The full rules: `docs/TECH_TREE.md` section 8.

**Size budgets** (enforced by `scripts/check-size.mjs`, part of `npm run check`)

| Kind | Soft limit (warn) | Hard limit (fail) |
|---|---|---|
| Rust / TS source file, excluding tests | 400 lines | 600 lines |
| CSS file (one per UI component) | 300 lines | 500 lines |
| Any `CLAUDE.md` | 60 lines | 100 lines |
| This plan (read every session) | 600 lines | 800 lines |
| Other docs (`CODEMAP`, `ROADMAP`, `WORKFLOW`) | 300 lines | 500 lines |

Count all lines, blank ones included: `(Get-Content f).Count`, not `Measure-Object -Line`.

- Before adding to a file over its soft limit, split it.
- **Tests never live inline in implementation files.** Declare `#[cfg(test)] mod tests;`, which resolves to
  `src/foo/tests.rs` from both `src/foo.rs` and `src/foo/mod.rs`. Reading a module must not pull in its
  tests.

**Readability: cheap to skim**

- **Every module starts with a header** (`//!` in Rust, a top comment in TS) of at most ~15 lines: what it
  owns, its key invariants, and how to extend it ("to add a machine: …").
- **Public items first, private helpers after**, so the top of a file is its interface.
- **Unique, searchable names** (`belt_step`, not a tenth `update`), so one search finds the right spot.
- **One source of truth.** Never mirror constants between Rust and TS; expose a getter (like `hand_yield()`)
  or generate it.
- **Delete dead code immediately.** No commented-out code, no old and new paths living side by side after a
  refactor.
- **Comments explain *why*,** briefly. Don't narrate what the code already says.

**Docs: maps, not history**

- `docs/CODEMAP.md`: one line per module (what it owns) plus "how to add X" recipes.
  **Update it in the same commit as any structural change.** It replaces most exploration.
- **Nested `CLAUDE.md` files** in `crates/engine/` and `web/src/` hold subsystem conventions. Claude Code loads
  them only when working in that folder, so they cost nothing elsewhere. Keep them short.
- **This plan details only the current milestone.** When a milestone finishes, compress it into section 8
  (a few lines) and move the next milestone in from `docs/ROADMAP.md`, detailed to the level of the
  current one (steps with where, how and done-when). Future milestones stay as short bullet lists in the
  roadmap, which is read only when planning.
- **Operational reference lives in `docs/WORKFLOW.md`** (commands, verification, gotchas); read only the
  section you need.
- The root `CLAUDE.md` holds rules and pointers only.

**Feedback loops: fast and quiet**

- **`npm run check`** runs format check, clippy with warnings as errors, engine tests, typecheck
  and size budgets, and prints only a summary and the failures. Run it before every commit.
- **Prefer headless tests over the browser.** Scenario tests in Rust (build a layout through actions, run
  ticks, assert on state or `describe` text) and text debug dumps (e.g. an ASCII map of an area with
  machines and belt contents) are far cheaper than driving the browser pane. The browser is for final
  visual proof only. From Milestone 1 on, bugs should become replayable action scripts.
- **Keep tool output small:** quiet flags (`cargo test -q`), and filter long output to what matters.
- **Let the compiler do the searching:** newtypes (`ItemId`, `PlayerId`) and exhaustive `match`es make the
  compiler list everything a change must touch.

**Session habits**

- One scoped step per session; finish green and committed. Big sessions are where tokens disappear.
- Read line ranges and symbols, not whole files. Use the code map. For a broad search, use a search
  sub-agent so only its conclusion enters the main context.
- End every session by updating this plan (checkboxes, status, change log) and the code map.
- **Every milestone ends with a cleanup step:** size check clean, code map current, plan compressed, dead
  code gone, the next milestone moved in from the roadmap.

### 3.2 Performance

- Hot loops and all game state stay in Rust. TypeScript stays thin.
- Pass bulk data zero-copy (views on wasm memory); don't serialise per frame.
- Measure before and after anything that could cost frame time.

### 3.3 Wasm size

- Report gzipped sizes when the build grows noticeably (`npx vite build` prints them).
- Avoid formatting floats in Rust (`{:.1}` pulls in about 25 KB). Round to integers, or format in
  TypeScript.
- Avoid new instantiations of the std sorts (about 5–9 KB each). Use the insertion sort
  `math::sort_small_by_key`.
- No new crates without a reason worth their size. Hand-written serialisation beats serde+bincode here.

### 3.4 Determinism (required for co-op; enforced from Milestone 1)

The **deterministic core** (world edits, factory, deposits, inventories, crafting, research) must produce
identical state on every machine given the same seed and the same actions. Therefore core code must not
depend on:

- frame time: advance only in fixed ticks,
- which chunks happen to be loaded or meshed locally: use the `*_anywhere` accessors, which generate or
  read the stored copy,
- the camera, the local player, UI state or presentation queries: queries must never create or modify core
  state,
- iteration order of hash maps: iterate `Vec`s or sorted keys wherever order affects results,
- wall-clock time, `Math.random`, or anything from JS,
- (for a future native server) platform math: `sin`/`cos`/`powf` may differ between wasm and native. The
  `libm` crate on both sides fixes that when the time comes.

Everything that changes the core goes through an **action** applied at a tick boundary. Things that are only
presentation (camera, sounds, particles, meshes, HUD, readouts) never feed back into the core.

### 3.5 Code style

- Section 3.1 comes first. Beyond it, match the surrounding code: comment density, naming, idioms.
- Every engine feature gets unit tests in its module. Run `cargo test --workspace --release`.
- Keep the player's manual (`docs/PLAYING.md`, `docs/FACTORY.md`) in sync for anything player-visible, and keep
  this plan in sync for everything else. README.md is the project brief: touch it only when the pitch or setup changes.

---

## 4. Now: the play-test response (steps R1–R8, agreed 2026-10-16)

Milestones 6 to 12 are built; their step lists are in `docs/CHANGELOG.md` ("Milestones 6 to 8"), `docs/CHANGELOG_M9.md`,
`docs/CHANGELOG_M10.md`, `docs/CHANGELOG_M11.md` and `docs/CHANGELOG_M12.md` (Milestone 12, the less-hand-crafting
interlude and the performance todo); Milestone 13's in `docs/CHANGELOG_M13.md`, the finished R steps' in
`docs/CHANGELOG_R.md`. On 2026-10-16 Claude reviewed the whole game and the user agreed with every point
(section 1); with the user's play-test save that review became steps R1–R8 below, the direct next work.

**Rules that still bind every step:**
- `docs/TECH_TREE.md` section 8: tiers are data, processing machines are spec rows, recipes belong to
  categories, unlocks are one enum. No new `bool` per variant, no block per tier.
- Old saves keep loading: each format change bumps `SAVE_VERSION` (now 53) with a migration and a fixture
  test. Golden hashes are re-recorded only on purpose, noted in the step.
- New blocks and items append (the next free block is 128, item 397, tech 85, machine recipe 136). Each new look gets a placeholder layer (`tex::COUNT` is 332) and a `docs/ART_HANDOVER.md` request line.
- Fluids other than water ride belts as **canister items** (pipes stay water-only); empty canisters come back.

### Where the last milestones stand

- **Milestones 10 and 11** are built and pushed (specs and balance in their changelogs). **The less-hand-crafting interlude**
  is done (L1–L3); crafting from nearby boxes (L4) was dropped by the user on 2026-10-11 and is never built.
- **Milestone 12 (Orbit) is done:** 12.1–12.4 and the orbit polish are built and pushed (`docs/CHANGELOG_M12.md`); 12.5 closed
  on 2026-10-16 when R6 made settlements Milestone 13. Not done: a beam drawn from power satellites down to rectennas. The
  pads' open question is answered: step "Pod channels by name" below.
- **Parked performance items** (details in `docs/CHANGELOG_M12.md`): P3 cheaper box instances (measure in the browser
  first), P4 an incremental co-op checksum (time a large save first), P5 small TS savings. Fit them in where a session has room.

### Play-test findings (2026-10-16) and the finished R steps

The user's 32-hour save (seed 1337): research outran need (78 of 84 techs, yet drones, trains and blueprints never used;
38 of 97 machines held back by full output). Findings and the finished steps' records: `docs/CHANGELOG_R.md`.

### Next steps, in this order (one per session; "talk" steps are a conversation with the user first)

- [x] **R1** (talked: settlements as demand, a blueprint prompt), **R1a** (`scripts/inspect-save.mjs`), **R1b**
  (`blueprint/repeat.rs`), **R6** (talked: settlements became Milestone 13, the space elevator Milestone 14 in the roadmap).
- [x] **R1c Natural rivers and lakes** (generator version 8, new worlds only; versions 7 and 8 pinned): winding reaches
  (`rivers/course.rs`), rapids and waterfalls of settled falling water instead of weirs (`worldgen::falls`), shores for
  lakes, ponds and ore stains (`shore.rs`), the spawn ring bent.
- [x] **R2 Plants (a living world, part 1; R2a to R2c built 2026-10-16 and 17).** Today: one tree shape, grass and saplings; deserts and basalt fields are bare.
  - [x] **R2a Ground cover (built 2026-10-16, record in `docs/CHANGELOG_R.md`), presentation only, every world from generator version 2:** grass tufts, flowers, ferns, reeds, dry brush
    and lichen chosen per biome by a hash of the column, drawn by the mesher as crossed quads (`Render::Plant`, faces 6 and 7) on
    top of grass, sand or dirt with air above. Never blocks, never saved; a remesh clears them when the ground changes. Watch
    `bench_meshing` and vertex counts. **Done when:** every biome has its own cover, meshing costs at most 15% more, seen in the pane.
  - [x] **R2b Tree species and biome plants (built 2026-10-17, record in `docs/CHANGELOG_R.md`; generator version 9, new worlds only, section 6; R1c released version 8):** species as rows in
    `worldgen/trees.rs` (shape, log and leaf blocks): today's broadleaf (plains), conifers (highlands), wide wet-ground trees
    (lowlands, by water), a pale-barked plains mix, cacti (desert), dead snags and heather (basalt). Each species has a sapling;
    every log saws into the same planks. Versions 7 and 8 are pinned (`released_versions_never_change`). **Done when:** each biome reads differently from afar.
  - [x] **R2c Indicator plants (built 2026-10-17, record in `docs/CHANGELOG_R.md`)** (real geobotany, like the stained soil): a flower or shrub per ore over shallow veins and lodes, a
    pure function of deposit placement; pointing at one names the ore below. In the ore guide too. **Done when:** a test shows
    they grow only over their ore.
- [x] **R3 Ambient sound (part 2; built 2026-10-17, record in `docs/CHANGELOG_R.md`):** `Game::surroundings` feeds `audio/ambience.ts`.
- [x] **R4 Music by context (part 3; built 2026-10-17, record in `docs/CHANGELOG_R.md`):** a tested director, streamed 64 kbit/s Opus
  pieces (1.3 to 2.1 MB each) composed in the user's MusicComposer project, modes By place / Always calm / Always adventurous.
  **Not built (waits for its first piece):** the era as a context (industrial, space). Next pieces: sea, industrial, space.
- [ ] **R5 Ruins of the last factory (moved up).** Design in `docs/ROADMAP.md`; shares generator version 9 (not pinned until released).
  - [x] **R5a Ruin sites, rusted blocks, scrap (built 2026-10-17, record in `docs/CHANGELOG_R.md`):** `worldgen/ruins.rs`, blocks
    124–126, Scrap Metal, smelter row 135. Deviation: rusted blocks are never held, so no recycler; they give scrap, smelted to iron.
  - [x] **R5b Logbooks (built 2026-10-17, record in `docs/CHANGELOG_R.md`):** block 127 on every slab, a reading panel (`ui/logbook.ts`),
    pages from `worldgen/logbook.rs`: a bearing with a map pin, a recipe tip, from far ruins a story fragment. Not an item (a block).
  - [ ] **R5c The vault (easter egg, the trimmed carry's only one):** a sealed vault (5-adic ring dial puzzle), modest loot.
- [ ] **R7 Decor building set.** Sketch in `docs/ROADMAP.md`: about a dozen machine-made decor blocks in a Decor group (coloured
  concrete, bricks, framed glass, metal panels, catwalks, railings, light fixtures). Slabs and stairs need non-cube meshing:
  a later step. **Done when:** they are made in machines, placed, and their art requests are listed.
- [ ] **R8 Pressure without combat (talk).** Pick one or two with the user from `docs/ROADMAP.md` (subsidence, mine flooding,
  storms, pollution that thins plants and solar). Each must reuse existing systems and stay deterministic.
- [ ] **Pod channels by name (small, any time).** Facing as a channel is unclear (user, 2026-10-16): a launcher and a pad pick
  their channel in their panel (a colour and a name), not by facing. Save version bump with a migration (facing becomes the
  colour).
- **The user's to do:** add the two TURN secrets (`docs/WORKFLOW.md` section 4) so friends behind strict networks can join.

### Milestone 13: Settlements (built 2026-10-16 but for 13.6b)

Towns the player founds are customers whose orders keep growing, so output has a purpose and distance and scale press
again (R1's main lever). Design notes: `docs/ROADMAP.md` ("The play-test response").
- [x] **13.1 to 13.5b built** (2026-10-16; each step's record is in `docs/CHANGELOG_M13.md`, the player's view in
  `docs/TOWNS.md`): four town kinds and their sites, the depot, orders, upkeep and stalls, plots built by the town's own
  drones, rebuilding and dormant towns, the four perks (forge, workshop, river relay, harbour contract work), the town
  panel and stall news, map markers, site shading and the town tips. Save versions 48 to 53.
- [x] **13.6a Cleanup without the play-test** (2026-10-16; record in `docs/CHANGELOG_M13.md`): `bench_towns`, stage
  planning 3.5× faster, `docs/TOWNS.md`. Milestone 14 moves into this plan when it is next.
- [ ] **13.6b Balance from the user's next play-test** (`scripts/inspect-save.mjs`): order sizes, upkeep rates, perk rates
  and the harbour's price, once the user has played Milestone 13 (pushed 2026-10-16; their saves are version 7 worlds,
  so towns there sit by the old rivers).

**After the R steps and Milestone 13:** the feel-and-feedback interlude (`docs/ROADMAP.md`; the instrument with few voices, a
distance cap and music quieting under it), the carry's era pieces (splitter addresses and the tally only), Milestone 14. **Art**
goes on in parallel (`docs/ART_HANDOVER.md` section 2b, 2026-10-16: shapes before textures).

Open items from Milestone 3 (the user's to unblock; do them when they come up):

- **The relay (TURN):** agreed on 2026-10-16 to add it (section 1): the user adds the TURN secrets
  (`docs/WORKFLOW.md` section 4); then test with `&relay` on both ends.
- **Instant feedback for your own edits** (was 3.9; measured, not built): an action applies about 90 ms after it is made
  on one machine (`INPUT_DELAY` 4 ticks), 110–170 ms over the internet. Only if placing feels laggy: show your own edits
  at once in the render cache, undo them if the core disagrees, never touch the core (`interaction.rs`).
- A client's every resync logs the tick and both hashes; turn any real one into a replayable test.

---

## 5. Roadmap after R1–R8

`docs/ROADMAP.md` holds the designs the R steps point to (ruins, settlements, decor, pressure without combat), the
feel-and-feedback interlude (still unplanned), the trimmed carry and Milestone 13. The tech tree is `docs/TECH_TREE.md`
(concept) and `docs/TECH_ERAS.md` (detail). Read them only when planning.

---

## 6. Open questions for the user

Ask these when the milestone that needs the answer comes up, not before. Record the answers in section 1
and adjust the steps.

| Needed by | Question |
|---|---|
| M7 | A colour-blind palette option for tier colours (pips and "Mk" text are there regardless)? Kits go in as one step at a time, never refunded: change? |
| M10 | Rivers and the new landforms are on new worlds only (generator version 7; built that way in 10.8): existing worlds keep their ground. Acceptable, or should they wait for a world-settings screen? Also: should far deposits seen once get a map pin (9.3 left it as a bearing only)? |
| R2b | New tree species and desert and basalt plants on new worlds only (generator version 9; the user's worlds keep their own version and get only R2a's ground cover), or another way? Default: new worlds only, like rivers. |
| R4 | ~~Which new music pieces?~~ Answered 2026-10-17: Claude composes them in the user's MusicComposer project, small files. Open: a sea piece; industrial and space pieces (with the era as a context). |
| R8 | Which pressure without combat, if any: subsidence, mine flooding, storms, pollution? |

---

## 7. Working in this repo

See `docs/WORKFLOW.md`: commands, browser verification, Windows gotchas, deploying, working with the user,
and the balance numbers. Read the section you need.

---

## 8. Recent changes

- **2026-10-17: Steps R5a and R5b built** (ruins on generator version 9: sites, blocks 124–127, Scrap Metal, logbooks; `tex::COUNT` 332). 909 tests.
- **2026-10-17: Steps R3 and R4 built** (ambient sound; music by context, composed in MusicComposer, six pieces in 9.3 MB;
  `npm run check` gained `web tests` and `music`).
- **2026-10-16 and 17: Steps R2a to R2c built** (ground cover; generator version 9, pinned in `released_versions_never_change`
  when pushed, with tree species per biome, blocks 117–123; indicator plants over stained soil).

Earlier entries live in [CHANGELOG_M13.md](CHANGELOG_M13.md), [CHANGELOG_M12.md](CHANGELOG_M12.md) and [CHANGELOG.md](CHANGELOG.md).
