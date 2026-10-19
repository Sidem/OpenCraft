//! The research center (Milestone 10): a 2×2×2 lab. It researches the world's chosen tech like a lab (`lab.rs`, same
//! rules: a unit takes one of each of the tech's packs when it starts, then `seconds` of work; never more units than
//! a tech has left; a unit in progress finishes for the tech it started on) but holds eight kinds of pack, works at
//! twice a lab's speed per tier and takes belts on every side. A small lab has `LAB_PACK_SLOTS` slots, so a tech that
//! uses a pack beyond them (`needs_center`) can be researched only here.
//!
//! - The input buffer has `CENTER_SLOTS` slots; each pack kind keeps one stack (`center_room`), so one pack can't
//!   crowd out another. `step_labs` (`lab.rs`) calls `step_centers` after the labs, sharing their unit counts.
//! - State (`Study`): the unit in progress, its work, paid units since the last free one and whether this one is
//!   free are saved for centers only; the status is last tick's (derived).
//!
//! To add a pack: an entry in `PACKS` (`research.rs`); the first four slots stay the lab's.

use crate::block::{tex, RESEARCH_CENTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, stack_size, ItemId};
use crate::research::{is_bonus, needs_ai_lab, pack_slot, Research, TECHS};

use super::super::buffer::Buffer;
use super::super::fibre::Data;
use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::lab::{LabStatus, LAB_PACK_SLOTS};
use super::super::power::{Power, FULL_SPEED, NOT_WIRED};
use super::super::ticks;
use super::model::{part, Look, Part};
use super::{Energy, Extra, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// Pack kinds a center holds (a small lab holds the first `LAB_PACK_SLOTS` of `PACKS`).
pub const CENTER_SLOTS: usize = 8;
/// Every this many units the last one takes no packs, by tier (0: never): the lab's rule.
const FREE_EVERY: [u8; 5] = [0, 0, 5, 3, 2];

/// A center's research state; other processors keep the default.
#[derive(Default)]
pub struct Study {
    /// The tech of the unit in progress, and the work it has had (thousandths of a full-power tick).
    pub unit: Option<u8>,
    pub progress: u32,
    /// Paid units since the last free one, and whether the unit in progress is free.
    pub since_free: u8,
    pub free: bool,
    /// Last tick's status and, when it waits for packs, the tech (derived, for the readout).
    pub status: Option<LabStatus>,
    pub waiting: Option<u8>,
}

const fn inlet(side: Side) -> Port {
    Port { side, role: Role::In, cell: Which::All }
}

pub const CENTER_SPEC: ProcessSpec = ProcessSpec {
    block: RESEARCH_CENTER,
    categories: &[],
    pick: Pick::Research,
    buffers: [CENTER_SLOTS, 0, 0],
    side: 0,
    tiers: &[
        ProcessTier { energy: Energy::Electric, speed: 2000, fuel: 0, power: 20 },
        ProcessTier { energy: Energy::Electric, speed: 4000, fuel: 0, power: 40 },
        ProcessTier { energy: Energy::Electric, speed: 6000, fuel: 0, power: 60 },
        ProcessTier { energy: Energy::Electric, speed: 8000, fuel: 0, power: 80 },
        ProcessTier { energy: Energy::Electric, speed: 10000, fuel: 0, power: 100 },
    ],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[inlet(Side::Back), inlet(Side::Front), inlet(Side::Left), inlet(Side::Right)],
    },
    verb: "Researching",
    products: "research",
    waiting: "",
    map_colour: 0xe8e8f0,
    compute: 0,
    parts: &CENTER_PARTS,
};

const BODY: [u16; 3] = [tex::LAB_TOP, tex::LAB_SIDE, tex::FRAME];

/// Across and deep 2, 2 high (±1): a banded plinth, a white cabinet, a glass dome with two frame ribs, a roof plate, a
/// copper mast and a status lamp.
const CENTER_PARTS: [Part; 7] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.45, 0.0], [1.9, 0.7, 1.9], Look::Tex(BODY)),
    part([0.0, 0.2, 0.0], [1.4, 0.6, 1.4], Look::Tex([tex::FLASK_GLASS; 3])),
    part([0.0, 0.2, 0.0], [0.16, 0.64, 1.5], Look::Tex([tex::FRAME; 3])),
    part([0.0, 0.62, 0.0], [1.6, 0.12, 1.6], Look::Tex(BODY)),
    part([-0.7, 0.95, -0.7], [0.14, 0.7, 0.14], Look::Tex([tex::COPPER_INGOT; 3])),
    part([0.78, -0.3, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];

/// Whether `tech` can be researched only in a center: it uses a pack a small lab has no slot for.
pub fn needs_center(tech: u8) -> bool {
    TECHS[tech as usize].packs.iter().any(|&p| pack_slot(p).is_some_and(|i| i >= LAB_PACK_SLOTS))
}

impl Processor {
    /// Whether it is a research center.
    pub fn is_center(&self) -> bool {
        self.spec.pick == Pick::Research
    }

    /// How many of `item` a center takes: science packs, up to a stack of each kind.
    pub(super) fn center_room(&self, item: ItemId) -> u32 {
        if pack_slot(item).is_none() {
            return 0;
        }
        stack_size(item).saturating_sub(self.input.count(item))
    }

    /// Whether it would work this tick if powered.
    pub(super) fn center_wants_power(&self, research: &Research) -> bool {
        self.study_unit().is_some()
            || research.current.is_some_and(|t| has_packs(&self.input, t) && (self.is_ai_lab() || !needs_ai_lab(t)))
    }

    /// The tech of a center's unit in progress (`None`: none, or another processor).
    pub(in crate::factory) fn study_unit(&self) -> Option<u8> {
        match &self.extra {
            Extra::Study(study) => study.unit,
            _ => None,
        }
    }

    /// One tick of research at `speed` (thousandths: its grid's share).
    fn study_step(&mut self, research: &mut Research, taken: &mut [u32], speed: u32) {
        self.speed = speed;
        let (ai, every, rate) = (self.is_ai_lab(), self.free_every(&FREE_EVERY), self.stats().speed);
        let Extra::Study(study) = &mut self.extra else { return };
        let tech = match study.unit {
            Some(t) => t,
            None => {
                let started = match study.can_start(&self.input, ai, every, research, taken) {
                    Ok(_) if speed == 0 => Err(LabStatus::NoPower),
                    other => other,
                };
                let Ok(t) = started else {
                    self.status = study.set_status(started.err());
                    return;
                };
                study.free = study.free_next(every);
                study.since_free = if study.free { 0 } else { study.since_free.saturating_add(1) };
                for &p in TECHS[t as usize].packs.iter().filter(|_| !study.free) {
                    self.input.remove(p, 1);
                    self.used.push(Stack { item: p, count: 1 });
                }
                taken[t as usize] += 1;
                (study.unit, study.progress) = (Some(t), 0);
                t
            }
        };
        if speed == 0 {
            self.status = study.set_status(Some(LabStatus::NoPower));
            return;
        }
        self.status = study.set_status(Some(LabStatus::Working));
        study.progress += speed * rate / 1000;
        if study.progress >= ticks(research.unit_seconds(tech)) * FULL_SPEED {
            research.add_unit(tech);
            taken[tech as usize] -= 1;
            (study.unit, study.progress, study.free) = (None, 0, false);
        }
    }

    /// The status line of a center (`None`: another processor).
    pub(super) fn center_text(&self) -> Option<String> {
        let Extra::Study(study) = &self.extra else { return None };
        let name = |t: Option<u8>| t.map_or("", |t| TECHS[t as usize].name);
        Some(match study.status {
            None | Some(LabStatus::NoResearch) => "No research chosen: pick one on the research screen (T)".to_string(),
            Some(LabStatus::Working) => {
                let slow =
                    if self.speed < FULL_SPEED { format!(" (low power: {}%)", self.speed / 10) } else { String::new() };
                format!("Researching {}{slow}", name(study.unit))
            }
            Some(LabStatus::NoPacks) => {
                let packs = study.waiting.map_or(&[][..], |t| TECHS[t as usize].packs);
                let names: Vec<&str> = packs.iter().map(|&p| item::name(p)).collect();
                format!("Waiting for {}", names.join(" and "))
            }
            Some(LabStatus::AllTaken | LabStatus::NeedsCenter) => {
                "Other labs are finishing the chosen research".to_string()
            }
            Some(LabStatus::NeedsAi) => format!("{} needs an AI lab", name(study.waiting)),
            Some(LabStatus::NoPower) if self.is_ai_lab() => {
                "No power or no compute: hang it on a pole and a fibre node whose grid has a datacenter".to_string()
            }
            Some(LabStatus::NoPower) => NOT_WIRED.to_string(),
        })
    }

    /// The readout's tech line for a center: its progress, speed and power (empty for another processor).
    pub(super) fn center_line(&self, research: &Research) -> String {
        if !self.is_center() {
            return String::new();
        }
        let mut out = String::new();
        if let Some(t) = self.study_unit().or(research.current) {
            let tech = &TECHS[t as usize];
            out += &if is_bonus(t) {
                format!("\n{}: level {}", tech.name, research.progress(t))
            } else {
                format!("\n{}: {} of {} units", tech.name, research.progress(t), tech.units)
            };
        }
        let free = match self.free_every(&FREE_EVERY) {
            0 => String::new(),
            n => format!(" · every {n}th unit is free"),
        };
        out + &format!("\n×{} speed · needs {} kW{free}", self.stats().speed / 1000, self.power())
    }

    /// An unfinished unit's packs, given back with the rest rather than lost.
    pub(super) fn study_contents(&self) -> Vec<Stack> {
        let Extra::Study(Study { unit: Some(t), free: false, .. }) = self.extra else { return Vec::new() };
        TECHS[t as usize].packs.iter().map(|&item| Stack { item, count: 1 }).collect()
    }
}

impl Study {
    /// Whether the next unit is free (`every`: the tier's `FREE_EVERY`).
    fn free_next(&self, every: u8) -> bool {
        every > 0 && self.since_free + 1 >= every
    }

    /// The tech a new unit would be for, or why none can start (`ai`: whether the center is an AI lab).
    fn can_start(
        &mut self,
        input: &Buffer,
        ai: bool,
        every: u8,
        research: &Research,
        taken: &[u32],
    ) -> Result<u8, LabStatus> {
        self.waiting = None;
        let t = research.current.ok_or(LabStatus::NoResearch)?;
        if needs_ai_lab(t) && !ai {
            self.waiting = Some(t);
            Err(LabStatus::NeedsAi)
        } else if research.progress(t) + taken[t as usize] >= TECHS[t as usize].units {
            Err(LabStatus::AllTaken)
        } else if !self.free_next(every) && !has_packs(input, t) {
            self.waiting = Some(t);
            Err(LabStatus::NoPacks)
        } else {
            Ok(t)
        }
    }

    /// Records last tick's status for the readout, and returns the `Status` the model's lamp reads.
    fn set_status(&mut self, why: Option<LabStatus>) -> Status {
        self.status = why;
        match why {
            Some(LabStatus::Working) => Status::Working,
            Some(LabStatus::NoPower) => Status::NoPower,
            Some(LabStatus::NoResearch) => Status::NoRecipe,
            _ => Status::NoInput,
        }
    }

    pub(super) fn write_state(&self, w: &mut ByteWriter) {
        w.u8(self.unit.unwrap_or(u8::MAX));
        w.u32(self.progress);
        w.u8(self.since_free);
        w.bool(self.free);
    }

    pub(super) fn read_state(&mut self, r: &mut ByteReader) -> Option<()> {
        let unit = r.u8()?;
        self.unit = (unit != u8::MAX).then_some(unit).filter(|&t| (t as usize) < TECHS.len());
        self.progress = r.u32()?;
        (self.since_free, self.free) = (r.u8()?, r.bool()?);
        Some(())
    }
}

/// Whether `input` holds one of each of `tech`'s packs.
fn has_packs(input: &Buffer, tech: u8) -> bool {
    TECHS[tech as usize].packs.iter().all(|&p| input.count(p) > 0)
}

/// One tick of every research center, each at its grid's speed (`taken`: units each tech has in progress).
pub(in crate::factory) fn step_centers(
    (processors, centers): (&mut [Processor], &[u32]),
    pole: &[Option<u32>],
    power: &Power,
    data: &Data,
    research: &mut Research,
    taken: &mut [u32],
) {
    for &i in centers {
        let (i, c) = (i as usize, &mut processors[i as usize]);
        let mut speed = power.speed(pole[i]);
        if c.spec.compute < 0 {
            speed = speed * data.satisfaction(data.process_node[i]) / FULL_SPEED;
        }
        c.study_step(research, taken, speed);
    }
}

#[cfg(test)]
mod tests;
