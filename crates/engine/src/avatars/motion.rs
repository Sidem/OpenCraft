//! Distance-paced gait and eased poses for the Kestrel rig. Only reads body state.
//! Gesture envelopes survive button release so a quick placement has a visible follow-through.

use crate::player::Player;
use std::f64::consts::TAU;

#[derive(Default)]
pub(super) struct Motion {
    /// Local crosshair surface relative to the camera; remote players use their reported look.
    pub focus: Option<crate::math::Vec3>,
    pub phase: f64,
    pub stride: f64,
    pub crouch: f64,
    pub air: f64,
    pub swim: f64,
    pub lean: f64,
    pub strafe: f64,
    pub mining: f64,
    pub using: f64,
    pub work_phase: f64,
    pub use_phase: f64,
}

impl Motion {
    pub fn advance(&mut self, body: &Player, dt: f64) {
        let k = 1.0 - (-12.0 * dt).exp();
        let speed = (body.vel.x * body.vel.x + body.vel.z * body.vel.z).sqrt();
        let (sy, cy) = body.yaw.sin_cos();
        let forward = body.vel.x * sy - body.vel.z * cy;
        self.stride += ((speed / 4.3).min(1.35) - self.stride) * k;
        self.phase = (self.phase + speed.min(7.0) * dt * TAU / 2.5).rem_euclid(TAU);
        self.crouch += (f64::from(body.crouched()) - self.crouch) * k;
        self.air += (f64::from(!body.on_ground && !body.in_water) - self.air) * k;
        self.swim += (f64::from(body.in_water) - self.swim) * k;
        self.lean += ((forward / 6.6).clamp(-1.0, 1.0) - self.lean) * k;
        self.strafe += (((body.vel.x * cy + body.vel.z * sy) / 6.6).clamp(-1.0, 1.0) - self.strafe) * k;
        let mine = body.gesture & 1 != 0;
        let using = body.gesture & 2 != 0 && !mine;
        self.mining += (f64::from(mine) - self.mining) * k;
        self.using += (f64::from(using) - self.using) * (1.0 - (-(if using { 24.0 } else { 7.0 }) * dt).exp());
        if mine || self.mining > 0.01 {
            self.work_phase = (self.work_phase + dt * TAU / 0.48).rem_euclid(TAU);
        } else {
            self.work_phase = 0.0;
        }
        if using || self.using > 0.01 {
            self.use_phase = (self.use_phase + dt * TAU / 0.22).rem_euclid(TAU);
        } else {
            self.use_phase = 0.0;
        }
    }
}
