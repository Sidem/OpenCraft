//! The blueprint prompt (presentation only, never saved): when the local player places the same working machine
//! a third time within [`RANGE`] blocks, a tip offers to copy a group as a blueprint instead, and the blueprint
//! selection is preset to a box around the previous copy, so B then Enter copies it. Play-tests showed players
//! never found blueprints by themselves (DEV_PLAN R1b).
//!
//! Fed by `SimEvent::BlockPlaced` in `events.rs`; the host shows the tip (`api/blueprint.rs` `blueprint_prompt`,
//! `web/src/ui/hints.ts`). Invariants: once per machine kind (block and tier) a session, never in creative
//! worlds, and never again once the world has a blueprint (the player knows the tool). To change the wording:
//! `prompt_text`.

use crate::block::{self, BlockId};
use crate::math::IVec3;
use crate::mode::Mode;
use crate::Game;

/// Earlier copies count within this many blocks across (and half that up or down).
pub const RANGE: i32 = 32;
/// The copy that brings the prompt (the third).
pub const COPIES: usize = 3;
/// The prompt shows this long, in ticks (90 s).
pub const SHOW_TICKS: u64 = 90 * 60;
/// Placements remembered this session.
const REMEMBER: usize = 64;
/// The preset box reaches this far around the previous copy's footprint, sideways.
const MARGIN: i32 = 2;

#[derive(Default)]
pub struct Repeat {
    /// The local player's recent working machines this session: block, tier and anchor.
    placed: Vec<(BlockId, u8, IVec3)>,
    /// Kinds already offered this session.
    offered: Vec<(BlockId, u8)>,
    /// The kind on offer and the tick its prompt ends.
    prompt: Option<(BlockId, u64)>,
}

impl Game {
    /// The local player placed `block` with its anchor at `pos`.
    pub(crate) fn note_placed(&mut self, pos: IVec3, block: BlockId) {
        if self.sim.mode == Mode::Creative || !self.library.list.is_empty() || !self.sim.factory.works_at(pos) {
            return;
        }
        let tier = self.sim.factory.placed_as(pos).map_or(0, |(_, tier)| tier);
        let (sim, repeat) = (&self.sim, &mut self.library.repeat);
        // Forget copies that were broken or replaced since.
        repeat.placed.retain(|&(b, _, p)| sim.world.block_anywhere(p) == Some(b) && sim.factory.works_at(p));
        let previous = repeat.placed.iter().rev().find(|&&(b, t, p)| b == block && t == tier && near(p, pos)).copied();
        let copies = 1 + repeat.placed.iter().filter(|&&(b, t, p)| b == block && t == tier && near(p, pos)).count();
        repeat.placed.push((block, tier, pos));
        if repeat.placed.len() > REMEMBER {
            repeat.placed.remove(0);
        }
        let (Some((_, _, before)), true) = (previous, copies >= COPIES) else { return };
        if repeat.offered.contains(&(block, tier)) {
            return;
        }
        repeat.offered.push((block, tier));
        repeat.prompt = Some((block, sim.tick + SHOW_TICKS));
        let lib = &mut self.library;
        if lib.held.is_none() && lib.corner.is_none() && lib.selection.is_none() {
            lib.selection = Some(around(&self.sim.factory.footprint_at(before).map_or(vec![before], |f| f.2)));
            lib.note = "Marked around your previous copy".into();
        }
    }

    /// The prompt's text (hint markup) while it shows, else empty.
    pub(crate) fn repeat_prompt(&self) -> String {
        match self.library.repeat.prompt {
            Some((block, end)) if self.sim.tick < end && self.library.list.is_empty() => prompt_text(block),
            _ => String::new(),
        }
    }

    /// H on the prompt: hide it (it does not come back for that kind this session).
    pub(crate) fn dismiss_repeat_prompt(&mut self) {
        self.library.repeat.prompt = None;
    }
}

fn prompt_text(block: BlockId) -> String {
    format!(
        "Building the same again: that's your third *{}* here. Copy a working group once and stamp it as \
         ghosts instead. Press [B]: a box is already marked around the previous one ([Z] marks new corners), \
         and [Enter] copies it. Then [Right mouse] stamps it; build the ghosts by hand or with drones.",
        block::def(block).name
    )
}

/// Within `RANGE` across and half that up or down.
fn near(a: IVec3, b: IVec3) -> bool {
    (a.x - b.x).abs() <= RANGE && (a.z - b.z).abs() <= RANGE && (a.y - b.y).abs() <= RANGE / 2
}

/// The box around `cells`: `MARGIN` to each side, from their bottom to one above their top.
fn around(cells: &[IVec3]) -> (IVec3, IVec3) {
    let mut lo = cells[0];
    let mut hi = cells[0];
    for c in cells {
        lo = IVec3::new(lo.x.min(c.x), lo.y.min(c.y), lo.z.min(c.z));
        hi = IVec3::new(hi.x.max(c.x), hi.y.max(c.y), hi.z.max(c.z));
    }
    (IVec3::new(lo.x - MARGIN, lo.y, lo.z - MARGIN), IVec3::new(hi.x + MARGIN, hi.y + 1, hi.z + MARGIN))
}
