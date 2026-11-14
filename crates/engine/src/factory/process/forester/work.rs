//! One tick of a forester's crane: the job steps of `forester.rs`'s header, over the world and its own buffers. Each
//! timed step adds this tick's work (`Crane::done`) and moves on when its seconds are done; a step that needs power
//! says `NoPower` without it. The jobs' order and meaning are in the header.

use crate::block::{is_log, is_sapling, sapling_of, BlockId, AIR, DIRT, GRASS};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::{SimEvent, SAPLING_CHANCE};
use crate::world::World;

use super::super::super::buffer::Buffer;
use super::super::super::power::FULL_SPEED;
use super::super::super::ticks;
use super::super::Status;
use super::fell::fell;
use super::patch::survey;
use super::{Forest, Job, Patch, Spot, HOME};
use super::{
    BUCK_SECONDS, COLUMNS_PER_TICK, CUT_SECONDS, FELL_SECONDS, GATHER_SECONDS, GRIP_SECONDS, HAUL_SECONDS,
    PLANT_SECONDS, REACH_SECONDS, REST_SECONDS, UNLOAD_SECONDS,
};

/// What one tick of the crane works with.
pub(super) struct Crane<'a> {
    pub pos: IVec3,
    pub dir: u8,
    pub tier: u8,
    pub world: &'a mut World,
    pub input: &'a mut Buffer,
    pub out: &'a mut Buffer,
    pub progress: &'a mut u32,
    pub made: &'a mut Vec<Stack>,
    pub changed: &'a mut Vec<(IVec3, BlockId)>,
    pub events: &'a mut Vec<SimEvent>,
    /// This tick's work: the grid's share times the tier's speed.
    pub share: u32,
}

impl Crane<'_> {
    fn block(&mut self, p: IVec3) -> BlockId {
        self.world.block_anywhere_or_generate(p)
    }

    /// Adds this tick's work; whether a task of `seconds` is done (its work then starts again from 0).
    fn done(&mut self, seconds: f64) -> bool {
        *self.progress += self.share;
        if *self.progress < ticks(seconds) * FULL_SPEED {
            return false;
        }
        *self.progress = 0;
        true
    }

    /// The input slot holding a sapling, if one does.
    fn sapling_slot(&self) -> Option<usize> {
        self.input.slots.iter().position(|s| !s.is_empty() && s.item.places().is_some_and(is_sapling))
    }

    /// Whether the column at `spot` is a plot of `patch` it can plant now: air over dirt or grass, and a sapling in hand.
    fn can_plant(&mut self, patch: Patch, spot: Spot) -> bool {
        let foot = foot(patch, spot);
        is_plot(patch, spot)
            && self.sapling_slot().is_some()
            && self.block(foot) == AIR
            && matches!(self.block(foot - IVec3::new(0, 1, 0)), DIRT | GRASS)
    }
}

/// The foot cell of the column at `spot`.
fn foot(patch: Patch, (ahead, side): Spot) -> IVec3 {
    patch.cell(i32::from(ahead), i32::from(side), patch.pos.y)
}

fn is_plot(patch: Patch, (ahead, side): Spot) -> bool {
    patch.is_plot_at(i32::from(ahead), i32::from(side))
}

fn spot_of(patch: Patch, c: u16) -> Spot {
    let (ahead, side) = patch.place(c);
    (ahead as i8, side as i8)
}

impl Forest {
    /// One tick of the crane; the processor's status.
    pub(super) fn tick(&mut self, c: &mut Crane) -> Status {
        let patch = self.patch(c.pos, c.dir);
        if self.job.moves() && c.share == 0 && !(self.job == Job::Unload && !self.room(c)) {
            return Status::NoPower;
        }
        match self.job {
            Job::Rest | Job::Blocked => {
                self.step = self.step.saturating_sub(1);
                if self.step == 0 {
                    // Both start with the look at column 0, so the ground is surveyed again first.
                    self.job = Job::Look;
                }
                Status::NoInput
            }
            Job::Look => self.look(c),
            Job::Reach => self.after(c, REACH_SECONDS, |f, _| (f.rest, f.job) = (f.tree, Job::Grip)),
            Job::Grip => {
                if !is_log(c.block(foot(patch, self.tree))) {
                    self.job = Job::Look;
                    return Status::NoInput;
                }
                self.after(c, GRIP_SECONDS, |f, _| f.job = Job::Cut)
            }
            Job::Cut => self.after(c, CUT_SECONDS, |f, c| f.cut_down(c, patch)),
            Job::Fell => self.after(c, FELL_SECONDS, |f, _| (f.job, f.cut) = (Job::Buck, 0)),
            Job::Buck => self.after(c, BUCK_SECONDS, |f, _| {
                f.cut += 1;
                if f.cut >= f.stem {
                    f.job = Job::Gather;
                }
            }),
            Job::Gather => self.after(c, GATHER_SECONDS, |f, _| (f.job, f.from) = (Job::Haul, f.tree)),
            Job::Haul => self.after(c, HAUL_SECONDS, |f, _| (f.rest, f.job) = (HOME, Job::Unload)),
            Job::Unload => {
                if !self.room(c) {
                    return Status::OutputFull;
                }
                self.after(c, UNLOAD_SECONDS, |f, c| f.unload(c, patch))
            }
            Job::Fetch => self.after(c, REACH_SECONDS, |f, _| (f.rest, f.job) = (f.tree, Job::Plant)),
            Job::Plant => {
                if !c.can_plant(patch, self.tree) {
                    self.job = Job::Look;
                    return Status::NoInput;
                }
                self.after(c, PLANT_SECONDS, |f, c| f.plant(c, patch))
            }
        }
    }

    /// Works on a step of `seconds`, then `next`.
    fn after(&mut self, c: &mut Crane, seconds: f64, next: impl FnOnce(&mut Forest, &mut Crane)) -> Status {
        if c.done(seconds) {
            next(self, c);
        }
        Status::Working
    }

    /// The log item its tree gives: its wood's (`wood.rs`).
    fn logs(&self) -> ItemId {
        crate::wood::item(crate::wood::of_tree(self.log).unwrap_or(0), crate::wood::Form::Log)
    }

    /// Whether its output has room for the logs it holds.
    fn room(&self, c: &Crane) -> bool {
        c.out.can_hold(self.logs(), u32::from(self.cut))
    }

    /// Looks over the next columns for a tree or a plot to plant, surveying the ground at the start of each round;
    /// rests after a round with neither.
    fn look(&mut self, c: &mut Crane) -> Status {
        if self.col == 0 {
            let world = &mut *c.world;
            self.ground = survey(c.pos, c.dir, c.tier, |p| world.block_anywhere_or_generate(p));
            if self.ground.size.is_none() {
                (self.job, self.step) = (Job::Blocked, ticks(REST_SECONDS) as u16);
                return Status::NoInput;
            }
        }
        let patch = self.patch(c.pos, c.dir);
        for _ in 0..COLUMNS_PER_TICK {
            let col = self.col;
            self.col = (self.col + 1) % patch.columns();
            let spot = spot_of(patch, col);
            if is_log(c.block(patch.column(col))) {
                (self.job, self.tree, self.from, self.busy) = (Job::Reach, spot, self.rest, true);
                return Status::Working;
            }
            if c.can_plant(patch, spot) {
                (self.job, self.tree, self.from, self.busy) = (Job::Fetch, spot, self.rest, true);
                return Status::Working;
            }
            if self.col == 0 {
                if !self.busy {
                    (self.job, self.step, self.from, self.rest) =
                        (Job::Rest, ticks(REST_SECONDS) as u16, self.rest, HOME);
                }
                self.busy = false;
                return Status::NoInput;
            }
        }
        Status::NoInput
    }

    /// The saw is through: the tree comes down into the head, its lone leaves with it (a sapling for every
    /// `1 / SAPLING_CHANCE` of them).
    fn cut_down(&mut self, c: &mut Crane, patch: Patch) {
        let felled = fell(c.world, foot(patch, self.tree), c.changed, c.events);
        if felled.logs == 0 {
            self.job = Job::Look;
            return;
        }
        (self.stem, self.cut, self.log, self.job) = (felled.logs, 0, felled.log, Job::Fell);
        let per = (1.0 / SAPLING_CHANCE).round() as u32;
        let leaves = u32::from(self.leaves) + felled.leaves;
        c.input.add(ItemId::block(sapling_of(felled.leaf)), leaves / per);
        self.leaves = (leaves % per) as u8;
    }

    /// The logs go onto its bunk (its output); then it plants the tree's plot, or looks on.
    fn unload(&mut self, c: &mut Crane, patch: Patch) {
        let drop = self.logs();
        c.out.add(drop, u32::from(self.cut));
        c.made.push(Stack { item: drop, count: u32::from(self.cut) });
        (self.felled, self.stem, self.cut) = (self.felled + 1, 0, 0);
        let replant = c.can_plant(patch, self.tree);
        (self.job, self.from) = if replant { (Job::Fetch, HOME) } else { (Job::Look, self.from) };
    }

    /// Plants a sapling on the plot; then looks on.
    fn plant(&mut self, c: &mut Crane, patch: Patch) {
        let slot = c.sapling_slot().expect("can_plant found one");
        let sapling = c.input.slots[slot].item.places().expect("a sapling is a block");
        c.input.take(slot, 1);
        let at = foot(patch, self.tree);
        c.world.set_block_anywhere(at, sapling);
        c.changed.push((at, AIR));
        self.job = Job::Look;
    }
}
