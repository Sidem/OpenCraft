//! Felling a tree whole (core state, deterministic): the trunk standing on a foot goes from the world into the
//! harvester's head, and with it the leaves round it that no log holds once it is gone. Those would only decay into
//! loose saplings no machine gathers; leaves another trunk still holds stay, so a neighbour keeps its crown.
//! Edits go through `set_block_anywhere_later` (many at once) and into `changed`, so the block timers see them.

use crate::block::{is_leaves, is_log, BlockId, AIR};
use crate::math::IVec3;
use crate::sim::{SimEvent, LEAF_SUPPORT_STEPS};
use crate::world::World;

use super::super::super::FACES;
use super::patch::PATCH_HEIGHT;

/// How far round the trunk the crown can reach (a broad oak's does 3).
const CROWN: i32 = 3;
/// The block of the world it looks at: the crown and the steps a log beyond it could hold a crown leaf from.
const REACH: i32 = CROWN + LEAF_SUPPORT_STEPS;
const SIDE: i32 = 2 * REACH + 1;
const CROWN_LAYERS: i32 = PATCH_HEIGHT + 2;
const LAYERS: i32 = CROWN_LAYERS + LEAF_SUPPORT_STEPS;

/// What came down: the trunk's logs and their kind, and the leaves stripped with it and theirs.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Felled {
    pub logs: u8,
    pub log: BlockId,
    pub leaves: u32,
    pub leaf: BlockId,
}

/// Fells the trunk standing on `foot` (its logs from the foot up) and strips the leaves round it that nothing holds
/// now. Nothing when no log stands on `foot`.
pub(super) fn fell(
    world: &mut World,
    foot: IVec3,
    changed: &mut Vec<(IVec3, BlockId)>,
    events: &mut Vec<SimEvent>,
) -> Felled {
    let mut out = Felled::default();
    for dy in 0..PATCH_HEIGHT {
        let p = foot + IVec3::new(0, dy, 0);
        let b = world.block_anywhere_or_generate(p);
        if !is_log(b) {
            break;
        }
        world.set_block_anywhere_later(p, AIR);
        changed.push((p, b));
        events.push(SimEvent::QuarryDug { pos: p, block: b });
        (out.logs, out.log) = (out.logs.saturating_add(1), b);
    }
    if out.logs == 0 {
        return out;
    }
    let origin = foot - IVec3::new(REACH, 0, REACH);
    let index = |x: i32, y: i32, z: i32| ((y * SIDE + z) * SIDE + x) as usize;
    let mut blocks = vec![AIR; (SIDE * SIDE * LAYERS) as usize];
    let mut held = vec![false; blocks.len()];
    let mut frontier = Vec::new();
    for y in 0..LAYERS {
        for z in 0..SIDE {
            for x in 0..SIDE {
                let b = world.block_anywhere_or_generate(origin + IVec3::new(x, y, z));
                blocks[index(x, y, z)] = b;
                if is_log(b) {
                    held[index(x, y, z)] = true;
                    frontier.push(IVec3::new(x, y, z));
                }
            }
        }
    }
    // Out from every log through leaves, as far as a log holds leaves (`sim/timers.rs`, `leaf_supported`).
    for _ in 0..LEAF_SUPPORT_STEPS {
        let mut next = Vec::new();
        for p in frontier {
            for n in FACES.map(|f| p + f) {
                let inside = (0..SIDE).contains(&n.x) && (0..LAYERS).contains(&n.y) && (0..SIDE).contains(&n.z);
                if inside && !held[index(n.x, n.y, n.z)] && is_leaves(blocks[index(n.x, n.y, n.z)]) {
                    held[index(n.x, n.y, n.z)] = true;
                    next.push(n);
                }
            }
        }
        frontier = next;
    }
    for y in 0..CROWN_LAYERS {
        for z in REACH - CROWN..=REACH + CROWN {
            for x in REACH - CROWN..=REACH + CROWN {
                let b = blocks[index(x, y, z)];
                if is_leaves(b) && !held[index(x, y, z)] {
                    let p = origin + IVec3::new(x, y, z);
                    world.set_block_anywhere_later(p, AIR);
                    changed.push((p, b));
                    (out.leaves, out.leaf) = (out.leaves + 1, b);
                }
            }
        }
    }
    out
}
