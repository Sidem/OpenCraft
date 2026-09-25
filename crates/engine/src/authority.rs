//! The authority's share of the game: every player's body (physics, falling out of the world), loose
//! items and who picks them up, throwing items, and players joining and leaving. In single-player the
//! local game is the authority; in co-op (Milestone 3) only the host is.
//!
//! Invariants: `Game.bodies` is indexed by `PlayerId`, like `Sim.players`. A body exists from `join`
//! on, and its core state from the tick the queued `Join` applies. The local player's body always
//! exists (`leave` refuses it). Bodies only move while the ground under them is loaded, and streaming
//! follows the local player, so other bodies far away wait (Milestone 3 loads around them too).
//! The authority never edits the core: pickups become `PickUp` actions.

use crate::action::Action;
use crate::block::BlockId;
use crate::entities::Collector;
use crate::math::Vec3;
use crate::player::{self, Player};
use crate::sim::PlayerId;
use crate::sound;
use crate::{Game, TICK};

/// Player physics substeps per tick (1/120 s each).
const PHYSICS_SUBSTEPS: u32 = 2;
/// Bodies that fall below this height are put back at spawn.
const FALL_LIMIT: f64 = -64.0;

impl Game {
    /// Adds a player: its body appears at spawn now, its (empty) inventory at the next tick. Returns
    /// `None` when all 256 ids are taken.
    pub(crate) fn join(&mut self) -> Option<PlayerId> {
        let slot = self.bodies.iter().position(Option::is_none).unwrap_or(self.bodies.len());
        let id = PlayerId(u8::try_from(slot).ok()?);
        if slot == self.bodies.len() {
            self.bodies.push(None);
        }
        self.bodies[slot] = Some(Player::new(self.spawn));
        self.act_as(id, Action::Join);
        Some(id)
    }

    /// Removes a player's body now and its core state at the next tick. The local player stays.
    pub(crate) fn leave(&mut self, id: PlayerId) {
        if id == self.local {
            return;
        }
        if let Some(body) = self.bodies.get_mut(id.0 as usize).filter(|b| b.is_some()) {
            *body = None;
            self.act_as(id, Action::Leave);
        }
    }

    /// Moves every body by one tick, and puts any that fell out of the world back at spawn.
    pub(crate) fn step_bodies(&mut self) {
        let world = &self.sim.world;
        let mut solid = |x, y, z| world.is_solid(x, y, z);
        let respawn = self.spawn + Vec3::new(0.0, 2.0, 0.0);
        let mut local_fell = false;
        for (slot, body) in self.bodies.iter_mut().enumerate() {
            let Some(body) = body else { continue };
            // Wait while the ground isn't loaded rather than fall through it.
            if world.is_loaded(body.pos) && world.is_loaded(body.pos - Vec3::new(0.0, 1.0, 0.0)) {
                for _ in 0..PHYSICS_SUBSTEPS {
                    body.step(TICK / PHYSICS_SUBSTEPS as f64, &mut solid);
                }
            }
            if body.pos.y < FALL_LIMIT {
                body.pos = respawn;
                body.vel = Vec3::ZERO;
                local_fell |= slot == self.local.0 as usize;
            }
        }
        if local_fell {
            // Cut the camera to spawn instead of gliding there.
            self.prev_eye = self.body().eye();
            self.render_eye = self.prev_eye;
        }
    }

    /// Steps loose items. They only predict what fits, on scratch copies of the inventories; the core
    /// applies each `PickUp` (and throws back anything that no longer fits).
    pub(crate) fn step_items(&mut self) {
        let mut ids = Vec::new();
        let mut collectors = Vec::new();
        for (slot, body) in self.bodies.iter().enumerate() {
            let id = PlayerId(slot as u8);
            let (Some(body), Some(core)) = (body, self.sim.player(id)) else { continue };
            ids.push(id);
            let center = body.pos + Vec3::new(0.0, player::HEIGHT * 0.5, 0.0);
            collectors.push(Collector { center, room: core.inventory.clone() });
        }
        let world = &self.sim.world;
        let mut solid = |x, y, z| world.is_solid(x, y, z);
        let loaded = |p: Vec3| world.is_loaded(p);
        let mut picked = Vec::new();
        self.items
            .update(TICK, &mut collectors, &mut solid, &loaded, |c, item, count| picked.push((ids[c], item, count)));
        for (id, item, count) in picked {
            self.act_as(id, Action::PickUp { item, count });
        }
    }

    /// Throws items out in front of a player (nothing happens if its body is gone).
    pub(crate) fn throw(&mut self, id: PlayerId, item: BlockId, n: u32) {
        let Some(Some(body)) = self.bodies.get(id.0 as usize) else { return };
        let dir = body.look_dir();
        let pos = body.eye() + dir * 0.4 - Vec3::new(0.0, 0.3, 0.0);
        self.items.spawn(pos, dir * 6.0 + Vec3::new(0.0, 1.5, 0.0), item, n, 1.5);
        self.play(sound::DROP, 0, pos, 1.0);
    }
}
