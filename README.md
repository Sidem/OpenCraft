# OpenCraft

**A voxel sandbox with a factory builder inside it, in the browser.** Mine and build in a fully editable block world,
then automate it with miners, belts, smelters, power lines, trains and drones. Peaceful, open-ended, co-op for 2–4
players. The whole game is written by AI coding agents.

**Play: <https://sidem.github.io/OpenCraft/>** (desktop browser, keyboard and mouse; no install, no account).

## How to play, in a paragraph

A new world opens in front of a broken production line with a missing belt; fixing it teaches the controls. Hand mining
is slow and lossy, deposits are finite, and machines are efficient, so the game is to automate: research (five science
packs) opens tiers Mk1–Mk5, steam, chemistry, nuclear power, compute, towns and orbit. Terrain is material: quarries leave
pits, and a planner with drones flattens, fills or tunnels whole sites. The manual: [docs/PLAYING.md](docs/PLAYING.md)
(controls, worlds, co-op) and [docs/FACTORY.md](docs/FACTORY.md) (deposits, machines, power, research, every era).

## Run, build, check

Needs Rust stable (`rust-toolchain.toml` adds the wasm target), [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)
and Node.js 20.19 or newer.

```bash
npm install
npm run dev
```

Open <http://localhost:5173>. `?seed=1234` starts a world with that seed; `?rd=12` sets the render distance in chunks.

| Command | What it does |
| ------- | ------------ |
| `npm run dev` | Vite plus a Rust watcher: a `.rs` change rebuilds the wasm and reloads the page |
| `npm run build:wasm` | The engine only (run once so the TS bindings exist); `-- --dev` is faster, `-- --profiling` keeps names |
| `npm run check` | The gate before a commit: fmt, clippy, tests, typecheck, size budgets, code map (quiet) |
| `npm test` | Engine tests only (`cargo test --workspace`; add `-q <name>` to filter) |
| `npm run build` | Production build into `dist/` |

## Where to read next

- **Contributors and agents:** [CLAUDE.md](CLAUDE.md), then [docs/DEV_PLAN.md](docs/DEV_PLAN.md) (status, rules, open
  steps) and [docs/CODEMAP.md](docs/CODEMAP.md) (one line per module). Recipes for adding content:
  [docs/EXTENDING.md](docs/EXTENDING.md); commands, browser driving, deploying: [docs/WORKFLOW.md](docs/WORKFLOW.md);
  settled choices: [docs/DECISIONS.md](docs/DECISIONS.md). See also [CONTRIBUTING.md](CONTRIBUTING.md).
- **Shape:** Rust → wasm (`crates/engine`) owns all state and hot loops, a fixed 60 Hz deterministic core that changes
  only through queued actions (saves, tests and lockstep co-op rest on it). TypeScript/WebGL2 (`web/src`) forwards
  input, draws, and reads meshes, instances and sounds zero-copy from wasm memory. `signal/` is a Cloudflare Worker that
  introduces co-op peers; no game data passes through it.
- **Tests without a browser:** headless scenario tests drive the full `Game`; the running game is scriptable through
  `window.opencraft.game` (recipes in `docs/WORKFLOW.md`).

Pushing to `main` runs `.github/workflows/pages.yml` (build, `npm run check`, publish): **a push deploys the live site.**

Licensed under [MIT](LICENSE). OpenCraft is independent and not affiliated with Mojang, Microsoft or Coffee Stain;
"Minecraft" and "Satisfactory" only describe the genre.
