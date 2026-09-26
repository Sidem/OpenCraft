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

- [ ] **A0. Direction and a test scene.** Take screenshots of the current look (terrain, a small
  factory, an avatar if you can host two tabs). Propose 2–3 short style options to the user (palette,
  texture treatment, machine colour language) with one quick sample each, for example a restyled stone
  and grass and a shader tint. Record the chosen direction in section 2b. Keep a way to rebuild the
  same test scene for later before/after screenshots (a `javascript_tool` snippet in the log is enough).
- [ ] **A1. Natural blocks.** Richer stone, dirt, grass (top and side lip), sand, logs (bark sides,
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

- [ ] (P5, upcoming) Pickaxe, axe and shovel in stone and iron tiers (steel later), each tier readable
  at a glance: icons, loose-item models and held models. The hotbar gets a wear bar.
- [ ] (P3, upcoming) A leaf-decay particle or puff, if cheap; otherwise none.

## 8. Log

- **2026-09-26:** Handover written; no art work yet.
