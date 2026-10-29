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

/// Every product of every maker, drawn in its display at any turn, stays inside the circle the slot turns in and
/// between its floor and top (small or big, flat or tall), and the plate is drawn too.
#[test]
fn every_product_fits_its_slot() {
    for spec in SPECS {
        let Some(slot) = spec.parts.iter().find(|p| matches!(p.look, Look::Product)) else { continue };
        for r in (0..MACHINE_RECIPES.len() as u16).filter(|&r| spec.recipe(r).is_some()) {
            let mut p = Processor::new(IVec3::ZERO, spec, 0);
            (p.recipe, p.shown, p.status) = (Some(r), Some(r), Status::Working);
            let name = item::name(p.product().unwrap());
            for frame in 0..12 {
                let mut out = Vec::new();
                draw(&p, &mut out, Vec3::ZERO, 0.0, slot.size, frame as f64 * MAX_STEP);
                assert!(out.len() > INSTANCE_FLOATS, "{name}: no product drawn");
                for b in out.chunks_exact(INSTANCE_FLOATS) {
                    for [x, y, z] in corners(b) {
                        let at = format!("{name} pokes out of block {}'s slot", spec.block);
                        assert!(x.hypot(z) <= slot.size[0] / 2.0 + 0.02, "{at} (across)");
                        assert!(y.abs() <= slot.size[1] / 2.0 + 0.02, "{at} (up)");
                    }
                }
            }
        }
    }
}

/// The display turns while its machine works, at `SPIN`, and holds still otherwise.
#[test]
fn the_display_turns_only_while_working() {
    let spec = SPECS.iter().find(|s| s.block == SMELTER).unwrap();
    let mut p = Processor::new(IVec3::ZERO, spec, 0);
    p.status = Status::Working;
    let start = p.display_turn(1.0);
    let turned = p.display_turn(1.05);
    assert!((turned - start - 0.05 * SPIN).abs() < 1e-5);
    p.status = Status::NoInput;
    assert_eq!(p.display_turn(2.0), turned);
    p.status = Status::Working;
    assert!(p.display_turn(30.0) - turned <= MAX_STEP as f32 * SPIN + 1e-5, "a long gap is one short step");
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

/// The eight corners of an instance's box, turned as the box shader turns it (`boxVert`: pitch about x, then yaw).
fn corners(b: &[f32]) -> [[f32; 3]; 8] {
    let (centre, yaw, size, pitch) = ([b[0], b[1], b[2]], b[3], [b[4], b[5], b[6]], b[12]);
    let (sy, cy, sp, cp) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
    std::array::from_fn(|i| {
        let [x, y, z] = [0, 1, 2].map(|a| if i >> a & 1 == 0 { -size[a] / 2.0 } else { size[a] / 2.0 });
        let (y, z) = (cp * y - sp * z, sp * y + cp * z);
        [centre[0] + cy * x - sy * z, centre[1] + y, centre[2] + sy * x + cy * z]
    })
}

/// `up` turns a point as a pitch of a quarter turn does, and `bounds` reaches every stood-up corner.
#[test]
fn standing_up_matches_the_shader() {
    let part = Part { center: [0.1, 0.02, -0.2], size: [0.3, 0.04, 0.5], yaw: 0.0, tex: [0; 3], form: Form::Box };
    let (lo, hi) = bounds(&[part], true);
    let mut b = [0.0; INSTANCE_FLOATS];
    (b[0], b[1], b[2]) = (up(part.center)[0], up(part.center)[1], up(part.center)[2]);
    (b[4], b[5], b[6], b[12]) = (part.size[0], part.size[1], part.size[2], FRAC_PI_2);
    for c in corners(&b) {
        for a in 0..3 {
            assert!(c[a] >= lo[a] - 1e-5 && c[a] <= hi[a] + 1e-5);
        }
    }
    assert!(lies_flat(&[part]));
}
