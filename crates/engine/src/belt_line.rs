//! Laying belts and rails in lines, part of the local player's hands (`interaction.rs`). With a belt selected,
//! holding the use button on a surface starts a line at the cell in front of that face; the pointer
//! (up to `LINE_REACH` away) sets the far end; releasing builds it, a few cells per tick, as ordinary
//! `PlaceBlock` actions, so the core, co-op and saves see plain placements. A left click cancels.
//!
//! The path ([`plan`]) runs along the longer horizontal axis first and turns once. It follows the
//! ground one block up or down at a time; the factory turns those steps into ramps by itself
//! (`factory::belt_shape::derive_slopes`). It stops before anything in the way, so a line ending at a
//! machine feeds it. Planning reads the loaded world (the render cache), never core state.
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
//! To change the path: `plan` / `plan_upgrade`. The preview: ghost belts (`write_line_preview`) and the
//! host's outline boxes and label (`api/hud.rs` `line_cells`, `line_label`), in the kit's colour when
//! upgrading.

use crate::action::Action;
use crate::block::{self, BlockId, BELT, FAST_BELT, RAIL, RAMP_DOWN, RAMP_UP, SOLID};
use crate::factory::{self, tiers, upgrades, Shape};
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::raycast::raycast;
use crate::research::Unlock;
use crate::upgrade_aim::{Aim, MAX_CHAIN};
use crate::Game;

/// The longest line one drag builds, in cells.
pub const MAX_LINE: usize = 64;
/// How far away the pointer can set the line's end.
const LINE_REACH: f64 = 48.0;
/// Cells placed per tick while a line is built (fast, but the placing sounds don't all land at once).
const BUILD_PER_TICK: usize = 3;
const UP: IVec3 = IVec3::new(0, 1, 0);

/// One cell of a planned line: where, which way it runs, and how it will slope.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineCell {
    pub pos: IVec3,
    pub dir: u8,
    pub shape: Shape,
}

/// One placement (or upgrade) of a line being built: the cell, the way it runs and the inventory slot
/// the belt comes from.
#[derive(Clone, Copy)]
struct Build {
    pos: IVec3,
    dir: u8,
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
    build_item: ItemId,
    /// The cells being built are upgrades (kits), not placements.
    upgrading: bool,
    /// What a held kit would upgrade at the crosshair (`upgrade_aim.rs`); empty while dragging.
    pub aim: Aim,
}

/// Whether block `b` is laid in lines: belts (`FAST_BELT` is a legacy block of old worlds) and rails, which follow
/// the same path (rails ignore the facing).
pub fn is_laid_in_lines(b: BlockId) -> bool {
    matches!(b, BELT | FAST_BELT | RAMP_UP | RAMP_DOWN | RAIL)
}

/// The columns (x, z) of a path from `start` towards the column of `end`, each with the way it runs:
/// along the longer axis first, turning once (the turning cell runs the new way); one column runs `facing`.
fn columns(start: IVec3, end: IVec3, facing: u8) -> Vec<((i32, i32), u8)> {
    let (dx, dz) = (end.x - start.x, end.z - start.z);
    let legs = if dx.abs() >= dz.abs() { [(dx, 0), (0, dz)] } else { [(0, dz), (dx, 0)] };
    let mut cols = vec![((start.x, start.z), facing)];
    for (lx, lz) in legs {
        let dir = match (lx.signum(), lz.signum()) {
            (0, 0) => continue,
            (0, -1) => 0,
            (1, 0) => 1,
            (0, 1) => 2,
            _ => 3,
        };
        cols.last_mut().expect("starts with one").1 = dir;
        let mut c = cols.last().expect("starts with one").0;
        for _ in 0..lx.abs().max(lz.abs()) {
            c = (c.0 + lx.signum(), c.1 + lz.signum());
            cols.push((c, dir));
        }
    }
    cols.truncate(MAX_LINE);
    cols
}

/// The cells of a line from `start` towards the column of `end`, reading blocks through `block`
/// (`None`: not loaded). A line of one cell runs `facing`. Empty when `start` isn't free.
pub fn plan(block: impl Fn(IVec3) -> Option<BlockId>, start: IVec3, end: IVec3, facing: u8) -> Vec<LineCell> {
    let free = |p: IVec3| block(p).is_some_and(block::replaceable);
    let solid = |p: IVec3| block(p).is_some_and(|b| SOLID[b as usize] && factory::machine(b).is_none());
    let cols = columns(start, end, facing);
    let mut cells: Vec<LineCell> = Vec::with_capacity(cols.len());
    let mut y = start.y;
    for (i, &((x, z), dir)) in cols.iter().enumerate() {
        let at = |y: i32| IVec3::new(x, y, z);
        if i > 0 {
            y = if free(at(y)) && solid(at(y - 1)) {
                y
            } else if solid(at(y)) && free(at(y + 1)) {
                y + 1 // a step up: the belt before it becomes an up ramp
            } else if free(at(y)) && free(at(y - 1)) && solid(at(y - 2)) {
                y - 1 // a step down: this belt becomes a down ramp
            } else if free(at(y)) {
                y // over a gap, level
            } else {
                break;
            };
        } else if !free(at(y)) {
            break;
        }
        cells.push(LineCell { pos: at(y), dir, shape: Shape::Flat });
    }
    // The slopes the factory will derive, for the preview.
    for i in 0..cells.len() {
        let y = cells[i].pos.y;
        if cells.get(i + 1).is_some_and(|n| n.pos.y == y + 1) {
            cells[i].shape = Shape::Up;
        } else if i > 0 && cells[i - 1].pos.y == y + 1 {
            cells[i].shape = Shape::Down;
        }
    }
    cells
}

/// The belts to upgrade from the belt at `start` towards the column of `end`: the path follows belts
/// (`tier_at`: the tier of the belt at a cell) one block up or down at a time and stops where none is;
/// the cells are those at tier `from`.
pub fn plan_upgrade(tier_at: impl Fn(IVec3) -> Option<u8>, start: IVec3, end: IVec3, from: u8) -> Vec<LineCell> {
    let mut cells = Vec::new();
    let mut y = start.y;
    for ((x, z), dir) in columns(start, end, 0) {
        let Some(pos) = [y, y + 1, y - 1].map(|y| IVec3::new(x, y, z)).into_iter().find(|&p| tier_at(p).is_some())
        else {
            break;
        };
        y = pos.y;
        if tier_at(pos) == Some(from) {
            cells.push(LineCell { pos, dir, shape: Shape::Flat });
        }
    }
    cells
}

impl Game {
    /// Runs the line tool for one tick. True while it owns the use button (a belt is selected and the
    /// button went down on a surface, or a line is being built).
    pub(crate) fn update_belt_line(&mut self) -> bool {
        self.line.aim = Aim::default();
        if self.send_line_placements() {
            return true;
        }
        let stack = self.inventory().selected_stack();
        let kit = upgrades::kit_tier(stack.item).filter(|_| !stack.is_empty());
        if kit.is_none() && (!stack.item.places().is_some_and(is_laid_in_lines) || stack.is_empty()) {
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
                    None => plan(|p| self.sim.world.get_block(p), start, end, facing),
                };
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
    /// held stack and then the other stacks, or, with `upgrading`, upgrades paid with kits.
    fn queue_build(&mut self, cells: &[LineCell], upgrading: bool) {
        let item = self.inventory().selected_stack().item;
        let n = cells.len().min(self.line_budget());
        let slots = if upgrading { vec![self.inventory().selected as u8; n] } else { self.belt_slots(item, n) };
        let mut queue: Vec<Build> =
            cells.iter().zip(slots).map(|(c, slot)| Build { pos: c.pos, dir: c.dir, slot }).collect();
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
                let cells: Vec<_> = chain.iter().map(|&pos| LineCell { pos, dir: 0, shape: Shape::Flat }).collect();
                self.queue_build(&cells, true);
                self.using = false;
            }
            Some((BELT, t)) => {
                self.line.start = Some((hit.block, 0));
                let cell = LineCell { pos: hit.block, dir: 0, shape: Shape::Flat };
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
            self.line.cells = vec![LineCell { pos: start, dir: facing, shape: Shape::Flat }];
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
        let item = self.line.build_item;
        for _ in 0..BUILD_PER_TICK {
            let Some(Build { pos, dir, slot }) = self.line.building.last().copied() else { break };
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
        let rails = held.places() == Some(RAIL);
        for c in &self.line.cells {
            let rel = c.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
            if rails {
                factory::write_rail(&mut self.instances, rel, factory::ghost_arms(c.dir, c.shape));
            } else {
                factory::belt_preview(&mut self.instances, c.pos, c.dir, c.shape, tier, rel, time);
            }
        }
    }

    /// Cells of the planned line for the host's outlines: x, y, z and 1 if it will be built, or the kit's
    /// tier colour (0xRRGGBB) if it will be upgraded; 0 past what the held stack pays for.
    pub(crate) fn planned_cells(&self) -> Vec<i32> {
        let have = self.line_budget();
        let kit = upgrades::kit_tier(self.inventory().selected_stack().item);
        let ok = kit.map_or(1, |t| upgrades::TIER_COLOURS[t as usize] as i32);
        let cell = |(i, pos): (usize, IVec3)| [pos.x, pos.y, pos.z, if i < have { ok } else { 0 }];
        let aimed = self.line.aim.belts.iter().copied();
        self.line.cells.iter().map(|c| c.pos).chain(aimed).enumerate().flat_map(cell).collect()
    }
}

#[cfg(test)]
mod tests;
