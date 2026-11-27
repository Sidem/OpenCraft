//! The Bladerang (GEAR-BLADE; it replaced the sickle and kept its item id): the use button throws it. It flies a loop
//! out [`REACH`] blocks along the view, swinging right, and back to the thrower in [`FLIGHT_SECONDS`]. Over natural
//! ground it cuts the plants of each 3 × 3 patch it crosses (`Action::ClearCover`, core state: `world/cover_clear.rs`),
//! and it gathers the loose items it passes, logs from felling included, and lets them fall at the thrower's feet.
//! A wall turns it home early.
//!
//! Invariants: the flight is the local hands' (presentation); only the cuts are core actions, and a patch already
//! cleared sends nothing. Loose items are carried only where this game is their authority (solo or a host; a co-op
//! client's items are the host's to move). The bladerang never leaves its slot. To change the loop: the constants.

use std::f64::consts::PI;

use crate::action::Action;
use crate::block::{self, sound::GRASS, AIR};
use crate::entities::push_item_model;
use crate::factory::hud_keys;
use crate::item::BLADERANG;
use crate::math::{IVec3, Vec3};
use crate::mesher::bears_cover;
use crate::net::Role;
use crate::sound;
use crate::world::PATCH_RADIUS;
use crate::{Game, TICK};

/// How far out the loop reaches, in blocks.
pub const REACH: f64 = 12.0;
/// How far right of the line out the loop swings.
const SWING: f64 = 3.5;
/// Seconds from the throw back to the hand.
pub const FLIGHT_SECONDS: f64 = 1.6;
/// How close a loose item must pass to be caught.
const CATCH: f64 = 1.4;
/// How far below the blade the ground it cuts may lie.
const CUT_DEPTH: i32 = 3;
/// Turns a second while it flies (drawn only).
const SPIN: f64 = 24.0;
/// How big it is drawn in flight, in blocks.
const DRAWN: f64 = 0.7;

/// A bladerang in the air: its time out, where it left from, its heading and swing, the items it carries (by entity
/// id) and the patches it cut.
pub struct Flight {
    t: f64,
    from: Vec3,
    fwd: Vec3,
    right: Vec3,
    carried: Vec<u32>,
    cut: Vec<IVec3>,
}

/// The local hands' bladerang: in flight or not.
#[derive(Default)]
pub struct Blade {
    pub flight: Option<Flight>,
}

impl Game {
    fn blade_held(&self) -> bool {
        self.inventory().selected_stack().item == BLADERANG
    }

    /// The use button with a bladerang in hand: a press throws it, unless it is still out. True when it owns the click.
    pub(crate) fn update_blade(&mut self) -> bool {
        if !self.using || !self.blade_held() {
            return false;
        }
        self.using = false;
        if self.blade.flight.is_none() {
            let body = self.body();
            let (sy, cy) = body.yaw.sin_cos();
            let (fwd, right) = (Vec3::new(sy, 0.0, -cy), Vec3::new(cy, 0.0, sy));
            let from = body.eye() - Vec3::new(0.0, 0.5, 0.0);
            self.blade.flight = Some(Flight { t: 0.0, from, fwd, right, carried: Vec::new(), cut: Vec::new() });
            self.play(sound::DIG, GRASS, from, 0.35);
        }
        true
    }

    /// Where the blade is now, if it is out.
    pub(crate) fn blade_pos(&self) -> Option<Vec3> {
        let f = self.blade.flight.as_ref()?;
        let tau = (f.t / FLIGHT_SECONDS).min(1.0);
        let home = self.body().eye() - Vec3::new(0.0, 0.5, 0.0);
        let out = (PI * tau).sin();
        Some(f.from * (1.0 - tau) + home * tau + f.fwd * (REACH * out) + f.right * (SWING * out))
    }

    /// Moves a thrown blade on by one tick: cuts the patch under it, catches items, comes home.
    pub(crate) fn step_blade(&mut self) {
        let Some(f) = self.blade.flight.as_mut() else { return };
        f.t += TICK;
        let Some(pos) = self.blade_pos() else { return };
        let world = &self.sim.world;
        let solid = |p: IVec3| world.get_block(p).is_some_and(|b| b != AIR && block::SOLID[b as usize]);
        let f = self.blade.flight.as_mut().expect("in flight");
        // A wall turns it home: on to the mirrored point of its way back.
        if f.t < FLIGHT_SECONDS * 0.5 && solid(pos.floor()) {
            f.t = FLIGHT_SECONDS - f.t;
        }
        let cell = pos.floor();
        let ground = (0..=CUT_DEPTH).map(|d| cell - IVec3::new(0, d, 0)).find(|&p| solid(p));
        let fresh = |c: IVec3| (c.x - cell.x).abs() > PATCH_RADIUS || (c.z - cell.z).abs() > PATCH_RADIUS;
        let cut = ground.filter(|&g| {
            world.get_block(g).is_some_and(bears_cover) && !world.patch_cleared(g) && f.cut.iter().all(|&c| fresh(c))
        });
        if !matches!(self.role, Role::Client(_)) {
            for e in &mut self.items.list {
                if !f.carried.contains(&e.id) && (e.pos - pos).length() <= CATCH {
                    f.carried.push(e.id);
                }
                if f.carried.contains(&e.id) {
                    (e.pos, e.vel) = (pos, Vec3::ZERO);
                }
            }
        }
        let done = f.t >= FLIGHT_SECONDS;
        if let Some(g) = cut {
            f.cut.push(g);
            self.act(Action::ClearCover { pos: g });
            self.play(sound::DIG, GRASS, g.as_vec3() + Vec3::new(0.5, 1.0, 0.5), 0.4);
        }
        if done {
            let feet = self.body().pos + Vec3::new(0.0, 0.5, 0.0);
            let carried = self.blade.flight.take().map(|f| f.carried).unwrap_or_default();
            for e in self.items.list.iter_mut().filter(|e| carried.contains(&e.id)) {
                (e.pos, e.vel, e.pickup_delay) = (feet, Vec3::ZERO, 0.0);
            }
        }
    }

    /// The spinning blade in flight (camera-relative instances).
    pub(crate) fn write_blade(&mut self, eye: Vec3) {
        let (Some(pos), Some(f)) = (self.blade_pos(), self.blade.flight.as_ref()) else { return };
        push_item_model(&mut self.instances, BLADERANG, pos - eye, (f.t * SPIN) as f32, DRAWN);
    }

    /// The HUD label with a bladerang in hand; `None` otherwise.
    pub(crate) fn blade_label(&self) -> Option<String> {
        self.blade_held().then(|| {
            let keys = hud_keys(&[("RMB", "Throw")]);
            format!("Bladerang\nCuts small plants on its loop · brings back loose items\n{keys}")
        })
    }
}

#[cfg(test)]
mod tests;
