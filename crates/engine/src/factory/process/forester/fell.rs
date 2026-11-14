//! Felling a tree whole (core state, deterministic): the trunk standing on a foot and every limb joined to it (logs
//! touching face, edge or corner) go from the world, the trunk into the harvester's head as timber and the limbs
//! chipped to nothing (so a tree yields what its trunk holds, as a planted tree always has), and with them the leaves round it that
//! no log holds once they are gone. Those would only decay into loose saplings no machine gathers; leaves another
//! trunk still holds stay, so a neighbour keeps its crown. Where crowns touch, a log belongs to the trunk standing
//! nearest it, so a neighbour's limbs stay too. Edits go through `set_block_anywhere_later` (many at once) and into
//! `changed`, so the block timers see them.

use crate::block::{is_leaves, is_log, BlockId, AIR};
use crate::math::IVec3;
use crate::sim::{SimEvent, LEAF_SUPPORT_STEPS};
use crate::world::World;

use super::super::super::FACES;
use super::patch::PATCH_HEIGHT;

/// How far round the trunk a planted tree's limbs and crown reach (a young oak's limbs do about 3, its leaves 5).
const CROWN: i32 = 5;
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

/// Fells the tree standing on `foot` (its trunk from the foot up and the limbs joined to it) and strips the leaves
/// round it that nothing holds now. Nothing when no log stands on `foot`.
pub(super) fn fell(
    world: &mut World,
    foot: IVec3,
    changed: &mut Vec<(IVec3, BlockId)>,
    events: &mut Vec<SimEvent>,
) -> Felled {
    let mut out = Felled::default();
    if !is_log(world.block_anywhere_or_generate(foot)) {
        return out;
    }
    let origin = foot - IVec3::new(REACH, 0, REACH);
    let index = |p: IVec3| ((p.y * SIDE + p.z) * SIDE + p.x) as usize;
    let mut blocks = vec![AIR; (SIDE * SIDE * LAYERS) as usize];
    for y in 0..LAYERS {
        for z in 0..SIDE {
            for x in 0..SIDE {
                let p = IVec3::new(x, y, z);
                blocks[index(p)] = world.block_anywhere_or_generate(origin + p);
            }
        }
    }
    // Other trunks: logs standing level with this one's foot.
    let centre = IVec3::new(REACH, 0, REACH);
    let others: Vec<IVec3> = (0..SIDE * SIDE)
        .map(|i| IVec3::new(i % SIDE, 0, i / SIDE))
        .filter(|&p| p != centre && is_log(blocks[index(p)]))
        .collect();
    let d2 = |a: IVec3, b: IVec3| (a.x - b.x).pow(2) + (a.z - b.z).pow(2);
    let ours = |p: IVec3| others.iter().all(|&o| d2(p, o) >= d2(p, centre));
    // The tree: logs joined to the foot, within the crown, nearer this trunk than any other.
    let mut tree = vec![centre];
    let mut taken = vec![false; blocks.len()];
    taken[index(centre)] = true;
    let mut i = 0;
    while i < tree.len() {
        let p = tree[i];
        i += 1;
        for d in (0..27).map(|k| IVec3::new(k % 3 - 1, k / 3 % 3 - 1, k / 9 - 1)) {
            let n = p + d;
            let inside = (centre.x - CROWN..=centre.x + CROWN).contains(&n.x)
                && (centre.z - CROWN..=centre.z + CROWN).contains(&n.z)
                && (0..CROWN_LAYERS).contains(&n.y);
            if inside && !taken[index(n)] && is_log(blocks[index(n)]) && ours(n) {
                taken[index(n)] = true;
                tree.push(n);
            }
        }
    }
    out.log = blocks[index(centre)];
    for &p in &tree {
        let b = blocks[index(p)];
        world.set_block_anywhere_later(origin + p, AIR);
        changed.push((origin + p, b));
        events.push(SimEvent::QuarryDug { pos: origin + p, block: b });
        blocks[index(p)] = AIR;
        if p.x == centre.x && p.z == centre.z {
            out.logs = out.logs.saturating_add(1);
        }
    }
    // Out from every log left through leaves, as far as a log holds leaves (`sim/timers.rs`, `leaf_supported`).
    let mut held = vec![false; blocks.len()];
    let mut frontier = Vec::new();
    for (i, &b) in blocks.iter().enumerate() {
        if is_log(b) {
            held[i] = true;
            let i = i as i32;
            frontier.push(IVec3::new(i % SIDE, i / (SIDE * SIDE), i / SIDE % SIDE));
        }
    }
    for _ in 0..LEAF_SUPPORT_STEPS {
        let mut next = Vec::new();
        for p in frontier {
            for n in FACES.map(|f| p + f) {
                let inside = (0..SIDE).contains(&n.x) && (0..LAYERS).contains(&n.y) && (0..SIDE).contains(&n.z);
                if inside && !held[index(n)] && is_leaves(blocks[index(n)]) {
                    held[index(n)] = true;
                    next.push(n);
                }
            }
        }
        frontier = next;
    }
    for y in 0..CROWN_LAYERS {
        for z in REACH - CROWN..=REACH + CROWN {
            for x in REACH - CROWN..=REACH + CROWN {
                let p = IVec3::new(x, y, z);
                let b = blocks[index(p)];
                if is_leaves(b) && !held[index(p)] {
                    world.set_block_anywhere_later(origin + p, AIR);
                    changed.push((origin + p, b));
                    (out.leaves, out.leaf) = (out.leaves + 1, b);
                }
            }
        }
    }
    out
}
