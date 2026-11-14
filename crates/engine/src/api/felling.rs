//! The host's view of felling (`sim/felling.rs`): where the mark is on the tree under the crosshair, and the trees
//! that just came down, meshed whole, for the host to draw falling (`web/src/render/falling.ts`). Presentation only.
//!
//! `take_fallen` layout, per part (a tree is two parts, its lower and upper 32 blocks): pivot x, y, z and the fall's
//! angle (radians from +x towards +z) as f32 bits; the part's origin x, y, z as i32; its opaque and cut-out quad
//! counts; the number of shaped words; then the words (`mesher/shaped.rs`). The first word is the part count.

use wasm_bindgen::prelude::*;

use crate::block::AIR;
use crate::chunk::Chunk;
use crate::math::IVec3;
use crate::mesher::{neighbor_index, Mesher, PAD_VOLUME};
use crate::Game;

/// Full daylight for a fallen tree's mesh: it is drawn in the open.
static DAYLIGHT: [u8; PAD_VOLUME] = [15; PAD_VOLUME];

#[wasm_bindgen]
impl Game {
    /// The mark on the grown tree under the crosshair, x, y, z; empty before its first swing.
    pub fn chop_mark_at(&mut self) -> Vec<f32> {
        self.chop_mark().map_or_else(Vec::new, |m| vec![m.x as f32, m.y as f32, m.z as f32])
    }

    /// The trees that came down since the last call, meshed (layout in the module header).
    pub fn take_fallen(&mut self) -> Vec<u32> {
        let fallen = std::mem::take(&mut self.sim.fallen);
        let mut out = vec![0u32];
        for tree in fallen {
            let base =
                IVec3::new(tree.foot.x.floor() as i32 - 16, tree.foot.y as i32 - 4, tree.foot.z.floor() as i32 - 16);
            let mut parts = [Chunk::uniform(AIR), Chunk::uniform(AIR)];
            let mut used = [false; 2];
            for &(p, b) in &tree.blocks {
                let l = p - base;
                if (0..32).contains(&l.x) && (0..64).contains(&l.y) && (0..32).contains(&l.z) {
                    parts[(l.y / 32) as usize].set(l.x as usize, (l.y % 32) as usize, l.z as usize, b);
                    used[(l.y / 32) as usize] = true;
                }
            }
            let air = Chunk::uniform(AIR);
            // Most trees fill only the lower part: an empty one is not meshed.
            for i in (0..2).filter(|&i| used[i]) {
                let mut refs = [&air; 27];
                refs[neighbor_index(0, 0, 0)] = &parts[i];
                refs[neighbor_index(0, if i == 0 { 1 } else { -1 }, 0)] = &parts[1 - i];
                let m = Mesher::new().mesh(&refs, &DAYLIGHT, None);
                if m.shaped.is_empty() {
                    continue;
                }
                out[0] += 1;
                let f = |v: f64| (v as f32).to_bits();
                out.extend([f(tree.foot.x), f(tree.foot.y), f(tree.foot.z), tree.angle.to_bits()]);
                out.extend([base.x as u32, (base.y + 32 * i as i32) as u32, base.z as u32]);
                out.extend([m.shaped_opaque, m.shaped_cutout, m.shaped.len() as u32]);
                out.extend_from_slice(&m.shaped);
            }
        }
        out
    }
}
