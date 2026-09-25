# OpenCraft development plan

**Status:** 2026-09-25 · Milestones 1 and 2 done · **Next up: Milestone 3 (co-op), step 3.1 (actions on
the wire)**.

> **This project is written entirely by AI coding agents.** Every session starts cold, and every line an
> agent has to read costs tokens and time. **Keeping the codebase small, modular and cheap to read is as
> important as any feature.** A sprawling codebase makes every future feature slower and more expensive,
> and that cost compounds. The rules in **section 3.1 are mandatory** and override convenience. When a
> change would break them, restructure first.

This is the plan of record. It says where the game is heading, what exists today, and exactly what to build
next. README.md covers setup and how the game works for players; this file covers direction and work.

---

## 0. Handover: read this first

You are picking up a working browser factory game (Rust → wasm engine, TypeScript/WebGL2 host), live at
<https://sidem.github.io/OpenCraft/>.

- **Milestone 1 (Foundation) is done** (section 8): a fixed 60 Hz tick, a deterministic core changed only
  by actions, several players in the engine, state hashes, and worlds that save in the browser.
- **Milestone 2 (Make it a game) is done** (section 8): items, smelter, constructor, belt logistics, power,
  research with science packs, Mk2 upgrades and onboarding tips.
- **The next job is Milestone 3: Co-op** (section 4): 2–4 players in one world over WebRTC, with the host's
  browser as the authority and a small Cloudflare service to connect them.

Before you change code:

1. Read sections 0–4 of this file (section 3.1 carefully), then `docs/CODEMAP.md`, then the nested
   `CLAUDE.md` of the area you work in. Skim README.md only if you need the player's view.
2. Run `npm run build:wasm` (if `web/src/wasm` is missing) and `npm run check` to confirm a green baseline
   (113 engine tests).
3. Work through the current milestone in step order. Each step lists where, how and when it's done. Do
   one step, or one clean part of a step, per session, and stop in a green, committed state.
4. When a step is done, tick its checkbox here, update the **Status** line at the top, and add a line to
   the change log (section 8). Update `docs/CODEMAP.md` in the same commit as any structural change. Keep
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

### Proposed, not yet confirmed by the user

- The Miner Mk1 and the smelter stay unpowered (a burner tier); newer machines need power. Built this way.
- Old saves keep loading across format changes where a migration is cheap. Every save since version 1
  still loads.

---

## 2. Where the code stands (after Milestone 2)

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

### What exists

- **World:** 32³ chunks, 256 tall; seeded terrain with cliffs, caves and trees; greedy mesher with AO;
  streaming nearest-first; edits kept when chunks unload (`World.saved`).
- **Deposits** (`deposits.rs`, placed by `worldgen/ore.rs`): outcrops, veins and lodes of coal, iron and
  copper (100 / 1,000 / 2,000 units per block, shared draw caps 60 / 240 / 1,200 per minute). A pool is
  shared per deposit, output tapers over the last 20%, and blocks turn to `SPENT_ROCK` as it drains, even
  in unloaded chunks. Hand mining keeps `HAND_YIELD` = 3 per block and costs the deposit one block.
- **Factory** (`factory/`, one file per machine kind, the `MACHINES` table and a `Machine` trait; one kind
  can serve several blocks through extra table rows):
  - Miners: Mk1 (1 unit/s, 60% recovery, unpowered) and Mk2 (2 units/s, 75%, 20 kW).
  - Belts (1 block/s; fast belts 2) with side-joins, corners, back-pressure, ramps, lifts and underpasses
    (`belt_shape.rs`); splitter and filter (`router.rs`); storage boxes (24 slots, open like a chest).
  - Smelter (ore plus coal or logs → ingots) and constructor (ingots → plates, rods, screws, wire), with
    machine recipes and fuels as data (`recipes.rs`). Right-click opens a machine panel (`panel.rs`,
    `ui/machine.ts`): status, buffers, recipe or filter choice, put-in and take buttons.
  - Power (`power.rs`): coal generators, poles that link within 10 blocks into grids, machines on the
    nearest pole within 5, brownouts as a speed factor.
  - Research labs (`lab.rs`) working through the tech tree (`research.rs`, key R) with red and green
    science packs; six techs unlock routing, climbing, underpasses, green packs, Mk2 and fast belts.
  - Models are instanced boxes; status readouts come from `describe`.
- **Inventory** (`inventory.rs`): 36 slots (hotbar 0–8), a cursor stack, click, shift-click and
  quick-move. **Crafting** (`recipes.rs`): hand recipes in the build menu (key E), greyed while locked.
- **Onboarding tips** (`hints.rs`, `ui/hints.ts`, key H skips): seven tips from finding ore to research.
- **Items** (`item.rs`): `ItemId(u16)`. Ids below 256 are the blocks with the same number (0–28, see
  `block.rs`); from 256: iron and copper ingots, iron plate, iron rod, screws, copper wire, red and green
  science packs. Every item is drawn as a textured box.
- **Sound:** procedural foley, 7 materials including metal, and a sound designer (key O).
- **Debug API** on `window.opencraft.game`: `give`, `teleport`, `run_ticks`, `skip_time`, `find_deposit`,
  `block_at`, `toggle_fly`, `set_look`, `add_player`, `remove_player`, `state_hash`.
- **Saving** (`save.rs`, `web/src/save/`): worlds autosave to IndexedDB; the menu lists, creates,
  exports and imports them. Save version 9; every version since 1 loads.
- **Size:** about 158 KB gzipped in total (wasm 120.3 KB, JS 31.8 KB, CSS 4.7 KB).

### Known limitations and technical debt

1. Still single-player-shaped: streaming centres on the local player (other bodies wait where the ground
   isn't loaded), only the local player has hands, nothing draws other players' bodies, and a leaving
   player's inventory is dropped. Milestone 3 handles these.
2. Two tabs on the same world overwrite each other's saves (the later save wins).
3. Veins and lodes can only be found by digging; there's no prospecting (Milestone 4).
4. The outcrop nearest spawn (about 11, 62, 2 with seed 1337) is buried under 1–2 blocks.
5. TypeScript mirrors a few engine constants: `INSTANCE_FLOATS`, the 6 floats per sound event, and the
   order of sound materials and event kinds. Replace them with getters when touching that code.
6. Item and belt instances aren't interpolated between ticks (only the camera is); optional polish.
7. Balance is untested by real play: pack costs, research times and Mk2 costs will need tuning
   (`docs/WORKFLOW.md` section 6 lists the numbers).

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
- **Every milestone ends with a cleanup step** (see step 3.10): size check clean, code map current, plan
  compressed, dead code gone.

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
- Keep README.md in sync for anything player-visible, and keep this plan in sync for everything else.

---

## 4. Milestone 3: Co-op (NEXT)

**Goal:** 2–4 players build in one world together. One player's browser hosts: it runs the world, saves
it, and orders everyone's actions. Every peer runs the same deterministic core from the same actions
(lockstep), so only actions travel, not world state. Movement is each player's own (co-op trust), and the
host alone runs loose items. Peers connect over WebRTC data channels; a Cloudflare Worker introduces them
and Cloudflare TURN relays when a direct connection fails.

How lockstep works here:

- Each peer sends its local actions to the host. The host stamps each with a tick,
  `current_tick + INPUT_DELAY` (4 ticks to start, tuned in play within 3–6), queues it and sends it to
  every peer. The host's own actions take the same path, so it has no advantage.
- The host sends one **frame** per tick it runs: the tick number plus the actions it stamped during that
  tick (usually none). A client steps its core only through the last tick it has a frame for. When frames
  arrive late it waits, then catches up (at most 8 extra ticks per rendered frame, as today).
- Every 60 ticks each peer compares `state_hash` with the host's. A mismatch is a bug: log it with the
  tick, then resync from a fresh host snapshot.

Rules for every step:

- The core never learns about the network. Networking moves actions, snapshots and presentation state
  (bodies, loose items) only. Solo play takes exactly today's path, and its golden hash stays unchanged
  unless core state changes on purpose.
- Engine networking lives in `net/` (Rust) with its API in `api/net.rs`; the host side lives in
  `web/src/net/`, one file per concern. The Worker lives in `signal/`.
- **The dev loop is two tabs on one machine** over `BroadcastChannel` (step 3.3), and before that two
  `Game`s wired together in a Rust test. WebRTC comes only once everything works over the channel.
- Budgets for 4 players: a star through the host (clients talk only to the host), under ~10 KB/s per
  client outside snapshots, and the host's frame time under 2 ms more than solo.
- A save format change bumps `SAVE_VERSION` and migrates older saves.
- Accounts and deploys on Cloudflare are the user's: agents write the code and the steps in
  `docs/WORKFLOW.md`, and the user runs the login and deploy commands.

### Steps

- [ ] **3.1 Actions on the wire** (`action/codec.rs`, `net/mod.rs`, `lib.rs`, `api/net.rs`)
  - A submodule `action/codec.rs` (beside `action/tests.rs`): each `Action` writes to and reads from bytes
    (`ByteWriter` / `ByteReader`), validated like save reads: bad bytes give `None`, never a panic. One
    exhaustive `match` each way, so a new action can't be forgotten.
  - `Game` gets a role: `Solo` (today), `Host` or `Client`. Outside `Solo`, `Game::act` puts the local
    player's actions in an outbox (`take_outbox() -> Vec<u8>`) instead of queuing them.
  - Host: `host_stamp(player, bytes)` stamps a peer's actions and queues them; after each tick the host
    produces that tick's frame (`take_frames() -> Vec<u8>`, all ticks run this frame).
  - Client: `push_frames(bytes)` queues the actions and moves the confirmed tick forward; `run_tick`
    keeps stepping bodies and hands every tick, but `Sim::step` only through the confirmed tick.
  - **Done when:** a round-trip test covers every `Action` variant, and garbage bytes fail cleanly. A
    Rust test wires a host `Game` and a client `Game` (made from the same seed) through byte buffers: both
    players act (break, place, craft, build a small line), and the hashes match at every 60th tick,
    including when the client's frames arrive in late bursts.

- [ ] **3.2 Joining and returning players** (`net/snapshot.rs`, `sim.rs`, `authority.rs`, `save.rs`)
  - A join snapshot is the save bytes (`save_bytes`) plus the actions already queued for future ticks
    (the codec from 3.1), since those aren't in a save. `Game::from_snapshot(bytes, local)` starts a
    client; the host queues the joiner's `Join` in the same order as any other action.
  - Players have a **key**: a random id each browser keeps in `localStorage` and sends when joining.
    `Leave` keeps the inventory and position under that key (`Sim.away`, saved and hashed), and `Join`
    with a known key gives them back. This fixes limitation 1's lost inventory.
  - The host's save stores the away players. Clients never save the host's world (`save/session.ts`
    skips autosave for a client).
  - Save version 10 (away players). Version 9 and older still load with none.
  - Cap: 4 players (`MAX_PLAYERS`, easy to raise). A fifth gets a readable refusal.
  - **Done when:** a test has the host run a factory for a while, a client join mid-run, and the hashes
    match 600 ticks later; a player who leaves and rejoins gets their inventory back; a save round trip
    keeps away players; a version-9 save loads.

- [ ] **3.3 Transport and two tabs** (`web/src/net/transport.ts`, `broadcast.ts`, `protocol.ts`,
  `session.ts`; `main.ts` one registration)
  - `Transport`: `send(bytes)`, `onMessage`, `onClose`, `close()`. Implementations: `Loopback` (a pair,
    for tests and debugging) and `BroadcastChannel` (a room name, for tabs on one machine).
  - `protocol.ts`: a one-byte message type, then the payload. `Hello` (build id, player key, name),
    `Welcome` (player id, snapshot) or `Refuse` (a readable reason), `Actions` (client to host),
    `Frames` (host to clients), `Checksum` (tick, hash), `Bye`. The build id comes from the build (Vite
    `define` with the git commit); a mismatch is refused with "the host runs a different version".
  - `session.ts` runs the host or client side each frame: send the outbox, deliver frames, compare
    checksums, and turn a closed transport into `Leave`.
  - For now, `?host=<room>` and `?join=<room>` in the URL start a session (the UI is step 3.7).
  - **Done when:** two tabs play one world: a block placed or a machine built in either tab shows in
    both, a miner fills a box that both see, and checksums match for 10 minutes (the console shows one
    line per mismatch, and there should be none). Screenshots of both tabs.

- [ ] **3.4 Seeing each other** (`net/players.rs`, `authority.rs`, `render.rs` or a new
  `avatars.rs`, `web/src/net/session.ts`)
  - `PlayerState` at 20 Hz: position, look and flying. The host relays each player's state to the
    others. Remote bodies take the latest state and are smoothed between updates. Only the local body
    runs physics.
  - Avatars: a simple box body and head per remote player, drawn through the instance renderer, with the
    name shown above it (a DOM label, positioned in TS).
  - Loose items: only the host runs item physics and pickups, and sends the items near each client at
    10 Hz (id, item, position). Clients draw that list and don't simulate items.
  - Host streaming: the host generates chunks around every player (no meshing for remote ones), so item
    physics works where remote players are. Limitation 1 goes away.
  - **Done when:** two tabs show each other's avatar moving smoothly; an item thrown in one tab lands in
    both and is picked up by whoever walks over it; the host's frame time is measured with 4 players
    (2 extra added with `add_player`) and recorded here.

- [ ] **3.5 A host in a hidden tab keeps ticking** (spike, then build; `web/src/net/` or `main.ts`)
  - Browsers pause `requestAnimationFrame` in background tabs and throttle timers, so a host that
    switches tabs would freeze everyone. Measure the options for 10 minutes each in a hidden tab:
    - (a) a tiny dedicated Worker that posts 60 Hz messages to the main thread, which runs
      `game.update` without rendering while hidden (cheapest, try first),
    - (b) the whole engine in a Worker, with meshes and instances sent as transferable buffers (one
      copy),
    - (c) the engine in a Worker with `SharedArrayBuffer` (needs COOP/COEP headers; GitHub Pages can't
      set them, so it needs `coi-serviceworker`).
  - Pick the cheapest that keeps a steady tick rate, and record the numbers here.
  - **Done when:** a host tab hidden for 10 minutes keeps a client in step (no growing lag, no checksum
    mismatch).

- [ ] **3.6 WebRTC and Cloudflare signalling** (`signal/` Worker, `web/src/net/webrtc.ts`)
  - `signal/`: a Cloudflare Worker with one Durable Object per room. `POST /room` gives a short room
    code; a WebSocket on `/room/<code>` passes WebRTC offers, answers and ICE candidates between the
    host and each joiner, then steps aside. `GET /ice` returns public STUN plus short-lived Cloudflare
    TURN credentials (the Worker keeps the TURN key as a secret). It gets its own `tsconfig.json`, which
    `npm run check` typechecks. No game logic in the Worker.
  - `webrtc.ts`: a `Transport` over one reliable, ordered data channel per peer; the host holds one per
    client.
  - The user creates the Cloudflare account, runs `npx wrangler login` and `npx wrangler deploy`, and
    sets the TURN secrets. The steps go in `docs/WORKFLOW.md`, and the Worker's URL goes in one constant
    in `web/src/net/`.
  - **Done when:** two browsers on different machines play together through the live site, and a
    session forced through the relay (`iceTransportPolicy: 'relay'`) works too.

- [ ] **3.7 Co-op UI** (`ui/coop.ts` + `.css`)
  - Menu: "Host this world" gives a share link (`?join=<code>`) and the code. "Join" takes a link or a
    code. Each player sets a name, kept in `localStorage`.
  - In game: a player list (Tab) with names and ping; notices when someone joins or leaves; a readable
    message when a join is refused or the connection drops. When the host leaves, the session ends for
    everyone, and clients return to the menu.
  - **Done when:** a friend can join from a pasted link with no URL editing, and every failure case
    (wrong version, full world, host gone, no connection) shows a message a player understands.

- [ ] **3.8 Resync and robustness** (`web/src/net/session.ts`, `net/snapshot.rs`)
  - On a checksum mismatch the client asks for a fresh snapshot and reloads from it, keeping its own
    body and view. It logs the tick and both hashes, so the bug can become a replayable test.
  - A client that falls more than about 5 seconds behind resyncs the same way. A silent peer times out
    and leaves.
  - **Done when:** a test forces a mismatch (a debug action that changes one client's core) and the
    client recovers to matching hashes; a tab closed abruptly leaves cleanly on the host.

- [ ] **3.9 Instant feedback for your own edits** (only if play shows it's needed; `interaction.rs`)
  - A client sees its own actions after the round trip plus `INPUT_DELAY`, about 100–200 ms. If placing
    and breaking feel laggy, show your own block edits at once in the render cache only, and undo that
    when the core disagrees once the action applies. The core stays untouched.
  - **Done when:** measured in play; built only if it helps, otherwise noted here as skipped.

- [ ] **3.10 Milestone cleanup** (every milestone ends with this step)
  - `npm run check` passes with no size warnings. Split anything that grew past its soft limit.
  - Remove dead code and leftover old paths.
  - The code map matches the tree, and the nested `CLAUDE.md` files are current.
  - Compress this plan: Milestone 3 becomes a few lines in section 8. Move Milestone 4 from
    `docs/ROADMAP.md` into section 4 and detail it to this level. Ask the user the open questions for
    M4 (section 6) before detailing it.

**Suggested commits:** one per step, or per clean part of a step. Every commit passes `npm run check`, and
the game still works.

---

## 5. Roadmap after Milestone 3

Milestones 4–7 (exploration, terrain and scale, fluids and depth, endgame) are in `docs/ROADMAP.md`. Read
it only when planning the next milestone; step 3.10 moves Milestone 4 from there into this plan.

---

## 6. Open questions for the user

Ask these when the milestone that needs the answer comes up, not before. Record the answers in section 1
and adjust the steps.

| Needed by | Question |
|---|---|
| Any time | Should the Miner Mk1 and the smelter stay unpowered (a burner tier) while newer machines need power? (Built that way; easy to change.) |
| M4 or later | Should factories keep running while the game is closed (simulate the missed time on load, capped)? |
| M4 | Which ores and rock types, and how rare should veins and lodes be once prospecting can find them? |
| M7 | Megaproject theme (rocket ship or something else) and what launching unlocks. |

---

## 7. Working in this repo

See `docs/WORKFLOW.md`: commands, browser verification, Windows gotchas, deploying, working with the user,
and the balance numbers. Read the section you need.

---

## 8. Change log of this plan

- **2026-09-25:** Plan created after the deposits and factory slice (`3239215`). It records the decisions
  (peaceful, open-ended, co-op), the co-op approach, Milestone 1 in detail, and the roadmap to Milestone 7.
- **2026-09-25:** The user stated the project is developed entirely by AI agents and that keeping the
  codebase from growing unwieldy is of utmost importance. Added section 3.1 (mandatory agent rules), the
  restructure-first step, the milestone cleanup step, and the rule that the plan details only the current
  milestone (milestones 2–7 moved to `docs/ROADMAP.md`, the operational reference to `docs/WORKFLOW.md`).
- **2026-09-25: Milestone 1 (Foundation) done**, commits `31bb05b` to `74fb476`.
  - Built: the restructure for agents (1.0); a fixed 60 Hz tick with at most 8 ticks per frame and an
    interpolated camera (1.1); the core `Sim` separated from the view (1.2); every core change an
    `Action` applied at a tick, answered by `SimEvent`s (1.3); several players, with `Join` / `Leave`
    actions and `authority.rs` (1.4); canonical bytes (`bytes.rs`) and an FNV-1a state hash with
    determinism tests (1.5); the save format (`save.rs`, 1.6); saving in the browser with a world list
    (`web/src/save/`, `ui/worlds.ts`, 1.7); the README (1.8); cleanup (1.9).
  - Deviations worth knowing: the player id sits beside each queued action, not inside it; all tracked
    deposits are hashed and saved, because tracking changes behaviour; world names and play time live in
    the browser's record, not the save; switching worlds reloads the page; each world keeps its previous
    save as a backup slot.
  - Cost: tests 56 → 74. Wasm 81.5 KB gzipped; JS +2.7 KB gzipped.
  - Lessons: new generic code shows up in the wasm size, so measure each step. Hash tests catch
    determinism bugs at the exact tick, and the browser pane can't lock the pointer, so drive
    `window.opencraft.game`.
- **2026-09-25: Milestone 2 (Make it a game) done**, commits `b49b766` to the step 2.11 commit.
  - Built: `ItemId` and the item table (2.1); the machine registry `MACHINES`, the `Machine` trait and
    `Buffer` (2.2); the smelter with machine recipes and fuels (2.3); the machine panel and the
    constructor, actions `SetRecipe` / `Insert` (2.4); splitter and filter as one `Router` kind (2.5);
    ramps, lifts and underpasses as belt shapes (2.6); boxes that open like a chest (user request);
    power with generators, poles, grids and brownouts (2.7); research with labs, red and green packs and
    six techs (2.8); Miner Mk2 (75% recovery) and fast belts behind research (2.9); onboarding tips as
    engine data (2.10); cleanup (2.11).
  - The user chose Factorio-style research, upgrades that raise recovery, and infinite bulk materials.
    The burner tier (Mk1 miner and smelter unpowered) was built as the default.
  - Deviations worth knowing: one kind serves several blocks through extra `MACHINES` rows plus a flag
    or shape; machines hang on the nearest pole, with no manual wiring; power balances at the start of
    each tick, before miners; labs count units in progress so they never overshoot; tips are engine data
    with checks, so TS mirrors no ids.
  - Saves are version 9, and every version since 1 loads (version 1 tested with the committed
    `save/v1.ocworld`, the others in the browser). The golden hash in `sim/tests.rs` was re-recorded on
    purpose at each format change, with the new machine added to the script.
  - Cost: tests 74 → 113. Wasm 81.5 → 120.3 KB gzipped (panels about 8 KB, power about 8, research
    about 6); JS 31.8 KB.
  - Lessons: the golden hash catches every unintended core change; scenario tests with a bare `Factory`
    replace most browser checks; exported getters cost wasm size, so group them per panel.
- **2026-09-25:** Step 2.11 (milestone cleanup): no size warnings and no dead code found; code map and
  `CLAUDE.md` files checked against the tree. The user answered the M3 questions (Cloudflare Worker plus
  Cloudflare TURN; 2–4 players). Milestone 3 moved in from `docs/ROADMAP.md` and detailed as steps
  3.1–3.10.
