//! Icons and loose models of machines drawn from their placed model: each machine in `PLACED` is put in a scratch
//! factory and what it draws there becomes its parts (`factory::shape_of`), so its icon is the machine as the player
//! sees it standing in the world and can never drift from it. A row picks the facing that turns the machine's face to
//! the icon's viewer (who looks from +x and +z, above) and the neighbours a joining piece is shown with (a pipe between
//! two pipes is a straight length), keeping only what stands in its own cell then. A model bigger than a
//! block is scaled down to fit the item cube; every model stands on the cube's floor, centred on its bounds.
//! Built once, on first use. Add a row to give another machine its placed look; a machine whose placed model does not
//! read at hotbar size gets a hand-built icon in `machines.rs` instead.

use std::cell::Cell;
use std::sync::OnceLock;

use super::icon_frame::outline_points;
use super::Part;
use crate::block::{self, BlockId};
use crate::factory::{shape_of, Factory, FACES, INSTANCE_FLOATS};
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::world::World;
use crate::worldgen::WorldGen;

/// A machine whose icon is its placed model: the block, the placing facing (a `DIRS` index), and the faces (`FACES`
/// bits) a piece of the same block stands on while it is drawn.
struct Placed {
    block: BlockId,
    facing: u8,
    joined: u8,
}

/// Faces of `FACES` along x (a straight run east to west).
const ALONG_X: u8 = 0b11;

const PLACED: &[Placed] = &[
    Placed { block: block::PIPE, facing: 1, joined: ALONG_X },
    Placed { block: block::RAIL, facing: 0, joined: 0 },
    Placed { block: block::FIBRE_NODE, facing: 0, joined: 0 },
];

thread_local! {
    /// Set while the models are built, so a machine drawing an item's model then (a product display) finds none
    /// instead of waiting on itself.
    static BUILDING: Cell<bool> = const { Cell::new(false) };
}

/// The parts of a machine in `PLACED`, else none.
pub(super) fn parts(item: ItemId) -> &'static [Part] {
    static BUILT: OnceLock<Vec<(ItemId, Vec<Part>)>> = OnceLock::new();
    if item.0 >= 256 || !PLACED.iter().any(|p| ItemId::block(p.block) == item) || BUILDING.get() {
        return &[];
    }
    let built = BUILT.get_or_init(|| {
        BUILDING.set(true);
        let world = &mut World::with_generator(WorldGen::new(0), 2);
        let all = PLACED.iter().map(|p| (ItemId::block(p.block), model(world, p))).collect();
        BUILDING.set(false);
        all
    });
    built.iter().find(|(i, _)| *i == item).map_or(&[], |(_, parts)| parts.as_slice())
}

/// One machine's parts: placed, drawn from its centre, fitted to the item cube.
fn model(world: &mut World, p: &Placed) -> Vec<Part> {
    let mut f = Factory::default();
    let pos = IVec3::new(0, 64, 0);
    let below = |q: IVec3| q - IVec3::new(0, 1, 0);
    f.place(world, p.block, pos, p.facing, below(pos), 0);
    let around: Vec<IVec3> = (0..6).filter(|&k| p.joined & 1 << k != 0).map(|k| pos + FACES[k]).collect();
    for &q in &around {
        f.place(world, p.block, q, p.facing, below(q), 0);
    }
    f.relink();
    let mut out = Vec::new();
    f.write_instances(&mut out, pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.0, 64.0);
    // With neighbours, only what stands in the piece's own cell (a joint's flange on its faces included).
    let own = |r: &&[f32]| around.is_empty() || r[..3].iter().all(|c| c.abs() <= 0.501);
    let mut parts: Vec<Part> = out
        .chunks_exact(INSTANCE_FLOATS)
        .filter(own)
        .map(|r| {
            let (form, center, yaw, size, tex) = shape_of(r);
            Part { center, size, yaw, tex, form }
        })
        .collect();
    fit(&mut parts);
    parts
}

/// Scales a model bigger than a block to fit the cube and stands it on the cube's floor, centred across.
fn fit(parts: &mut [Part]) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in parts.iter() {
        for q in outline_points(p) {
            for k in 0..3 {
                lo[k] = lo[k].min(q[k]);
                hi[k] = hi[k].max(q[k]);
            }
        }
    }
    let span = (0..3).map(|k| hi[k] - lo[k]).fold(1.0f32, f32::max);
    let shift = [-(lo[0] + hi[0]) / 2.0, -0.5 * span - lo[1], -(lo[2] + hi[2]) / 2.0];
    for p in parts.iter_mut() {
        p.center = [0, 1, 2].map(|k| (p.center[k] + shift[k]) / span);
        p.size = p.size.map(|s| s / span);
    }
}
