//! The core's bytes: `Sim::write_state` (what `state_hash`, saves and join snapshots carry) and `Sim::read_state`.
//!
//! Invariants: every piece of core state is written here, in this order, and read back the same way; a reader
//! follows `ByteReader::version` for older saves (`save.rs` lists the versions).
//!
//! To add state: append its write at the end of `write_state` and its read, behind a new `SAVE_VERSION`, at the end
//! of `read_state`.

use super::{Away, BlockTimers, Felling, Fire, PlayerCore, Sim, WaterQueue};
use crate::bytes::{ByteReader, ByteWriter};
use crate::codex::Codex;
use crate::crafting::CraftQueue;
use crate::drones::{cargo::Cargo, Drones};
use crate::factory::Factory;
use crate::ghosts::Ghosts;
use crate::helpers::Helpers;
use crate::inventory::Inventory;
use crate::led::Leds;
use crate::math::Rng;
use crate::mode::Mode;
use crate::opening::Opening;
use crate::settlements::Settlements;
use crate::signs::Signs;

impl Sim {
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
        self.felling.write_state(w);
        self.leds.write_state(w);
        self.factory.write_manual_off(w);
        self.fire.write_state(w);
        self.codex.write_state(w);
        self.world.write_shapes(w);
        self.world.write_paint(w);
        self.paint.write_state(w);
        crate::elevator::write_state(&self.elevator, w);
        self.world.write_bare(w);
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
        if r.version < 68 {
            self.taught |= crate::lessons::KNOWN_BEFORE_68;
        }
        if r.version < 74 {
            self.taught |= crate::lessons::KNOWN_BEFORE_74;
        }
        if r.version < 75 {
            self.taught |= crate::lessons::KNOWN_BEFORE_75;
        }
        if r.version < 76 {
            self.taught |= crate::lessons::KNOWN_BEFORE_76;
        }
        self.signs = if r.version >= 63 { Signs::read_state(r)? } else { Signs::default() };
        self.felling = if r.version >= 73 { Felling::read_state(r)? } else { Felling::default() };
        self.leds = if r.version >= 79 { Leds::read_state(r)? } else { Leds::default() };
        if r.version >= 80 {
            self.factory.read_manual_off(r)?;
        }
        self.fire = if r.version >= 82 { Fire::read_state(r)? } else { Fire::default() };
        self.codex = if r.version >= 83 {
            Codex::read_state(r)?
        } else {
            Codex::seed_from(&self.players, &self.away, &self.factory)
        };
        self.world.read_shapes(r)?;
        self.world.read_paint(r)?;
        self.paint = if r.version >= 85 {
            crate::colour::PaintCharges::read_state(r)?
        } else {
            crate::colour::PaintCharges::default()
        };
        self.elevator = if r.version >= 89 { crate::elevator::read_state(r)? } else { None };
        if r.version >= 90 {
            self.world.read_bare(r)?;
        }
        Some(())
    }
}
