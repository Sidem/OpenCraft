//! Laying belts in lines, part of the local player's hands (`interaction.rs`). With a belt selected,
//! holding the use button on a surface starts a line at the cell in front of that face; the pointer
//! (up to `LINE_REACH` away) sets the far end; releasing builds it, a few cells per tick, as ordinary
//! `PlaceBlock` actions, so the core, co-op and saves see plain placements. A left click cancels.
//!
//! The path ([`plan`]) runs along the longer horizontal axis first and turns once. It follows the
//! ground one block up or down at a time; the factory turns those steps into ramps by itself
//! (`factory::belt_shape::derive_slopes`). It stops before anything in the way, so a line ending at a
//! machine feeds it. Planning reads the loaded world (the render cache), never core state. Where something
//! is in the way (a belt, a machine, a wall) the line dives under it with an underpass pair, if one can
//! cover the width (`pass.rs`: which underpass, what the inventory pays for, the red marks).
//!
//! With upgrade kits selected the same drag upgrades belts instead (`factory/upgrades.rs`): the path
//! ([`plan_upgrade`]) follows existing belts, the cells are the belts one tier below the kit, and
//! releasing sends `Upgrade` actions as far as the kits last. Shift-clicking a belt upgrades every belt
//! joined to it of the same tier (`Factory::belt_chain`). Clicking another tiered machine (a miner) with
//! kits upgrades it at once. Before the click, `upgrade_aim.rs` outlines what the kit would upgrade.
//!
//! Belts and kits are paid for from the whole inventory, not just the held stack: a line takes belts from
//! the held stack first, then from the other stacks (each placement names the slot it comes from).
//!
//! To change the path: `plan` / `plan_upgrade` (`path.rs`). The preview: ghost belts (`write_line_preview`) and the
//! host's outline boxes and label (`api/hud.rs` `line_cells`, `line_label`), in the kit's colour when
//! upgrading.

use crate::action::Action;
use crate::block::{self, BlockId, BELT, FAST_BELT, RAMP_DOWN, RAMP_UP};
use crate::factory::{self, tiers, upgrades, Shape};
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::raycast::raycast;
use crate::research::Unlock;
use crate::upgrade_aim::{Aim, MAX_CHAIN};
use crate::Game;

pub use path::{plan, plan_upgrade};

/// The longest line one drag builds, in cells.
pub const MAX_LINE: usize = 64;
/// How far away the pointer can set the line's end.
const LINE_REACH: f64 = 48.0;
/// Cells placed per tick while a line is built (fast, but the placing sounds don't all land at once).
const BUILD_PER_TICK: usize = 3;
const UP: IVec3 = IVec3::new(0, 1, 0);

/// What a planned cell is: a belt, or one end of a tunnel under an obstacle (`pass.rs`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    Belt,
    /// The underpass that dives in, with the blocks it passes under.
    Entry(u8),
    Exit,
}

/// One cell of a planned line: where, which way it runs, and how it will slope. An underpass has its
/// `tier` (which one of the family) and is `short` when the inventory has no pair of it to place.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineCell {
    pub pos: IVec3,
    pub dir: u8,
    pub shape: Shape,
    pub piece: Piece,
    pub tier: u8,
    pub short: bool,
}

impl LineCell {
    pub fn belt(pos: IVec3, dir: u8, shape: Shape) -> LineCell {
        LineCell { pos, dir, shape, piece: Piece::Belt, tier: 0, short: false }
    }
}

/// One placement (or upgrade) of a line being built: the cell, the way it runs, the item it uses and the
/// inventory slot that item comes from.
#[derive(Clone, Copy)]
struct Build {
    pos: IVec3,
    dir: u8,
    item: ItemId,
    slot: u8,
}

/// A line being dragged out, or being built.
#[derive(Default)]
pub struct BeltLine {
    /// The cell the drag started in, and the direction a single click lays a belt.
    start: Option<(IVec3, u8)>,
    pub cells: Vec<LineCell>,
    /// Placements still to send (the next one last), and the item their slots must hold.
    building: Vec<Build>,
    /// The kit an upgrade line spends.
    build_item: ItemId,
    /// Where the line was stopped by something too wide to go under (red in the preview).
    pub blocked: Option<IVec3>,
    /// The cells being built are upgrades (kits), not placements.
    upgrading: bool,
    /// What a held kit would upgrade at the crosshair (`upgrade_aim.rs`); empty while dragging.
    pub aim: Aim,
}

/// Whether block `b` is a belt (`FAST_BELT` is a legacy block of old worlds).
fn is_belt(b: BlockId) -> bool {
    matches!(b, BELT | FAST_BELT | RAMP_UP | RAMP_DOWN)
}

impl Game {
    /// Runs the line tool for one tick. True while it owns the use button (a belt is selected and the
    /// button went down on a surface, or a line is being built).
    pub(crate) fn update_belt_line(&mut self) -> bool {
        self.line.aim = Aim::default();
        if self.line.start.is_none() {
            self.line.blocked = None;
        }
        if self.send_line_placements() {
            return true;
        }
        let stack = self.inventory().selected_stack();
        let kit = upgrades::kit_tier(stack.item).filter(|_| !stack.is_empty());
        if kit.is_none() && (!stack.item.places().is_some_and(is_belt) || stack.is_empty()) {
            self.line.start = None;
            self.line.cells.clear();
            return false;
        }
        if kit.is_some() && self.line.start.is_none() {
            self.line.aim = self.upgrade_aim(stack.item);
        }
        match (self.line.start, self.using) {
            (None, false) => false,
            (None, true) => match kit {
                Some(tier) => self.start_upgrade(tier),
                None => self.start_line(),
            },
            (Some(_), _) if self.mining => {
                self.cancel_belt_line();
                true
            }
            (Some((start, facing)), true) => {
                let end = self.line_end(start).unwrap_or_else(|| self.line.cells.last().map_or(start, |c| c.pos));
                self.line.cells = match kit {
                    Some(tier) => {
                        let belt_tier = |p| self.sim.factory.tiered_at(p).filter(|t| t.0 == BELT).map(|t| t.1);
                        plan_upgrade(belt_tier, start, end, tier - 1)
                    }
                    None => {
                        let (cells, blocked) = plan(|p| self.sim.world.get_block(p), start, end, facing);
                        self.line.blocked = blocked;
                        cells
                    }
                };
                self.assign_passes();
                true
            }
            (Some(_), false) => {
                self.line.start = None;
                let cells = std::mem::take(&mut self.line.cells);
                self.queue_build(&cells, kit.is_some());
                true
            }
        }
    }

    /// Queues `cells` to be built, as far as the inventory pays for (`line_budget`): belts placed from the
    /// held stack and then the other stacks (underpasses from their own stacks, left out when there is no
    /// pair), or, with `upgrading`, upgrades paid with kits.
    fn queue_build(&mut self, cells: &[LineCell], upgrading: bool) {
        let item = self.inventory().selected_stack().item;
        let mut queue = Vec::with_capacity(cells.len());
        if upgrading {
            let slot = self.inventory().selected as u8;
            let n = cells.len().min(self.line_budget());
            queue.extend(cells.iter().take(n).map(|c| Build { pos: c.pos, dir: c.dir, item, slot }));
        } else {
            let mut pools: Vec<(ItemId, std::vec::IntoIter<u8>)> = Vec::new();
            for (c, _) in cells.iter().zip(pass::affordable(cells, self.line_budget())).filter(|p| p.1) {
                let it = c.item(item);
                let at = pools.iter().position(|p| p.0 == it).unwrap_or_else(|| {
                    pools.push((it, self.belt_slots(it, cells.len()).into_iter()));
                    pools.len() - 1
                });
                if let Some(slot) = pools[at].1.next() {
                    queue.push(Build { pos: c.pos, dir: c.dir, item: it, slot });
                }
            }
        }
        queue.reverse();
        self.line.upgrading = upgrading;
        self.line.build_item = item;
        self.line.building = queue;
    }

    /// The slots `n` belts of `item` come from: the selected stack first, then the others in slot order.
    fn belt_slots(&self, item: ItemId, n: usize) -> Vec<u8> {
        let inv = self.inventory();
        let mut slots = Vec::with_capacity(n);
        for s in std::iter::once(inv.selected).chain((0..inv.capacity()).filter(|&s| s != inv.selected)) {
            if inv.slots[s].item == item {
                let take = (inv.slots[s].count as usize).min(n - slots.len());
                slots.resize(slots.len() + take, s as u8);
            }
        }
        slots
    }

    /// Drops the line being dragged (the pointer was freed, or a left click).
    pub(crate) fn cancel_belt_line(&mut self) {
        self.line.start = None;
        self.line.cells.clear();
    }

    /// Cells the inventory pays for (none while research locks the kit's upgrade): all the belts of the
    /// held kind it holds, or the belt upgrades all the held kits cover.
    pub(crate) fn line_budget(&self) -> usize {
        let held = self.inventory().selected_stack().item;
        match upgrades::kit_tier(held) {
            Some(tier) if !self.sim.factory.research.has(Unlock::Upgrade(BELT, tier)) => 0,
            Some(_) => {
                let per = tiers::family(BELT).map_or(1, |f| f.kits);
                (self.inventory().count(held) / per) as usize
            }
            None => self.inventory().count(held) as usize,
        }
    }

    /// With kits: start an upgrade line at the targeted belt, or upgrade another tiered machine at once.
    fn start_upgrade(&mut self, tier: u8) -> bool {
        let Some(hit) = self.target else { return false };
        match self.sim.factory.tiered_at(hit.block) {
            Some((BELT, t)) if t + 1 == tier && self.body().input.sprint => {
                // Shift: the whole line of belts joined to this one.
                let chain = self.sim.factory.belt_chain(hit.block, MAX_CHAIN);
                let cells: Vec<_> = chain.iter().map(|&pos| LineCell::belt(pos, 0, Shape::Flat)).collect();
                self.queue_build(&cells, true);
                self.using = false;
            }
            Some((BELT, t)) => {
                self.line.start = Some((hit.block, 0));
                let cell = LineCell::belt(hit.block, 0, Shape::Flat);
                self.line.cells = if t + 1 == tier { vec![cell] } else { Vec::new() };
            }
            Some(_) => {
                self.act(Action::Upgrade { pos: hit.block });
                self.using = false;
            }
            None => return false,
        }
        true
    }

    /// The use button went down: start a line at the cell in front of the targeted face. Boxes and
    /// machine panels keep their right-click (unless crouching).
    fn start_line(&mut self) -> bool {
        let Some(hit) = self.target else { return false };
        let machine = factory::machine(hit.id).is_some_and(|m| m.panel || m.slots > 0);
        if (machine && !self.body().input.crouch) || hit.normal == IVec3::ZERO {
            return false;
        }
        let start = hit.block + hit.normal;
        if self.sim.world.get_block(start).is_some_and(block::replaceable) {
            let facing = factory::dir_from_yaw(self.body().yaw);
            self.line.start = Some((start, facing));
            self.line.cells = vec![LineCell::belt(start, facing, Shape::Flat)];
        }
        true
    }

    /// The cell the pointer picks as the line's end: in front of the face it points at, or where it
    /// crosses the start's level when it points at the sky.
    fn line_end(&self, start: IVec3) -> Option<IVec3> {
        let (eye, dir) = (self.body().eye(), self.body().look_dir());
        let world = &self.sim.world;
        if let Some(hit) = raycast(eye, dir, LINE_REACH, |p| world.get_block(p).filter(|&b| !block::replaceable(b))) {
            return (hit.normal != IVec3::ZERO).then_some(hit.block + hit.normal);
        }
        let t = (start.y as f64 + 0.5 - eye.y) / dir.y;
        (dir.y < -1e-3 && t < LINE_REACH).then(|| (eye + dir * t).floor())
    }

    /// Sends the next few placements of a line being built. False when there are none. Stops if a slot
    /// no longer holds the belts it was to give (or the kits ran out).
    fn send_line_placements(&mut self) -> bool {
        if self.line.building.is_empty() {
            return false;
        }
        for _ in 0..BUILD_PER_TICK {
            let Some(Build { pos, dir, item, slot }) = self.line.building.last().copied() else { break };
            let holds = if self.line.upgrading {
                self.inventory().count(item) > 0
            } else {
                self.inventory().slots[slot as usize].item == item
            };
            if !holds {
                self.line.building.clear();
                return false;
            }
            self.line.building.pop();
            if self.line.upgrading {
                self.act(Action::Upgrade { pos });
            } else {
                self.act(Action::PlaceBlock { pos, slot, facing: dir, against: pos - UP });
            }
        }
        true
    }

    /// Ghost belts along the planned line, drawn with the machines.
    pub(crate) fn write_line_preview(&mut self, eye: Vec3, time: f64) {
        let held = self.inventory().selected_stack().item;
        if upgrades::kit_tier(held).is_some() {
            return; // upgrades show as coloured outlines only; the belts are already there
        }
        let tier = tiers::placed_by(held).map_or(0, |(_, t)| t);
        for c in &self.line.cells {
            let rel = c.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
            let tier = if c.piece == Piece::Belt { tier } else { c.tier };
            factory::belt_preview(&mut self.instances, c.pos, c.dir, c.shape, tier, rel, time);
        }
    }

    /// Cells of the planned line for the host's outlines: x, y, z and 1 if it will be built, or the kit's
    /// tier colour (0xRRGGBB) if it will be upgraded; 0 (red) past what the held stack pays for, for
    /// underpasses the inventory has no pair of, and for what is too wide to go under.
    pub(crate) fn planned_cells(&self) -> Vec<i32> {
        let have = self.line_budget();
        let kit = upgrades::kit_tier(self.inventory().selected_stack().item);
        let colour = kit.map_or(1, |t| upgrades::TIER_COLOURS[t as usize] as i32);
        let paid: Vec<bool> = match kit {
            Some(_) => (0..self.line.cells.len()).map(|i| i < have).collect(),
            None => pass::affordable(&self.line.cells, have),
        };
        let cell = |p: IVec3, ok: bool| [p.x, p.y, p.z, if ok { colour } else { 0 }];
        let planned = self.line.cells.iter().zip(paid).map(|(c, ok)| cell(c.pos, ok));
        let aimed = self.line.aim.belts.iter().enumerate().map(|(i, &p)| cell(p, self.line.cells.len() + i < have));
        let blocked = self.line.blocked.map(|p| cell(p, false));
        planned.chain(aimed).chain(blocked).flatten().collect()
    }
}

mod pass;
mod path;
#[cfg(test)]
mod tests;
