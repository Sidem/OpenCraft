//! Research lab: uses science packs to research the world's chosen tech (`research.rs`). Buffer slot
//! `i` holds `PACKS[i]` only, so one pack can't crowd out another; belts, miners and the panel bring
//! packs in. A unit takes one of each of the tech's packs when it starts, then `seconds` of work at
//! full power; the finished unit counts towards the tech. Powered, like the constructor.
//!
//! Invariants: labs never start more units than a tech has left (`step_labs` counts the units
//! already in progress in any lab); a unit in progress finishes for the tech it started on, even if
//! the chosen tech changes. Work is counted in thousandths of a tick at full power. `status` is
//! last tick's (for the view, not saved).

use crate::block::{tex, LAB};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, stack_size, ItemId};
use crate::math::{IVec3, Vec3};
use crate::research::{pack_slot, Research, PACKS, TECHS};

use super::buffer::Buffer;
use super::panel::{Panel, ROLE_INPUT};
use super::power::{Power, FULL_SPEED, POLE_REACH};
use super::render::push_box;
use super::{ticks, Factory, Kind, Machine};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabStatus {
    NoResearch,
    Working,
    NoPacks,
    /// Other labs are already doing the tech's last units.
    AllTaken,
    NoPower,
}

pub struct Lab {
    pub pos: IVec3,
    /// Slot `i` holds `PACKS[i]`.
    pub packs: Buffer,
    /// The tech of the unit in progress, and the work it has had (thousandths of a full-power tick).
    pub unit: Option<u8>,
    pub progress: u32,
    /// Last tick's speed from its grid, in thousandths (derived, for the readout).
    pub speed: u32,
    pub status: LabStatus,
}

/// One tick of every lab, each at its grid's speed.
pub fn step_labs(labs: &mut [Lab], lab_pole: &[Option<u32>], power: &Power, research: &mut Research) {
    let mut taken = [0u32; TECHS.len()];
    for t in labs.iter().filter_map(|l| l.unit) {
        taken[t as usize] += 1;
    }
    for (l, &p) in labs.iter_mut().zip(lab_pole) {
        l.step(research, &mut taken, power.speed(p));
    }
}

impl Lab {
    pub fn new(pos: IVec3) -> Lab {
        let packs = Buffer::new(Kind::Lab.def().slots);
        Lab { pos, packs, unit: None, progress: 0, speed: 0, status: LabStatus::NoResearch }
    }

    /// How many of `item` it would take now: science packs, up to their slot's room.
    pub fn room_for(&self, item: ItemId) -> u32 {
        let Some(s) = pack_slot(item).map(|i| self.packs.slots[i]) else { return 0 };
        stack_size(item) - if s.is_empty() { 0 } else { s.count }
    }

    pub fn accept(&mut self, item: ItemId) -> bool {
        self.room_for(item) > 0 && self.add(item, 1) == 0
    }

    /// Puts up to `n` of `item` in its slot; returns what didn't fit.
    pub fn add(&mut self, item: ItemId, n: u32) -> u32 {
        let put = n.min(self.room_for(item));
        if let Some(i) = pack_slot(item).filter(|_| put > 0) {
            self.packs.slots[i] = Stack { item, count: self.packs.slots[i].count + put };
        }
        n - put
    }

    /// Whether it would work this tick if powered (its grid counts it as demand).
    pub fn wants_power(&self, research: &Research) -> bool {
        self.unit.is_some() || research.current.is_some_and(|t| self.has_packs(t))
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
                for &p in TECHS[t as usize].packs {
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
        self.progress += speed;
        if self.progress >= ticks(TECHS[tech as usize].seconds) * FULL_SPEED {
            research.add_unit(tech);
            taken[tech as usize] -= 1;
            (self.unit, self.progress) = (None, 0);
        }
    }

    /// The tech a new unit would be for, or why none can start.
    fn can_start(&self, research: &Research, taken: &[u32]) -> Result<u8, LabStatus> {
        let t = research.current.ok_or(LabStatus::NoResearch)?;
        if research.progress(t) + taken[t as usize] >= TECHS[t as usize].units {
            Err(LabStatus::AllTaken)
        } else if !self.has_packs(t) {
            Err(LabStatus::NoPacks)
        } else {
            Ok(t)
        }
    }

    fn has_packs(&self, tech: u8) -> bool {
        TECHS[tech as usize].packs.iter().all(|&p| pack_slot(p).is_some_and(|i| !self.packs.slots[i].is_empty()))
    }

    /// The first readout line, also the panel's status.
    pub fn status_text(&self, research: &Research) -> String {
        let name = |t: Option<u8>| t.map_or("", |t| TECHS[t as usize].name);
        match self.status {
            LabStatus::NoResearch => "No research chosen: pick one on the research screen (R)".to_string(),
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
            LabStatus::NoPower => {
                format!("No power: needs a power pole within {POLE_REACH} blocks, linked to a generator")
            }
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
    }

    fn read_state(r: &mut ByteReader) -> Option<Lab> {
        let mut l = Lab::new(r.ivec3()?);
        l.packs = Buffer::read_state(r, PACKS.len())?;
        let slots_ok = l.packs.slots.iter().zip(PACKS).all(|(s, p)| s.is_empty() || s.item == p);
        let unit = r.u8()?;
        l.unit = (unit != u8::MAX).then_some(unit);
        l.progress = r.u32()?;
        (slots_ok && l.unit.is_none_or(|t| (t as usize) < TECHS.len())).then_some(l)
    }

    /// Its packs, plus those of an unfinished unit (given back rather than lost).
    fn contents(&self) -> Vec<Stack> {
        let mut all = self.packs.contents();
        if let Some(t) = self.unit {
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
        lines.push("Right-click to open".to_string());
        lines.join("\n")
    }

    /// A white cabinet with a glass dome that glows while it works, and a status lamp.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let body = [tex::LAB_TOP, tex::LAB_SIDE, tex::FRAME];
        push_box(out, rel + Vec3::new(0.0, -0.12, 0.0), 0.0, [0.92, 0.76, 0.92], 0.0, body, false);
        let working = self.status == LabStatus::Working;
        let spin = if working { (time * 1.5) as f32 } else { 0.0 };
        let dome = [tex::LAB_TOP; 3];
        push_box(out, rel + Vec3::new(0.0, 0.34, 0.0), spin, [0.5, 0.16, 0.5], 0.0, dome, false);
        let lamp = match self.status {
            LabStatus::Working => tex::LAMP_GREEN,
            LabStatus::NoPacks | LabStatus::AllTaken => tex::LAMP_YELLOW,
            LabStatus::NoResearch | LabStatus::NoPower => tex::LAMP_RED,
        };
        push_box(out, rel + Vec3::new(0.32, 0.34, 0.32), 0.0, [0.14, 0.1, 0.14], 0.0, [lamp; 3], false);
    }
}
