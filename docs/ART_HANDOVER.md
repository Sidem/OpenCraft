# Art handover: textures and 3D models

Brief for the agent that improves OpenCraft's looks (block textures, item icons, machine and avatar
models, held items) **in parallel** with the agent building the roadmap (`docs/DEV_PLAN.md`). Everything
here is presentation: it never changes game rules, saves or co-op. Keep this file current: tick requests,
add to the log at the bottom, keep it under 300 lines.

## 1. Set-up: your own worktree and branch

The roadmap agent works on `main` in `X:\Programming\Projects\OpenCraft`. Never edit files there.

1. Your worktree exists already (made from the main checkout with
   `git worktree add ..\OpenCraft-art -b art main`). Work only in `X:\Programming\Projects\OpenCraft-art`.
2. Its `npm install` and `npm run build:wasm` have been run; confirm `npm run check` is green before you
   start.
3. Browser: `preview_start` name `opencraft` from the worktree. `.claude/launch.json` has `autoPort`, so
   it picks a free port if the main checkout's dev server holds 5173.
4. Stay close to `main`: before each step and before asking for a merge, `git rebase main` (the branch
   `main` is shared by both worktrees, so no fetch is needed), then `npm run check`.
5. **Commits and merges:** ask the user whether you may commit on `art` at will. **Never merge into
   `main` and never push.** Every push to `main` deploys the live site. When a batch is done, green
   and rebased, tell the user. They (or the roadmap agent, on their say) run `git merge --ff-only art` on
   `main`.
6. Windows: use PowerShell and the file tools; the Bash tool fails here. Never round-trip files through
   `Get-Content` / `Set-Content`: they turn UTF-8 (→ · × —) into mojibake and add a BOM. Use the Edit/Write
   tools, or Node `.mjs` scripts for bulk edits. For commit messages, write a file and use `git commit -F <file>`.

Read first: `CLAUDE.md`, `crates/engine/CLAUDE.md`, `web/src/CLAUDE.md`, `docs/CODEMAP.md`, and
`docs/DEV_PLAN.md` section 3 (engineering rules; mandatory). Don't read the rest of the plan; it's the
roadmap agent's.

## 2. How the looks work today

- **Textures** are generated in Rust, procedurally, at start-up: `crates/engine/src/textures.rs`
  (`generate()`, the noise helpers `n` / `smooth` / `rgb`, and natural patterns) and
  `textures/machines.rs` (machine and item patterns). `pixel(layer, x, y)` maps each layer to a pattern.
  They are 16 × 16 (`TEX_SIZE`), RGBA, one layer of a WebGL2 texture array per `block::tex` constant
  (`tex::COUNT` = 51 today). The magenta placeholder test (`textures/tests.rs`) fails if a layer has no
  pattern.
- **Blocks** (`block.rs`): each `BlockDef` names 6 face layers (+X, -X, +Y, -Y, +Z, -Z). `Render::Opaque`
  and `Cutout` (leaves: alpha below 0.5 is cut) go through the greedy chunk mesher. Merged quads *tile*
  their texture (UVs from world position), so textures must tile seamlessly. `Render::None` blocks
  (machines) are drawn as models instead.
- **Models** are boxes, all drawn in one instanced draw call (`web/src/render/boxes.ts`, shader `boxVert`
  in `render/shaders.ts`). Each box is 12 floats written by `factory::push_box(out, centre, yaw, size,
  uv_scroll, [top, side, bottom layers], world_uv)` (`factory/render.rs`). `world_uv` true = texels at
  world scale like terrain; false = the whole texture on each face (items).
  - Machines: the `fn model(&self, out, rel, time)` of each kind in `crates/engine/src/factory/*.rs`
    (miner, belt, belt_shape, smelter, constructor, lab, generator, power: poles and wires, router).
    `time` animates (drills pump, fans spin); lamps show state.
  - Loose items: `entities::push_item_box` (one box from the item's `ItemDef.tex` and `size`, bobbing).
  - Avatars (co-op players): `avatars.rs` (body, head, visor; `tex::AVATAR_*` layers).
- **Item icons** (hotbar, inventory, panels): `web/src/ui/hud.ts` `itemIcon` draws an isometric box from
  `game.item_icon(id)` (`api/content.rs`: the item's 3 layers and proportions).
- **Lighting** is simple: fixed face shading plus ambient occlusion in the mesher, and fog. No day/night
  yet (the roadmap agent adds that in Milestone 4).
- **No held item** in first person yet, and avatars hold nothing.

## 2b. Art direction: its own look, not a cheap Minecraft clone

**Chosen: B · Alpine (2026-09-26).** Blue slate, fern greens and cool earth; ivory machinery with
burnt-orange accents when A4 lands. Natural textures use small palettes, angular mineral plates,
quiet grass tufts, bark/end grain and clustered leaf cutouts. Low-frequency, world-anchored terrain
tint softens repetition without changing lighting or gameplay. The A0 alternatives are retired;
`/art-preview.html` now uses the production textures and renderer. `?view=materials` tiles every
natural texture 3×3 for inspection. Original screenshots remain under `artifacts/a0/`.

**A4 silhouette brief from user feedback:** recognition must come from geometry at a glance, before
the viewer reads a label or colour. Miner/extractor: low tracked base and a large ground-facing
drill. Generator: broad turbine/flywheel, fuel hopper and a high exhaust. Smelter: tall furnace
body, chimney, dark ore inlet and glowing crucible. Storage container: broad stackable chest with
a hinged lid/seam and reinforced corners. Constructor (composer): open workbench with a moving
press or arm over the input tray. Lab: glass chamber and a short antenna. Keep family colour
consistent, but give each a distinct outline; stay near 12 boxes per machine and stop work
animation when idle or blocked. Implement in A4 after A2–A3, per the agreed step order.

The user's brief: make the existing textures and models **more beautiful and less like a cheap
Minecraft clone**. The game is a frontier-industry sandbox (voxel world + Satisfactory-style factories),
peaceful and open-ended. Ideas to start from; settle the direction with the user in step A0:

- **Palette over noise.** Minecraft's look is per-pixel random noise in saturated flat colours. Instead:
  a small, harmonious palette per material (3–5 tones), soft value ramps, and shapes (strata in stone,
  clumps and blades in grass, grain in wood, crystal facets in ore) rather than salt-and-pepper speckle.
- **Large-scale variation.** Repeating 16 × 16 tiles are the clone giveaway. Break the grid in the
  chunk shader: a low-frequency tint and brightness from world position (a cheap hash noise over a few
  blocks), so a hillside shifts in hue. Presentation only, in `shaders.ts`.
- **Materials that read.** Rock looks heavy, grass soft, metal machined: edge highlights and shadowed
  seams in the texture itself, since lighting is simple.
- **Machines with character.** Industrial but friendly: chunky bevelled housings (several boxes, not a
  cube with a texture), a consistent colour language (safety-yellow trims for moving parts, teal or
  orange accents per machine family), visible working parts, readable status lamps.
- **Avatars** that don't look like Steve: a suited explorer with a helmet, proportions of their own.

## 3. What you own, what you share, what you never touch

**Yours to change freely:**
- `crates/engine/src/textures.rs` and `crates/engine/src/textures/` (split into more files by theme
  as they grow, for example `textures/nature.rs`, `textures/items.rs`).
- `avatars.rs` (the model part), `entities::push_item_box` (looks only), `factory/render.rs` `push_box`.
- The bodies of the `fn model` functions in the factory files, **only those functions** and look
  constants next to them.
- `web/src/render/shaders.ts`, `render/boxes.ts`, and `hud.ts` `itemIcon`.
- New presentation-only modules you create (for example `viewmodel.rs` for the held item,
  `item_models.rs`), each with a header and a CODEMAP row.

**Shared: small, append-only edits, expect merges:**
- `block.rs`: the `tex` constants and `tex::COUNT`, and a block's `faces`. Append layers at the end;
  never renumber. If both branches appended, the second to merge renumbers its new layers after the
  other's (a rebase conflict you resolve by keeping both).
- `item.rs`: an item's `tex` and `size`. `render/renderer.ts`: small edits for your uniforms (for
  example the camera's world position for the tint) and one registration line per new pass.
- `docs/CODEMAP.md`: rows for your files.

**Never touch** (they are the roadmap agent's, or change the game):
- Core state and rules: `sim*`, `action*`, `world/`, `worldgen/`, `deposits.rs`, `save.rs`, `net/`,
  factory logic, recipes, block ids, item ids, `BlockDef` fields other than `faces`, balance numbers.
- `docs/DEV_PLAN.md`, `README.md` (ask the roadmap agent through the user), UI panels and CSS other than
  icons, audio.

Rule of thumb: **if the golden hash test (`sim/tests.rs` `GOLDEN_HASH`) or any non-texture test changes,
you touched the game.** Undo that part. Looks never need to change it.

## 4. Rules that bite

- **Budgets** (`npm run check` enforces them): source files warn at 400 lines and fail at 600. Split
  before you add to a file near the limit (`textures/machines.rs` is at 289). Tests go in `tests.rs`,
  never inline. Every module starts with a `//!` header (what it owns, how to extend).
- **Wasm size:** today the wasm is 131.5 KB gzipped. Your budget is **+12 KB gzipped for all art work**
  unless the user agrees to more. Measure after every step (`npx vite build`; what grew:
  `node scripts/wasm-sizes.mjs`). No float formatting, no new std sorts, no new crates. Share noise
  helpers; prefer a few parameterised patterns over many one-off functions.
- **Procedural stays procedural.** Don't add PNG or model files or loaders without asking the user
  first (it changes the pipeline and the download size).
- **Performance:** a machine model may have about 12 boxes at most, since big factories draw thousands.
  The texture array is generated once at start-up; keep `generate()` under 20 ms. Check frame time in
  the debug overlay (F3) near a busy factory.
- **Readability at distance:** keep textures at 16 × 16 pixel art unless the user asks for more
  (`TEX_SIZE`; the crack overlay in `shaders.ts` assumes 16 cells). Ores must stay distinct from each
  other and from stone at a glance, for players with colour-vision deficiency too (vary pattern and
  brightness, not just hue).
- **Tiling:** block textures tile across greedy-merged faces, so no borders or edge seams on natural
  blocks. Machines use `FRAME`-style borders on purpose.
- **Newer clippy in CI** (Rust 1.98; local is 1.87): write `x.is_multiple_of(n)`, not `x % n == 0`.
- Tests: `cargo test --workspace -q` while iterating; `npm run check` before telling the user a step
  is done.

## 5. Verifying

- `preview_start` name `opencraft`, then drive `window.opencraft.game` with `javascript_tool`: hide the
  menu (`document.getElementById('menu').classList.add('hidden')`), give yourself items (`game.give(id,
  count)`), place blocks and machines in view, and take screenshots. Pointer lock never engages in the
  automated pane, and `requestAnimationFrame` pauses while the pane is hidden: call
  `game.update(1/60)` yourself (details in `web/src/CLAUDE.md`).
- Show the user before/after screenshots for every batch. Ask their taste on anything bold.

## 6. Work list (in this order; one step per session, green at the end)

Start by restyling what exists (A0–A4, then A6's avatar look); new things (A5, A7, A8) come after.

- [x] **A0. Direction and a test scene.** Take screenshots of the current look (terrain, a small
  factory, an avatar if you can host two tabs). Propose 2–3 short style options to the user (palette,
  texture treatment, machine colour language) with one quick sample each, for example a restyled stone
  and grass and a shader tint. Record the chosen direction in section 2b. Keep a way to rebuild the
  same test scene for later before/after screenshots (a `javascript_tool` snippet in the log is enough).
- [x] **A1. Natural blocks.** Richer stone, dirt, grass (top and side lip), sand, logs (bark sides,
  rings on the ends), leaves (cutout holes), spent rock; the rest of `DEFS` in `block.rs` to match.
  Seamless tiling.
- [ ] **A2. Ores.** Every ore readable at 20 blocks and distinct without colour (shape of flecks,
  brightness). Keep each ore's family look in sync with its ingot and plate icons.
- [ ] **A3. Item icons and loose items.** Ingots, plates, rods, screws, wire, science packs. If single
  boxes aren't enough, add multi-box item models (`item_models.rs`, keyed by item id, used by
  `push_item_box` and a new icon getter). One getter in `api/content.rs`.
- [ ] **A4. Machine models.** Clearer silhouettes, so each machine is recognisable from its outline;
  animation that shows work (and stops when idle, no power or blocked; the lamps already carry state).
  Miners, smelter, constructor, lab, generator, pole, belts and belt shapes, splitter and filter; the
  storage box is a plain textured cube (texture only).
- [ ] **A5. Held item in first person** (`viewmodel.rs` plus a pass in `renderer.ts`): the selected
  hotbar item or an empty hand, a swing while mining, a bob while walking. Presentation only: read the
  selected slot and the hands' mining progress through getters.
- [ ] **A6. Avatars.** A nicer figure with a walk cycle (from speed), a head that follows the pitch, the
  held item, and a colour per player.
- [ ] **A7. Tools** (when the roadmap agent's step P5 lands, see requests): icons and held models
  for the pickaxe, axe and shovel.
- [ ] **A8. Shader polish** (ask the user, and coordinate with Milestone 4's day/night and lighting):
  unlit lamp texels, better fog colour, subtle per-face variation.

## 7. Requests from the roadmap agent

The roadmap agent adds a line here when gameplay needs a look. It will already have appended a `tex`
layer with a plain placeholder pattern, so nothing is blocked. Tick the line when you replace it.

- [ ] (P5, landed) Pickaxe, axe and shovel in stone and iron tiers (steel later), each tier readable
  at a glance: icons, loose-item models and held models. Placeholders: layers `tex::STONE_PICKAXE`..`IRON_SHOVEL`
  (51–56) drawn by `textures/tools.rs` on flat plates (`item.rs` `tool()` size). The hotbar wear bar is
  `.slot.tool::after` in `hud.css`.
- [ ] (Saplings, landed) A sapling: layer `tex::SAPLING` (57) by `textures/plants.rs`, drawn on two
  crossed quads (`Render::Plant`), so keep the background transparent. Also its icon and loose item.
- [ ] (4.2, landed) Rocks and minerals for Milestone 4's biomes: granite, sandstone, basalt (building
  rocks that fill whole regions underground, so they must tile well and read apart from stone),
  limestone and quartz ore (deposit ores), glass (cutout: keep most texels transparent). Layers
  `tex::GRANITE`..`tex::GLASS` (58–63) in `textures/geology.rs`; quartz uses `ore()` in `textures.rs`.
- [ ] (4.6, landed) Prospecting devices, drawn like the tools (flat item plates): a handheld scanner
  (`tex::SCANNER`, 68) and a core drill (`tex::CORE_DRILL`, 69); `scanner()` and `core_drill()` in
  `textures/tools.rs`.
- [ ] (4.5, landed) Stained soils that hint at ore below: rusty (iron), dark (coal), verdigris
  (copper), pale (limestone, quartz). Readable from a distance and on the minimap, but still soil.
  Layers `tex::RUSTY_SOIL`..`tex::PALE_SOIL` (64–67), `soil()` in `textures/geology.rs`.
- [ ] (P3, landed) A leaf-decay particle or puff, if cheap; otherwise none. There is no particle system;
  the `LeafDecayed` event (`events.rs`) only plays a sound today.

## 8. Log

- **2026-09-26:** Handover written; no art work yet.
- **2026-09-26 (roadmap agent):** `main` gained P1–P5: new `tex` layers 51–56 (tools) with `tex::COUNT` = 57,
  an arm in `textures.rs` `pixel` and `textures/tools.rs`. Rebase `art` onto `main`; renumber your appended
  layers after 56 if you added any.
- **2026-09-26 (roadmap agent):** `main` gained saplings: `tex::SAPLING` = 57 (`tex::COUNT` = 58) in the new
  `textures/plants.rs`, and plant quads in the chunk shader (`shaders.ts` faces 6 and 7). Renumber your
  appended layers after 57. Then 4.2 added layers 58–63 (`textures/geology.rs`; `tex::COUNT` = 64), and 4.5 layers 64–67 (`tex::COUNT` = 68), and 4.6 layers 68–69 (`tex::COUNT` = 70).
- **2026-09-26, A0 review:** Read all required rules; `git rebase main` was already up to date.
  Added `web/art-preview.html` and `web/art/{scene,palettes}.js` for comparison only (Vite's production
  entry does not import them). Rebuild with `npm run dev`, visit `/art-preview.html?style=0..3`.
  The fixture creates a fresh seed-2024 Game, finds the same clearing, places an 8×6 grass station,
  stone sample wall, miner/belt/box, smelter, constructor, generator and pole through public actions.
  Fixed camera: eye `(30,75.62,19)`, yaw `-0.65`, pitch `-0.55`; 25 model boxes. No saved world used.
  Browser screenshots: `artifacts/a0/{before,a-fieldworks,b-alpine,c-claylands}.png` (review evidence,
  not game texture assets). Avatar comparison not captured. `preview_start` was unavailable; ran the
  configured `npm run dev` directly in the art worktree and used the Codex browser pane on port 5173.
  `npm run check`: green, 127 tests; golden hash unchanged. `npx vite build`: wasm 358.39 kB raw,
  **131.49 kB gzip (+0 art)**; `node scripts/wasm-sizes.mjs` inspected. The user subsequently chose B · Alpine; A0 is complete. No commit, merge or push; first commit still requires user permission.
- **2026-09-26, A1:** Rebased `art` on `main`; replaced all 10 natural texture layers with
  Alpine palettes, mineral cleavage, fine grain, grass tufts and side lip, pale rippled sand,
  bark/end rings, cutout foliage, bedrock and spent rock. The chunk shader applies periodic
  world-position tint and a small continuous UV drift to break repetition. Shader leaves keep
  their cutouts crisp. User feedback requested more material detail and recognisable machine
  silhouettes; the latter is recorded as the A4 brief above. Same fixture screenshots:
  `artifacts/a0/before.png`, `artifacts/a1/after.png` and `artifacts/a1/materials.png`.
  Tile-edge and foliage tests pass; `npm run check` green (129 tests), release tests pass,
  golden hash unchanged. Vite wasm **133.39 kB gzip**, up **1.90 kB** from A0 and within the
  +12 kB art budget. Whole `Game::new` median 0.76 ms (includes texture generation, under
  20 ms); synthetic 1,600-box browser render 0.10 ms median with or without tint at the
  available timer resolution, GL error 0. User approved commits on `art`; no merge or push.
