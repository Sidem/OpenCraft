//! Flares (GEAR-FLARE, a co-op marker): a key raises a column of light where you look (`flare_spot_at`), seen by
//! every player for [`FLARE_SECONDS`], in the colour of whoever raised it. The host passes each flare on to every peer
//! like a chat line (`web/src/net/protocol.ts`, message `flare`), and each game draws its own copy as a telescope pin's
//! column (`telescope/beacons.rs`, seen from any distance) until it burns out.
//!
//! Invariants: presentation only, never core state, never saved: no `Action`, no hash. Pins stay per browser
//! (`telescope.rs`). At most [`MAX_FLARES`] burn at once; a new one puts out the oldest.

use crate::block::{self, AIR};
use crate::math::Vec3;
use crate::raycast::raycast;
use crate::telescope::push_beacon;
use crate::Game;

/// Seconds a flare burns.
pub const FLARE_SECONDS: f64 = 60.0;
/// How far the view may meet the ground to raise one, in blocks (loaded chunks only).
const AIM_REACH: f64 = 192.0;
/// Flares burning at once.
const MAX_FLARES: usize = 12;
/// A flare's colour by player slot: orange, cyan, lime, magenta.
const COLOURS: [u32; 4] = [0xFF7A30, 0x38D4FF, 0x8CFF4A, 0xFF4FD8];
/// The last seconds, in which a flare flickers as it burns out.
const DYING: f64 = 8.0;

/// One burning flare: where its column stands (the foot), how long it has burned and its colour.
pub struct Flare {
    pub foot: Vec3,
    pub age: f64,
    pub rgb: u32,
}

/// Every flare this game draws.
#[derive(Default)]
pub struct Flares {
    pub list: Vec<Flare>,
}

impl Game {
    /// Where a flare raised now would stand: on top of the block the view meets, `None` if it meets none in reach.
    pub(crate) fn flare_spot_at(&self) -> Option<Vec3> {
        let world = &self.sim.world;
        let solid = |p| world.get_block(p).filter(|&b| b != AIR && block::SOLID[b as usize]);
        let hit = raycast(self.body().eye(), self.body().look_dir(), AIM_REACH, solid)?;
        let top = hit.block + crate::math::IVec3::new(0, 1, 0);
        Some(top.as_vec3() + Vec3::new(0.5, 0.0, 0.5))
    }

    /// Lights a flare at `foot` for player slot `player` (its own, or one a peer raised).
    pub(crate) fn light_flare_at(&mut self, foot: Vec3, player: u8) {
        if !(foot.x.is_finite() && foot.y.is_finite() && foot.z.is_finite()) {
            return;
        }
        if self.flares.list.len() >= MAX_FLARES {
            self.flares.list.remove(0);
        }
        let rgb = COLOURS[usize::from(player) % COLOURS.len()];
        self.flares.list.push(Flare { foot, age: 0.0, rgb });
    }

    /// Burns the flares on by `dt` seconds and writes their columns (camera-relative instances).
    pub(crate) fn write_flares(&mut self, dt: f64, eye: Vec3) {
        self.flares.list.retain_mut(|f| {
            f.age += dt;
            f.age < FLARE_SECONDS
        });
        let reach = self.sim.world.view_distance() * 1.5;
        for f in &self.flares.list {
            let left = FLARE_SECONDS - f.age;
            // Burning out: the column blinks off for a moment twice a second.
            if left < DYING && (left * 4.0).fract() < 0.25 {
                continue;
            }
            push_beacon(&mut self.instances, f.foot - eye, reach, f.rgb);
        }
    }
}

#[cfg(test)]
mod tests;
