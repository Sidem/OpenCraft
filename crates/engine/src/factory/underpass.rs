//! Underpasses: one item (and block) in tiers Mk1 to Mk4. Pass pieces standing in a line facing the same
//! way pair up by themselves: the first piece takes items under whatever lies between it and the next piece
//! within its reach, which comes out as an exit. A piece with nothing in reach stays a dangling entry, and
//! the next piece starts a new pair. The tier sets the reach (`UNDERPASS_SPAN`) and, as for belts, the speed.
//!
//! Invariants: roles are derived at every relink (`derive_passes`, before `derive_slopes`) from positions,
//! directions and tiers alone, so the order of the belts doesn't matter and nothing about them is saved but
//! "this belt is a pass piece" (`Shape::Entry` or `Shape::Exit`; both old blocks place the same piece).
//! To change the reach or the price: `UNDERPASS_SPAN`; a piece costs `span / 2` belts of its own Mk
//! (`recipes/tiers.rs`, checked by a test).

use crate::math::sort_small_by_key;

use super::belt::Belt;
use super::belt_shape::Shape;
use super::DIRS;

/// How many blocks a pair of Mk1..Mk4 underpasses can pass under (the cells between entry and exit).
pub const UNDERPASS_SPAN: [i32; 4] = [4, 6, 8, 10];

/// Blocks an underpass of `tier` can pass under.
pub fn span(tier: u8) -> i32 {
    UNDERPASS_SPAN[usize::from(tier).min(UNDERPASS_SPAN.len() - 1)]
}

/// Gives every pass piece its role and returns, per belt, the exit its entry hands items to.
pub(super) fn derive_passes(belts: &mut [Belt]) -> Vec<Option<u32>> {
    let mut exit_of = vec![None; belts.len()];
    let mut pieces: Vec<u32> = (0..belts.len() as u32).filter(|&i| belts[i as usize].shape.is_pass()).collect();
    // Along each line, in the direction of travel.
    let key = |b: &Belt| {
        let d = DIRS[b.dir as usize];
        let (along, side) = (b.pos.x * d.x + b.pos.z * d.z, if d.x != 0 { b.pos.z } else { b.pos.x });
        (b.dir, b.pos.y, side, along)
    };
    sort_small_by_key(&mut pieces, |&i| key(&belts[i as usize]));
    let mut open: Option<u32> = None;
    for &i in &pieces {
        let (dir, y, side, along) = key(&belts[i as usize]);
        let reached = open.filter(|&e| {
            let (d2, y2, s2, a2) = key(&belts[e as usize]);
            (d2, y2, s2) == (dir, y, side) && along - a2 <= span(belts[e as usize].tier) + 1
        });
        if let Some(e) = reached {
            exit_of[e as usize] = Some(i);
            belts[i as usize].shape = Shape::Exit;
            open = None;
        } else {
            belts[i as usize].shape = Shape::Entry;
            open = Some(i);
        }
    }
    exit_of
}

#[cfg(test)]
mod tests;
