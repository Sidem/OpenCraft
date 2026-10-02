//! First-person character controller: walking, sprinting, crouching (no falling off edges),
//! jumping, swimming, climbing, flying, and collision through `physics`. `step` advances one physics
//! substep; the caller decides the substep size and answers which cells are solid and which block is in a cell.
//! Tuning constants sit at the top of the file.
//!
//! Swimming: a body whose feet are in water moves at half speed, sinks slowly (a tenth of gravity
//! against strong drag) and rises while jump is held until its chest is out, where it floats. Jump
//! with the chest out, pushing against a wall or standing on the bottom, leaps like a jump on land:
//! that is how a swimmer climbs out onto a bank. Flying ignores water.
//!
//! Climbing: a body with a ladder or a belt lift (`climbable`) at its feet or waist doesn't fall; it rises at `CLIMB_SPEED` with jump
//! held and sinks with crouch held. Climbing past the top ladder carries the feet just over it, so walking
//! on steps onto a ledge level with it.

use crate::block::{BlockId, LADDER, LIFT, LIQUID};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::Vec3;
use crate::physics::{move_axis, Aabb};

pub const HALF_WIDTH: f64 = 0.3;
pub const HEIGHT: f64 = 1.8;
const EYE_HEIGHT: f64 = 1.62;
const CROUCH_EYE_HEIGHT: f64 = 1.32;
/// How much lower the eye sits when crouched.
pub const CROUCH_DIP: f64 = EYE_HEIGHT - CROUCH_EYE_HEIGHT;
const GRAVITY: f64 = 28.0;
const TERMINAL_VELOCITY: f64 = 60.0;
const JUMP_VELOCITY: f64 = 8.6;
const WALK_SPEED: f64 = 4.3;
const SPRINT_SPEED: f64 = 6.6;
const CROUCH_SPEED: f64 = 1.9;
const FLY_SPEED: f64 = 11.0;
const FLY_SPRINT_SPEED: f64 = 24.0;
/// Horizontal speed in water, as a share of the speed on land.
const SWIM_SPEED_FACTOR: f64 = 0.5;
/// Vertical drag in water (per second): slows a dive and caps sinking at `GRAVITY / 10 / WATER_DRAG`.
const WATER_DRAG: f64 = 4.0;
/// Rising speed with jump held under water.
const SWIM_UP_SPEED: f64 = 2.5;
/// Height above the feet that must be out of water to float, and to leap out.
const CHEST_HEIGHT: f64 = 1.2;
/// Speed up and down a ladder.
const CLIMB_SPEED: f64 = 3.0;
/// Heights above the feet that hold on to a ladder or lift.
const GRIP_HEIGHTS: [f64; 2] = [0.1, 0.9];

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
    /// The feet are in water (swimming), as of the last step.
    pub in_water: bool,
    pub flying: bool,
    pub input: PlayerInput,
    /// Presentation-only hands: bit 0 mining, bit 1 using. Relayed to co-op peers, never saved.
    pub gesture: u8,
    /// Downward speed at the most recent touchdown; the game consumes and resets it (landing sound).
    pub landing_speed: f64,
    /// Downward speed when the body last entered water; the game consumes and resets it (splash).
    pub splash_speed: f64,
    /// A sideways move was blocked in the last step (for climbing out of water).
    against_wall: bool,
}

impl Player {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            in_water: false,
            flying: false,
            input: PlayerInput::default(),
            gesture: 0,
            landing_speed: 0.0,
            splash_speed: 0.0,
            against_wall: false,
        }
    }

    /// What a save keeps of a body: position, velocity, view and flying (input and ground contact
    /// are live and come back by themselves).
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.vec3(self.pos);
        w.vec3(self.vel);
        w.f64(self.yaw);
        w.f64(self.pitch);
        w.bool(self.flying);
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Player> {
        let mut p = Player::new(r.vec3()?);
        p.vel = r.vec3()?;
        p.yaw = r.f64()?;
        p.pitch = r.f64()?;
        p.flying = r.bool()?;
        Some(p)
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::from_feet(self.pos, HALF_WIDTH, HEIGHT)
    }

    pub fn eye(&self) -> Vec3 {
        self.pos + Vec3::new(0.0, self.eye_height(), 0.0)
    }

    /// How far the eye is above the feet.
    pub fn eye_height(&self) -> f64 {
        if self.crouched() {
            CROUCH_EYE_HEIGHT
        } else {
            EYE_HEIGHT
        }
    }

    /// Whether the eye is at crouching height (`eye`), which the camera eases (camera.rs).
    pub fn crouched(&self) -> bool {
        self.input.crouch && !self.flying
    }

    /// Unit view direction. yaw = 0 looks towards -Z, positive yaw turns right.
    pub fn look_dir(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(sy * cp, sp, -cy * cp)
    }

    pub fn step(
        &mut self,
        dt: f64,
        solid: &mut impl FnMut(i32, i32, i32) -> bool,
        block: &mut impl FnMut(i32, i32, i32) -> BlockId,
    ) {
        let inp = self.input;
        let (sy, cy) = self.yaw.sin_cos();
        let (fwd, right) = (Vec3::new(sy, 0.0, -cy), Vec3::new(cy, 0.0, sy));
        let mut wish = fwd * inp.forward + right * inp.strafe;
        let len = wish.length();
        if len > 1.0 {
            wish = wish * (1.0 / len);
        }

        let feet = self.pos;
        let mut block_at = |height: f64| {
            let p = (feet + Vec3::new(0.0, height, 0.0)).floor();
            block(p.x, p.y, p.z)
        };
        let mut water_at = |height: f64| LIQUID[block_at(height) as usize];
        let was_in_water = self.in_water;
        self.in_water = !self.flying && water_at(0.1);
        if self.in_water && !was_in_water {
            self.splash_speed = self.splash_speed.max(-self.vel.y);
        }
        let swimming = self.in_water;
        let chest_out = swimming && !water_at(CHEST_HEIGHT);
        let climbing = !self.flying && !swimming && GRIP_HEIGHTS.iter().any(|&h| climbable(block_at(h)));

        let mut speed = match (self.flying, inp.sprint, inp.crouch) {
            (true, true, _) => FLY_SPRINT_SPEED,
            (true, false, _) => FLY_SPEED,
            (false, _, true) => CROUCH_SPEED,
            (false, true, false) => SPRINT_SPEED,
            (false, false, false) => WALK_SPEED,
        };
        if swimming {
            speed *= SWIM_SPEED_FACTOR;
        }
        // Exponential approach to the wished velocity: snappy on the ground, floaty in the air.
        let rate = if self.flying {
            10.0
        } else if swimming {
            6.0
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
        } else if swimming {
            self.swim_vertical(dt, chest_out);
        } else if climbing {
            self.vel.y = (f64::from(u8::from(inp.jump)) - f64::from(u8::from(inp.crouch))) * CLIMB_SPEED;
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
        self.against_wall = false;
        for axis in [0, 2] {
            let want = self.vel.get(axis) * dt;
            let before = bb;
            let moved = move_axis(&mut bb, axis, want, solid);
            if edge_guard && !bb.has_support(solid) {
                bb = before;
                self.vel.set(axis, 0.0);
            } else if (moved - want).abs() > 1e-9 {
                self.vel.set(axis, 0.0);
                self.against_wall = true;
            }
        }

        self.pos = Vec3::new((bb.min.x + bb.max.x) * 0.5, bb.min.y, (bb.min.z + bb.max.z) * 0.5);
    }

    /// Vertical speed for one substep in water: a leap out, rising with jump held, or sinking.
    fn swim_vertical(&mut self, dt: f64, chest_out: bool) {
        let jump = self.input.jump;
        if jump && chest_out && (self.on_ground || self.against_wall) {
            self.vel.y = JUMP_VELOCITY;
        } else if self.vel.y > SWIM_UP_SPEED {
            // A leap carries on through the water's surface.
            self.vel.y -= GRAVITY * 0.1 * dt;
        } else if jump && !chest_out {
            self.vel.y += (SWIM_UP_SPEED - self.vel.y) * (1.0 - (-WATER_DRAG * dt).exp());
        } else {
            // Sinking; with jump held the chest bobs at the surface.
            self.vel.y = (self.vel.y - GRAVITY * 0.1 * dt) * (-WATER_DRAG * dt).exp();
        }
    }
}

/// Blocks a body climbs by standing in them: ladders, and belt lifts so a lift stack is easy to build.
fn climbable(b: BlockId) -> bool {
    b == LADDER || b == LIFT
}

#[cfg(test)]
mod tests;
