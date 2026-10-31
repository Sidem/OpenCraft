//! The deterministic core: the tick counter, the world's blocks (edits included), the factory with its
//! deposits, each player's inventory, the core random stream, and the queue of pending actions.
//! Players join and leave through actions too, so every peer agrees on who exists at each tick.
//!
//! Invariants (DEV_PLAN section 3.4): state changes only in `step`, which first applies the actions due
//! this tick (`action.rs`) in (tick, player, sequence) order and then advances the factory by `TICK`.
//! It never takes frame time, the camera or `Sounds`, and never asks which chunks are loaded. Core code
//! reads and writes blocks through `World::block_anywhere_or_generate` / `set_block_anywhere`. `World`
//! also still holds the render cache (loaded chunks, meshes, streaming), which the core must not read.
//! Anything the rest of the game should react to leaves as a [`SimEvent`] (`sim/event.rs`) in `events`;
//! `Game::handle_sim_events` (events.rs) drains them every tick.
//!
//! To add core state: a field here (or on the type that owns it) and its bytes in that type's
//! `write_state`, which `state_hash` and saves use. To change it: an `Action`. Core code that changes a
//! block calls `block_changed` (sim/timers.rs), so timed rules (leaf decay, grass) and water react.
//! To tell the game about something: a `SimEvent` variant (`sim/event.rs`) and its arm in `handle_sim_events`.

use crate::action::Action;
use crate::bytes::{fnv1a, ByteReader, ByteWriter};
use crate::crafting::CraftQueue;
use crate::drones::{cargo::Cargo, Drones};
use crate::factory::Factory;
use crate::ghosts::Ghosts;
use crate::helpers::Helpers;
use crate::inventory::Inventory;
use crate::math::{hash2, Rng, Vec3};
use crate::mode::Mode;
use crate::opening::Opening;
use crate::settlements::Settlements;
use crate::signs::Signs;
use crate::world::World;
use crate::worldgen::WorldGen;

mod event;
mod saplings;
mod scrap;
mod timers;
mod torches;
pub mod vault;
mod water;

pub use event::SimEvent;
pub use timers::BlockTimers;
pub use water::WaterQueue;

/// Index of a player in `Sim::players`. Ids are reused after a player leaves.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PlayerId(pub u8);

/// A player's core state. The body (position, physics) is not core: it belongs to the authority.
#[derive(Default)]
pub struct PlayerCore {
    /// Slots, cursor stack and selected hotbar slot.
    pub inventory: Inventory,
    /// Who this is across visits (a random id the player's browser keeps); 0 for nobody in
    /// particular (the world's owner, test players), whose things aren't kept when they leave.
    pub key: u64,
    /// Hand crafts waiting or in progress (`crafting.rs`).
    pub crafts: CraftQueue,
    /// Jetpack and personal drone (`helpers/`).
    pub helpers: Helpers,
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
    /// Pending block changes: leaf decay, grass spreading (timers.rs).
    pub timers: BlockTimers,
    /// Pending flowing-water checks (water.rs).
    pub water: WaterQueue,
    /// Planned blocks and machines (ghosts.rs).
    pub ghosts: Ghosts,
    /// Construction drones in flight (drones/).
    pub drones: Drones,
    /// Cargo drones in flight and the routes between ports (drones/cargo.rs).
    pub cargo: Cargo,
    /// Normal or creative, fixed when the world was made (mode.rs).
    pub mode: Mode,
    /// Where a new normal world's opening scene is (opening.rs); None in older and creative worlds.
    pub opening: Option<Opening>,
    /// The towns the players founded (settlements/).
    pub settlements: Settlements,
    /// The players' note signs (signs/).
    pub signs: Signs,
    /// How far the main tips and the lessons got in this world (hints.rs, lessons.rs); neither ever goes back.
    pub hints_reached: u8,
    pub taught: u32,
    /// Events from the ticks since the last drain.
    pub events: Vec<SimEvent>,
    /// Actions not yet applied, sorted by (tick, player, sequence).
    pending: Vec<Queued>,
    next_seq: u32,
}

impl Sim {
    /// A fresh world from the newest generator with one player, `PlayerId(0)`.
    #[cfg(test)]
    pub fn new(seed: u32, view_radius: i32) -> Sim {
        Sim::with_generator(WorldGen::new(seed), view_radius)
    }

    pub fn with_generator(generator: WorldGen, view_radius: i32) -> Sim {
        let seed = generator.seed();
        Sim {
            tick: 0,
            world: World::with_generator(generator, view_radius),
            factory: Factory::default(),
            players: vec![Some(PlayerCore::default())],
            away: Vec::new(),
            rng: Rng::new(hash2(seed, 17, 42) as u64),
            timers: BlockTimers::default(),
            water: WaterQueue::default(),
            ghosts: Ghosts::default(),
            drones: Drones::default(),
            cargo: Cargo::default(),
            mode: Mode::Normal,
            opening: None,
            settlements: Settlements::default(),
            signs: Signs::default(),
            hints_reached: 0,
            taught: 0,
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

    /// Advances the core by one tick of `TICK` seconds: this tick's actions, the timers and water checks due, the
    /// factory (its block changes start new ones), then towns, drones, cargo and the tips.
    pub fn step(&mut self) {
        self.sync_depots();
        let due = self.pending.iter().take_while(|q| q.tick <= self.tick).count();
        let later = self.pending.split_off(due);
        for q in std::mem::replace(&mut self.pending, later) {
            self.apply(q.player, q.action);
        }
        self.run_crafting();
        self.run_helpers();
        self.run_timers();
        self.run_water();
        let made_from = self.events.len();
        self.factory.update(&mut self.world, self.tick, &mut self.events);
        for (pos, old) in std::mem::take(&mut self.factory.changed) {
            self.block_changed(pos, old);
        }
        self.step_towns();
        self.settlements.made.note(&self.events[made_from..]);
        self.step_drones();
        self.step_cargo();
        self.step_tips();
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
    /// edits, factory and deposits, block timers, water checks. Pending actions and undrained events are not state:
    /// peers may hold different queues for future ticks (a join snapshot sends them along: net/snapshot.rs).
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
                p.crafts.write_state(w);
                p.helpers.write_state(w);
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
        self.timers.write_state(w);
        self.water.write_state(w);
        self.ghosts.write_state(w);
        self.drones.write_state(w);
        self.cargo.write_state(w);
        w.u8(self.mode.byte());
        Opening::write_state(&self.opening, w);
        self.settlements.write_state(w);
        w.u8(self.hints_reached);
        w.u32(self.taught);
        self.signs.write_state(w);
    }

    /// Restores what `write_state` wrote into a fresh `Sim` made with the same seed (`save.rs` lists the versions).
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
                let (inventory, key) = (Inventory::read_state(r)?, if keyed { r.u64()? } else { 0 });
                let crafts = if r.version >= 23 { CraftQueue::read_state(r)? } else { CraftQueue::default() };
                let helpers = if r.version >= 28 { Helpers::read_state(r)? } else { Helpers::default() };
                Some(PlayerCore { inventory, key, crafts, helpers })
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
        self.timers = if r.version >= 11 { BlockTimers::read_state(r)? } else { BlockTimers::default() };
        self.water = if r.version >= 13 { WaterQueue::read_state(r)? } else { WaterQueue::default() };
        self.ghosts = if r.version >= 26 { Ghosts::read_state(r)? } else { Ghosts::default() };
        self.drones = if r.version >= 27 { Drones::read_state(r)? } else { Drones::default() };
        self.cargo = if r.version >= 37 { Cargo::read_state(r)? } else { Cargo::default() };
        let mode = if r.version >= 38 { Mode::from_byte(r.u8()?)? } else { Mode::Normal };
        self.set_mode(mode);
        self.opening = if r.version >= 47 { Opening::read_state(r)? } else { None };
        if let Some(o) = self.opening.as_mut().filter(|_| r.version < 57) {
            o.migrate(&self.factory);
        }
        // Save 49 added each town's stage and order, 50 its plots and drones, 51 dormancy, 53 the harbour's product and
        // what machines have made (`Settlements::read_state` follows the version).
        self.settlements = if r.version >= 48 { Settlements::read_state(r)? } else { Settlements::default() };
        self.hints_reached = if r.version >= 57 { r.u8()? } else { 0 };
        self.taught = if r.version >= 58 { r.u32()? } else { crate::lessons::STALL_MASK };
        self.signs = if r.version >= 63 { Signs::read_state(r)? } else { Signs::default() };
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
