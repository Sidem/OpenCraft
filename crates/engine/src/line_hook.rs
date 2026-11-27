//! The Line Hook (GEAR-HOOK, a left-hand device): a press of `G` fires it where you look. At a block within [`RANGE`]
//! it pulls the body there and holds it while `G` stays down; jump lets go with a hop, to climb over a ledge. At a
//! power wire between two poles it clips on and rides the span to its far pole, then on along the span there that best
//! keeps the heading, until jump lets go; at the end of the line it hangs from the last pole.
//!
//! Invariants: body and presentation only (the left hand informs: no core `Action`). The movement goes through
//! `Player::pull`, set here every tick and read by `step_bodies`; bodies are replicated, never core state, so the hook
//! costs no state hash. Wires are read from the factory as drawn (`Factory::for_each_wire`).
//! To tune it: the constants below.

use crate::block::{self, AIR};
use crate::item::LINE_HOOK;
use crate::math::Vec3;
use crate::raycast::raycast;
use crate::{Game, TICK};

/// How far the hook flies, in blocks.
pub const RANGE: f64 = 24.0;
/// How fast the hook reels the body in, in blocks per second.
const PULL_SPEED: f64 = 16.0;
/// How fast a clipped-on body rides a wire, in blocks per second.
const RIDE_SPEED: f64 = 9.0;
/// The fastest a hook drags the body (catching up with the wire after a clip), in blocks per second.
const MAX_PULL: f64 = 30.0;
/// How far below the wire the feet hang.
const HANG: f64 = 2.1;
/// How close to the hook point the eye stops.
const ARRIVE: f64 = 1.2;
/// How close the aim must pass to a wire to clip on.
const GRAB: f64 = 0.9;
/// Span ends this close belong to the same pole (its insulators sit apart on the crossarm).
const JOIN: f64 = 1.6;
/// Upward speed of the hop when jump lets go of a block.
const HOP: f64 = 6.5;
/// The line's colour (a steel cable).
const LINE_LAYER: u16 = block::tex::MAT_GUNMETAL;

/// What the hook holds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Hook {
    #[default]
    Off,
    /// Stuck at `at`, the middle of the free cell in front of the face it hit.
    Block { at: Vec3 },
    /// Clipped to the wire from `a` to `b`, `t` blocks along it, riding towards `b`.
    Wire { a: Vec3, b: Vec3, t: f64 },
}

impl Hook {
    /// Where the line ends now, in the world.
    pub fn point(self) -> Option<Vec3> {
        match self {
            Hook::Off => None,
            Hook::Block { at } => Some(at),
            Hook::Wire { a, b, t } => Some(a + (b - a) * (t / (b - a).length().max(1e-6))),
        }
    }
}

impl Game {
    /// Runs the hook for one tick: a press of `G` fires it; then it drags the body (`Player::pull`).
    pub(crate) fn update_line_hook(&mut self) {
        let held = self.left_item() == LINE_HOOK;
        let pressed = self.left.using && !self.left.was_using;
        self.left.was_using = self.left.using;
        if !held {
            self.left.hook = Hook::Off;
        } else if pressed {
            self.left.hook = self.fire_hook();
        }
        let jump = self.body().input.jump;
        let pull = match self.left.hook {
            Hook::Off => None,
            Hook::Block { .. } if jump || !self.left.using => {
                self.left.hook = Hook::Off;
                if jump {
                    self.body_mut().vel.y = HOP;
                }
                None
            }
            Hook::Block { at } => {
                let to = at - self.body().eye();
                let dist = to.length();
                Some(if dist <= ARRIVE { Vec3::ZERO } else { to * (PULL_SPEED.min(dist / TICK) / dist) })
            }
            Hook::Wire { .. } if jump => {
                self.left.hook = Hook::Off;
                None
            }
            Hook::Wire { a, b, t } => {
                self.left.hook = self.ride(a, b, t + RIDE_SPEED * TICK);
                let point = self.left.hook.point().unwrap_or(b);
                let to = point - Vec3::new(0.0, HANG, 0.0) - self.body().pos;
                let v = to * (1.0 / TICK);
                let speed = v.length();
                Some(if speed > MAX_PULL { v * (MAX_PULL / speed) } else { v })
            }
        };
        self.body_mut().pull = pull;
    }

    /// What a hook fired along the view catches: the nearest wire the aim passes by before any block, else a block.
    pub(crate) fn fire_hook(&self) -> Hook {
        let (eye, dir) = (self.body().eye(), self.body().look_dir());
        let world = &self.sim.world;
        let solid = |p| world.get_block(p).filter(|&b| b != AIR && block::SOLID[b as usize]);
        let hit = raycast(eye, dir, RANGE, solid);
        let block_at = hit.map(|h| (h.block + h.normal).as_vec3() + Vec3::new(0.5, 0.5, 0.5));
        let block_dist = block_at.map_or(f64::INFINITY, |p| (p - eye).length());
        let mut best: Option<(f64, Hook)> = None;
        self.sim.factory.for_each_wire(eye, RANGE + 48.0, |a, b, _, poles| {
            let (s, u, gap) = ray_segment(eye, dir, a, b);
            if !poles || gap > GRAB || s > RANGE || s >= block_dist || best.is_some_and(|(d, _)| d <= s) {
                return;
            }
            let len = (b - a).length();
            // Ride the way the view points along the span.
            let hook = if (b - a).dot(dir) >= 0.0 {
                Hook::Wire { a, b, t: u * len }
            } else {
                Hook::Wire { a: b, b: a, t: (1.0 - u) * len }
            };
            best = Some((s, hook));
        });
        match (best, block_at) {
            (Some((_, wire)), _) => wire,
            (None, Some(at)) => Hook::Block { at },
            (None, None) => Hook::Off,
        }
    }

    /// The hook `t` blocks along the wire from `a` to `b`: past `b`, on along the next span that keeps the heading, or
    /// held at `b` at the end of the line.
    fn ride(&self, a: Vec3, b: Vec3, t: f64) -> Hook {
        let len = (b - a).length();
        if t < len {
            return Hook::Wire { a, b, t };
        }
        let heading = (b - a) * (1.0 / len.max(1e-6));
        let mut next: Option<(f64, Vec3, Vec3)> = None;
        self.sim.factory.for_each_wire(b, 64.0, |p, q, _, poles| {
            for (from, to) in [(p, q), (q, p)] {
                let span = to - from;
                let along = span.dot(heading) / span.length().max(1e-6);
                if poles && (from - b).length() <= JOIN && along > 0.3 && next.is_none_or(|n| n.0 < along) {
                    next = Some((along, from, to));
                }
            }
        });
        match next {
            Some((_, from, to)) => Hook::Wire { a: from, b: to, t: (t - len).min((to - from).length()) },
            None => Hook::Wire { a, b, t: len },
        }
    }

    /// The line from the left hand to the hook, drawn like a wire (camera-relative instances).
    pub(crate) fn write_hook_line(&mut self, eye: Vec3) {
        let Some(point) = self.left.hook.point() else { return };
        let body = self.body();
        let (sy, cy) = body.yaw.sin_cos();
        let left = Vec3::new(-cy, 0.0, -sy);
        let hand = body.eye() + left * 0.3 + body.look_dir() * 0.4 - Vec3::new(0.0, 0.35, 0.0);
        crate::factory::preview_wire(&mut self.instances, hand - eye, point - eye, LINE_LAYER);
    }

    /// Whether the hook holds something (for the HUD and the sound of letting go).
    pub fn hook_holds(&self) -> bool {
        self.left.hook != Hook::Off
    }
}

/// The closest approach of the ray from `eye` along unit `dir` to the segment `a`–`b`: how far along the ray, how far
/// along the segment (0 at `a`, 1 at `b`) and the gap between them.
pub(crate) fn ray_segment(eye: Vec3, dir: Vec3, a: Vec3, b: Vec3) -> (f64, f64, f64) {
    let seg = b - a;
    let (bb, c, e) = (dir.dot(seg), seg.dot(seg), seg.dot(eye - a));
    let d = dir.dot(eye - a);
    let den = c - bb * bb;
    let u = if den.abs() < 1e-9 { e / c.max(1e-9) } else { (e - bb * d) / den }.clamp(0.0, 1.0);
    let on_seg = a + seg * u;
    let s = dir.dot(on_seg - eye).max(0.0);
    (s, u, (eye + dir * s - on_seg).length())
}

#[cfg(test)]
mod tests;
