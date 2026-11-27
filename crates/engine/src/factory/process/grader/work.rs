//! One tick of the Site Grader's serpentine cut-fill-pave state machine (core state, deterministic).
//!
//! Strict conservation invariant: every non-leaf block excavated is accounted for 1:1 — either placed into a hollow
//! (`fill_count`), held by exact `BlockId` in `hopper`, or delivered into `out` (blocking with `Status::OutputFull`
//! rather than ever discarding a block). Once no hollows remain at the end of a pass, any leftover earth in `hopper`
//! automatically flushes into `out`.

use crate::block::{
    self, is_leaves, is_log, is_ore, is_turf, sapling_of, BlockId, AIR, BASALT, CLAY, DARK_SAND, DIRT, GRANITE,
    GREEN_SAND, KAOLIN, MACHINE_PART, PALE_SAND, RUSTY_SAND, SAND, SANDSTONE, SIGN, SLAG, SPENT_ROCK, STONE, TAILINGS,
};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::SimEvent;
use crate::world::World;

use super::super::super::buffer::Buffer;
use super::super::super::power::FULL_SPEED;
use super::super::super::{machine, ticks};
use super::super::forester::fell_tree;
use super::super::Status;
use super::{
    Grader, ACT_CUT, ACT_FELL, ACT_FILL, ACT_IDLE, ACT_PAVE, FELL_SECONDS, FILL_CAP, MAX_CUT_HEIGHT, MAX_FILL_DEPTH,
    REST_TICKS, WORK_SECONDS,
};

/// Maximum already-level columns scanned in one tick so finished areas are skipped briskly.
const SCAN_PER_TICK: usize = 12;
/// Minimum free log room required in `out` before felling a whole tree so zero trunk logs are ever lost.
const MIN_LOG_ROOM: u32 = 12;

/// Whether `item` can be selected as a floor material or fed into the grader's paving intake port.
pub(in crate::factory::process) fn is_paving_block(item: ItemId) -> bool {
    let Some(b) = item.places() else { return false };
    b != AIR
        && b != MACHINE_PART
        && b != SIGN
        && machine(b).is_none()
        && block::OPAQUE[b as usize]
        && block::SOLID[b as usize]
        && !is_log(b)
        && !is_leaves(b)
        && !is_deposit(b)
}

/// Whether `b` is a mineral deposit (ore or soft clay/kaolin bed) spared when `spare_ores` is enabled.
fn is_deposit(b: BlockId) -> bool {
    is_ore(b) || matches!(b, CLAY | KAOLIN)
}

/// Whether `b` is bulk natural earthworks material that can temporarily buffer in the typed cut-and-fill hopper.
fn bulk_fill_block(b: BlockId) -> Option<BlockId> {
    if is_turf(b) || b == DIRT {
        Some(DIRT)
    } else if matches!(b, SAND | RUSTY_SAND | DARK_SAND | GREEN_SAND | PALE_SAND) {
        Some(SAND)
    } else if matches!(b, STONE | GRANITE | SANDSTONE | BASALT | SPENT_ROCK | SLAG | TAILINGS) {
        Some(b)
    } else {
        None
    }
}

/// Whether the grader must never break `b` (air, water, bedrock, machines, signs, or spared ore deposits).
fn protected(b: BlockId, spare_ores: bool) -> bool {
    b == AIR
        || block::LIQUID[b as usize]
        || block::def(b).break_time < 0.0
        || b == MACHINE_PART
        || b == SIGN
        || machine(b).is_some()
        || (spare_ores && is_deposit(b))
}

pub(super) struct Site<'a> {
    pub pos: IVec3,
    pub dir: u8,
    pub world: &'a mut World,
    pub input: &'a mut Buffer,
    pub out: &'a mut Buffer,
    pub progress: &'a mut u32,
    pub made: &'a mut Vec<Stack>,
    pub used: &'a mut Vec<Stack>,
    pub changed: &'a mut Vec<(IVec3, BlockId)>,
    pub events: &'a mut Vec<SimEvent>,
    pub share: u32,
}

impl Site<'_> {
    fn block(&mut self, p: IVec3) -> BlockId {
        self.world.block_anywhere_or_generate(p)
    }

    /// Input slot holding the chosen `floor` block, if present.
    fn floor_input(&self, floor: BlockId) -> Option<(usize, ItemId)> {
        if floor == AIR {
            return None;
        }
        self.input
            .slots
            .iter()
            .enumerate()
            .find_map(|(i, s)| (!s.is_empty() && s.item.places() == Some(floor)).then_some((i, s.item)))
    }

    fn set_world(&mut self, p: IVec3, to: BlockId) -> BlockId {
        let was = self.block(p);
        if was != to {
            self.world.set_block_anywhere_later(p, to);
            self.changed.push((p, was));
        }
        was
    }
}

impl Grader {
    pub(super) fn tick(&mut self, s: &mut Site) -> Status {
        // When resting after a completed sweep, automatically flush any leftover earth from `hopper` into `out`
        // so 100% of unused excavated dirt, stone, sand and sandstone exits cleanly onto the rear belt!
        if self.rest > 0 {
            self.active = ACT_IDLE;
            if self.hopper_total() > 0 {
                if s.share == 0 {
                    return Status::NoPower;
                }
                return match self.flush_one_hopper_block(s) {
                    Ok(()) => Status::Working,
                    Err(status) => status,
                };
            }
            self.rest -= 1;
            if self.rest == 0 {
                self.col = 0;
                self.pass_worked = false;
                self.needs_floor = false;
            }
            return Status::NoInput;
        }
        if s.share == 0 {
            return Status::NoPower;
        }
        let grade_y = s.pos.y - 1;

        // If we already locked onto a cell, advance work and commit on completion.
        if self.active != ACT_IDLE {
            let need_sec = if self.active == ACT_FELL { FELL_SECONDS } else { WORK_SECONDS };
            let goal = ticks(need_sec) * FULL_SPEED;
            if *s.progress + s.share < goal {
                *s.progress += s.share;
                return Status::Working;
            }
            match self.commit(s, grade_y) {
                Ok(()) => {
                    *s.progress = 0;
                    self.pass_worked = true;
                    return self.acquire(s, grade_y);
                }
                Err(status) => {
                    *s.progress = goal;
                    return status;
                }
            }
        }

        self.acquire(s, grade_y)
    }

    /// Moves one leftover block from the internal cut-and-fill hopper into `out` without losing a single block.
    fn flush_one_hopper_block(&mut self, s: &mut Site) -> Result<(), Status> {
        let Some(&(b, _)) = self.hopper.last() else {
            return Ok(());
        };
        let item = ItemId::block(b);
        if item.is_valid() {
            if s.out.space_for(item) == 0 {
                return Err(Status::OutputFull);
            }
            self.hopper_pop(Some(b));
            let added = s.out.add(item, 1);
            if added > 0 {
                s.made.push(Stack { item, count: added });
            }
        } else {
            self.hopper_pop(Some(b));
        }
        Ok(())
    }

    /// Scans from `self.col` for the next cut, fell, fill or pave target.
    fn acquire(&mut self, s: &mut Site, grade_y: i32) -> Status {
        let total = self.columns();
        for _ in 0..SCAN_PER_TICK {
            let col_base = self.column_pos(s.pos, s.dir, self.col);
            let (found, missing_floor) = self.inspect_column(s, col_base.x, col_base.z, grade_y);
            if missing_floor {
                self.needs_floor = true;
            }
            if let Some((act, target)) = found {
                if self.cur != IVec3::ZERO && self.cur != target {
                    self.prev = self.cur;
                } else if self.prev == IVec3::ZERO {
                    self.prev = target;
                }
                self.cur = target;
                self.active = act;
                return Status::Working;
            }
            self.col += 1;
            if self.col >= total {
                self.col = 0;
                if !self.pass_worked {
                    self.active = ACT_IDLE;
                    self.rest = REST_TICKS;
                    return if self.hopper_total() > 0 { Status::Working } else { Status::NoInput };
                }
                self.pass_worked = false;
                self.needs_floor = false;
            }
        }
        self.active = ACT_IDLE;
        Status::Working
    }

    /// Whether the chosen `floor` block is currently available (from the left intake buffer OR from matching natural
    /// earth already sitting in the cut-and-fill hopper).
    fn has_floor_material(&self, s: &Site) -> bool {
        if self.floor == AIR {
            return false;
        }
        s.floor_input(self.floor).is_some() || self.hopper.iter().any(|&(b, n)| b == self.floor && n > 0)
    }

    /// Determines the highest-priority edit needed in column `(x, z)` for target surface `grade_y`.
    /// Returns `(Some((action, pos)), missing_floor)`.
    fn inspect_column(&self, s: &mut Site, x: i32, z: i32, grade_y: i32) -> (Option<(u8, IVec3)>, bool) {
        // 1. Cut anything above `grade_y`, top-down (skipping protected blocks and spared ore deposits).
        for y in ((grade_y + 1)..=(grade_y + MAX_CUT_HEIGHT)).rev() {
            let p = IVec3::new(x, y, z);
            let b = s.block(p);
            if protected(b, self.spare_ores) {
                continue;
            }
            if is_log(b) {
                let mut foot_y = y;
                while foot_y > grade_y + 1 && is_log(s.block(IVec3::new(x, foot_y - 1, z))) {
                    foot_y -= 1;
                }
                return (Some((ACT_FELL, IVec3::new(x, foot_y, z))), false);
            }
            return (Some((ACT_CUT, p)), false);
        }

        let has_hopper = self.hopper_total() > 0;
        let has_floor = self.has_floor_material(s);

        // 2. Fill hollows / water strictly below `grade_y`, bottom-up.
        if has_hopper || s.floor_input(self.floor).is_some() {
            let bottom = grade_y - MAX_FILL_DEPTH;
            let mut floor_y = bottom;
            for y in (bottom..grade_y).rev() {
                if !block::replaceable(s.block(IVec3::new(x, y, z))) {
                    floor_y = y + 1;
                    break;
                }
            }
            if floor_y < grade_y && block::replaceable(s.block(IVec3::new(x, floor_y, z))) {
                return (Some((ACT_FILL, IVec3::new(x, floor_y, z))), false);
            }
        }

        // 3. Grade surface layer `y == grade_y`.
        let surf = IVec3::new(x, grade_y, z);
        let sb = s.block(surf);
        if block::replaceable(sb) {
            if self.floor != AIR {
                if has_floor {
                    return (Some((ACT_PAVE, surf)), false);
                }
                // Fill the empty gap with natural earth if we have it, and flag that we still want `floor`.
                if has_hopper {
                    return (Some((ACT_FILL, surf)), true);
                }
                return (None, true);
            }
            if has_hopper {
                return (Some((ACT_FILL, surf)), false);
            }
            return (None, false);
        }
        if self.floor != AIR && sb != self.floor && !protected(sb, self.spare_ores) {
            if has_floor {
                return (Some((ACT_PAVE, surf)), false);
            }
            return (None, true);
        }
        (None, false)
    }

    /// Commits the finished action on `self.cur`, returning `Err(Status::OutputFull)` if the output buffer is full.
    fn commit(&mut self, s: &mut Site, grade_y: i32) -> Result<(), Status> {
        let target = self.cur;
        match self.active {
            ACT_FELL => {
                let b = s.block(target);
                if !is_log(b) {
                    return Ok(());
                }
                let log_item = crate::wood::item(crate::wood::of_tree(b).unwrap_or(0), crate::wood::Form::Log);
                // Ensure room for the entire trunk before felling so zero logs are ever truncated.
                if s.out.space_for(log_item) < MIN_LOG_ROOM {
                    return Err(Status::OutputFull);
                }
                let felled = fell_tree(s.world, target, s.changed, s.events);
                if felled.logs > 0 {
                    let count = u32::from(felled.logs);
                    let added = s.out.add(log_item, count);
                    if added > 0 {
                        s.made.push(Stack { item: log_item, count: added });
                    }
                    self.trees_felled += 1;
                    self.cut_count += count;
                }
                if felled.leaves > 0 {
                    let sap = ItemId::block(sapling_of(felled.leaf));
                    if s.out.space_for(sap) > 0 {
                        let added = s.out.add(sap, 1);
                        if added > 0 {
                            s.made.push(Stack { item: sap, count: added });
                        }
                    }
                }
                Ok(())
            }
            ACT_CUT => {
                let b = s.block(target);
                if protected(b, self.spare_ores) {
                    return Ok(());
                }
                self.harvest_block(s, b)?;
                s.set_world(target, AIR);
                s.events.push(SimEvent::QuarryDug { pos: target, block: b });
                self.cut_count += 1;
                Ok(())
            }
            ACT_FILL => {
                let b = s.block(target);
                if !block::replaceable(b) {
                    return Ok(());
                }
                let place = if let Some(h) = self.hopper_pop(None) {
                    h
                } else if let Some((slot, item)) = s.floor_input(self.floor) {
                    s.input.take(slot, 1);
                    s.used.push(Stack { item, count: 1 });
                    self.floor
                } else {
                    return Ok(());
                };
                s.set_world(target, place);
                self.fill_count += 1;
                Ok(())
            }
            ACT_PAVE => {
                let pave_block = self.floor;
                if pave_block == AIR {
                    return Ok(());
                }
                let b = s.block(target);
                if b == pave_block {
                    return Ok(());
                }
                // Check availability BEFORE harvesting the underlying block so we never remove without replacing.
                let from_input = s.floor_input(pave_block);
                let from_hopper = from_input.is_none() && self.hopper.iter().any(|&(hb, n)| hb == pave_block && n > 0);
                if from_input.is_none() && !from_hopper {
                    return Ok(());
                }
                if !block::replaceable(b) {
                    if protected(b, self.spare_ores) || target.y != grade_y {
                        return Ok(());
                    }
                    self.harvest_block(s, b)?;
                    s.events.push(SimEvent::QuarryDug { pos: target, block: b });
                    self.cut_count += 1;
                }
                if let Some((slot, item)) = from_input {
                    s.input.take(slot, 1);
                    s.used.push(Stack { item, count: 1 });
                } else {
                    self.hopper_pop(Some(pave_block));
                }
                s.set_world(target, pave_block);
                self.paved_count += 1;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Routes an excavated block either into the typed cut-and-fill hopper (up to `FILL_CAP` bulk natural earth) or
    /// straight to the rear output belt (ores, coal, clay, excess earth). Never drops or loses a block.
    fn harvest_block(&mut self, s: &mut Site, b: BlockId) -> Result<(), Status> {
        if is_leaves(b) {
            return Ok(());
        }
        let drop_block = block::def(b).drop;
        if drop_block == AIR {
            return Ok(());
        }
        if !is_deposit(b) {
            if let Some(fill_kind) = bulk_fill_block(b) {
                if self.hopper_total() < FILL_CAP {
                    self.hopper_push(fill_kind);
                    return Ok(());
                }
            }
        }
        let drop_item = ItemId::block(drop_block);
        if !drop_item.is_valid() {
            return Ok(());
        }
        if s.out.space_for(drop_item) == 0 {
            return Err(Status::OutputFull);
        }
        let added = s.out.add(drop_item, 1);
        if added > 0 {
            s.made.push(Stack { item: drop_item, count: added });
        }
        Ok(())
    }
}
