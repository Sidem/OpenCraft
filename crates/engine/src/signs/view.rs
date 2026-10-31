//! Drawing the note signs (presentation only: nothing here changes the core). A sign is a wooden board across its two
//! cells, flat against the block behind it, or on two posts when nothing is there, with its text on two thin face boxes
//! in front. The host paints the text into texture layers (`tex::SIGN_FACE`, two a slot: the left and right half;
//! `web/src/render/sign-faces.ts`): the [`SLOTS`] nearest signs within [`NEAR`] blocks each keep a slot while they stay
//! near, so a layer is painted again only when another sign takes its slot or the text changes (`stamp`). A board
//! without a slot shows bare wood.
//!
//! To change the model: `write_signs` and the sizes below.

use std::f32::consts::FRAC_PI_2;

use super::Sign;
use crate::block::{tex, SOLID};
use crate::bytes::fnv1a;
use crate::factory::{push_box, DIRS};
use crate::math::{sort_small_by_key, IVec3, Vec3};
use crate::Game;

/// Signs whose text is drawn at once (two texture layers each).
pub const SLOTS: usize = 12;
/// Which sign's anchor each slot shows.
pub type Slots = [Option<IVec3>; SLOTS];
/// Text is drawn on signs within this many blocks; at 1080p a line is about 6 px tall there.
const NEAR: f64 = 40.0;
/// Boards farther away are not drawn at all.
const FAR: f64 = 160.0;
/// The board across both cells, and its face's square halves (the text is twice as wide as tall, as its layers).
const BOARD: [f32; 3] = [1.96, 0.96, 0.06];
const FACE: f32 = 0.86;
const FACE_DEPTH: f32 = 0.01;
/// A post holding up a sign with no wall behind it, and how far from the middle each one stands.
const POST: [f32; 3] = [0.08, 1.0, 0.08];
const POST_OUT: f64 = 0.7;

/// A fingerprint of what a sign's faces show (never 0, which `sign_faces` uses for an empty slot).
pub fn stamp(s: &Sign) -> u32 {
    let mut bytes = Vec::with_capacity(64);
    for v in [s.pos.x, s.pos.y, s.pos.z, s.back as i32, s.ink as i32] {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    for line in &s.lines {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(0);
    }
    (fnv1a(&bytes) as u32) | 1
}

impl Game {
    /// Draws every sign within `FAR` of the camera at `eye`, the text of those holding a slot.
    pub(crate) fn write_signs(&mut self, eye: Vec3) {
        self.assign_sign_slots(eye);
        let world = &self.sim.world;
        let solid = |p: IVec3| world.get_block(p).is_some_and(|b| SOLID[b as usize]);
        for s in &self.sim.signs.list {
            let (right, back) = (DIRS[(s.dir as usize + 1) % 4].as_vec3(), DIRS[s.dir as usize].as_vec3());
            let wall = s.cells().iter().any(|&c| solid(c + DIRS[s.dir as usize]));
            let depth = if wall { 0.5 - BOARD[2] as f64 / 2.0 } else { 0.0 };
            let centre = s.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) + right * 0.5 + back * depth - eye;
            if centre.length() > FAR {
                continue;
            }
            let yaw = s.dir as f32 * FRAC_PI_2;
            push_box(&mut self.instances, centre, yaw, BOARD, 0.0, [tex::PLANKS; 3], true);
            if !wall {
                for side in [-1.0, 1.0] {
                    let at = centre + right * (side * POST_OUT) + back * (BOARD[2] as f64 / 2.0 + POST[2] as f64 / 2.0);
                    push_box(&mut self.instances, at, yaw, POST, 0.0, [tex::PLANKS; 3], true);
                }
            }
            let Some(slot) = self.sign_slots.iter().position(|&p| p == Some(s.pos)) else { continue };
            let front = centre - back * (BOARD[2] as f64 / 2.0 + FACE_DEPTH as f64 / 2.0);
            for (half, side) in [-0.5, 0.5].into_iter().enumerate() {
                let layer = tex::SIGN_FACE + 2 * slot as u16 + half as u16;
                let at = front + right * (side * FACE as f64);
                push_box(&mut self.instances, at, yaw, [FACE, FACE, FACE_DEPTH], 0.0, [layer; 3], false);
            }
        }
    }

    /// Frees the slots of signs gone or out of `NEAR`, then gives free slots to the nearest signs without one.
    fn assign_sign_slots(&mut self, eye: Vec3) {
        let signs = &self.sim.signs.list;
        let dist = |s: &Sign| (s.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye).length();
        let near = |p: IVec3| signs.iter().find(|s| s.pos == p).is_some_and(|s| dist(s) <= NEAR);
        for slot in &mut self.sign_slots {
            if slot.is_some_and(|p| !near(p)) {
                *slot = None;
            }
        }
        let held = self.sign_slots;
        let mut waiting: Vec<(i64, IVec3)> = signs
            .iter()
            .filter(|s| dist(s) <= NEAR && !held.contains(&Some(s.pos)))
            .map(|s| ((dist(s) * 64.0) as i64, s.pos))
            .collect();
        sort_small_by_key(&mut waiting, |w| w.0);
        let mut free = self.sign_slots.iter_mut().filter(|s| s.is_none());
        for (_, pos) in waiting {
            let Some(slot) = free.next() else { break };
            *slot = Some(pos);
        }
    }

    /// Each slot's sign as the host paints it: x, y, z and `stamp` (0 for an empty slot) a slot.
    pub(crate) fn sign_slot_list(&self) -> Vec<i32> {
        let mut out = Vec::with_capacity(SLOTS * 4);
        for slot in self.sign_slots {
            match slot.and_then(|p| self.sim.signs.list.iter().find(|s| s.pos == p)) {
                Some(s) => out.extend_from_slice(&[s.pos.x, s.pos.y, s.pos.z, stamp(s) as i32]),
                None => out.extend_from_slice(&[0; 4]),
            }
        }
        out
    }
}
