//! Read-only readings of the factory for the analytics (`analytics/`): what the power grids could give, gave and
//! were asked for, and how well each working machine runs right now. Nothing here changes core state.
//!
//! A machine's [`Sample`] is the share of its full pace it ran at, in thousandths, and, when that falls short, why.
//! `None` means it has nothing to do (no recipe, no research, paused): that is not a loss.
//! To cover a new machine kind: one loop in `Factory::samples`.

use crate::math::IVec3;

use super::lab::{Lab, LabStatus};
use super::miner::{Miner, MinerStatus};
use super::power::FULL_SPEED;
use super::process::{Pick, Processor, Status};
use super::quarry::{Quarry, QuarryStatus};
use super::Factory;

/// Why a machine runs below its full pace.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cause {
    Power,
    Input,
    Output,
    Fuel,
    Water,
    Deposit,
    Blocked,
}

/// How many [`Cause`]s there are (the array size for per-cause figures).
pub const CAUSES: usize = 7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sample {
    /// Share of full pace, in thousandths.
    pub work: u32,
    /// Why `work` is short of full (`None` at full pace).
    pub short: Option<Cause>,
}

impl Sample {
    /// Running at `share` of full speed: short only because of the power (or nothing, at full).
    fn run(share: u32) -> Sample {
        let work = share.min(FULL_SPEED);
        Sample { work, short: (work < FULL_SPEED).then_some(Cause::Power) }
    }

    fn stalled(cause: Cause) -> Sample {
        Sample { work: 0, short: Some(cause) }
    }
}

/// Kilowatts over every grid: what fuelled generators could give, what was delivered, what machines asked for.
#[derive(Clone, Copy, Default, Debug)]
pub struct PowerTotals {
    pub capacity: u32,
    pub supply: u32,
    pub demand: u32,
}

impl Factory {
    /// The grids' totals from the last tick.
    pub fn power_totals(&self) -> PowerTotals {
        let sum = |v: &[u32]| v.iter().sum();
        PowerTotals {
            capacity: sum(&self.power.capacity),
            supply: sum(&self.power.supply),
            demand: sum(&self.power.demand),
        }
    }

    /// Calls `f` with the anchor and the latest [`Sample`] of every machine that does work (miners, quarries, labs,
    /// processors; not belts, boxes, power sources or docks).
    pub fn samples(&self, mut f: impl FnMut(IVec3, Option<Sample>)) {
        self.miners.iter().for_each(|m| f(m.pos, miner_sample(m)));
        self.quarries.iter().for_each(|q| f(q.pos, quarry_sample(q)));
        self.labs.iter().for_each(|l| f(l.pos, lab_sample(l)));
        for p in self.processors.iter().filter(|p| works(p)) {
            f(p.pos, process_sample(p));
        }
    }
}

/// Whether a processor is a worker (not a power source, a store, a hangar or a research center).
fn works(p: &Processor) -> bool {
    !p.energy().is_source() && !p.spec.pick.stores() && !matches!(p.spec.pick, Pick::Hangar | Pick::Research)
}

fn process_sample(p: &Processor) -> Option<Sample> {
    let share = if p.draws_power() { p.speed } else { FULL_SPEED };
    Some(match p.status {
        Status::Working => Sample::run(share),
        Status::NoRecipe => return None,
        Status::NoInput => Sample::stalled(Cause::Input),
        Status::OutputFull => Sample::stalled(Cause::Output),
        Status::NoPower => Sample::stalled(Cause::Power),
        Status::NoFuel => Sample::stalled(Cause::Fuel),
        Status::NoWater => Sample::stalled(Cause::Water),
        Status::NoDeposit | Status::Exhausted => Sample::stalled(Cause::Deposit),
        Status::Overheated => Sample::stalled(Cause::Blocked),
    })
}

/// A running miner's pace is what it actually draws against what it could at full power, so a thin or capped
/// deposit shows as a loss too (blamed on the power only when the grid is short).
fn miner_sample(m: &Miner) -> Option<Sample> {
    Some(match m.status {
        MinerStatus::Running => {
            let work = ((m.draw_rate / m.rate() * FULL_SPEED as f64) as u32).min(FULL_SPEED);
            let short = (work < FULL_SPEED).then_some(if m.speed < FULL_SPEED { Cause::Power } else { Cause::Deposit });
            Sample { work, short }
        }
        MinerStatus::OutputFull => Sample::stalled(Cause::Output),
        MinerStatus::NoDeposit | MinerStatus::Exhausted => Sample::stalled(Cause::Deposit),
        MinerStatus::NoPower => Sample::stalled(Cause::Power),
    })
}

fn quarry_sample(q: &Quarry) -> Option<Sample> {
    Some(match q.status {
        QuarryStatus::Digging => Sample::run(q.speed),
        QuarryStatus::OutputFull => Sample::stalled(Cause::Output),
        QuarryStatus::NoPower => Sample::stalled(Cause::Power),
        QuarryStatus::Flooded => Sample::stalled(Cause::Blocked),
        QuarryStatus::Paused | QuarryStatus::Done => return None,
    })
}

fn lab_sample(l: &Lab) -> Option<Sample> {
    Some(match l.status {
        LabStatus::Working => Sample::run(l.speed),
        LabStatus::NoPacks | LabStatus::NeedsCenter => Sample::stalled(Cause::Input),
        LabStatus::NoPower => Sample::stalled(Cause::Power),
        LabStatus::NoResearch | LabStatus::AllTaken => return None,
    })
}
