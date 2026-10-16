//! What a launch or a landing looks and sounds like (presentation only, never saved): rockets and drop pods report
//! `SimEvent::Launched`, `PodSent` and `PodLanded` (`factory/process/pods.rs`, `Factory::update`), and this plays the
//! sound, starts a burst of dust and exhaust at the machine and, for a satellite, queues a toast ("in orbit") the HUD
//! pops with `next_launch` (`api/orbit.rs`). A burst is a ring of smoke puffs pushed out from a point that swell, rise
//! and thin over `LIFE` seconds; drawn with the other box instances (`drone_view.rs`).
//!
//! To add an effect: an arm in `Game::orbit_event` and, for a new sound, a `sound` id (synthesised in `web/src/audio`).

use std::collections::VecDeque;
use std::f64::consts::TAU;

use crate::block::tex;
use crate::factory::push_box;
use crate::item::ItemId;
use crate::math::Vec3;
use crate::sim::SimEvent;
use crate::{sound, Game, TICK};

/// Seconds a burst lasts, puffs in its ring, and bursts kept at once (the oldest go first).
const LIFE: f64 = 1.8;
const PUFFS: usize = 10;
const MAX_BURSTS: usize = 32;
/// Bursts further than this from the eye are not drawn.
const DRAW_RANGE: f64 = 400.0;
/// Burst sizes: a rocket leaving its pad, a pod leaving its tube, a pod landing.
const ROCKET: f32 = 3.0;
const POD_UP: f32 = 0.8;
const POD_DOWN: f32 = 1.4;
/// Heights above a machine's anchor where the burst starts: a launch site's deck, a launcher's tube mouth, a pad's top.
const DECK: f64 = 1.0;
const MOUTH: f64 = 3.0;
const PAD: f64 = 0.4;

#[cfg(test)]
mod tests;

struct Burst {
    at: Vec3,
    start: f64,
    scale: f32,
}

/// The bursts in the air and the launches the HUD has not shown yet.
#[derive(Default)]
pub(crate) struct OrbitFx {
    bursts: Vec<Burst>,
    pub news: VecDeque<ItemId>,
}

impl OrbitFx {
    fn burst(&mut self, at: Vec3, time: f64, scale: f32) {
        if self.bursts.len() >= MAX_BURSTS {
            self.bursts.remove(0);
        }
        self.bursts.push(Burst { at, start: time, scale });
    }

    /// Appends every live burst's puffs to the box instances and forgets the finished ones; `time` in seconds.
    pub(crate) fn write(&mut self, out: &mut Vec<f32>, eye: Vec3, time: f64) {
        self.bursts.retain(|b| time - b.start < LIFE);
        for b in &self.bursts {
            let age = ((time - b.start) / LIFE).clamp(0.0, 1.0);
            if (b.at - eye).length() > DRAW_RANGE {
                continue;
            }
            let (scale, spread) = (f64::from(b.scale), 1.0 - (1.0 - age) * (1.0 - age));
            let layer = if age < 0.5 { tex::SMOKE } else { tex::SMOKE_THIN };
            for k in 0..PUFFS {
                let angle = TAU * k as f64 / PUFFS as f64 + b.start;
                let r = scale * (0.4 + 2.2 * spread);
                let p = b.at - eye + Vec3::new(angle.cos() * r, scale * 0.5 * age, angle.sin() * r);
                let size = b.scale * (0.5 + 0.9 * age as f32);
                push_box(out, p, p.x.atan2(p.z) as f32, [size; 3], 0.0, [layer; 3], false);
            }
        }
    }
}

impl Game {
    /// The look, sound and toast of a launch or a landing (from `handle_sim_events`); other events are ignored.
    pub(crate) fn orbit_event(&mut self, event: SimEvent) {
        let time = self.sim.tick as f64 * TICK;
        let (pos, rise, scale, kind) = match event {
            SimEvent::Launched { pos, item } => {
                self.orbit_fx.news.push_back(item);
                (pos, DECK, ROCKET, sound::LAUNCH)
            }
            SimEvent::PodSent { pos } => (pos, MOUTH, POD_UP, sound::POD_UP),
            SimEvent::PodLanded { pos } => (pos, PAD, POD_DOWN, sound::POD_DOWN),
            _ => return,
        };
        let at = self.sim.factory.base_middle(pos) + Vec3::new(0.0, rise, 0.0);
        self.orbit_fx.burst(at, time, scale);
        self.play(kind, 0, at, 1.0);
    }
}
