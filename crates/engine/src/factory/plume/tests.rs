//! A plume: everything below its nozzle and within its length, a pale diamond at every pinch, all of it glowing.

use crate::block::tex;
use crate::factory::render::{light_boxes, INSTANCE_FLOATS};
use crate::math::Vec3;

use super::{push_plume, CELLS};

fn plume(lean: f32) -> Vec<f32> {
    let mut out = Vec::new();
    push_plume(&mut out, Vec3::new(0.0, 10.0, 0.0), 0.0, lean, 1.0, 6.0, 1.5, 0.2);
    out
}

#[test]
fn a_plume_hangs_below_its_nozzle_with_a_diamond_at_every_pinch() {
    let out = plume(0.0);
    let recs: Vec<&[f32]> = out.chunks_exact(INSTANCE_FLOATS).collect();
    assert!(recs.iter().all(|r| r[1] < 10.0 && r[1] > 10.0 - 6.6), "below the nozzle, within its length");
    let diamonds = recs.iter().filter(|r| r[8] as u16 == tex::PLUME_DIAMOND).count();
    assert_eq!(diamonds, 2 * CELLS, "two cones tip to tip at each of the pinches");
    let mut lit = out.clone();
    light_boxes(&mut lit, Vec3::ZERO, |_| 0);
    assert!(lit.chunks_exact(INSTANCE_FLOATS).all(|r| r[15] == 1.0), "every part glows");
}

#[test]
fn a_canted_plume_leans_out_along_its_yaw() {
    let out = plume(0.4);
    let last = out.chunks_exact(INSTANCE_FLOATS).last().unwrap();
    assert!(last[0].abs() < 1e-4 && last[2] < -1.0, "the tail swings out along -z for yaw 0: {:?}", &last[..3]);
}
