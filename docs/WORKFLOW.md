# Working in the OpenCraft repo

Operational reference: commands, verification, environment gotchas, deploying, working with the user, and
composing music. Read the section you need; there's no need to read it all every session.
Rules and direction live in `docs/DEV_PLAN.md`.

## 1. Commands

| Command | Purpose |
|---|---|
| `npm run check` | Everything before a commit: format, clippy with warnings as errors, tests, typecheck (web and `signal/`), size budgets. Quiet output: one line per step, details only for failures. Needs `web/src/wasm` (`npm run build:wasm`). |
| `npm run dev` | Dev server (Vite) plus a Rust watcher. Any `.rs` change rebuilds the wasm and reloads the page. |
| `npm run build:wasm` | Build the engine only (release). |
| `npm run typecheck` | TypeScript check. |
| `npm run build` | Wasm, typecheck, and the Vite production bundle into `dist/`. |
| `cargo test --workspace -q` | Engine tests only. Add a name to filter. |
| `cargo fmt --all` | Format the engine (`rustfmt.toml`: width 120). |
| `node scripts/check-size.mjs` | Size budgets only (lines for code, bytes for docs and the session-start read). |
| `node scripts/check-map.mjs` | The code map's paths exist, every module is on it, the core names no presentation module. |
| `npx vite build` | Prints gzipped bundle sizes; check wasm size here. |
| `node scripts/wasm-sizes.mjs [n] [filter]` | After `npm run build:wasm`: the largest wasm functions by name, to find what grew. Compare with the same list from the last commit (build it in a `git worktree`). |
| `node scripts/inspect-save.mjs <file.ocworld>` | After `npm run build:wasm`: reads a player's exported world headlessly (play time, techs, the tree's pack demand, machines and what holds them back, the inventory). Use it on the user's play-test saves before judging balance. |
| `node scripts/bench-flight.mjs <file.ocworld> [seconds] [yaw] [rd] [sprint]` | After `npm run build:wasm`: loads a save headlessly, flies a line and prints the engine's share of the frame (`update`, streaming `work`, event `drain`: avg, p50, p99, max, frames over 16.6 ms; `SPIKES=1` lists spikes). Run it on a big play-test save after any per-frame change. |
| `OC_SAVE=<unpacked save> cargo test --release -q bench_save_frame -- --ignored --nocapture` | Times each presentation step of `Game::update` on a real save (`src/tests/frame_bench.rs`); gunzip the `.ocworld` first. Add `CARGO_PROFILE_RELEASE_LTO=off` for a fast build. |

Frame budget (60 fps = 16.6 ms): `Game::update` about 2 ms, streaming work at most 6 ms (`main.ts` sizes it from what `update` left), drawing under 6 ms of GPU on a mid card. In the browser pane the page cannot run `requestAnimationFrame` while hidden: step `game.update`, `work_step` and `renderer.render` by hand and time them, and use `EXT_disjoint_timer_query_webgl2` for real GPU time.

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
  `add_player()` / `remove_player(id)` add a second body at spawn. `state_hash()` (a BigInt) fingerprints the core state; it changes every tick.
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

### The co-op signalling Worker (`signal/`, Cloudflare)

Deployed at **<https://opencraft-signal.opencraft.workers.dev>** (the user's account, subdomain
`opencraft`); `SIGNAL_URL` in `web/src/net/signal.ts` points there. Wrangler needs Node 22 or newer
(the machine has Node 24). Agents write code and run it locally (`cd signal; npm run dev`, then
`npm run smoke -- http://127.0.0.1:8787`); the user deploys and holds the secrets:

1. **Deploy:** `cd signal`, `npm install`, `npx wrangler login` (opens the browser), `npx wrangler deploy`.
   Redeploy after changing `signal/src/` or `wrangler.jsonc`. A new `workers.dev` subdomain can refuse
   TLS for a few minutes until its certificate is issued.
2. **TURN (optional; relays only when a direct connection fails):** Cloudflare dashboard → Realtime
   (formerly Calls; `dash.cloudflare.com/?to=/:account/calls`) → TURN → create a key; note its Key ID
   and API token (shown once). Then `npx wrangler secret put TURN_KEY_ID` and `npx wrangler secret put
   TURN_KEY_API_TOKEN`. The first 1,000 GB a month are free, then $0.05/GB; without the secrets `/ice`
   gives free STUN only. `npx wrangler secret delete TURN_KEY_API_TOKEN` turns the relay off.
3. **Check:** `npm run smoke -- https://opencraft-signal.opencraft.workers.dev` ends with "smoke test
   passed" (and "TURN included" once step 2 is done).
4. **Play test:** on the site, "Host this world" in the menu, then open the link it shows on another
   machine. To force the TURN relay, use `<site>/?host&relay` and `<site>/?join=<code>&relay`.

A site on another domain goes in `ALLOWED_ORIGINS` (`wrangler.jsonc`). Durable Objects on the Free plan
allow 100,000 requests a day, far beyond signalling for 2–4 players; over it, requests fail until the
next day (no charge).

## 5. Working with the user

- Performance-first; justify dependencies.
- They are not a sound-design expert. Explain audio, and generally anything technical, in plain language.
- The game is not a Minecraft clone; break conventions when it makes the game better.
- Balance numbers are expected to change; keep them as named constants near the top of their module.
- Offer recommendations, not surveys of options.

## 6. Composing music (the MusicComposer project)

Pieces are written in the user's MusicComposer project (`X:\Programming\Projects\MusicComposer`, LilyPond 2.26 bundled, ffmpeg and Python
with numpy and scipy needed): a score `.ly` becomes MIDI, `tools/render_midi.py` plays it through a small additive synth and a hall
reverb, ffmpeg encodes it. `opencraft/music-by-context/` holds the four place pieces and its `build.ps1` (`.\build.ps1 night` builds
one); `opencraft/overworld-day/` holds the two Open Ground versions (its build also writes the `*_once.ogg` the game uses).
- A piece plays **once** (`render_midi.py --once`, a faded reverb tail), is encoded as **64 kbit/s Opus in an .ogg** (1.3 to 2.1 MB), and
  is rendered to about -20 dBFS RMS (about -18 LUFS). The user wants the music small but not thin (24 kbit/s sounded worse than the old Vorbis loops, 48 to 64 matches them); `scripts/check-music.mjs` fails a file over 2.5 MB.
- To add a piece: write the score, add it to `build.ps1`, copy the `.ogg` to `web/src/audio/music/`, add a row in `audio/music-pieces.ts`
  (places, mood, `'any'` if it may fill in), run `npm run check`. The synth's voices are listed in `render_midi.py` (`GM`, `DRUMS`).
- The agent cannot hear: measure with ffmpeg (`ebur128`, `astats`) and leave taste to the user, who tunes a piece's `gain` in its row.
- Tests for the director run with `node --test web/test/*.test.mjs` (Node 24 runs the TypeScript directly: no enums or constructor parameter
  properties in `music-director.ts` and `surroundings.ts`).
