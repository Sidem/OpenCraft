//! Underpasses in a dragged belt line (part of `belt_line.rs`). Where the path meets something it can't
//! lay a belt on (a belt, a machine, a wall), `plan` asks [`bridge`] for the nearest free cell ahead at the
//! same level, up to the reach of the best underpass, and [`dive`] turns the last belt into an entry and
//! puts an exit there. [`assign`] then picks which underpass each pair is: the lowest one that covers the
//! width, is at least the belt's Mk (so it isn't slower than the line) and that the inventory holds a pair
//! of; a pair it has none of is `short`, drawn red and left out when the line is built ([`affordable`]).
//! Too wide to go under, or no free cell before the path ends: the line stops there, with the obstacle
//! marked red (`BeltLine::blocked`).
//!
//! Invariants: the factory pairs placed underpasses by itself (`factory/underpass.rs`), so a pair is just
//! two same-tier pieces in a line facing the same way, no further apart than the entry's reach.
//! To change which underpass is picked: `assign`; what the HUD says: `belt_line_label`.

use crate::block::UNDERPASS_IN;
use crate::factory::underpass::UNDERPASS_SPAN;
use crate::factory::{tiers, Shape};
use crate::item::ItemId;
use crate::math::IVec3;
use crate::Game;

use super::{LineCell, Piece};

/// The most blocks the best underpass passes under.
const MAX_UNDER: usize = UNDERPASS_SPAN[UNDERPASS_SPAN.len() - 1] as usize;

impl LineCell {
    /// The item that places this cell: the held belt, or its underpass.
    pub(super) fn item(&self, belt: ItemId) -> ItemId {
        match self.piece {
            Piece::Belt => belt,
            Piece::Entry(_) | Piece::Exit => {
                tiers::item_of(UNDERPASS_IN, self.tier).unwrap_or(ItemId::block(UNDERPASS_IN))
            }
        }
    }
}

/// How many blocks the line goes under at column `i` (blocked at level `y`): the nearest later column on
/// the same straight run whose cell at `y` is free. None when the belt before it can't be an entry (an
/// underpass already, or fed from a level above) or nothing free lies within reach.
pub(super) fn bridge(
    cols: &[((i32, i32), u8)],
    i: usize,
    y: i32,
    cells: &[LineCell],
    free: &impl Fn(IVec3) -> bool,
) -> Option<usize> {
    let entry = cells.last().filter(|c| c.piece == Piece::Belt)?;
    if cells.len() >= 2 && cells[cells.len() - 2].pos.y > y {
        return None; // a belt coming down onto the entry would need a ramp the entry can't be
    }
    (1..=MAX_UNDER)
        .map_while(|k| cols.get(i + k).filter(|c| c.1 == entry.dir).map(|c| (k, c.0)))
        .find_map(|(k, (x, z))| free(IVec3::new(x, y, z)).then_some(k))
}

/// Turns the last cell into an entry going under `k` blocks and pushes the exit at column `i + k`.
pub(super) fn dive(cells: &mut Vec<LineCell>, cols: &[((i32, i32), u8)], i: usize, k: usize, y: i32) {
    let entry = cells.last_mut().expect("bridge needs an entry belt");
    (entry.piece, entry.shape) = (Piece::Entry(k as u8), Shape::Entry);
    let ((x, z), dir) = cols[i + k];
    let exit = LineCell::belt(IVec3::new(x, y, z), dir, Shape::Exit);
    cells.push(LineCell { piece: Piece::Exit, ..exit });
}

/// Chooses the underpass of every pair in `cells` and whether the inventory (`have`: how many of each tier
/// it holds) pays for it. The lowest tier that covers the width, is at least `belt_tier` and has a pair
/// in stock wins; with none in stock the lowest that would do is shown, `short`.
pub(super) fn assign(cells: &mut [LineCell], belt_tier: u8, mut have: [u32; UNDERPASS_SPAN.len()]) {
    let mut entry: Option<usize> = None;
    for i in 0..cells.len() {
        match cells[i].piece {
            Piece::Entry(under) => {
                let range = belt_tier as usize..UNDERPASS_SPAN.len();
                let covers = |t: &usize| UNDERPASS_SPAN[*t] >= i32::from(under);
                let stocked = range.clone().filter(covers).find(|&t| have[t] >= 2);
                let tier = stocked.or_else(|| range.clone().find(covers)).unwrap_or(UNDERPASS_SPAN.len() - 1);
                if stocked.is_some() {
                    have[tier] -= 2;
                }
                cells[i].tier = tier as u8;
                cells[i].short = stocked.is_none();
                entry = Some(i);
            }
            Piece::Exit => {
                if let Some(e) = entry.take() {
                    (cells[i].tier, cells[i].short) = (cells[e].tier, cells[e].short);
                }
            }
            Piece::Belt => {}
        }
    }
}

/// Which cells the inventory builds: belts while the `belts` it holds last (the line ends where they run
/// out), and every underpass pair it has in stock before that.
pub(super) fn affordable(cells: &[LineCell], belts: usize) -> Vec<bool> {
    let (mut used, mut ended) = (0, false);
    cells
        .iter()
        .map(|c| {
            if ended {
                return false;
            }
            if c.piece != Piece::Belt {
                return !c.short;
            }
            ended = used >= belts;
            used += 1;
            !ended
        })
        .collect()
}

impl Game {
    /// Picks the underpasses of the dragged line from what the inventory holds (every frame while dragging).
    pub(super) fn assign_passes(&mut self) {
        let held = self.inventory().selected_stack().item;
        let family = tiers::family(UNDERPASS_IN).map_or(&[][..], |f| f.items);
        let mut have = [0; UNDERPASS_SPAN.len()];
        for (n, &item) in have.iter_mut().zip(family) {
            *n = self.inventory().count(item);
        }
        let belt_tier = tiers::placed_by(held).map_or(0, |(_, t)| t);
        assign(&mut self.line.cells, belt_tier, have);
    }

    /// The HUD card of a belt line being dragged (not an upgrade): belts, underpasses, what is missing.
    pub(crate) fn belt_line_label(&self) -> String {
        let cells = &self.line.cells;
        let plural = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        let belts = cells.iter().filter(|c| c.piece == Piece::Belt).count();
        let passes = cells.iter().filter(|c| c.piece != Piece::Belt).count();
        let short = cells.iter().filter(|c| c.piece != Piece::Belt && c.short).count();
        let sloped = cells.iter().filter(|c| matches!(c.shape, Shape::Up | Shape::Down)).count();
        let have = self.inventory().count(self.inventory().selected_stack().item) as usize;
        let mut s = format!("Belt line\n{}", plural(belts, "belt", "belts"));
        if sloped > 0 {
            s += &format!(", {sloped} on slopes");
        }
        if passes > 0 {
            s += &format!(", {}", plural(passes, "underpass", "underpasses"));
        }
        if have < belts {
            s += &format!(" · you have {have}");
        }
        if short > 0 {
            s += &format!(" · {} missing (red), left out", plural(short, "underpass", "underpasses"));
        }
        if self.line.blocked.is_some() {
            s += " · too wide to go under (red)";
        }
        s + " · release to build, left-click to cancel"
    }
}
