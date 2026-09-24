//! First-person character controller.

use crate::math::Vec3;
use crate::physics::{move_axis, Aabb};

pub const HALF_WIDTH: f64 = 0.3;
pub const HEIGHT: f64 = 1.8;
const EYE_HEIGHT: f64 = 1.62;
const CROUCH_EYE_HEIGHT: f64 = 1.32;
const GRAVITY: f64 = 28.0;
const TERMINAL_VELOCITY: f64 = 60.0;
const JUMP_VELOCITY: f64 = 8.6;
const WALK_SPEED: f64 = 4.3;
const SPRINT_SPEED: f64 = 6.6;
const CROUCH_SPEED: f64 = 1.9;
const FLY_SPEED: f64 = 11.0;
const FLY_SPRINT_SPEED: f64 = 24.0;

#[derive(Clone, Copy, Default)]
pub struct PlayerInput {
    /// -1..1, positive = forward.
    pub forward: f64,
    /// -1..1, positive = right.
    pub strafe: f64,
    pub jump: bool,
    pub sprint: bool,
    pub crouch: bool,
}

pub struct Player {
    /// Bottom-centre of the collision box.
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f64,
    pub pitch: f64,
    pub on_ground: bool,
    pub flying: bool,
    pub input: PlayerInput,
    /// Downward speed at the most recent touchdown; the game consumes and resets it (landing sound).
    pub landing_speed: f64,
}

impl Player {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            flying: false,
            input: PlayerInput::default(),
            landing_speed: 0.0,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::from_feet(self.pos, HALF_WIDTH, HEIGHT)
    }

    pub fn eye(&self) -> Vec3 {
        let crouched = self.input.crouch && !self.flying;
        self.pos + Vec3::new(0.0, if crouched { CROUCH_EYE_HEIGHT } else { EYE_HEIGHT }, 0.0)
    }

    /// Unit view direction. yaw = 0 looks towards -Z, positive yaw turns right.
    pub fn look_dir(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(sy * cp, sp, -cy * cp)
    }

    pub fn step(&mut self, dt: f64, solid: &mut impl FnMut(i32, i32, i32) -> bool) {
        let inp = self.input;
        let (sy, cy) = self.yaw.sin_cos();
        let (fwd, right) = (Vec3::new(sy, 0.0, -cy), Vec3::new(cy, 0.0, sy));
        let mut wish = fwd * inp.forward + right * inp.strafe;
        let len = wish.length();
        if len > 1.0 {
            wish = wish * (1.0 / len);
        }

        let speed = match (self.flying, inp.sprint, inp.crouch) {
            (true, true, _) => FLY_SPRINT_SPEED,
            (true, false, _) => FLY_SPEED,
            (false, _, true) => CROUCH_SPEED,
            (false, true, false) => SPRINT_SPEED,
            (false, false, false) => WALK_SPEED,
        };
        // Exponential approach to the wished velocity: snappy on the ground, floaty in the air.
        let rate = if self.flying {
            10.0
        } else if self.on_ground {
            16.0
        } else {
            3.0
        };
        let k = 1.0 - (-rate * dt).exp();
        self.vel.x += (wish.x * speed - self.vel.x) * k;
        self.vel.z += (wish.z * speed - self.vel.z) * k;

        if self.flying {
            let vy = (f64::from(u8::from(inp.jump)) - f64::from(u8::from(inp.crouch))) * speed * 0.8;
            self.vel.y += (vy - self.vel.y) * k;
        } else {
            self.vel.y = (self.vel.y - GRAVITY * dt).max(-TERMINAL_VELOCITY);
            if inp.jump && self.on_ground {
                self.vel.y = JUMP_VELOCITY;
            }
        }

        let mut bb = self.aabb();
        let want_y = self.vel.y * dt;
        let dy = move_axis(&mut bb, 1, want_y, solid);
        if (dy - want_y).abs() > 1e-9 {
            if want_y < 0.0 {
                if !self.on_ground {
                    self.landing_speed = self.landing_speed.max(-self.vel.y);
                }
                self.on_ground = true;
            }
            self.vel.y = 0.0;
        } else {
            self.on_ground = false;
        }

        // Crouching on the ground never walks off an edge.
        let edge_guard = inp.crouch && self.on_ground && !self.flying;
        for axis in [0, 2] {
            let want = self.vel.get(axis) * dt;
            let before = bb;
            let moved = move_axis(&mut bb, axis, want, solid);
            if edge_guard && !bb.has_support(solid) {
                bb = before;
                self.vel.set(axis, 0.0);
            } else if (moved - want).abs() > 1e-9 {
                self.vel.set(axis, 0.0);
            }
        }

        self.pos = Vec3::new((bb.min.x + bb.max.x) * 0.5, bb.min.y, (bb.min.z + bb.max.z) * 0.5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(_x: i32, y: i32, _z: i32) -> bool {
        y < 10
    }

    fn settle(p: &mut Player, seconds: f64) {
        let steps = (seconds * 120.0) as usize;
        for _ in 0..steps {
            p.step(1.0 / 120.0, &mut flat);
        }
    }

    #[test]
    fn lands_and_walks() {
        let mut p = Player::new(Vec3::new(0.5, 14.0, 0.5));
        settle(&mut p, 2.0);
        assert!(p.on_ground);
        assert!((p.pos.y - 10.0).abs() < 1e-6, "y = {}", p.pos.y);
        p.input.forward = 1.0;
        settle(&mut p, 1.0);
        assert!(p.pos.z < -3.0, "should walk towards -Z, z = {}", p.pos.z);
    }

    #[test]
    fn jump_clears_one_block() {
        let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
        settle(&mut p, 0.2);
        p.input.jump = true;
        let mut apex: f64 = 0.0;
        for _ in 0..120 {
            p.step(1.0 / 120.0, &mut flat);
            apex = apex.max(p.pos.y - 10.0);
        }
        assert!(apex > 1.05 && apex < 1.6, "apex {apex}");
    }
}
