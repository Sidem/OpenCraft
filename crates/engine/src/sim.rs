//! The deterministic core: the tick counter, the world's blocks (edits included), the factory with its
//! deposits, each player's inventory, the core random stream, and the queue of pending actions.
//! Players join and leave through actions too, so every peer agrees on who exists at each tick.
//!
//! Invariants (DEV_PLAN section 3.4): state changes only in `step`, which first applies the actions due
//! this tick (`action.rs`) in (tick, player, sequence) order and then advances the factory by `TICK`.
//! It never takes frame time, the camera or `Sounds`, and never asks which chunks are loaded. Core code
//! reads and writes blocks through `World::block_anywhere_or_generate` / `set_block_anywhere`. `World`
//! also still holds the render cache (loaded chunks, meshes, streaming), which the core must not read.
//! Anything the rest of the game should react to leaves as a [`SimEvent`] in `events`;
//! `Game::handle_sim_events` (events.rs) drains them every tick.
//!
//! To add core state: a field here (or on the type that owns it) and its bytes in that type's
//! `write_state`, which `state_hash` and saves use. To change it: an `Action`.
//! To tell the game about something: a `SimEvent` variant and its arm in `handle_sim_events`.

use crate::action::Action;
use crate::block::BlockId;
use crate::bytes::{fnv1a, ByteReader, ByteWriter};
use crate::factory::Factory;
use crate::inventory::Inventory;
use crate::item::ItemId;
use crate::math::{hash2, IVec3, Rng, Vec3};
use crate::world::World;

/// Index of a player in `Sim::players`. Ids are reused after a player leaves.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PlayerId(pub u8);

/// Something that happened in the core that the authority or the view reacts to.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SimEvent {
    /// A miner is drawing ore; `pos` is the block it drills. Sent every `MINER_PULSE_TICKS`.
    MinerWorking {
        pos: IVec3,
    },
    BlockBroken {
        player: PlayerId,
        pos: IVec3,
        block: BlockId,
    },
    BlockPlaced {
        player: PlayerId,
        pos: IVec3,
        block: BlockId,
    },
    /// Items entered a player's inventory from the world (a pickup or a machine's contents).
    Gained {
        player: PlayerId,
        item: ItemId,
        count: u32,
    },
    /// A player crafted `count` of `item` (some may have been thrown for lack of room).
    Crafted {
        player: PlayerId,
        item: ItemId,
        count: u32,
    },
    /// Loose items to spawn at `pos` with velocity `vel` (a broken block's drops).
    Dropped {
        pos: Vec3,
        vel: Vec3,
        item: ItemId,
        count: u32,
    },
    /// Loose items to throw out in front of a player (dropping, or no room in the inventory).
    Thrown {
        player: PlayerId,
        item: ItemId,
        count: u32,
    },
}

/// A player's core state. The body (position, physics) is not core: it belongs to the authority.
#[derive(Default)]
pub struct PlayerCore {
    /// Slots, cursor stack and selected hotbar slot.
    pub inventory: Inventory,
    /// Who this is across visits (a random id the player's browser keeps); 0 for nobody in
    /// particular (the world's owner, test players), whose things aren't kept when they leave.
    pub key: u64,
}

/// A player who left, kept under their key until they join again.
pub struct Away {
    pub key: u64,
    /// Where their body was, so they come back there.
    pub pos: Vec3,
    pub inventory: Inventory,
}

pub struct Sim {
    /// Ticks run so far; game time is `tick as f64 * TICK`. Actions queued for `tick` run in the next `step`.
    pub tick: u64,
    pub world: World,
    pub factory: Factory,
    /// Indexed by `PlayerId`; `None` where no player is (it left, or never joined). Changed only by the
    /// `Join` and `Leave` actions.
    pub players: Vec<Option<PlayerCore>>,
    /// Players who left with a key, oldest first; `Join` with that key gives their things back.
    pub away: Vec<Away>,
    pub rng: Rng,
    /// Events from the ticks since the last drain.
    pub events: Vec<SimEvent>,
    /// Actions not yet applied, sorted by (tick, player, sequence).
    pending: Vec<Queued>,
    next_seq: u32,
}

impl Sim {
    /// A fresh world with one player, `PlayerId(0)`.
    pub fn new(seed: u32, view_radius: i32) -> Sim {
        Sim {
            tick: 0,
            world: World::new(seed, view_radius),
            factory: Factory::default(),
            players: vec![Some(PlayerCore::default())],
            away: Vec::new(),
            rng: Rng::new(hash2(seed, 17, 42) as u64),
            events: Vec::new(),
            pending: Vec::new(),
            next_seq: 0,
        }
    }

    /// A player's core state; `None` before its `Join` applies and after its `Leave`.
    pub fn player(&self, id: PlayerId) -> Option<&PlayerCore> {
        self.players.get(id.0 as usize)?.as_ref()
    }

    /// Schedules `action` for `tick` (a tick already run means the next one).
    pub fn queue(&mut self, tick: u64, player: PlayerId, action: Action) {
        let q = Queued { tick: tick.max(self.tick), player, seq: self.next_seq, action };
        self.next_seq = self.next_seq.wrapping_add(1);
        let at = self.pending.iter().position(|p| p.key() > q.key()).unwrap_or(self.pending.len());
        self.pending.insert(at, q);
    }

    /// The actions queued for coming ticks, in the order they will apply: (tick, player, action) each.
    /// A join snapshot carries them, since a save doesn't.
    pub fn write_pending(&self, w: &mut ByteWriter) {
        w.count(self.pending.len());
        for q in &self.pending {
            w.u64(q.tick);
            w.u8(q.player.0);
            q.action.write(w);
        }
    }

    /// Queues what `write_pending` wrote, keeping its order.
    pub fn read_pending(&mut self, r: &mut ByteReader) -> Option<()> {
        for _ in 0..r.count()? {
            let (tick, player) = (r.u64()?, PlayerId(r.u8()?));
            self.queue(tick, player, Action::read(r)?);
        }
        Some(())
    }

    /// Advances the core by one tick of `TICK` seconds: this tick's actions, then the factory.
    pub fn step(&mut self) {
        let due = self.pending.iter().take_while(|q| q.tick <= self.tick).count();
        let later = self.pending.split_off(due);
        for q in std::mem::replace(&mut self.pending, later) {
            self.apply(q.player, q.action);
        }
        self.factory.update(&mut self.world, self.tick, &mut self.events);
        self.tick += 1;
    }

    /// Fingerprint of the core state. Two cores with equal hashes behave the same from here on,
    /// given the same actions. Compare it between peers, runs or a save and its reload.
    pub fn state_hash(&self) -> u64 {
        let mut w = ByteWriter::default();
        self.write_state(&mut w);
        fnv1a(&w.bytes)
    }

    /// The canonical core state: tick, rng, players (up to the last one here), away players, world
    /// edits, factory and deposits. Pending actions and undrained events are not state: peers may
    /// hold different queues for future ticks (a join snapshot sends them along: net/snapshot.rs).
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u64(self.tick);
        w.u64(self.rng.state());
        let players = self.players.iter().rposition(Option::is_some).map_or(0, |last| last + 1);
        w.count(players);
        for p in &self.players[..players] {
            w.bool(p.is_some());
            if let Some(p) = p {
                p.inventory.write_state(w);
                w.u64(p.key);
            }
        }
        w.count(self.away.len());
        for a in &self.away {
            w.u64(a.key);
            w.vec3(a.pos);
            a.inventory.write_state(w);
        }
        self.world.write_state(w);
        self.factory.write_state(w);
    }

    /// Restores what `write_state` wrote into a fresh `Sim` made with the same seed. Saves before
    /// version 10 have no keys (0) and nobody away.
    pub fn read_state(&mut self, r: &mut ByteReader) -> Option<()> {
        self.tick = r.u64()?;
        self.rng = Rng::new(r.u64()?);
        let players = r.count()?;
        if players > 256 {
            return None;
        }
        let keyed = r.version >= 10;
        self.players.clear();
        for _ in 0..players {
            let present = r.bool()?;
            self.players.push(if present {
                Some(PlayerCore { inventory: Inventory::read_state(r)?, key: if keyed { r.u64()? } else { 0 } })
            } else {
                None
            });
        }
        self.away.clear();
        for _ in 0..if keyed { r.count()? } else { 0 } {
            let (key, pos) = (r.u64()?, r.vec3()?);
            self.away.push(Away { key, pos, inventory: Inventory::read_state(r)? });
        }
        self.world.read_state(r)?;
        self.factory = Factory::read_state(&mut self.world, r)?;
        Some(())
    }
}

struct Queued {
    tick: u64,
    player: PlayerId,
    seq: u32,
    action: Action,
}

impl Queued {
    fn key(&self) -> (u64, PlayerId, u32) {
        (self.tick, self.player, self.seq)
    }
}

#[cfg(test)]
pub(crate) mod tests;
