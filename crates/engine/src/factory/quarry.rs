//! Quarry: a powered machine (its tier's kW, `QUARRY_TIERS`) that digs the ground in a box in front of it for real,
//! leaving a pit. The box (`DigBox`) is `width` Ã— `width` cells from one cell in front of its face,
//! from the quarry's own level down to its depth choice (`DEPTHS`). It works through the box top
//! layer first, row by row back and forth (`DigBox::cell`), one block every `dig_ticks` at full
//! power. Each ground block (`QUARRIABLE`) becomes its normal drop in the output buffer, which feeds
//! belts leading away and machines beside it like a miner's. Everything else stays: ore (listed in
//! `found` as it is uncovered), bedrock, logs, machines. Water in the next cell stops it (flooded)
//! until a pump takes it away.
//!
//! Invariants: cells before `next` are done; it looks past at most `SCAN_PER_TICK` cells a tick. A
//! new or re-sized quarry starts at 0 and skips air, so a picked-up quarry keeps nothing yet carries
//! on. Digging reads and edits through the `*_anywhere` accessors and reports each edit in `changed`
//! (water and block timers react). `head`, `from` and `speed` are derived (the model, this tick's
//! power). The panel's choices arrive through `Factory::set_quarry` (`Action::SetQuarry`).

mod dig_box;
mod model;

pub use dig_box::{survey, DigBox, DEFAULT_DEPTH, DEFAULT_WIDTH, DEPTHS, QUARRIABLE, WIDTHS};

use crate::block::{self, BlockId, AIR, LIQUID, QUARRY};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::{owner_of, DepositKey};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};
use crate::sim::SimEvent;
use crate::world::World;

use super::belt::Belt;
use super::buffer::Buffer;
use super::describe::fmt_int;
use super::links::{deliver, Link, Sinks};
use super::panel::{Panel, ROLE_OUTPUT};
use super::power::{FULL_SPEED, POLE_REACH};
use super::{Factory, Kind, Machine};

/// What a quarry tier does, Mk1 first.
pub struct QuarryTier {
    /// Ticks per block at full power: 30, 15 and 10 make 2, 4 and 6 blocks a second (a 7 Ã— 7 Ã— 16 pit takes
    /// about 7 minutes at Mk1).
    pub dig_ticks: u32,
    /// kW while it digs.
    pub power: u32,
}

pub const QUARRY_TIERS: [QuarryTier; 3] = [
    QuarryTier { dig_ticks: 30, power: 10 },
    QuarryTier { dig_ticks: 15, power: 20 },
    QuarryTier { dig_ticks: 10, power: 30 },
];
/// Cells with nothing to dig it looks past in one tick.
const SCAN_PER_TICK: u32 = 64;
/// Deposits it remembers uncovering.
const MAX_FOUND: usize = 6;

/// What it did last tick.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QuarryStatus {
    Digging,
    OutputFull,
    NoPower,
    Flooded,
    Paused,
    Done,
}

/// Every status, in declaration order: saves store `status as u8`.
const STATUSES: [QuarryStatus; 6] = [
    QuarryStatus::Digging,
    QuarryStatus::OutputFull,
    QuarryStatus::NoPower,
    QuarryStatus::Flooded,
    QuarryStatus::Paused,
    QuarryStatus::Done,
];

pub struct Quarry {
    pub pos: IVec3,
    pub tier: u8,
    /// Where the box lies, as a `DIRS` index (the placing player's facing, turned with R).
    pub facing: u8,
    /// Indices into `WIDTHS` and `DEPTHS`.
    pub width: u8,
    pub depth: u8,
    pub paused: bool,
    /// The next cell of the box (`DigBox::cell`).
    pub next: u32,
    /// Work on that cell, in thousandths of a tick at full speed.
    pub progress: u32,
    pub dug: u32,
    pub out: Buffer,
    pub outs: Vec<Link>,
    pub next_out: usize,
    /// Deposits it uncovered, with the layer where it first met each.
    pub found: Vec<(DepositKey, i32)>,
    pub status: QuarryStatus,
    /// This tick's speed from its grid, in thousandths (derived).
    pub speed: u32,
    /// The cell being dug and the one dug before it, for the model (derived).
    pub head: IVec3,
    pub from: IVec3,
}

impl Quarry {
    pub fn new(pos: IVec3, facing: u8) -> Quarry {
        let mut q = Quarry {
            pos,
            tier: 0,
            facing: facing % 4,
            width: DEFAULT_WIDTH,
            depth: DEFAULT_DEPTH,
            paused: false,
            next: 0,
            progress: 0,
            dug: 0,
            out: Buffer::new(Kind::Quarry.def().slots),
            outs: Vec::new(),
            next_out: 0,
            found: Vec::new(),
            status: QuarryStatus::Digging,
            speed: 0,
            head: pos,
            from: pos,
        };
        q.park();
        q
    }

    pub fn stats(&self) -> &'static QuarryTier {
        &QUARRY_TIERS[self.tier as usize]
    }

    /// Work one block takes, in thousandths of a tick at full speed.
    pub fn full_work(&self) -> u32 {
        self.stats().dig_ticks * FULL_SPEED
    }

    pub fn dig_box(&self) -> DigBox {
        DigBox::new(self.pos, self.facing, self.width, self.depth)
    }

    /// Whether it would dig this tick if powered (its grid counts it as demand).
    pub fn wants_power(&self) -> bool {
        matches!(self.status, QuarryStatus::Digging | QuarryStatus::NoPower)
    }

    /// Takes the panel's choices; a different box starts over from its first cell.
    pub fn set(&mut self, width: u8, depth: u8, paused: bool) {
        if (width, depth) != (self.width, self.depth) {
            (self.width, self.depth) = (width, depth);
            (self.next, self.progress) = (0, 0);
            self.park();
        }
        self.paused = paused;
        if !paused && self.status == QuarryStatus::Paused {
            self.status = QuarryStatus::Digging;
        }
    }

    /// One tick: push one item on, then look for the next block and dig at it.
    pub fn step(
        &mut self,
        world: &mut World,
        belts: &mut [Belt],
        sinks: &mut Sinks,
        changed: &mut Vec<(IVec3, BlockId)>,
        events: &mut Vec<SimEvent>,
    ) {
        self.push_out(belts, sinks);
        if self.paused {
            self.status = QuarryStatus::Paused;
            return;
        }
        let Some((cell, id)) = self.seek(world) else {
            self.status = if self.next >= self.dig_box().cells() { QuarryStatus::Done } else { QuarryStatus::Digging };
            return;
        };
        self.head = cell;
        let drop = ItemId::block(block::def(id).drop);
        self.status = match () {
            _ if LIQUID[id as usize] => QuarryStatus::Flooded,
            _ if !self.out.can_accept(drop) => QuarryStatus::OutputFull,
            _ if self.speed == 0 => QuarryStatus::NoPower,
            _ => QuarryStatus::Digging,
        };
        if self.status != QuarryStatus::Digging {
            return;
        }
        self.progress += self.speed;
        if self.progress < self.full_work() {
            return;
        }
        world.set_block_anywhere(cell, AIR);
        changed.push((cell, id));
        events.push(SimEvent::QuarryDug { pos: cell, block: id });
        self.out.add(drop, 1);
        (self.dug, self.next, self.progress, self.from) = (self.dug + 1, self.next + 1, 0, cell);
    }

    /// Moves `next` past cells with nothing to dig (at most `SCAN_PER_TICK`), noting ore; the cell to
    /// dig and its block (ground or water), or `None` when done or still looking.
    fn seek(&mut self, world: &mut World) -> Option<(IVec3, BlockId)> {
        let dig = self.dig_box();
        for _ in 0..SCAN_PER_TICK {
            if self.next >= dig.cells() {
                return None;
            }
            let cell = dig.cell(self.next);
            let id = world.block_anywhere_or_generate(cell);
            if QUARRIABLE[id as usize] || LIQUID[id as usize] {
                return Some((cell, id));
            }
            if block::is_ore(id) && self.found.len() < MAX_FOUND {
                if let Some(d) = owner_of(world, cell).filter(|d| self.found.iter().all(|f| f.0 != d.key)) {
                    self.found.push((d.key, cell.y));
                }
            }
            (self.next, self.progress) = (self.next + 1, 0);
        }
        None
    }

    /// Pushes one held item into the next output that accepts it (round-robin), last slot first.
    fn push_out(&mut self, belts: &mut [Belt], sinks: &mut Sinks) {
        let Some(src) = self.out.slots.iter().rposition(|s| !s.is_empty()) else { return };
        let (item, n) = (self.out.slots[src].item, self.outs.len());
        for i in 0..n {
            let slot = (self.next_out + i) % n;
            if deliver(belts, sinks, self.outs[slot], item, 0.0) {
                self.out.take(src, 1);
                self.next_out = (slot + 1) % n;
                return;
            }
        }
    }

    /// Puts the model's head over the next cell.
    fn park(&mut self) {
        let dig = self.dig_box();
        self.head = dig.cell(self.next.min(dig.cells().saturating_sub(1)));
        self.from = self.head;
    }

    /// The layer being dug (1 = the top) and how many there are.
    pub fn layer(&self) -> (u32, u32) {
        let dig = self.dig_box();
        let per = (dig.width * dig.width) as u32;
        ((self.next / per + 1).min(dig.layers()), dig.layers())
    }

    /// What it is doing, in a line.
    pub fn status_text(&self, f: &Factory) -> String {
        match self.status {
            QuarryStatus::Digging => "Digging".to_string(),
            QuarryStatus::OutputFull => "Output full: put a belt leading away, or a box, next to it".to_string(),
            QuarryStatus::NoPower => {
                let me = f.quarries.iter().position(|q| q.pos == self.pos);
                let pole = me.and_then(|i| f.power.quarry_pole.get(i).copied()).flatten();
                match pole {
                    Some(_) => f.power.grid_line(pole),
                    None => format!("No power: needs a power pole within {POLE_REACH} blocks"),
                }
            }
            QuarryStatus::Flooded => "Flooded: pump the water out of the pit".to_string(),
            QuarryStatus::Paused => "Paused".to_string(),
            QuarryStatus::Done => "Finished: every block it can dig is dug".to_string(),
        }
    }

    /// One line per deposit it uncovered ("Iron vein exposed at y 41").
    pub fn found_lines(&self) -> Vec<String> {
        let label = |k: &DepositKey| format!("{} {}", block::ore_label(k.ore), k.tier.name());
        self.found.iter().map(|(k, y)| format!("{} exposed at y {y}", label(k))).collect()
    }

    /// The panel: status, the dig progress of the current block, and the output buffer.
    pub fn panel(&self, f: &Factory) -> Panel {
        Panel {
            block: QUARRY,
            recipe: None,
            choosable: false,
            progress: self.progress / self.stats().dig_ticks,
            fire: 0,
            slots: self.out.slots.iter().map(|&s| (ROLE_OUTPUT, s)).collect(),
            status: self.status_text(f),
            filter: None,
        }
    }
}

impl Factory {
    /// Blocks quarries within `range` of `eye` are digging: x, y, z and progress in thousandths each.
    pub fn quarry_cracks(&self, eye: Vec3, range: f64) -> Vec<i32> {
        let digging = self.quarries.iter().filter(|q| q.status == QuarryStatus::Digging && q.progress > 0);
        let near = digging.filter(|q| (q.head.as_vec3() - eye).length() <= range);
        near.flat_map(|q| [q.head.x, q.head.y, q.head.z, (q.progress * 1000 / q.full_work()) as i32]).collect()
    }
}

impl Machine for Quarry {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state (`outs` is rebuilt by `relink`; `head`, `from` and `speed` are derived).
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        for b in [self.facing, self.width, self.depth] {
            w.u8(b);
        }
        w.bool(self.paused);
        for n in [self.next, self.progress, self.dug] {
            w.u32(n);
        }
        self.out.write_state(w);
        w.u32(self.next_out as u32);
        w.u8(self.status as u8);
        w.u8(self.tier);
        w.count(self.found.len());
        for (key, y) in &self.found {
            key.write_state(w);
            w.i32(*y);
        }
    }

    fn read_state(r: &mut ByteReader) -> Option<Quarry> {
        let (pos, facing, width, depth) = (r.ivec3()?, r.u8()?, r.u8()?, r.u8()?);
        if facing > 3 || width as usize >= WIDTHS.len() || depth as usize >= DEPTHS.len() {
            return None;
        }
        let mut q = Quarry::new(pos, facing);
        (q.width, q.depth, q.paused) = (width, depth, r.bool()?);
        (q.next, q.progress, q.dug) = (r.u32()?, r.u32()?, r.u32()?);
        q.out = Buffer::read_state(r, q.out.slots.len())?;
        q.next_out = r.u32()? as usize;
        q.status = *STATUSES.get(r.u8()? as usize)?;
        if r.version >= 21 {
            q.tier = r.u8()?;
        }
        if q.tier as usize >= QUARRY_TIERS.len() {
            return None;
        }
        for _ in 0..r.count()?.min(MAX_FOUND) {
            q.found.push((DepositKey::read_state(r)?, r.i32()?));
        }
        q.park();
        Some(q)
    }

    fn contents(&self) -> Vec<Stack> {
        self.out.contents()
    }

    /// Status, layer and count, and what it uncovered.
    fn describe(&self, f: &Factory) -> String {
        let (layer, layers) = self.layer();
        let dig = self.dig_box();
        let mut lines = vec![
            self.status_text(f),
            format!(
                "{w}Ã—{w}, {} Â· layer {layer} of {layers} Â· {} dug Â· needs {} kW",
                DEPTHS[self.depth as usize].label(),
                fmt_int(self.dug as u64),
                self.stats().power,
                w = dig.width
            ),
        ];
        lines.extend(self.found_lines());
        let held = self.out.contents();
        if !held.is_empty() {
            let names: Vec<String> = held.iter().map(|s| format!("{} {}", s.count, item::name(s.item))).collect();
            lines.push(format!("Holding {} Â· right-click for its panel", names.join(", ")));
        }
        lines.join("\n")
    }

    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        model::draw(self, out, rel, time);
    }
}

#[cfg(test)]
mod tests;
