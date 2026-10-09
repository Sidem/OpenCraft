//! Auto-routing (part of `belt_line.rs`; tech Auto-Routing): [`find`] searches the voxels for a belt route from the
//! start cell to the cell in front of a machine, round walls and over steps, and the Game side ([`Game::plan_route`])
//! runs it when a dragged line's pointer rests on a machine. In ghost mode the line is planted as ghosts instead
//! of belts (`queue_ghosts`), for drones to build.
//!
//! The search is an A* over (cell, heading): it reads blocks through a closure and never touches the core, so a
//! route is a plain query. Cells are free blocks; it prefers cells with ground below (floating belts over a gap
//! cost extra) and few corners. A step up or down is a belt pair running the same way, which the factory turns
//! into a ramp by itself (`factory::belt_shape::derive_slopes`), so neither belt of the pair may turn.
//! It does not use underpasses: a wall too long to walk round (or past [`MAX_EXPANDED`] cells searched) is "no route".
//!
//! To change what a route prefers: the cost constants. To change which machines it routes to: `aimed_machine`.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use rustc_hash::FxHashMap;

use crate::action::Action;
use crate::block::{self, BlockId, SOLID};
use crate::factory::{self, Shape, DIRS};
use crate::math::IVec3;
use crate::research::{Feature, Unlock};
use crate::Game;

use super::path::slopes;
use super::{is_belt, LineCell, Piece, UP};

/// The longest route, in belts.
pub const MAX_ROUTE: usize = 128;
/// Cells the search looks at before giving up.
const MAX_EXPANDED: usize = 30_000;
/// How far outside the box of the two ends the search may wander.
const MARGIN: i32 = 12;
const HEIGHT_MARGIN: i32 = 8;
/// Costs of a step: every cell, a corner, a cell with nothing below it, a step up or down.
const STEP: u32 = 1;
const TURN: u32 = 2;
const FLOAT: u32 = 3;
const CLIMB: u32 = 1;

/// A search state: where, which way the belt before it ran (4: none yet) and whether this belt must run on the same
/// way (it ends a step up or down).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    pos: IVec3,
    came: u8,
    straight: bool,
}

/// The cells of a route from `start` to `end`, reading blocks through `block` (`None`: not loaded). The last belt
/// runs `into` (the way the machine it feeds lies) if given. None when there is no route within the limits.
pub fn find(
    block: impl Fn(IVec3) -> Option<BlockId>,
    start: IVec3,
    end: IVec3,
    into: Option<u8>,
) -> Option<Vec<LineCell>> {
    let free = |p: IVec3| block(p).is_some_and(block::replaceable);
    let ground = |p: IVec3| block(p - UP).is_some_and(|b| SOLID[b as usize] && factory::machine(b).is_none());
    if !free(start) || !free(end) {
        return None;
    }
    let (lo, hi) = (
        IVec3::new(start.x.min(end.x), start.y.min(end.y), start.z.min(end.z)),
        IVec3::new(start.x.max(end.x), start.y.max(end.y), start.z.max(end.z)),
    );
    let inside = |p: IVec3| {
        (lo.x - MARGIN..=hi.x + MARGIN).contains(&p.x)
            && (lo.z - MARGIN..=hi.z + MARGIN).contains(&p.z)
            && (lo.y - HEIGHT_MARGIN..=hi.y + HEIGHT_MARGIN).contains(&p.y)
    };
    let h = |p: IVec3| ((p.x - end.x).abs() + (p.z - end.z).abs() + (p.y - end.y).abs()) as u32;

    let first = State { pos: start, came: 4, straight: false };
    let mut best: FxHashMap<State, (u32, Option<State>)> = FxHashMap::default();
    let mut open = BinaryHeap::new();
    let mut seen: Vec<State> = vec![first];
    best.insert(first, (0, None));
    open.push(Reverse((h(start), 0u32, 0usize)));
    let mut expanded = 0;
    while let Some(Reverse((_, g, i))) = open.pop() {
        let s = seen[i];
        if best[&s].0 < g {
            continue;
        }
        if s.pos == end && into.is_none_or(|d| !s.straight || s.came == d) {
            return Some(cells(&best, s, into));
        }
        expanded += 1;
        if expanded > MAX_EXPANDED {
            return None;
        }
        for d in 0..4u8 {
            let turn = s.came < 4 && d != s.came;
            if (s.came < 4 && d == (s.came + 2) % 4) || (s.straight && turn) {
                continue;
            }
            let ahead = s.pos + DIRS[d as usize];
            // Level, a step up (this belt becomes the ramp) or a step down (the next one is the ramp).
            for dy in [0, 1, -1] {
                let n = ahead + IVec3::new(0, dy, 0);
                let step = dy != 0;
                if (step && turn) || !inside(n) || !free(n) || (dy == -1 && !free(ahead)) {
                    continue;
                }
                if dy == 1 && block(ahead).is_some_and(is_belt) {
                    continue; // an up ramp has nothing straight ahead
                }
                let cost =
                    STEP + if turn { TURN } else { 0 } + if ground(n) { 0 } else { FLOAT } + CLIMB * dy.unsigned_abs();
                let next = State { pos: n, came: d, straight: step };
                let g2 = g + cost;
                if best.get(&next).is_none_or(|b| g2 < b.0) {
                    best.insert(next, (g2, Some(s)));
                    seen.push(next);
                    open.push(Reverse((g2 + h(n), g2, seen.len() - 1)));
                }
            }
        }
    }
    None
}

/// The route that ends in `last`, start first, each belt running the way it leaves.
fn cells(best: &FxHashMap<State, (u32, Option<State>)>, last: State, into: Option<u8>) -> Vec<LineCell> {
    let mut states = vec![last];
    while let Some(prev) = best[states.last().expect("starts with one")].1 {
        states.push(prev);
    }
    states.reverse();
    let mut out: Vec<LineCell> = states.iter().map(|s| LineCell::belt(s.pos, s.came, Shape::Flat)).collect();
    // Each belt runs the way the next one was entered; the first has no heading of its own yet.
    for i in 0..out.len().saturating_sub(1) {
        out[i].dir = out[i + 1].dir;
    }
    if let Some(end) = out.last_mut() {
        end.dir = into.unwrap_or(last.came % 4);
    }
    slopes(&mut out);
    out
}

/// The way (0..4) the horizontal vector `v` points.
fn dir_of(v: IVec3) -> Option<u8> {
    (0..4u8).find(|&d| DIRS[d as usize] == v)
}

impl Game {
    /// Whether Auto-Routing is researched: routes, and lines in ghost mode.
    pub(crate) fn auto_route_known(&self) -> bool {
        self.sim.factory.research.has(Unlock::Feature(Feature::AutoRoute))
    }

    /// The route from `start` to the machine the pointer rests on, if it does: Some(None) when it is a machine but
    /// no route reaches it (`line.route` says so), None when the pointer is elsewhere (the plain path applies).
    pub(super) fn plan_route(&self, start: IVec3, end: IVec3) -> Option<Option<Vec<LineCell>>> {
        let into = self.aimed_machine()?;
        let world = &self.sim.world;
        Some(find(|p| world.get_block(p), start, end, into).filter(|r| r.len() <= MAX_ROUTE))
    }

    /// The machine under the pointer (a block with a machine, or part of a multi-block one) and the way a belt
    /// in front of the aimed face runs to feed it (None for a top or bottom face).
    fn aimed_machine(&self) -> Option<Option<u8>> {
        let hit = self.line_hit()?;
        let machine = self.sim.world.get_block(hit.block).is_some_and(|b| factory::machine(b).is_some())
            || self.sim.factory.footprint_at(hit.block).is_some();
        machine.then(|| dir_of(IVec3::ZERO - hit.normal))
    }

    /// Queues the line as ghosts (free; the item only has to be in the inventory): what ghost mode does with a
    /// dragged line. A line of one cell on a ghost removes it, as a click does without the line tool.
    pub(super) fn queue_ghosts(&mut self, cells: &[LineCell]) {
        let item = self.inventory().selected_stack().item;
        if let [c] = cells {
            if self.sim.ghosts.covering(c.pos).is_some() {
                self.act(Action::RemoveGhost { pos: c.pos });
                return;
            }
        }
        let inv = self.inventory();
        let slot_of = |it| {
            let held = inv.selected;
            (inv.slots[held].item == it && !inv.slots[held].is_empty())
                .then_some(held)
                .or_else(|| (0..inv.capacity()).find(|&s| inv.slots[s].item == it && !inv.slots[s].is_empty()))
        };
        let mut queue = Vec::with_capacity(cells.len());
        for c in cells.iter().filter(|c| c.piece == Piece::Belt || !c.short) {
            let it = c.item(item);
            if let Some(slot) = slot_of(it) {
                queue.push(super::Build { pos: c.pos, dir: c.dir, item: it, slot: slot as u8 });
            }
        }
        queue.reverse();
        self.line.upgrading = false;
        self.line.ghosting = true;
        self.line.build_item = item;
        self.line.building = queue;
    }
}

#[cfg(test)]
mod tests;
