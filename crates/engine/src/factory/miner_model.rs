//! The miner's model (presentation only): a drill rig standing on the ore it works. Seen with the drill
//! pointing down (it usually stands on an ore block): a base plate on the ore, four posts forming a cage with a
//! screw auger inside it, a motor housing on top with a fan and the status lamp. While it runs the auger turns
//! and works up and down, the fan spins and the motor housing shivers; stopped, everything rests.
//!
//! The model is written once in the drill's frame (`u`, `v`, `w`: `v` runs from the back of the cell, -0.5,
//! to the ore face, +0.5) and [`Frame`] places each box for whichever of the six directions the drill faces.
//! To add a part, `put` it; a turning part passes `Some(angle)` (a turn about the drill axis).

use crate::block::tex;
use crate::math::Vec3;

use super::render::{push_box, INSTANCE_FLOATS};
use super::{smoke, tier_mark};

/// What the model shows besides its place.
pub(super) struct Look {
    /// Face index (`FACES`) from the miner into the ore.
    pub drill: u8,
    /// The tier's side texture and its number (the band's colour).
    pub side: u16,
    pub tier: u8,
    /// The status lamp's texture.
    pub lamp: u16,
    pub running: bool,
    /// A coal miner (`miner_burner.rs`): the smoke's seed. It gets a chimney instead of the tier's band.
    pub coal_seed: Option<f64>,
}

/// Boxes placed in the drill's frame: `u` and `w` across it, `v` along it towards the ore.
struct Frame<'a> {
    out: &'a mut Vec<f32>,
    at: Vec3,
    /// 0, 1 or 2: the world axis the drill points along.
    axis: usize,
    /// +1 when the drill points to the positive side of that axis.
    sign: f64,
    /// A sideways shiver of the parts placed next.
    shake: f64,
}

impl Frame<'_> {
    /// The world axes `u` and `w` run along.
    fn across(&self) -> (usize, usize) {
        match self.axis {
            0 => (1, 2),
            1 => (0, 2),
            _ => (0, 1),
        }
    }

    /// The world position of the frame's point `c` (`u`, `v`, `w`), without the shiver.
    fn point(&self, c: [f32; 3]) -> Vec3 {
        let (ua, wa) = self.across();
        let mut d = [0.0; 3];
        d[ua] = c[0] as f64;
        d[self.axis] = self.sign * c[1] as f64;
        d[wa] = c[2] as f64;
        self.at + Vec3::new(d[0], d[1], d[2])
    }

    /// One box at `c` (`u`, `v`, `w`) of `size` (the same order). `turn` spins it about the drill axis; the box's
    /// own `u` side is then the one that sweeps round.
    fn put(&mut self, c: [f32; 3], size: [f32; 3], turn: Option<f32>, layers: [u16; 3], world_uv: bool) {
        let (ua, wa) = self.across();
        let mut s = [0.0; 3];
        s[ua] = size[0];
        s[self.axis] = size[1];
        s[wa] = size[2];
        let at = self.point([c[0] + self.shake as f32, c[1], c[2]]);
        push_box(self.out, at, 0.0, s, 0.0, layers, world_uv);
        if let Some(angle) = turn {
            // A turn about x is the instance's pitch, about y its yaw, about z its roll.
            let i = self.out.len() - INSTANCE_FLOATS;
            self.out[i + [12, 3, 13][self.axis]] = angle;
        }
    }
}

pub(super) fn write(out: &mut Vec<f32>, rel: Vec3, look: &Look, time: f64) {
    let (spin, fan, bob, shiver) = if look.running {
        (
            (time * 9.0) as f32,
            (time * 26.0) as f32,
            (0.05 * (0.5 + 0.5 * (time * 5.0).sin())) as f32,
            0.004 * (time * 53.0).sin(),
        )
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };
    let sign = if look.drill.is_multiple_of(2) { 1.0 } else { -1.0 };
    let mut f = Frame { out, at: rel, axis: look.drill as usize / 2, sign, shake: 0.0 };
    let frame = [tex::FRAME; 3];
    let ribbed = [tex::DRILL; 3];

    // Standing on the ore: a plate with a post at each corner.
    f.put([0.0, 0.45, 0.0], [0.92, 0.1, 0.92], None, frame, false);
    for (u, w) in [(-0.4, -0.4), (0.4, -0.4), (-0.4, 0.4), (0.4, 0.4)] {
        f.put([u, 0.13, w], [0.1, 0.54, 0.1], None, frame, false);
    }
    // The screw auger inside the cage: a shaft down into the ore and a flight every few centimetres, each
    // turned a little further, so it reads as a thread when it turns.
    f.put([0.0, 0.21 + bob, 0.0], [0.08, 0.82, 0.08], None, ribbed, true);
    for k in 0..8 {
        f.put([0.0, -0.08 + 0.06 * k as f32 + bob, 0.0], [0.3, 0.06, 0.07], Some(spin + 0.7 * k as f32), ribbed, true);
    }
    // The motor housing with the tier's band, a fan on its back and the status lamp.
    f.shake = shiver;
    f.put([0.0, -0.35, 0.0], [0.72, 0.3, 0.72], None, [tex::MINER_TOP, look.side, tex::FRAME], false);
    match look.coal_seed {
        None => {
            f.put([0.0, -0.17, 0.0], [0.76, 0.06, 0.76], None, [tex::FRAME, tex::stripe(look.tier), tex::FRAME], false);
            // The Mk studs on the housing's sides that face the camera.
            if tier_mark::near(rel) {
                let (paint, s, d, half) = ([tex::paint(look.tier); 3], tier_mark::STUD, tier_mark::DEPTH, 0.36 + 0.015);
                for side in [-half, half] {
                    let (u_face, w_face) = (f.point([side, -0.35, 0.0]), f.point([0.0, -0.35, side]));
                    let toward = |face: Vec3| {
                        let n = face - f.point([0.0, -0.35, 0.0]);
                        n.x * face.x + n.y * face.y + n.z * face.z < 0.0
                    };
                    let (u_seen, w_seen) = (toward(u_face), toward(w_face));
                    for o in tier_mark::offsets(look.tier) {
                        if u_seen {
                            f.put([side, -0.35, o], [d, s, s], None, paint, false);
                        }
                        if w_seen {
                            f.put([o, -0.35, side], [s, s, d], None, paint, false);
                        }
                    }
                }
            }
        }
        Some(seed) => {
            // A coal miner burns its fuel: a soot-black chimney rises from the housing and smokes while it digs.
            f.put([-0.2, -0.62, -0.2], [0.16, 0.54, 0.16], None, [tex::SOOT; 3], false);
            f.put([-0.2, -0.9, -0.2], [0.22, 0.06, 0.22], None, [tex::SOOT; 3], false);
            if look.running {
                smoke::puffs(f.out, f.point([-0.2, -0.94, -0.2]), time, seed, 0.4);
            }
        }
    }
    f.put([0.0, -0.49, 0.0], [0.1, 0.04, 0.1], None, ribbed, true);
    f.put([0.0, -0.485, 0.0], [0.36, 0.03, 0.06], Some(fan), ribbed, true);
    f.put([0.0, -0.488, 0.0], [0.36, 0.03, 0.06], Some(fan + std::f32::consts::FRAC_PI_2), ribbed, true);
    f.put([0.25, -0.49, 0.25], [0.12, 0.03, 0.12], None, [look.lamp; 3], false);
}
