# OpenCraft roadmap

Future work only; read it when a milestone ends and the next is planned. `docs/DEV_PLAN.md` details the current work;
at each milestone's cleanup step, move the next item from here into the plan and detail it there (steps with where, how
and done-when). Finished designs live in `docs/history/`, settled choices in `docs/DECISIONS.md`.

**Order (2026-10-16):** R8, then the feel-and-feedback interlude, the carry's era pieces, Milestone 14. The order can
change with the user's priorities. Determinism (DEV_PLAN section 3.4), the agent rules (section 3.1) and the content
architecture (`docs/TECH_TREE.md` section 8) apply to all of it. Every milestone ends with a cleanup step.

**The arc the user asked for:** a steady, intuitive and fun growth of the player's reach (building by hand → planning an
area → machines and drones doing the work → building from afar), carried by a tech tree where every technology opens
something, improves efficiency or helps the player. Since 2026-10-16: needs before techs (`docs/DECISIONS.md`).

## R8: pressure without combat (talk first)

- **Pressure without combat (R8).** What enemies give Factorio is pressure: the factory can't be left alone. Peaceful options,
  all deterministic and built on existing systems: **subsidence** (hollowed lodes collapse unless backfilled or supported;
  already in "Later / maybe"), **groundwater** (deep mines below the water table seep and need pumps), **storms** (rare, from
  the core tick; lines in the open stop or wear unless roofed), **pollution** (smelters and generators spread a haze value
  per region that thins plants and dims solar; scrubbers clean it). Pick one or two; none may destroy the player's work.
  Later, with pollution, plants thin around a dirty base and come back when it is cleaned.

## Interlude: feel and feedback (chosen by the user, 2026-10-08)

Small features that make the factory more satisfying to watch, hear and read. All but the site bonuses are
presentation only (the state hash never moves; DEV_PLAN section 3.4), so they can be built in any order and
moved earlier if a session has room. Each one is its own module.

- **Timelapse of the world (S–M).** Every 5 game minutes, capture the world map's explored image
  (downsampled, about 256×256) and keep the frames in IndexedDB beside the save, never in core state. A
  History screen plays them from the first pickaxe to now in about 30 seconds, and exports a video with
  the browser's `MediaRecorder` from a canvas (no dependency). In co-op each player records their own.
  Later, maybe: a 3D replay from an action log, since the core is deterministic.
- **The factory as an instrument (S).** Machine cycles already land on fixed ticks: give each family a
  pitched note, quantised to a shared beat (e.g. every 15 ticks), so a balanced line has a groove and a
  starved or blocked machine leaves a gap you can hear before you see it. Volume falls off with distance,
  and a setting turns it off. **Limits (2026-10-16):** only the few nearest families sound at once, a hard distance
  cap, and the music (R4) quiets while it plays, so a big base never turns into noise. Lives in `web/src/audio/` from the existing sound events. Later, maybe: a
  sequencer block (core state, driven by logic) to compose with machines.
- **Follow one item (S).** Tag an item (one tag per player) on a belt or from an inventory; the tag moves
  with it, and through a machine it passes to the first output made with it. Its route is drawn on the
  world map and as a faint line in the world. **No camera follow** (the user gets motion sick). When it
  ends (in a pack, a silo or the recycler) a postcard shows where it was mined, distance, time, machines
  and transport used. Decide when planning whether the tag is core state (one field on a lane slot,
  co-op safe, a save bump) or a host-side watcher.
- **Seismic prospecting (M).** A thumper (powered, tech after the scanner Mk2) shows a vertical
  cross-section along its facing, 64 blocks wide down to bedrock: dense bands where ore lies, caves and
  voids, water, the player's own tunnels. It reads the true world (`*_anywhere`) blurred by noise that
  shrinks with its Mk, so reading it well is a skill. A query (only placing the thumper is an action). It
  sits between AI survey (11.12: wide but only a prediction) and survey satellites (Milestone 12: every
  deposit, but no depth structure), and doubles as an X-ray when planning tunnels and shafts (and
  subsidence, if that comes).
- **Where you build matters (S per bonus).** Places get real properties, each a pure function of seed and
  position (biome, height, depth below the surface), so never dependent on loaded chunks: solar by biome
  (desert strongest), datacenter cooling underground or by the sea (Milestone 11), and new machines as
  rows when wanted: a geothermal plant on vents in basalt fields (a worldgen feature, so a generator
  version) and a wind turbine whose output grows with height. The value is computed at placement, stored,
  and **shown on the ghost before placing** ("Solar here: 125%, desert"), so the player can choose a site.

## Thread: the carry (p-adic numbers and fractals, by discovery; chosen 2026-10-11, trimmed 2026-10-16)

Mechanisms that behave exactly like p-adic numbers, so players find the ideas by playing. **Trimmed on 2026-10-16** (the
user agreed with Claude's review: the puzzles cost a lot of design for a small audience, and "names only, never explained"
would leave most players at a locked door): two cheap era pieces that come out of normal play, and one vault in the ruins
as an easter egg. Dropped: the 3-door, the 10-door, the Cantor sieve, the Hanoi crane and the layer printer. Parked, only if
a later step wants more depth for datacenters or comms: fractal radiators with the analyzer, and antenna bands.

- **Pure discovery, names only** (user's choice). When a player does the thing, a discovery shows only its name ("3-adic
  address"); digits are shown right to left with a leading "…" ("…0110"), so carries and endless digits are visible.
- **The thread, for agents** (never shown in game): a binary counter's carry length is the ruler sequence 1 2 1 3 1 2 1 4;
  a carry that never ends gives …1111 = −1, so 1 + 2 + 4 + … = −1 in the 2-adics. Two numbers are p-adically close when
  their last digits agree, which is a tree.
- **Discoveries (S).** One module: a list of named entries, each a pure check on state, a toast with the name, a page in the
  menu. Decide at planning whether it is core state (shared per world in co-op, a save bump) or kept beside the save.
- **Splitter addresses (S, primes 2 and 3, presentation only).** A splitter round-robins its outputs (`factory/router.rs`),
  so a cascade sends item n to leaf n mod 2^k (two outputs) or n mod 3^k (three). Looking at a belt fed by a tree at least 3
  deep shows its address, first split rightmost ("…0110", "…210"). Discoveries "2-adic integers", "3-adic integers".
- **The tally (S, Electronics retrofit, prime 2, core state).** A logic block that counts the box or belt behind it: +1 for
  each item in, −1 for each item out, shown on 16 binary lamps. Below zero it rolls to all lamps lit (…1111). A sensor rule
  can fire on one bit, so it doubles as a divider ("every 8th item"). Discoveries "2-adic −1", "two's complement".
- **The vault (ruins, the one easter egg, prime 5).** A door of rings, each a dial of 5 digits, showing …4444 (−1 in base
  5). Ring k opens when the number on rings 1..k, squared, ends in the target's last k digits: the first ring is found by
  trial (2 or 3), then each ring has exactly one right digit (2 → 12 → 212 → …). Modest loot; rewards stop after about 6
  rings. Discoveries "5-adic integers", "√−1".

**Order:** discoveries, splitter addresses and the tally after the feel-and-feedback interlude; the vault with the ruins (R5).

## Milestone 14: The megaproject, a space elevator (era 10; optional, non-ending; theme chosen 2026-10-16)

Milestone 13 (Settlements) took the number 13. Optional, built in phases, physically in the world, needing enormous
resources; completing it **unlocks** something rather than ending the game. Play continues.
- **Where:** it rises from the player's biggest town (a town stage unlocks its foundation), so the towns are its workforce
  and their supply routes its supply routes.
- **Phases** (a first sketch): the anchor and foundation, the tether climbing in sections (each a large order delivered to
  its base: steel, aluminium, plastics, chips, rocket fuel for the climbers), a counterweight station sent up by rocket,
  the climber cars. Each finished section shows: the tower grows visibly into the sky, seen from far away, lit at night.
- **What finishing unlocks:** asteroid mining. Climbers carry mining craft up; asteroid ore returns by drop pod to landing
  pads, resources beyond the finite ground (pillar 2's answer at the end).
- **Engine notes for planning:** a very tall structure must stay cheap to draw (boxes or a dedicated mesh, not 4,000 blocks),
  and its sections are core state like a machine's; the sky already draws satellites (`render/sky.ts`).

## Later / maybe

- A tower-defence mode as a world setting (the user's idea; explicitly not now; 2026-10-16: still peaceful, no combat).
- Subsidence: hollowed-out lodes collapse the ground above unless backfilled or supported (an R8 candidate).
- Slabs, stairs and beams for building (R7's decor set is cubes only; needs non-cube meshing and hit boxes).
- A world settings screen: ore richness, deposit size, peaceful toggle.
- Catch-up on return: factories run forward for the time the game was closed, capped.
- Mod support (the data-driven content tables are the first step towards mods).
- A dedicated server: the engine crate compiles natively (feature-gate wasm-bindgen) and speaks WebSocket,
  with `libm` on both sides for identical math. Co-op itself is player-hosted (DEV_PLAN section 2).
