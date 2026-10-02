//! Kestrel survey robots: presentation-only poses, interpolated co-op bodies and name anchors.
//! `motion` blends locomotion and hand gestures; `model` draws the articulated procedural rig.
//! No animation changes physics, saves or the deterministic core. Tune the rig in `model.rs`.

use std::f64::consts::{PI, TAU};

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

#[derive(Default)]
pub struct Avatars {
    shown: Vec<Option<Shown>>,
    local_motion: Motion,
    pub labels: Vec<f32>,
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
        let held = self.inventory().selected_stack().item;
        let focus = self.avatar_focus() - eye;
        let a = &mut self.avatars;
        a.labels.clear();
        a.shown.resize_with(self.bodies.len(), || None);
        for (slot, body) in self.bodies.iter().enumerate() {
            let Some(body) = body.as_ref() else {
                a.shown[slot] = None;
                continue;
            };
            if slot == local {
                a.local_motion.advance(body, dt);
                a.local_motion.focus = Some(focus);
                if let Some(feet) = self.third_person.feet {
                    model::push_avatar(&mut self.instances, feet - eye, body.yaw, body.pitch, &a.local_motion, held);
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
            let held = self
                .sim
                .players
                .get(slot)
                .and_then(Option::as_ref)
                .map_or(ItemId::NONE, |p| p.inventory.selected_stack().item);
            if model::push_avatar(&mut self.instances, at, s.yaw, s.pitch, &s.motion, held) && at.length() < LABEL_RANGE
            {
                let tag = at + Vec3::new(0.0, 2.05 - s.motion.crouch * 0.3, 0.0);
                a.labels.extend_from_slice(&[slot as f32, tag.x as f32, tag.y as f32, tag.z as f32]);
            }
        }
    }
}

#[cfg(test)]
mod tests;
