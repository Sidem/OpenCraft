//! Other players as the local player sees them: a body, a head with a helmet and a visor that shows
//! where they look, drawn with the box instances, and a name-tag anchor above each head (`labels`,
//! positioned by the host page). What is drawn glides towards each body's latest position and view
//! (bodies moved by another machine update 20 times a second: net/players.rs) and jumps when more than
//! `SNAP` blocks away. Presentation only: nothing here touches the core or the bodies.
//! To change the look: the sizes below and the `AVATAR_*` textures (`textures.rs`).

use std::f64::consts::{PI, TAU};

use crate::block::tex;
use crate::factory::push_box;
use crate::math::Vec3;
use crate::Game;

/// Floats per name-tag anchor: player id, camera-relative x, y, z.
pub const LABEL_FLOATS: usize = 4;
const BODY: [f32; 3] = [0.6, 1.3, 0.34];
const HEAD: [f32; 3] = [0.46, 0.46, 0.46];
const VISOR: [f32; 3] = [0.36, 0.13, 0.06];
/// Height of a name tag's anchor above the feet.
const LABEL_HEIGHT: f64 = 2.05;
/// Name tags show within this many blocks.
const LABEL_RANGE: f64 = 64.0;
/// An avatar whose head is closer than this to the camera isn't drawn (players who share a spot,
/// such as two who just joined at spawn, would otherwise fill each other's view).
const NEAR: f64 = 0.9;
/// How fast what is drawn closes the gap to the body (per second).
const GLIDE_RATE: f64 = 14.0;
const SNAP: f64 = 8.0;

#[derive(Default)]
pub struct Avatars {
    /// What is drawn of each body, indexed by `PlayerId` (none for the local player).
    shown: Vec<Option<Shown>>,
    /// This frame's name-tag anchors, `LABEL_FLOATS` each.
    pub labels: Vec<f32>,
}

#[derive(Clone, Copy)]
struct Shown {
    pos: Vec3,
    yaw: f64,
    pitch: f64,
}

impl Game {
    /// Glides every other player's avatar on by `dt` seconds and pushes its boxes and name tag.
    pub(crate) fn write_avatars(&mut self, dt: f64, eye: Vec3) {
        let k = 1.0 - (-GLIDE_RATE * dt).exp();
        let (a, local) = (&mut self.avatars, self.local.0 as usize);
        a.labels.clear();
        a.shown.resize(self.bodies.len(), None);
        for (slot, body) in self.bodies.iter().enumerate() {
            let Some(body) = body.as_ref().filter(|_| slot != local) else {
                a.shown[slot] = None;
                continue;
            };
            let s = match &mut a.shown[slot] {
                Some(s) if (body.pos - s.pos).length() < SNAP => {
                    s.pos += (body.pos - s.pos) * k;
                    s.yaw += ((body.yaw - s.yaw + PI).rem_euclid(TAU) - PI) * k;
                    s.pitch += (body.pitch - s.pitch) * k;
                    *s
                }
                shown => *shown.insert(Shown { pos: body.pos, yaw: body.yaw, pitch: body.pitch }),
            };
            let at = s.pos - eye;
            let head = at + up((BODY[1] + HEAD[1] * 0.5) as f64);
            if head.length() < NEAR {
                continue;
            }
            let yaw = s.yaw as f32;
            push_box(&mut self.instances, at + up(BODY[1] as f64 * 0.5), yaw, BODY, 0.0, [tex::AVATAR_SUIT; 3], false);
            let skin = [tex::AVATAR_HELMET, tex::AVATAR_SKIN, tex::AVATAR_SKIN];
            push_box(&mut self.instances, head, yaw, HEAD, 0.0, skin, false);
            let ahead = ((HEAD[2] + VISOR[2]) * 0.5) as f64 - 0.01;
            let (sin, cos) = s.yaw.sin_cos();
            let visor = head + Vec3::new(sin * ahead, 0.03 + s.pitch.sin() * 0.1, -cos * ahead);
            push_box(&mut self.instances, visor, yaw, VISOR, 0.0, [tex::AVATAR_VISOR; 3], false);
            if at.length() < LABEL_RANGE {
                let tag = at + up(LABEL_HEIGHT);
                a.labels.extend_from_slice(&[slot as f32, tag.x as f32, tag.y as f32, tag.z as f32]);
            }
        }
    }
}

fn up(y: f64) -> Vec3 {
    Vec3::new(0.0, y, 0.0)
}
