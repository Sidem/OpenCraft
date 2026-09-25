# Working in the OpenCraft repo

Operational reference: commands, verification, environment gotchas, deploying, working with the user, and
the balance numbers. Read the section you need; there's no need to read it all every session.
Rules and direction live in `docs/DEV_PLAN.md`.

## 1. Commands

| Command | Purpose |
|---|---|
| `npm run check` | Everything before a commit: format, clippy with warnings as errors, tests, typecheck, size budgets. Quiet output: one line per step, details only for failures. Needs `web/src/wasm` (`npm run build:wasm`). |
| `npm run dev` | Dev server (Vite) plus a Rust watcher. Any `.rs` change rebuilds the wasm and reloads the page. |
| `npm run build:wasm` | Build the engine only (release). |
| `npm run typecheck` | TypeScript check. |
| `npm run build` | Wasm, typecheck, and the Vite production bundle into `dist/`. |
| `cargo test --workspace -q` | Engine tests only (74 after Milestone 1). Add a name to filter. |
| `cargo fmt --all` | Format the engine (`rustfmt.toml`: width 120). |
| `node scripts/check-size.mjs` | Size budgets only. |
| `npx vite build` | Prints gzipped bundle sizes; check wasm size here. |

Line counts: use `(Get-Content <file>).Count`. `Measure-Object -Line` skips blank lines and undercounts.

## 2. Verifying in the browser pane

Driving the pane is the most expensive way to verify something. Prefer headless scenario tests (DEV_PLAN
section 3.1) and use the browser for final visual proof.

- Start with `preview_start` name `opencraft`. `.claude/launch.json` has `autoPort: true`, and
  `scripts/dev.mjs` honours `PORT`, so several sessions can each run a server.
- Pointer lock never engages in the automated pane, so real play can't be tested there.
  `requestAnimationFrame` is paused while the pane is hidden. Drive the engine directly through
  `window.opencraft.game` and keep each `javascript_tool` script synchronous.
- Streaming starts only after `update()` has run. Load the world with:
  `for (i…; !g.ready();) { g.update(1/60); g.begin_work(); while (g.work_step() && n++ < 200); }`
- Hide the menu with `document.getElementById('menu').classList.add('hidden')`.
- Aim by teleporting the eye above a face. `teleport` takes the feet position; the eye is 1.62 higher.
  Then call `set_look(atan2(dx, -dz), asin(dy / len))`. Mine with `set_mining(true)` plus `update` steps;
  place with `set_using(true)`. Take a screenshot to force a render.
- Useful: `block_at`, `find_deposit(tier)` (0 lode, 1 vein, 2 outcrop), `skip_time(s)` (silent),
  `run_ticks(n)`, `give(id, n)`, `craft(r, n)`, `target_detail()`, and `opencraft.inventory.open()`.
  `add_player()` / `remove_player(id)` add an invisible second body at spawn (nothing draws other
  players yet). `state_hash()` (a BigInt) fingerprints the core state; it changes every tick.
- `update(dt)` runs whole 60 Hz ticks, so a single `update(1/144)` may run none. `run_ticks(n)` steps the
  simulation exactly; follow it with `update(0)` to refresh the camera and box instances.
- Mutating calls (`give`, `craft`, `click_slot`, `select_slot`, `drop_selected`, placing, breaking) are
  actions applied at the next tick: run `update(1/60)` or `run_ticks(1)` before reading the result.
- Any edit reloads the page. The world is saved as the page closes and comes back, but keep scene setup
  as one re-runnable script anyway. `?seed=N` starts a new saved world; `opencraft.session.save()`
  saves now. Test worlds pile up in the pane's IndexedDB: `indexedDB.deleteDatabase('opencraft')`.
- A tested scene: the iron outcrop at (11, 62, 2), seed 1337. Dig (11, 64, z) for z = 2..6. Place a box
  at z = 6, belts facing +Z (yaw π) at z = 5, 4, 3, and the miner at z = 2 against the ore top face.

## 3. Environment gotchas (Windows)

- The shell is PowerShell 5.1: no `&&` (use `; if ($?) { … }`). **The Bash tool fails on this machine.**
- `git commit -F -` with a here-string doesn't reach stdin. Write the message to a file in the
  scratchpad and use `git commit -F <file>`. Write that file with the Write tool or `-Encoding ascii`:
  `Out-File -Encoding utf8` adds a BOM, which ends up in the commit subject.
- `Set-Content -Encoding utf8` writes a BOM, which broke `package.json` once. Use the Write and Edit tools
  or Node for files.
- Cargo's "Blocking waiting for file lock" is reported as a NativeCommandError; it's harmless.
- If another session's dev server also writes `web/src/wasm`, `wasm-opt` can fail with "os error 32".
  Wait, then rerun `npm run build:wasm`.

## 4. Deploying

Every push to `main` runs `.github/workflows/pages.yml`: engine tests, build, then publish to GitHub Pages.
**Ask the user before any push to `main`.** The user may ask to push without waiting for the deployment.

## 5. Working with the user

- Performance-first; justify dependencies.
- They are not a sound-design expert. Explain audio, and generally anything technical, in plain language.
- The game is not a Minecraft clone; break conventions when it makes the game better.
- Balance numbers are expected to change; keep them as named constants near the top of their module.
- Offer recommendations, not surveys of options.

## 6. Tuning knobs (values after Milestone 1)

`docs/CODEMAP.md` lists where every tuning constant lives; this is the current balance at a glance.

| Where | Constant | Value |
|---|---|---|
| `deposits.rs` | `HAND_YIELD`, `TAPER_START`, `TAPER_FLOOR` | 3, 0.2, 0.25 |
| `deposits.rs` | `Tier::grade` / `draw_cap` (units per s) | lode 2000 / 20, vein 1000 / 4, outcrop 100 / 1 |
| `factory/miner.rs` | `MINER_RATE`, `MINER_RECOVERY` | 1.0 units/s, 0.6 |
| `factory/belt.rs` | `BELT_SPEED`, `ITEM_SPACING` | 1.0 blocks/s, 0.35 |
| `factory/mod.rs` | `MACHINES` slots | miner 1 stack (64 ore), box 24, smelter and constructor 1 per buffer |
| `recipes.rs` | `MACHINE_RECIPES`, `FUELS` | smelter: 1 ore → 1 ingot in 1.5 s · coal ore 8 s, log 4 s of fire · constructor: 2 iron ingots → plate 2 s, 1 → rod 2 s, rod → 4 screws 3 s, copper ingot → 2 wire 2 s |
| `worldgen/ore.rs` | `ORE_GEN`, `LODE_CHANCE`, `ORE_SPAWN_CLEARING` | per ore (outcrops per column, vein chance, lode weight); 1/40; 10 |
| `recipes.rs` | `RECIPES` | Miner 10 iron, 6 copper, 12 stone · 4 belts 1 iron, 2 stone · Box 6 log, 2 iron · Smelter 16 stone, 4 iron · Constructor 10 iron ingots, 4 copper ingots, 8 stone · Splitter 2 plates, 2 belts · Filter 2 plates, 2 wire, 2 belts |
