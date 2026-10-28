use super::*;
use crate::block::{COAL_ORE, IRON_ORE, SMELTER};
use crate::factory::render::INSTANCE_FLOATS;
use crate::factory::Factory;
use crate::item::IRON_INGOT;
use crate::math::IVec3;
use crate::recipes::MACHINE_RECIPES;
use crate::world::World;

use super::super::model::Look;
use super::super::specs::SPECS;

/// Every product of every maker, drawn in its display, stays inside the slot (small or big, flat or tall), and the
/// plate is drawn too.
#[test]
fn every_product_fits_its_slot() {
    for spec in SPECS {
        let Some(slot) = spec.parts.iter().find(|p| matches!(p.look, Look::Product)) else { continue };
        for r in (0..MACHINE_RECIPES.len() as u16).filter(|&r| spec.recipe(r).is_some()) {
            let mut p = Processor::new(IVec3::ZERO, spec, 0);
            (p.recipe, p.shown) = (Some(r), Some(r));
            let mut out = Vec::new();
            draw(&p, &mut out, Vec3::ZERO, 0.0, slot.size);
            let name = item::name(p.product().unwrap());
            assert!(out.len() > INSTANCE_FLOATS, "{name}: no product drawn");
            for b in out.chunks_exact(INSTANCE_FLOATS).skip(1) {
                let (centre, size) = ([b[0], b[1], b[2]], [b[4], b[5], b[6]]);
                // A part with a turn of its own is let off by its size.
                let spare = if b[3] != 0.0 { size[0].max(size[2]) / 2.0 } else { 0.0 };
                for (k, c) in centre.iter().enumerate() {
                    let far = c.abs() + corners_reach(size, k) - spare;
                    assert!(far <= slot.size[k] / 2.0 + 0.02, "{name} pokes out of block {}'s slot", spec.block);
                }
            }
        }
    }
}

#[test]
fn the_tilt_constants_agree() {
    let (s, c) = TILT.sin_cos();
    assert!((s - TILT_SIN).abs() < 1e-6 && (c - TILT_COS).abs() < 1e-6);
}

/// A smelter picks its recipe by what it is fed: between batches, with nothing left in it, it still shows its last
/// product.
#[test]
fn a_smelter_keeps_showing_its_last_product() {
    let (mut world, mut f) = (World::new(1, 2), Factory { by_hand: true, ..Factory::default() });
    let pos = IVec3::new(0, 100, 0);
    f.place(&mut world, SMELTER, pos, 0, pos - IVec3::new(0, 1, 0), 0);
    assert_eq!(f.processors[0].product(), None, "a new smelter shows nothing");
    f.insert(pos, ItemId::block(IRON_ORE), 1);
    f.insert(pos, ItemId::block(COAL_ORE), 1);
    for tick in 0..60 * 30 {
        f.update(&mut world, tick, &mut Vec::new());
    }
    let p = &f.processors[0];
    assert!(p.batch.is_none() && p.out.count(IRON_INGOT) == 1, "the batch is done");
    assert_eq!(p.product(), Some(IRON_INGOT));
    // `shown` is not saved: a loaded smelter tells it from the ingot it holds.
    let mut w = crate::bytes::ByteWriter::default();
    f.write_state(&mut w);
    let back = Factory::read_state(&mut world, &mut crate::bytes::ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back.processors[0].product(), Some(IRON_INGOT), "after loading");
}

/// How far a tipped box of `size` reaches along `k`, from all eight of its corners (the closed form `reach` is checked
/// against this).
fn corners_reach(size: [f32; 3], k: usize) -> f32 {
    let corner = |i: usize| tip([0, 1, 2].map(|a| if i >> a & 1 == 0 { -size[a] / 2.0 } else { size[a] / 2.0 }));
    (0..8).map(|i| corner(i)[k].abs()).fold(0.0, f32::max)
}

#[test]
fn the_closed_form_reach_matches_the_corners() {
    for size in [[0.3, 0.1, 0.5], [1.0, 1.0, 1.0], [0.05, 0.6, 0.2]] {
        for k in 0..3 {
            assert!((reach(size)[k] - corners_reach(size, k)).abs() < 1e-5);
        }
    }
}
