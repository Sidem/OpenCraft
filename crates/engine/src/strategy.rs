//! Overhead follow/free views and click-to-walk orders. Camera state is local presentation;
//! only the body discovers terrain. Navigation drives normal player physics on known loaded ground.
//! Extend the view here; path planning and terrain queries live in strategy/navigation.rs.

use crate::block;
use crate::math::{IVec3, Vec3};
use crate::player::PlayerInput;
use crate::raycast::{raycast, RayHit};
use crate::{Game, TICK};

mod navigation;
#[cfg(test)]
mod tests;

const PITCH: f64 = -0.9;
const YAW: f64 = std::f64::consts::FRAC_PI_4;
/// Orbiting keeps the camera between straight down and level with the ground (radians below the horizon).
const PITCH_RANGE: (f64, f64) = (-1.5, -0.03);
const PAN_SPEED: f64 = 24.0;
const HEIGHT_RATE: f64 = 2.0;
const HEIGHT_SPEED: f64 = 6.0;
const FOLLOW_RATE: f64 = 5.0;

pub(crate) struct Strategy {
    /// 0 first person, 1 shoulder, 2 overhead follow, 3 overhead free.
    pub mode: u32,
    pub focus: Vec3,
    ground: f64,
    pub height: f64,
    /// Where the orbiting camera wants to look; `render_yaw` / `render_pitch` ease towards it.
    pub yaw: f64,
    pub pitch: f64,
    zoom: f64,
    pub shoulder_distance: f64,
    pub pan: Vec3,
    pub cursor: [f64; 2],
    pub projection: [f64; 2],
    pub render_yaw: f64,
    pub render_pitch: f64,
    transition: f64,
    pub hover: Option<RayHit>,
    pub navigation: navigation::Navigation,
}

impl Default for Strategy {
    fn default() -> Self {
        Self {
            mode: 0,
            focus: Vec3::ZERO,
            ground: 0.0,
            height: 30.0,
            zoom: 30.0,
            yaw: YAW,
            pitch: PITCH,
            pan: Vec3::ZERO,
            cursor: [0.0; 2],
            projection: [1.25, 1.0],
            hover: None,
            shoulder_distance: 4.0,
            navigation: navigation::Navigation::default(),
            render_yaw: 0.0,
            render_pitch: 0.0,
            transition: 0.0,
        }
    }
}

impl Strategy {
    pub fn active(&self) -> bool {
        self.mode >= 2
    }

    pub fn eye(&self) -> Vec3 {
        self.focus + Vec3::new(0.0, self.ground - self.focus.y, 0.0) - direction(self.yaw, self.pitch) * self.zoom
    }

    /// Turns the view around the point it looks at (`focus`) on a sphere whose radius is the zoom distance.
    pub fn orbit(&mut self, yaw: f64, pitch: f64) {
        self.yaw = (self.yaw + yaw).rem_euclid(std::f64::consts::TAU);
        self.pitch = (self.pitch + pitch).clamp(PITCH_RANGE.0, PITCH_RANGE.1);
    }

    pub fn cursor_ray(&self) -> Vec3 {
        let (yaw, pitch) = (self.render_yaw, self.render_pitch);
        let forward = direction(yaw, pitch);
        let right = Vec3::new(yaw.cos(), 0.0, yaw.sin());
        let up = Vec3::new(-yaw.sin() * pitch.sin(), pitch.cos(), yaw.cos() * pitch.sin());
        let tangent = (self.projection[0] * 0.5).tan();
        let ray = forward + right * (self.cursor[0] * tangent * self.projection[1]) + up * (self.cursor[1] * tangent);
        ray * (1.0 / ray.length())
    }

    pub fn advance(&mut self, feet: Vec3, ground: Option<f64>, dt: f64) {
        let dt = dt.clamp(0.0, 0.1);
        self.zoom += (self.height - self.zoom) * (1.0 - (-6.0 * dt).exp());
        if self.mode == 2 {
            let k = 1.0 - (-FOLLOW_RATE * dt).exp();
            self.focus.x += (feet.x - self.focus.x) * k;
            self.focus.z += (feet.z - self.focus.z) * k;
        } else {
            self.focus += self.pan * (PAN_SPEED * dt);
        }
        if let Some(target) = ground {
            let delta = (target - self.ground) * (1.0 - (-HEIGHT_RATE * dt).exp());
            self.ground += delta.clamp(-HEIGHT_SPEED * dt, HEIGHT_SPEED * dt);
        }
    }
}

impl Game {
    pub(crate) fn switch_view(&mut self, mode: u32) {
        let mode = mode.min(3);
        let was_active = self.strategy.active();
        if was_active != (mode >= 2) {
            self.strategy.transition = 1.0;
        }
        self.strategy.mode = mode;
        self.strategy.pan = Vec3::ZERO;
        self.mining = false;
        self.using = false;
        self.cancel_belt_line();
        self.body_mut().input = PlayerInput::default();
        if mode >= 2 && !was_active {
            let feet = self.body().pos;
            self.strategy.focus = feet;
            self.strategy.ground = feet.y;
        } else if mode < 2 {
            self.strategy.navigation.cancel();
            self.third_person.distance = if mode == 1 { self.strategy.shoulder_distance } else { 0.0 };
        }
    }

    pub(crate) fn advance_strategy(&mut self, feet: Vec3, dt: f64) {
        let ground =
            self.camera_area_known().then(|| navigation::camera_ground(&self.sim.world, self.strategy.focus)).flatten();
        self.strategy.advance(feet, ground, dt);
        self.third_person.feet = Some(feet);
    }

    pub(crate) fn settle_view(&mut self, before: Vec3, wanted: Vec3, dt: f64) {
        let (yaw, pitch) = if self.strategy.active() {
            (self.strategy.yaw, self.strategy.pitch)
        } else {
            (self.body().yaw, self.body().pitch)
        };
        let s = &mut self.strategy;
        let k = if s.transition > 0.001 { 1.0 - (-6.0 * dt.clamp(0.0, 0.1)).exp() } else { 1.0 };
        self.render_eye = before + (wanted - before) * k;
        s.render_yaw +=
            ((yaw - s.render_yaw + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI) * k;
        s.render_pitch += (pitch - s.render_pitch) * k;
        s.transition *= 1.0 - k;
        if self.strategy.active() {
            self.refresh_strategy_hover();
        }
    }

    pub(crate) fn refresh_strategy_hover(&mut self) {
        let s = &self.strategy;
        self.strategy.hover = raycast(self.render_eye, s.cursor_ray(), 180.0, |p| {
            self.minimap.atlas.tile(p.x >> 5, p.z >> 5)?;
            self.sim.world.get_block(p).filter(|&b| !block::replaceable(b))
        });
    }

    pub(crate) fn order_strategy_move(&mut self) {
        if !self.strategy.active() {
            return;
        }
        self.refresh_strategy_hover();
        let Some(hit) = self.strategy.hover else {
            self.strategy.navigation.reject();
            return;
        };
        let goal = hit.block + IVec3::new(0, 1, 0);
        let body = self.body();
        let start = (body.pos + Vec3::new(0.0, 0.1, 0.0)).floor();
        let path =
            (!body.flying && !body.in_water).then(|| navigation::find_path(&self.sim.world, start, goal)).flatten();
        self.strategy.navigation.order(path, goal);
    }

    pub(crate) fn step_navigation(&mut self) {
        if !self.strategy.active() {
            return;
        }
        let local = self.local.0 as usize;
        let body = self.bodies[local].as_mut().expect("local body");
        self.strategy.navigation.step(body, &self.sim.world, TICK);
    }

    pub(crate) fn strategy_target(&mut self) {
        let origin = self.render_eye;
        let dir = self.strategy.cursor_ray();
        self.update_target_ray(origin, dir);
    }

    pub(crate) fn observer_columns(&self) -> Vec<IVec3> {
        if !self.strategy.active() {
            return Vec::new();
        }
        let p = self.strategy.focus.floor();
        let (cx, cz, radius) = (p.x >> 5, p.z >> 5, (self.sim.world.view_distance() / 32.0) as i32);
        let mut columns = Vec::new();
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dz * dz <= radius * radius && self.minimap.atlas.tile(cx + dx, cz + dz).is_some() {
                    columns.push(IVec3::new(cx + dx, 0, cz + dz));
                }
            }
        }
        columns
    }

    pub(crate) fn write_move_marker(&mut self, eye: Vec3) {
        if !self.strategy.active() || self.strategy.navigation.status != 1 {
            return;
        }
        let Some(goal) = self.strategy.navigation.goal else {
            return;
        };
        let center = goal.as_vec3() + Vec3::new(0.5, 0.06, 0.5) - eye;
        for (dx, dz) in [(-0.45, 0.0), (0.45, 0.0), (0.0, -0.45), (0.0, 0.45)] {
            let size = if dx == 0.0 { [0.9, 0.06, 0.08] } else { [0.08, 0.06, 0.9] };
            crate::factory::push_box(
                &mut self.instances,
                center + Vec3::new(dx, 0.0, dz),
                0.0,
                size,
                0.0,
                [block::tex::AVATAR_TRIM; 3],
                false,
            );
        }
    }

    pub(crate) fn hide_unexplored_instances(&mut self, eye: Vec3) {
        if !self.strategy.active() {
            return;
        }
        let stride = crate::factory::INSTANCE_FLOATS;
        let mut kept = 0;
        for i in (0..self.instances.len()).step_by(stride) {
            let x = (eye.x + f64::from(self.instances[i])).floor() as i32;
            let z = (eye.z + f64::from(self.instances[i + 2])).floor() as i32;
            if self.minimap.atlas.tile(x >> 5, z >> 5).is_some() {
                self.instances.copy_within(i..i + stride, kept);
                kept += stride;
            }
        }
        self.instances.truncate(kept);
    }

    pub(crate) fn write_strategy_fog(&mut self, eye: Vec3) {
        if !self.strategy.active() {
            return;
        }
        let p = self.strategy.focus.floor();
        let (cx, cz) = (p.x >> 5, p.z >> 5);
        // Cover is deliberately flat: it must not give away unseen heights, objects or shorelines.
        let radius = ((self.sim.world.view_distance() / 32.0) as i32 + 2).min(8);
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let (x, z) = (cx + dx, cz + dz);
                if self.minimap.atlas.tile(x, z).is_none() {
                    let center =
                        Vec3::new(f64::from(x * 32 + 16), self.strategy.ground - 1.0, f64::from(z * 32 + 16)) - eye;
                    crate::factory::push_box(
                        &mut self.instances,
                        center,
                        0.0,
                        [32.01, 0.05, 32.01],
                        0.0,
                        [block::tex::STRATEGY_FOG; 3],
                        false,
                    );
                }
            }
        }
    }
}

pub(crate) fn direction(yaw: f64, pitch: f64) -> Vec3 {
    Vec3::new(yaw.sin() * pitch.cos(), pitch.sin(), -yaw.cos() * pitch.cos())
}
