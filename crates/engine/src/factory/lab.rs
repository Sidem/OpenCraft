//! Research lab: uses science packs to research the world's chosen tech (`research.rs`). Buffer slot
//! `i` holds `PACKS[i]` only, so one pack can't crowd out another; belts, miners and the panel bring
//! packs in. A unit takes one of each of the tech's packs when it starts, then `seconds` of work at
//! full power; the finished unit counts towards the tech. Powered, like the constructor. A tier
//! (`LAB_TIERS`) works faster and draws more; the top one gets every `free_every`th unit without packs.
//!
//! Invariants: labs never start more units than a tech has left (`step_labs` counts the units
//! already in progress in any lab); a unit in progress finishes for the tech it started on, even if
//! the chosen tech changes. Work is counted in thousandths of a tick at full power, times the tier's
//! speed. `since_free` counts the paid units since the last free one; `free` says the unit in progress
//! is free (nothing to give back). `status` is last tick's (for the view, not saved).

use crate::block::{tex, LAB};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, stack_size, ItemId};
use crate::math::{IVec3, Vec3};
use crate::research::{needs_ai_lab, pack_slot, Research, PACKS, TECHS};

use super::buffer::Buffer;
use super::fibre::Data;
use super::panel::{Panel, ROLE_INPUT};
use super::power::{Power, FULL_SPEED, NOT_WIRED};
use super::process::{needs_center, step_centers, Processor};
use super::render::push_box;
use super::{ticks, Factory, Kind, Machine};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabStatus {
    NoResearch,
    Working,
    NoPacks,
    /// Other labs are already doing the tech's last units.
    AllTaken,
    /// The chosen tech uses a pack a small lab has no slot for.
    NeedsCenter,
    /// The chosen tech costs compute: only an AI lab can research it.
    NeedsAi,
    NoPower,
}

/// What a lab tier does, Mk1 first.
pub struct LabTier {
    /// Work per tick relative to Mk1.
    pub speed: u32,
    /// kW while researching.
    pub power: u32,
    /// Every this many units the last one takes no packs (0: never).
    pub free_every: u8,
}

/// Pack kinds a small lab holds (the first of `PACKS`); a research center holds more (`process/center.rs`).
pub const LAB_PACK_SLOTS: usize = 4;

pub const LAB_TIERS: [LabTier; 4] = [
    LabTier { speed: 1, power: 10, free_every: 0 },
    LabTier { speed: 2, power: 20, free_every: 0 },
    LabTier { speed: 3, power: 30, free_every: 5 },
    LabTier { speed: 4, power: 40, free_every: 3 },
];

pub struct Lab {
    pub pos: IVec3,
    pub tier: u8,
    /// Paid units since the last free one, and whether the unit in progress is free.
    pub since_free: u8,
    pub free: bool,
    /// Slot `i` holds `PACKS[i]`.
    pub packs: Buffer,
    /// The tech of the unit in progress, and the work it has had (thousandths of a full-power tick).
    pub unit: Option<u8>,
    pub progress: u32,
    /// Last tick's speed from its grid, in thousandths (derived, for the readout).
    pub speed: u32,
    pub status: LabStatus,
}

/// One tick of every lab and research center, each at its grid's speed.
pub fn step_labs(
    labs: &mut [Lab],
    processors: &mut [Processor],
    poles: (&[Option<u32>], &[Option<u32>]),
    power: &Power,
    data: &Data,
    research: &mut Research,
) {
    let mut taken = [0u32; TECHS.len()];
    for t in labs.iter().filter_map(|l| l.unit).chain(processors.iter().filter_map(|p| p.study.unit)) {
        taken[t as usize] += 1;
    }
    for (l, &p) in labs.iter_mut().zip(poles.0) {
        l.step(research, &mut taken, power.speed(p));
    }
    step_centers(processors, poles.1, power, data, research, &mut taken);
}

impl Lab {
    pub fn new(pos: IVec3) -> Lab {
        let packs = Buffer::new(Kind::Lab.def().slots);
        Lab {
            pos,
            tier: 0,
            since_free: 0,
            free: false,
            packs,
            unit: None,
            progress: 0,
            speed: 0,
            status: LabStatus::NoResearch,
        }
    }

    pub fn stats(&self) -> &'static LabTier {
        &LAB_TIERS[self.tier as usize]
    }

    /// Makes it tier `tier`; a unit in progress keeps the way it was paid for.
    pub fn set_tier(&mut self, tier: u8) {
        self.tier = tier;
        self.since_free = 0;
    }

    /// Whether the next unit it starts is the free one.
    fn free_next(&self) -> bool {
        let every = self.stats().free_every;
        every > 0 && self.since_free + 1 >= every
    }

    /// How many of `item` it would take now: science packs, up to their slot's room.
    pub fn room_for(&self, item: ItemId) -> u32 {
        let Some(s) = pack_slot(item).and_then(|i| self.packs.slots.get(i)) else { return 0 };
        stack_size(item) - if s.is_empty() { 0 } else { s.count }
    }

    pub fn accept(&mut self, item: ItemId) -> bool {
        self.room_for(item) > 0 && self.add(item, 1) == 0
    }

    /// Puts up to `n` of `item` in its slot; returns what didn't fit.
    pub fn add(&mut self, item: ItemId, n: u32) -> u32 {
        let put = n.min(self.room_for(item));
        if let Some(i) = pack_slot(item).filter(|&i| put > 0 && i < self.packs.slots.len()) {
            self.packs.slots[i] = Stack { item, count: self.packs.slots[i].count + put };
        }
        n - put
    }

    /// Whether it would work this tick if powered (its grid counts it as demand).
    pub fn wants_power(&self, research: &Research) -> bool {
        self.unit.is_some() || research.current.is_some_and(|t| !needs_ai_lab(t) && self.has_packs(t))
    }

    fn step(&mut self, research: &mut Research, taken: &mut [u32], speed: u32) {
        self.speed = speed;
        let tech = match self.unit {
            Some(t) => t,
            None => {
                let t = match self.can_start(research, taken) {
                    Ok(_) if speed == 0 => Err(LabStatus::NoPower),
                    started => started,
                };
                let t = match t {
                    Ok(t) => t,
                    Err(why) => {
                        self.status = why;
                        return;
                    }
                };
                self.free = self.free_next();
                self.since_free = if self.free { 0 } else { self.since_free.saturating_add(1) };
                for &p in TECHS[t as usize].packs.iter().filter(|_| !self.free) {
                    self.packs.take(pack_slot(p).unwrap_or(0), 1);
                }
                taken[t as usize] += 1;
                (self.unit, self.progress) = (Some(t), 0);
                t
            }
        };
        if speed == 0 {
            self.status = LabStatus::NoPower;
            return;
        }
        self.status = LabStatus::Working;
        self.progress += speed * self.stats().speed;
        if self.progress >= ticks(TECHS[tech as usize].seconds) * FULL_SPEED {
            research.add_unit(tech);
            taken[tech as usize] -= 1;
            (self.unit, self.progress, self.free) = (None, 0, false);
        }
    }

    /// The tech a new unit would be for, or why none can start.
    fn can_start(&self, research: &Research, taken: &[u32]) -> Result<u8, LabStatus> {
        let t = research.current.ok_or(LabStatus::NoResearch)?;
        if needs_ai_lab(t) {
            Err(LabStatus::NeedsAi)
        } else if needs_center(t) {
            Err(LabStatus::NeedsCenter)
        } else if research.progress(t) + taken[t as usize] >= TECHS[t as usize].units {
            Err(LabStatus::AllTaken)
        } else if !self.free_next() && !self.has_packs(t) {
            Err(LabStatus::NoPacks)
        } else {
            Ok(t)
        }
    }

    fn has_packs(&self, tech: u8) -> bool {
        TECHS[tech as usize]
            .packs
            .iter()
            .all(|&p| pack_slot(p).is_some_and(|i| self.packs.slots.get(i).is_some_and(|s| !s.is_empty())))
    }

    /// The first readout line, also the panel's status.
    pub fn status_text(&self, research: &Research) -> String {
        let name = |t: Option<u8>| t.map_or("", |t| TECHS[t as usize].name);
        match self.status {
            LabStatus::NoResearch => "No research chosen: pick one on the research screen (T)".to_string(),
            LabStatus::Working => {
                let slow =
                    if self.speed < FULL_SPEED { format!(" (low power: {}%)", self.speed / 10) } else { String::new() };
                format!("Researching {}{slow}", name(self.unit))
            }
            LabStatus::NoPacks => {
                let packs = research.current.map_or(&[][..], |t| TECHS[t as usize].packs);
                let names: Vec<&str> = packs.iter().map(|&p| item::name(p)).collect();
                format!("Waiting for {}", names.join(" and "))
            }
            LabStatus::AllTaken => format!("Other labs are finishing {}", name(research.current)),
            LabStatus::NeedsCenter => format!("{} needs a research center", name(research.current)),
            LabStatus::NeedsAi => format!("{} needs an AI lab", name(research.current)),
            LabStatus::NoPower => NOT_WIRED.to_string(),
        }
    }

    pub fn panel(&self, research: &Research) -> Panel {
        let total = self.unit.map_or(1, |t| ticks(TECHS[t as usize].seconds));
        Panel {
            block: LAB,
            recipe: None,
            choosable: false,
            progress: self.progress / total,
            fire: 0,
            slots: self.packs.slots.iter().map(|&s| (ROLE_INPUT, s)).collect(),
            status: self.status_text(research),
            filter: None,
        }
    }
}

impl Machine for Lab {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: packs and the unit in progress.
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        self.packs.write_state(w);
        w.u8(self.unit.unwrap_or(u8::MAX));
        w.u32(self.progress);
        w.u8(self.tier);
        w.u8(self.since_free);
        w.bool(self.free);
    }

    fn read_state(r: &mut ByteReader) -> Option<Lab> {
        let mut l = Lab::new(r.ivec3()?);
        // Saves before version 20 had no blue pack slot.
        let held = match r.version {
            22.. => LAB_PACK_SLOTS,
            20..=21 => LAB_PACK_SLOTS - 1,
            _ => LAB_PACK_SLOTS - 2,
        };
        l.packs = Buffer::read_state(r, held)?;
        l.packs.slots.resize(LAB_PACK_SLOTS, Stack::default());
        let slots_ok = l.packs.slots.iter().zip(PACKS).all(|(s, p)| s.is_empty() || s.item == p);
        let unit = r.u8()?;
        l.unit = (unit != u8::MAX).then_some(unit);
        l.progress = r.u32()?;
        if r.version >= 21 {
            (l.tier, l.since_free, l.free) = (r.u8()?, r.u8()?, r.bool()?);
        }
        let ok = slots_ok && l.unit.is_none_or(|t| (t as usize) < TECHS.len()) && (l.tier as usize) < LAB_TIERS.len();
        ok.then_some(l)
    }

    /// Its packs, plus those of an unfinished unit (given back rather than lost).
    fn contents(&self) -> Vec<Stack> {
        let mut all = self.packs.contents();
        if let Some(t) = self.unit.filter(|_| !self.free) {
            all.extend(TECHS[t as usize].packs.iter().map(|&item| Stack { item, count: 1 }));
        }
        all
    }

    fn describe(&self, f: &Factory) -> String {
        let r = &f.research;
        let mut lines = vec![self.status_text(r)];
        if let Some(t) = self.unit.or(r.current) {
            let tech = &TECHS[t as usize];
            lines.push(format!("{}: {} of {} units", tech.name, r.progress(t), tech.units));
        }
        let held: Vec<String> =
            self.packs.contents().iter().map(|s| format!("{} {}", s.count, item::name(s.item))).collect();
        if !held.is_empty() {
            lines.push(held.join(" · "));
        }
        let LabTier { speed, power, free_every } = *self.stats();
        let nth = match free_every {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        };
        let free = if free_every > 0 { format!(" · every {free_every}{nth} unit is free") } else { String::new() };
        lines.push(format!("×{speed} speed · needs {power} kW{free}"));
        lines.push("Right-click to open".to_string());
        lines.join("\n")
    }

    /// A white cabinet with a glass dome that glows while it works, and a status lamp.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let body = [tex::LAB_TOP, tex::LAB_SIDE, tex::FRAME];
        push_box(out, rel + Vec3::new(0.0, -0.4, 0.0), 0.0, [0.94, 0.18, 0.94], 0.0, body, false);
        push_box(out, rel + Vec3::new(0.0, -0.1, 0.0), 0.0, [0.67, 0.43, 0.67], 0.0, [tex::FLASK_GLASS; 3], false);
        for x in [-0.37, 0.37] {
            push_box(out, rel + Vec3::new(x, -0.02, 0.0), 0.0, [0.08, 0.61, 0.75], 0.0, [tex::FRAME; 3], false);
        }
        push_box(out, rel + Vec3::new(0.0, 0.26, 0.0), 0.0, [0.78, 0.11, 0.78], 0.0, body, false);
        let working = self.status == LabStatus::Working;
        let pulse = if working { 0.03 * (time * 3.0).sin().abs() } else { 0.0 };
        push_box(out, rel + Vec3::new(0.0, -0.22 + pulse, 0.0), 0.0, [0.35, 0.13, 0.35], 0.0, [tex::LAB_TOP; 3], false);
        push_box(out, rel + Vec3::new(-0.28, 0.46, -0.28), 0.0, [0.06, 0.37, 0.06], 0.0, [tex::FRAME; 3], false);
        push_box(out, rel + Vec3::new(-0.28, 0.68, -0.28), 0.0, [0.14, 0.12, 0.14], 0.0, [tex::COPPER_INGOT; 3], false);
        let lamp = match self.status {
            LabStatus::Working => tex::LAMP_GREEN,
            LabStatus::NoPacks | LabStatus::AllTaken | LabStatus::NeedsCenter | LabStatus::NeedsAi => tex::LAMP_YELLOW,
            LabStatus::NoResearch | LabStatus::NoPower => tex::LAMP_RED,
        };
        push_box(out, rel + Vec3::new(0.31, 0.35, 0.31), 0.0, [0.13, 0.08, 0.13], 0.0, [lamp; 3], false);
    }
}
