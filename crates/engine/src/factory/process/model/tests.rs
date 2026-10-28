use super::super::specs::{ProcessSpec, SPECS};
use super::*;
use crate::block::LAUNCH_SITE;
use crate::factory::tier_mark::{DEPTH, ROW_MAX, STUD};
use crate::factory::tiers;
use crate::recipes::MACHINE_RECIPES;

/// Makers whose product is no product to show: a launch site's batch launches a rocket; its canisters are change.
const NO_DISPLAY: [crate::block::BlockId; 1] = [LAUNCH_SITE];
/// The smallest slot side: 16 px at 16 blocks on a 1080p screen.
const MIN_SLOT: f32 = 0.34;

/// The faces of two parts that lie in one plane, face the same way and overlap z-fight: the GPU can't tell which is in
/// front, so they flicker. Turned parts are skipped (their faces aren't axis-aligned in the model's frame), and so is
/// the product display's slot, which is no box.
#[test]
fn no_model_has_two_faces_in_one_plane() {
    let mut clashes = Vec::new();
    for spec in SPECS {
        let parts: Vec<&Part> = spec.parts.iter().filter(|p| p.turn == 0.0 && boxed(p)).collect();
        for (i, a) in parts.iter().enumerate() {
            for (j, b) in parts.iter().enumerate().skip(i + 1) {
                if let Some(axis) = shared_face(a, b) {
                    clashes.push(format!("block {} parts {i} and {j} along {axis}", spec.block));
                }
            }
        }
    }
    assert!(clashes.is_empty(), "coplanar faces:\n{}", clashes.join("\n"));
}

/// Every machine that makes items has one product display and no other machine has one: a new maker can't ship
/// without it.
#[test]
fn every_maker_has_one_product_display() {
    for spec in SPECS {
        let makes = MACHINE_RECIPES.iter().any(|r| spec.categories.contains(&r.category) && !r.outputs.is_empty());
        let want = usize::from(makes && !NO_DISPLAY.contains(&spec.block));
        assert_eq!(displays(spec).count(), want, "block {}", spec.block);
    }
}

/// A display's slot is big enough to read, inside the machine's cells across (a neighbour never clips it; a one-cell
/// machine's sign may rise above its cell, as its chimney does), overlaps no part, and no part stands in front of it
/// (towards +z, the front).
#[test]
fn every_product_display_is_in_sight() {
    const EPS: f32 = 1e-3;
    for spec in SPECS {
        for d in displays(spec) {
            let at = format!("block {}", spec.block);
            assert!(d.size[0] >= MIN_SLOT && d.size[1] >= MIN_SLOT, "{at}: slot too small");
            let [x, z, y] = spec.footprint.size.map(|n| n as f32 / 2.0);
            let room = [x, y, z];
            let inside = |k: usize| hi(d, k) <= room[k] + EPS && lo(d, k) >= -room[k] - EPS;
            assert!(inside(0) && inside(2) && lo(d, 1) >= -room[1], "{at}: outside its cells");
            for p in spec.parts.iter().filter(|p| boxed(p) && p.turn == 0.0) {
                let across = |k: usize| hi(d, k).min(hi(p, k)) - lo(d, k).max(lo(p, k)) > EPS;
                assert!(!(across(0) && across(1) && across(2)), "{at}: a part overlaps the display");
                assert!(!(across(0) && across(1) && lo(p, 2) >= hi(d, 2) - EPS), "{at}: a part hides the display");
            }
        }
    }
}

/// A machine that comes in tiers has a family (`tiers.rs`) and one band, whose sides take a row of Mk5 studs.
#[test]
fn every_tiered_processor_has_a_band_for_its_studs() {
    for spec in SPECS {
        let tiered = spec.tiers.len() > 1;
        assert_eq!(tiered, tiers::family(spec.block).is_some(), "block {}", spec.block);
        let bands: Vec<&Part> = spec.parts.iter().filter(|p| matches!(p.look, Look::Band(_))).collect();
        if tiered {
            assert_eq!(bands.len(), 1, "block {}", spec.block);
            let b = bands[0];
            assert!(b.size[0].min(b.size[2]) >= ROW_MAX && b.size[1] >= STUD, "block {}: band too small", spec.block);
        }
    }
}

/// No part covers a row of Mk5 studs on any side of the band, or stands just before it (a constructor's copper strip
/// hid its front row).
#[test]
fn no_part_covers_the_studs() {
    const EPS: f32 = 1e-3;
    const CLEAR: f32 = 0.05;
    for spec in SPECS.iter().filter(|s| s.tiers.len() > 1) {
        let band = spec.parts.iter().find(|p| matches!(p.look, Look::Band(_))).unwrap();
        for (axis, side) in [(0, -1.0), (0, 1.0), (2, -1.0), (2, 1.0)] {
            let face = band.at[axis] + side * band.size[axis] / 2.0;
            let out = face + side * (DEPTH + CLEAR);
            let across = 2 - axis;
            let mut row_lo = [0.0; 3];
            let mut row_hi = [0.0; 3];
            (row_lo[axis], row_hi[axis]) = (face.min(out), face.max(out));
            (row_lo[1], row_hi[1]) = (band.at[1] - STUD / 2.0, band.at[1] + STUD / 2.0);
            (row_lo[across], row_hi[across]) = (band.at[across] - ROW_MAX / 2.0, band.at[across] + ROW_MAX / 2.0);
            for (i, p) in
                spec.parts.iter().enumerate().filter(|(_, p)| boxed(p) && p.turn == 0.0 && !std::ptr::eq(*p, band))
            {
                let hit = (0..3).all(|k| hi(p, k).min(row_hi[k]) - lo(p, k).max(row_lo[k]) > EPS);
                assert!(!hit, "block {}: part {i} covers the studs on side {side} of axis {axis}", spec.block);
            }
        }
    }
}

fn displays(spec: &ProcessSpec) -> impl Iterator<Item = &Part> {
    spec.parts.iter().filter(|p| matches!(p.look, Look::Product))
}

/// Whether a part is drawn as its box (smoke and the display's slot are not).
fn boxed(p: &Part) -> bool {
    !matches!(p.look, Look::Smoke | Look::Product)
}

fn lo(p: &Part, k: usize) -> f32 {
    p.at[k] - p.size[k] / 2.0
}

fn hi(p: &Part, k: usize) -> f32 {
    p.at[k] + p.size[k] / 2.0
}

/// The axis (0 x, 1 y, 2 z) on which `a` and `b` have a face in one plane, facing the same way and overlapping.
fn shared_face(a: &Part, b: &Part) -> Option<usize> {
    const EPS: f32 = 1e-3;
    let lo = |p: &Part, k: usize| p.at[k] - p.size[k] / 2.0;
    let hi = |p: &Part, k: usize| p.at[k] + p.size[k] / 2.0;
    for axis in 0..3 {
        let same_plane = (lo(a, axis) - lo(b, axis)).abs() < EPS || (hi(a, axis) - hi(b, axis)).abs() < EPS;
        let overlap = (0..3).filter(|&k| k != axis).all(|k| hi(a, k).min(hi(b, k)) - lo(a, k).max(lo(b, k)) > EPS);
        if same_plane && overlap {
            return Some(axis);
        }
    }
    None
}
