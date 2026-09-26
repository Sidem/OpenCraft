//! The authority's share of the game: every player's body (physics, falling out of the world), loose
//! items and who picks them up, throwing items, players joining and leaving, and loading the ground
//! around every body. In single-player the local game is the authority; in co-op only the host is,
//! and a client spawns no items (`spawn_item`).
//!
//! Invariants: `Game.bodies` is indexed by `PlayerId`, like `Sim.players`. A body exists from `join`
//! on, and its core state from the tick the queued `Join` applies. The local player's body always
//! exists (`leave` refuses it). A body moved by another machine (`Role::drives`, net/players.rs) takes
//! that machine's states and runs no physics here. Bodies only move while the ground under them is
//! loaded; the authority loads it around every body (`stream_around_players`).
//! The authority never edits the core: pickups become `PickUp` actions.

use crate::action::Action;
use crate::entities::Collector;
use crate::item::ItemId;
use crate::math::Vec3;
use crate::net::Role;
use crate::player::{self, Player};
use crate::sim::PlayerId;
use crate::sound;
use crate::{Game, TICK};

/// Player physics substeps per tick (1/120 s each).
const PHYSICS_SUBSTEPS: u32 = 2;
/// Bodies that fall below this height are put back at spawn.
const FALL_LIMIT: f64 = -64.0;
/// Players in one world at once, the host included (co-op sets the bandwidth budget by it).
pub const MAX_PLAYERS: usize = 4;

impl Game {
    /// Adds a player with `key` (0 for nobody in particular): its body appears now, where it stood
    /// when that key left or else at spawn, and its inventory when the queued `Join` applies.
    /// `None` when `MAX_PLAYERS` are here.
    pub(crate) fn join(&mut self, key: u64) -> Option<PlayerId> {
        if self.bodies.iter().flatten().count() >= MAX_PLAYERS {
            return None;
        }
        let slot = self.bodies.iter().position(Option::is_none).unwrap_or(self.bodies.len());
        let id = PlayerId(u8::try_from(slot).ok()?);
        if slot == self.bodies.len() {
            self.bodies.push(None);
        }
        let back = self.sim.away.iter().find(|a| key != 0 && a.key == key);
        self.bodies[slot] = Some(Player::new(back.map_or(self.spawn, |a| a.pos)));
        self.act_as(id, Action::Join { key });
        Some(id)
    }

    /// Removes a player's body now and its core state when the queued `Leave` applies (kept under its
    /// key, if it has one). The local player stays.
    pub(crate) fn leave(&mut self, id: PlayerId) {
        if id == self.local {
            return;
        }
        if let Some(body) = self.bodies.get_mut(id.0 as usize).and_then(Option::take) {
            self.act_as(id, Action::Leave { pos: body.pos });
        }
    }

    /// Streams the world around the local player and, where this game is the authority, around every
    /// other body too (without meshing: world/streaming.rs).
    pub(crate) fn stream_around_players(&mut self) {
        let mut others = Vec::new();
        if !matches!(self.role, Role::Client(_)) {
            let local = self.local.0 as usize;
            others.extend(
                self.bodies.iter().enumerate().filter(|b| b.0 != local).filter_map(|b| b.1.as_ref()).map(|b| b.pos),
            );
        }
        self.sim.world.update_streaming(self.body().pos, &others);
    }

    /// Moves every body this game runs by one tick, and puts any that fell out of the world back at spawn.
    pub(crate) fn step_bodies(&mut self) {
        let world = &self.sim.world;
        let mut solid = |x, y, z| world.is_solid(x, y, z);
        let respawn = self.spawn + Vec3::new(0.0, 2.0, 0.0);
        let mut local_fell = false;
        for (slot, body) in self.bodies.iter_mut().enumerate() {
            let Some(body) = body else { continue };
            if self.role.drives(PlayerId(slot as u8), self.local) {
                continue;
            }
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
    pub(crate) fn throw(&mut self, id: PlayerId, item: ItemId, n: u32) {
        let Some(Some(body)) = self.bodies.get(id.0 as usize) else { return };
        let dir = body.look_dir();
        let pos = body.eye() + dir * 0.4 - Vec3::new(0.0, 0.3, 0.0);
        self.spawn_item(pos, dir * 6.0 + Vec3::new(0.0, 1.5, 0.0), item, n, 1.5);
        self.play(sound::DROP, 0, pos, 1.0);
    }

    /// Spawns a loose item, unless this game is a co-op client (it draws the host's: net/items.rs).
    pub(crate) fn spawn_item(&mut self, pos: Vec3, vel: Vec3, item: ItemId, n: u32, pickup_delay: f32) {
        if !matches!(self.role, Role::Client(_)) {
            self.items.spawn(pos, vel, item, n, pickup_delay);
        }
    }
}
