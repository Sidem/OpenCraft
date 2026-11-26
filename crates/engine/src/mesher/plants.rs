//! Plants (`block::PLANT`): two crossed quads per block, each emitted in both windings into the cutout list, unshaded
//! by AO and never merged.
//!
//! Invariants: faces 6 and 7 of the vertex format (`mesher.rs` header) are a plant's two diagonals; the texture is the
//! block's first face layer.

use super::{pidx, Mesher, N};
use crate::block::{FACE_TEX, PLANT};

impl Mesher {
    /// Two diagonal quads per plant block (faces 6 and 7), each in both windings, unshaded by AO.
    pub(super) fn mesh_plants(&mut self, light: &[u8]) {
        for y in 0..N {
            for z in 0..N {
                for x in 0..N {
                    let p = pidx(x + 1, y + 1, z + 1);
                    let b = self.pad[p];
                    if !PLANT[b as usize] {
                        continue;
                    }
                    let layer = FACE_TEX[b as usize][0] as u32;
                    self.push_plant(x as u32, y as u32, z as u32, layer, light[p]);
                }
            }
        }
    }

    /// The crossed quads of a plant in chunk cell (x, y, z) drawn with texture `layer`, lit by `light`.
    pub(super) fn push_plant(&mut self, x: u32, y: u32, z: u32, layer: u32, light: u8) {
        let (lo, hi) = (layer & 0x1FF, ((layer >> 9) as u8) << 5);
        for _ in 0..16 {
            self.cutout_light.push(light);
            self.cutout_light.push(hi);
        }
        let v = |dx: u32, dy: u32, dz: u32, face: u32| {
            (x + dx) | ((y + dy) << 6) | ((z + dz) << 12) | (face << 18) | (3 << 21) | (lo << 23)
        };
        let a = [v(0, 0, 0, 6), v(1, 0, 1, 6), v(1, 1, 1, 6), v(0, 1, 0, 6)];
        let b = [v(1, 0, 0, 7), v(0, 0, 1, 7), v(0, 1, 1, 7), v(1, 1, 0, 7)];
        for q in [a, b] {
            self.cutout.extend_from_slice(&q);
            self.cutout.extend_from_slice(&[q[3], q[2], q[1], q[0]]);
        }
    }
}
