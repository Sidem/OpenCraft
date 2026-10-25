//! Kestrel survey robots: presentation-only poses, interpolated co-op bodies and name anchors.
//! `motion` blends locomotion and hand gestures; `model` draws the articulated procedural rig.
//! No animation changes physics, saves or the deterministic core. Tune the rig in `model.rs`.

use std::f64::consts::{PI, TAU};

use crate::held_light;
use crate::item::ItemId;
use crate::math::Vec3;
use crate::Game;
use motion::Motion;

mod model;
mod motion;

pub const LABEL_FLOATS: usize = 4;
const LABEL_RANGE: f64 = 64.0;
const GLIDE_RATE: f64 = 14.0;
const SNAP: f64 = 8.0;
/// From an avatar's feet to where its held light shines.
const CHEST: Vec3 = Vec3::new(0.0, 1.2, 0.0);

#[derive(Default)]
pub struct Avatars {
    shown: Vec<Option<Shown>>,
    local_motion: Motion,
    pub labels: Vec<f32>,
    /// The held lights this frame: `held_light::LIGHT_FLOATS` a lit player, camera-relative.
    pub lights: Vec<f32>,
}

/// Adds a light at `at` (relative to the camera) to `lights` if `reach` is more than none.
fn add_light(lights: &mut Vec<f32>, at: Vec3, reach: f32) {
    if reach > 0.0 && lights.len() < held_light::MAX_LIGHTS * held_light::LIGHT_FLOATS {
        lights.extend_from_slice(&[at.x as f32, at.y as f32, at.z as f32, reach]);
    }
}

struct Shown {
    pos: Vec3,
    yaw: f64,
    pitch: f64,
    motion: Motion,
}

impl Game {
    pub(crate) fn write_avatars(&mut self, dt: f64, eye: Vec3) {
        let dt = dt.clamp(0.0, 0.1);
        let k = 1.0 - (-GLIDE_RATE * dt).exp();
        let local = self.local.0 as usize;
        let inv = self.inventory();
        let (hands, reach) = ([inv.selected_stack().item, inv.offhand.item], held_light::reach_of(inv));
        let focus = self.avatar_focus() - eye;
        let a = &mut self.avatars;
        a.labels.clear();
        a.lights.clear();
        a.shown.resize_with(self.bodies.len(), || None);
        for (slot, body) in self.bodies.iter().enumerate() {
            let Some(body) = body.as_ref() else {
                a.shown[slot] = None;
                continue;
            };
            if slot == local {
                a.local_motion.advance(body, dt);
                a.local_motion.focus = Some(focus);
                // In first person the light sits just below the eye; seen from behind, at the avatar's chest.
                let at = self.third_person.feet.map_or(Vec3::new(0.0, -0.3, 0.0), |feet| feet - eye + CHEST);
                add_light(&mut a.lights, at, reach);
                if let Some(feet) = self.third_person.feet {
                    model::push_avatar(&mut self.instances, feet - eye, body.yaw, body.pitch, &a.local_motion, hands);
                }
                continue;
            }
            let s = a.shown[slot].get_or_insert_with(|| Shown {
                pos: body.pos,
                yaw: body.yaw,
                pitch: body.pitch,
                motion: Motion::default(),
            });
            if (body.pos - s.pos).length() >= SNAP {
                s.pos = body.pos;
                s.motion = Motion::default();
            } else {
                s.pos += (body.pos - s.pos) * k;
            }
            s.yaw += ((body.yaw - s.yaw + PI).rem_euclid(TAU) - PI) * k;
            s.pitch += (body.pitch - s.pitch) * k;
            s.motion.advance(body, dt);
            let at = s.pos - eye;
            let inv = self.sim.players.get(slot).and_then(Option::as_ref).map(|p| &p.inventory);
            let hands = inv.map_or([ItemId::NONE; 2], |i| [i.selected_stack().item, i.offhand.item]);
            add_light(&mut a.lights, at + CHEST, inv.map_or(0.0, held_light::reach_of));
            if model::push_avatar(&mut self.instances, at, s.yaw, s.pitch, &s.motion, hands)
                && at.length() < LABEL_RANGE
            {
                let tag = at + Vec3::new(0.0, 2.05 - s.motion.crouch * 0.3, 0.0);
                a.labels.extend_from_slice(&[slot as f32, tag.x as f32, tag.y as f32, tag.z as f32]);
            }
        }
    }
}

#[cfg(test)]
mod tests;
