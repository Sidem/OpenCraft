//! Camera easing and the optional third-person view (presentation only, never core state).
//! `CrouchGlide`: the eye glides between standing and crouching height instead of snapping, because a
//! sudden vertical jump of the whole view is a common trigger of motion sickness.
//! `ThirdPerson`: a comfort option (the host sets the distance, 0 is first person). The camera sits
//! behind the right shoulder, leaving the avatar left of the crosshair. Interaction traces this
//! camera ray, then checks reach and visibility from the hands (interaction/aim.rs).
//! The local player's avatar is drawn from `feet` (avatars.rs). To ease another view change, add it here
//! and apply it in `Game::update`.

use crate::math::{IVec3, Vec3};
use crate::player::CROUCH_DIP;

/// How fast the eye settles, per second: about 95% of the way in a fifth of a second.
const GLIDE_RATE: f64 = 15.0;
/// Longest distance behind the eye the host may ask for.
pub const MAX_DISTANCE: f64 = 10.0;
/// Gap kept between the camera and a block behind it, so the near plane never cuts into it.
const CLEARANCE: f64 = 0.3;
/// Spacing of the samples looked at along the line behind the eye.
const PROBE_STEP: f64 = 0.1;
/// How fast the camera backs away from the eye (per second) and how fast it closes in on a wall.
const ZOOM_OUT_RATE: f64 = 3.0;
const ZOOM_IN_RATE: f64 = 25.0;
/// Shoulder offset grows with distance, then caps so a distant camera keeps aiming near the body.
const SHOULDER_RATIO: f64 = 0.24;
const MAX_SHOULDER: f64 = 1.15;

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

#[derive(Default)]
pub struct ThirdPerson {
    /// The distance the player wants behind the eye; 0 is first person.
    pub distance: f64,
    /// The distance in use now: eased, and shortened by blocks in the way.
    current: f64,
    /// Where the local player's feet are drawn this frame, when the camera is out of the head.
    pub feet: Option<Vec3>,
}

impl ThirdPerson {
    pub fn active(&self) -> bool {
        self.current > 0.01
    }

    pub fn aim_eye(&self, eye: Vec3, look: Vec3) -> Vec3 {
        eye + offset(look, self.current)
    }

    /// The camera position for a body with its `eye` and `feet` this frame, looking along `look`.
    /// `solid` says whether a cell stops the camera.
    pub fn camera(&mut self, eye: Vec3, feet: Vec3, look: Vec3, dt: f64, solid: impl Fn(IVec3) -> bool) -> Vec3 {
        let wanted = self.distance.clamp(0.0, MAX_DISTANCE);
        let mut free = wanted;
        let mut t = PROBE_STEP;
        while t <= wanted + CLEARANCE {
            let at = eye + offset(look, t);
            // A small swept volume protects the near plane beside walls, ceilings and corners too.
            if [
                Vec3::ZERO,
                Vec3::new(0.15, 0.0, 0.0),
                Vec3::new(-0.15, 0.0, 0.0),
                Vec3::new(0.0, 0.15, 0.0),
                Vec3::new(0.0, -0.15, 0.0),
                Vec3::new(0.0, 0.0, 0.15),
                Vec3::new(0.0, 0.0, -0.15),
            ]
            .iter()
            .any(|p| solid((at + *p).floor()))
            {
                free = (t - CLEARANCE).clamp(0.0, wanted);
                break;
            }
            t += PROBE_STEP;
        }
        let rate = if free < self.current { ZOOM_IN_RATE } else { ZOOM_OUT_RATE };
        self.current += (free - self.current) * (1.0 - (-rate * dt.max(0.0)).exp());
        // Obstructions are a hard bound: easing must never leave the camera inside newly met terrain.
        if wanted > 0.0 {
            self.current = self.current.min(free);
        }
        self.feet = (self.current > 0.01).then_some(feet);
        self.aim_eye(eye, look)
    }
}

fn offset(look: Vec3, distance: f64) -> Vec3 {
    let horizontal = (look.x * look.x + look.z * look.z).sqrt().max(0.001);
    let right = Vec3::new(-look.z / horizontal, 0.0, look.x / horizontal);
    look * -distance + right * (distance * SHOULDER_RATIO).min(MAX_SHOULDER)
}

#[cfg(test)]
mod tests;
