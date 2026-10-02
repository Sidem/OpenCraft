//! Camera easing (presentation only, never core state): the eye glides between standing and crouching
//! height instead of snapping, because a sudden vertical jump of the whole view is a common trigger of
//! motion sickness. To ease another view change, add it beside `CrouchGlide` and apply it in `Game::update`.

use crate::math::Vec3;
use crate::player::CROUCH_DIP;

/// How fast the eye settles, per second: about 95% of the way in a fifth of a second.
const GLIDE_RATE: f64 = 15.0;

#[derive(Default)]
pub struct CrouchGlide {
    /// How far the eye is lowered right now, 0 standing to `CROUCH_DIP` crouched.
    dip: f64,
}

impl CrouchGlide {
    /// `eye` as the body reports it (already lowered the whole `CROUCH_DIP` when crouched) with the
    /// lowering eased over `dt` seconds.
    pub fn apply(&mut self, eye: Vec3, crouched: bool, dt: f64) -> Vec3 {
        let target = if crouched { CROUCH_DIP } else { 0.0 };
        self.dip += (target - self.dip) * (1.0 - (-GLIDE_RATE * dt.max(0.0)).exp());
        Vec3::new(eye.x, eye.y + target - self.dip, eye.z)
    }
}

#[cfg(test)]
mod tests;
