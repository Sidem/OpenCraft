# Space program: feasibility (study of 2026-11-16, not planned work)

The user asked: could OpenCraft later have rockets built from parts, flight through an atmosphere, real orbits, other
planets and moons worth settling, on a **finite spherical home planet with an atmosphere gradient**? It is not for now;
`docs/ROADMAP_ORDER.md` is unchanged. This file holds the verdict, the measurements behind it, the staged path and the
red team. The rules that keep the path open until then, and the planet's size, are rows in `docs/DECISIONS.md`.

**Verdict: feasible, conditional go.** Every risk that could be measured cheaply passed by a wide margin (flight
determinism, orbit stability, cost per craft, block distortion, far-terrain cost). The real risk is one cross-cutting
change: teaching the world, generator, mesher and maps about a round world while flat worlds stay bit-identical.
Cost: about 80–140 sessions over four stages, the first of which gives a playable round world on its own.

**The home planet (user, 2026-11-16): a low orbit takes about 15 minutes**, so at the player's `GRAVITY` (`player.rs`)
the radius is **about 375,700 blocks**: 2,360 km round, 0.63× KSP's Kerbin. It stays cheap by level of detail
(section 2): real chunks only near players, generated low-detail terrain farther out, one small mesh from orbit.

## 1. Measurements

Spike sources (throwaway, not in the build): `artifacts/spacespike/` (git-ignored). `cargo run --release --bin spike`
prints the tables; `spike size` the planet for a 15-minute orbit; `spike lod <radius> <px>` the far-terrain tiles;
`spike hash` and `node wasm-hash.mjs <spacespike.wasm>` (`cargo build --release --lib --target
wasm32-unknown-unknown`) compare native and wasm; `node spread.mjs <save>` measures a base. 1 block = 1 m, 60 Hz
ticks, f64, gravity = the player's `GRAVITY` unless noted.

**How big a base gets.** After 37.6 h (`artifacts/bench/factory.ocworld` and the user's three other big saves) machines
span about 590 × 740 blocks; half lie within 37 blocks of the centre, 90% within 184, the farthest 513.

**Flight is deterministic.** A rocket ascent through an exponential atmosphere (gravity turn, cut-off at the target
apoapsis, circularisation burn) followed by 100 coasted orbits, with every tick's state folded into a hash: native
release, native debug and wasm in V8 give **the same hash bit for bit** over 0.56–1.13 million ticks, with `exp`,
`sin`, `cos` and `sqrt` in the path. Lockstep co-op can simulate flight in the core like any other machine; the
native-server caveat (`libm` on both sides, `docs/ROADMAP.md`) still applies.

**Orbits are stable and cheap.** Leapfrog (kick-drift-kick) at 60 Hz: worst energy drift over 100 orbits 0.2–19 parts
per billion. One craft-tick costs **20 ns** native: a hundred powered craft is 2 µs a tick. A craft off the throttle
goes "on rails" (orbital elements, position a closed form of time, like `satellites.rs` today): 0.6 µs a query, any
time warp. Rails and leapfrog differ by 0.4–17 blocks after 100 orbits, so a craft switches between them by taking
the state from one to start the other, never by comparing them.

**Scale.** Atmosphere top at 11.7% of the radius (Kerbin's ratio), density falling 10,000× by the top; orbit 30% above it.

| Radius | v orbit | Lap | Δv to orbit | Air at height 256 / 2,000 | Horizon from the eye / from 256 up |
|---|---|---|---|---|---|
| 16,384 | 631 m/s | 3.2 min | 0.92 km/s | 29% / 0% | 229 / 2,908 |
| **375,700 (chosen)** | **3.0 km/s** | **15 min** | **≤ 4.6 km/s** | 95% / 66% | 1,096 / 13,900 |
| 131,600 at g 9.81 | 1.06 km/s | 15 min | 1.6 km/s | 86% / 30% | 649 / 8,200 |

On the chosen planet the spike's crude autopilot reached orbit with 4.6 km/s (1.53× orbital speed, 93 m/s lost to drag;
smaller planets needed 1.42–1.46×; Kerbin needs about 1.5×): with exhaust at 3 km/s that is a mass ratio of about 4.6,
so **rockets need stages**, as in KSP. The atmosphere is 44 km tall: the gradient is felt in flight (66% of sea-level
air at 2 km, gone by 44 km), barely while building (95% at the build limit). How much the air costs is a tuning knob
(sea-level density), not a given. Lower gravity than the player's would shrink the planet (last row); rejected: one
gravity for everything.

**Blocks on a sphere.** The usual cube-sphere: six faces of N × N columns (N = 590,109 on the chosen planet, well inside
`i32`), each column's up pointing out from the centre. One face, every column measured at radius 4,096 and 16,384 (the
ratios do not depend on the radius):

| Mapping | Block edge, shortest..longest | Area, largest/smallest | Area change across 512 blocks (R 4,096 / 16,384) |
|---|---|---|---|
| Plain normalise (gnomonic) | 0.60..1.27 | 5.20 | - |
| Equal-angle | 0.71..1.00 | **1.41** | 1.12 / 1.03 |
| Nowell | 0.74..1.27 | 1.33 | 1.18 / 1.07 |

Equal-angle passes the 1.5× bar. The change across a base falls with the radius (about 0.1% on the chosen planet, scaled,
not measured). The worst cells, 30° off square, sit only at the cube's 8 corners; 0.02% of columns lie within one chunk
of a face seam. A 32-bit float steps 4.5 cm at the planet's radius, so everything drawn stays relative to the camera
(as now) and planet-scale positions stay `f64`.

**What shipped games ran into** (Bowerbyte's *Blocky Planet*, StarMade's 2013 write-up, KSP): blocks widening with
height (Blocky Planet: shells that quadruple the columns each time the width doubles, a hollow core); neighbours
across the 12 seams and the 8 corners where three faces meet (structures wrap and overwrite); ships built on one grid
not matching a curved one; maintaining flat and round code at once; floating-point jitter far from the origin (KSP:
floating origin, "Krakensbane"); and n-body cost (KSP: patched conics, planets on rails).

## 2. Level of detail: vast but cheap

KSP draws a body two ways: **PQS** (a procedural quad sphere: each cube face a quadtree of terrain tiles that split near
the camera) and **scaled space** (one static low-poly mesh per body, always drawn far away), cross-fading between them by
altitude (Kerbin's PQS stops at about 160 km). KSP 2 built its tiles relative to the floating origin. OpenCraft would add
a third, nearest tier, the voxel chunks it has:

1. **Chunks** within render distance (`web/src/render/quality.ts` `RADII`): the real, editable world, as now.
2. **Far tiles**: each face a quadtree of 32 × 32-quad heightfield tiles from the generator's heights, coloured by biome
   (the world map's colours), split while a quad would cover more than 8 pixels, dropped below the horizon (plus the
   terrain's 256-block relief). Built off the main thread, nearest first; edits show only where chunks are loaded.
3. **The whole planet** as one baked mesh (6 × 64 × 64 quads from about 25,000 height samples, 15 ms once) when the camera
   is a planet radius or more away, and for every other body until approached. An atmosphere shader around it.

Measured on the chosen planet (`spike lod 375675 8`), counting every direction (a view sees about a quarter):

| Camera | Far tiles | Quads | Samples to build all | New tiles a second, moving at |
|---|---|---|---|---|
| On the ground | 736 | 754k | 0.80 M (0.5 s once) | 0 at 6 m/s |
| 2 km up | 796 | 815k | 0.87 M | 14 at 300 m/s |
| 10 km up | 564 | 578k | 0.61 M | 6 at 1 km/s |
| Low orbit, 57 km | 336 | 344k | 0.37 M | 2 at 3 km/s |
| One radius away | 88 | 90k | 0.10 M | 0 |

A view's share, about 190k quads on the ground, is the size of today's whole chunk scene (211k quads, `docs/DEV_PLAN.md`
section 2); at 16 pixels it is a third of that, at 4 pixels three times (2,112 tiles). Streaming is small: at most
15,000 samples a second, about 1% of one core.

**The generator needs a far-sample path.** One surface height (`WorldGen::height_at`, timed with a temporary bench, since
removed) costs 0.7–0.8 µs for neighbouring columns but **350 µs for columns 4,096 apart**, because the river network is
rebuilt round every sample. The raw terrain (`base_height`) costs 0.5–0.6 µs at any spacing. Far tiles therefore use the
raw terrain and draw rivers and lakes as a separate coarse layer; a planet generator keeps that split as a rule.

## 3. Why OpenCraft dodges most of those traps

- **The world is already a crust.** Blocks exist only in a 256-tall band above bedrock (`WORLD_HEIGHT`,
  `worldgen/mod.rs`). On the chosen planet the top of that band is 0.07% wider than the bottom: **one shell, no layer
  doubling, no hollow-core problem**, and the core below is simply not voxels.
- **The grid stays a grid.** A column is addressed (face, u, v) and a block (face, u, v, h): integers, as today with
  a face added. Belts, machines, footprints, water, towns and deposits keep working in grid steps; only stepping across
  one of the 12 seams rotates the axes (a lookup table of 24 cases). Multi-block footprints refuse to straddle a seam;
  the generator puts ocean or mountains at the 8 corners.
- **Physics runs in grid space, the picture is bent.** Within a base the grid is flat to a fraction of a percent, so the
  swept-box physics (`physics.rs`) can stay as it is in (u, v, h); the vertex shader maps grid positions onto the sphere,
  relative to the camera. Up and gravity come from one function of position.
- **The deterministic core already has the right orbit model.** Satellites and pods are pure functions of the tick
  (`satellites.rs`, `pods.rs`); real orbits are the same idea with Kepler elements in place of the 4,096-block torus.
- **Rockets are not voxels.** A rocket is a list of parts drawn as boxes (`factory/process/rocket.rs` already draws
  one): one rigid body, no grid to match, no wobbly joints to blow up.
- **Generation is already lazy.** The planet has 2.1 × 10¹² columns; like today's endless world, only what players
  visit is generated and only edited chunks are saved. Noise already has a 3D form (the caves use it), so terrain sampled
  on the unit sphere has no seams.

## 4. Staged path (each stage ends playable)

Cost in sessions, from this project's history (one step ≈ one commit): orbit (Milestone 12) took 12 commits for 1.1k
lines, trains 13 for 2.9k, towns 19 for 1.6k; world and generator took 75, the renderer 54. Ranges are 80% guesses
from those.

| Stage | What | Sessions | Riskiest part |
|---|---|---|---|
| A Round home world | New generator version "planet": face in chunk keys, seam steps, columns from sphere directions, biomes by latitude, rivers on a sphere with a far layer; radial up for player, items, drones, camera; curved meshes in the shader; far tiles and the baked planet (section 2) with an atmosphere shader whose sky darkens with height; world map as a globe; satellites on real orbits. Flat worlds unchanged. | 30–50 | Rivers and other plane-wide generator passes; keeping every flat-world golden hash equal |
| B Flight | Parts as data rows (mass, thrust, burn rate, drag, fuel), stages, a build gantry, rockets as core state stepped by actions (throttle, attitude, stage), on rails when unpowered, flight HUD, orbit lines on the map, landing, parachutes | 20–35 | Flight controls through the co-op input delay (about 70 ms: fine for rockets, test early) |
| C Other bodies | A moon and one more planet as generator profiles (radius, gravity, air, materials), one chunk store per body, spheres of influence (patched conics), transfer planning, a time-warp rule for co-op (only when every player agrees, or never in co-op) | 15–30 | Many bodies loaded at once by players spread across them |
| D Reasons to go | Materials found only off-world, outposts (towns reused), automated cargo rockets on rails, comms cover from real geometry, the space elevator on the round world | 15–25 | "Needs before techs": each body must solve a need, not add a multiplier |

Total 80–140 sessions. Stage A alone is worth having: it makes the ground finite, which `docs/DEV_PLAN.md` pillar 2
already wants. Travel becomes its own problem on a 2,360 km planet (trains, then flight), as pillar 5 asks.

## 5. Red team

- **"It is a different game."** KSP is about hand-flying skill, OpenCraft about automation. Answer: fly the first
  rocket by hand, then automate the route (pillar 4); cargo rockets on rails cost no frame time.
- **"Two worlds forever."** StarMade dropped spheres partly over this. Answer: a flat world is one infinite face with
  up = +y, and the existing golden hashes are the judge: stage A is not done while any flat-world hash moves.
- **"The 8 corners will bite."** They will (Blocky Planet admits it). Keep them out of reach by generation, and test
  every seam case with a scenario that walks, belts and pipes across each of the 24 seam orientations.
- **"Far terrain doubles the scene."** At 8 pixels a view's far tiles match today's chunk scene. Answer: they are flat
  heightfield quads in a few draw calls, unlike chunk faces; if the frame bench says otherwise, 16 pixels cuts them to a
  third. Measure with `scripts/frame-bench.mjs` in stage A before adding detail.
- **"The planet is too big to fill."** 2,360 km round is mostly untouched ground, as today's endless world already is;
  the cost is travel, which the game answers with trains, drones and flight.
- **"It costs a quarter of everything built so far."** True. It waits until the roadmap is done, and stage A can stop
  there with a finished, round, finite world.

## 6. Sources

[Blocky Planet — Making Minecraft Spherical](https://www.bowerbyte.com/posts/blocky-planet) ·
[StarMade: why spherical planets don't fit in a block world](https://starmadedock.net/threads/starmade-dev-update-why-spherical-planets-dont-fit-in-a-block-world.22996) ·
[KSP API: Krakensbane](https://www.kerbalspaceprogram.com/ksp/api/class_krakensbane.html) ·
[KSP forum: patched conics](https://forum.kerbalspaceprogram.com/topic/69339-patched-conics) ·
[KSP 2 Developer Insights #12: planet tech](https://www.kerbalspaceprogram.com/news/dev-diaries-developer-insights-12-planet-tech) ·
[KSP forum: at what altitudes PQS stops rendering](https://forum.kerbalspaceprogram.com/topic/184506-at-what-altitudes-does-pqs-terrain-surface-stop-rendering)
