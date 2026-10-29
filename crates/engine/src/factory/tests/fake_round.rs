//! The standing check that no model fakes a round body from boxes (`render::fake_round`): every machine block at every
//! tier as the factory draws it, and every item's model. Round things are drawn with `push_round` (`Form::Round`).

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::block::BlockId;
use crate::item::{ItemId, ITEM_COUNT};
use crate::item_models::parts;
use crate::math::{IVec3, Vec3};
use crate::world::World;

use crate::factory::render::fake_round;
use crate::factory::tiers::item_of;
use crate::factory::{machine, push_shape, Factory, INSTANCE_FLOATS};

#[test]
fn no_machine_fakes_a_round_body_from_boxes() {
    let mut drawn = 0;
    for id in 1..=255u16 {
        let block = id as BlockId;
        if machine(block).is_none() {
            continue;
        }
        // Each tier the block comes in (a block outside a tier family has only its own).
        for tier in (0..8).filter(|&t| t == 0 || item_of(block, t).is_some()) {
            let (mut world, mut f) = (World::new(1, 2), Factory { by_hand: true, ..Factory::default() });
            let pos = IVec3::new(0, 64, 0);
            f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
            let mut out = Vec::new();
            f.write_instances(&mut out, Vec3::new(0.5, 66.0, 0.5), 0.0, 40.0);
            drawn += out.len();
            if let Some((a, b)) = fake_round(&out) {
                let at = |i: usize| &out[i * INSTANCE_FLOATS..i * INSTANCE_FLOATS + 7];
                panic!(
                    "block {block} tier {tier}: boxes {:?} and {:?} fake a round body; use push_round",
                    at(a),
                    at(b)
                );
            }
        }
    }
    assert!(drawn > 0, "the machines were drawn");
}

#[test]
fn no_item_model_fakes_a_round_body_from_boxes() {
    for id in 0..ITEM_COUNT as u16 {
        let mut out = Vec::new();
        for p in parts(ItemId(id)) {
            let c = Vec3::new(p.center[0] as f64, p.center[1] as f64, p.center[2] as f64);
            push_shape(&mut out, p.form, c, p.yaw, p.size, p.tex, false);
        }
        if let Some((a, b)) = fake_round(&out) {
            panic!("item {id}: parts {a} and {b} fake a round body; use a cylinder");
        }
    }
}

#[test]
fn the_detector_finds_crossed_and_turned_boxes_and_passes_thin_crosses() {
    let mut out = Vec::new();
    let o = Vec3::new(0.0, 0.0, 0.0);
    let pair = |out: &mut Vec<f32>, a: ([f32; 3], f32), b: ([f32; 3], f32)| {
        out.clear();
        push_shape(out, crate::factory::Form::Box, o, a.1, a.0, [0; 3], false);
        push_shape(out, crate::factory::Form::Box, o, b.1, b.0, [0; 3], false);
        fake_round(out).is_some()
    };
    assert!(pair(&mut out, ([0.4, 0.5, 0.3], 0.0), ([0.3, 0.5, 0.4], 0.0)), "a cross about y");
    assert!(pair(&mut out, ([0.6, 0.4, 0.3], 0.0), ([0.6, 0.3, 0.4], 0.0)), "a cross about x");
    assert!(pair(&mut out, ([0.4, 0.5, 0.4], 0.0), ([0.4, 0.5, 0.4], FRAC_PI_4)), "an octagon");
    assert!(!pair(&mut out, ([0.6, 0.05, 0.06], 0.0), ([0.06, 0.05, 0.6], 0.0)), "a thin plus is not round");
    assert!(
        pair(&mut out, ([0.4, 0.5, 0.3], 0.0), ([0.4, 0.5, 0.3], FRAC_PI_2)),
        "a box crossed by its own quarter turn"
    );
    assert!(
        !pair(&mut out, ([0.4, 0.5, 0.3], 0.0), ([0.3, 0.5, 0.4], FRAC_PI_2)),
        "one box turned a quarter is itself"
    );
}
