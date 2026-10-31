use super::super::specs::{ProcessSpec, SPECS};
use super::*;
use crate::block::LAUNCH_SITE;
use crate::factory::render::{Axis, Form};
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

/// A display's slot is square (it turns), big enough to read, inside the machine's cells across (a neighbour never
/// clips it; it may rise above them, as a chimney does), overlaps no part and stands on one.
#[test]
fn every_product_display_stands_on_a_roof() {
    const EPS: f32 = 1e-3;
    for spec in SPECS {
        for d in displays(spec) {
            let at = format!("block {}", spec.block);
            assert!(d.size[0] == d.size[2] && d.size[0] >= MIN_SLOT && d.size[1] >= MIN_SLOT, "{at}: slot shape");
            let [x, z, y] = spec.footprint.size.map(|n| n as f32 / 2.0);
            let room = [x, y, z];
            let inside = |k: usize| hi(d, k) <= room[k] + EPS && lo(d, k) >= -room[k] - EPS;
            assert!(inside(0) && inside(2) && lo(d, 1) >= -room[1], "{at}: outside its cells");
            let solid = || spec.parts.iter().filter(|p| boxed(p) && p.turn == 0.0);
            let across = |p: &Part, k: usize| hi(d, k).min(hi(p, k)) - lo(d, k).max(lo(p, k)) > EPS;
            for p in solid() {
                assert!(!(across(p, 0) && across(p, 1) && across(p, 2)), "{at}: a part overlaps the display");
            }
            let under = |p: &&Part| (hi(p, 1) - lo(d, 1)).abs() < EPS && across(p, 0) && across(p, 2);
            assert!(solid().any(|p| under(&p)), "{at}: the display stands on nothing");
        }
    }
}

/// A display is seen from every side: looking at it level from any of 16 directions, at least two of five sight lines
/// across the middle of its product pass every part (a slim post may hide a strip of it, a chimney or tank no more).
#[test]
fn every_product_display_is_seen_from_every_side() {
    const EPS: f32 = 1e-3;
    for spec in SPECS {
        for d in displays(spec) {
            for i in 0..16 {
                let a = i as f32 * std::f32::consts::TAU / 16.0;
                let (dx, dz) = (a.cos(), a.sin());
                let clear = [-0.4, -0.2, 0.0, 0.2, 0.4]
                    .iter()
                    .filter(|&&k| {
                        let (x, z) = (d.at[0] - dz * k * d.size[0], d.at[2] + dx * k * d.size[0]);
                        !spec.parts.iter().filter(|p| boxed(p) && p.turn == 0.0).any(|p| {
                            let level = d.at[1] > lo(p, 1) + EPS && d.at[1] < hi(p, 1) - EPS;
                            level && ray_hits([x, z], [dx, dz], [lo(p, 0), lo(p, 2)], [hi(p, 0), hi(p, 2)])
                        })
                    })
                    .count();
                assert!(clear >= 2, "block {}: hidden from direction {i} of 16", spec.block);
            }
        }
    }
}

/// Whether the ray from `from` along `dir` (on the ground plane) crosses the rectangle `lo`..`hi`.
fn ray_hits(from: [f32; 2], dir: [f32; 2], lo: [f32; 2], hi: [f32; 2]) -> bool {
    let (mut near, mut far) = (0.0f32, f32::MAX);
    for k in 0..2 {
        if dir[k].abs() < 1e-6 {
            if from[k] <= lo[k] || from[k] >= hi[k] {
                return false;
            }
            continue;
        }
        let (a, b) = ((lo[k] - from[k]) / dir[k], (hi[k] - from[k]) / dir[k]);
        (near, far) = (near.max(a.min(b)), far.min(a.max(b)));
    }
    near < far
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

/// Whether a part is drawn as its box (smoke, the display's slot, the rocket's stand and a pour are not).
fn boxed(p: &Part) -> bool {
    !matches!(p.look, Look::Smoke | Look::Product | Look::Rocket | Look::Pour)
}

fn lo(p: &Part, k: usize) -> f32 {
    p.at[k] - p.size[k] / 2.0
}

fn hi(p: &Part, k: usize) -> f32 {
    p.at[k] + p.size[k] / 2.0
}

/// Whether `p` has flat faces across `axis` (0 x, 1 y, 2 z): a box on every axis, a round body only its two ends (its
/// side meets a plane in a line, which cannot flicker).
fn flat_across(p: &Part, axis: usize) -> bool {
    match p.form {
        Form::Box => true,
        Form::Lean(_) => false,
        Form::Round(along, _) => axis == [Axis::X, Axis::Up, Axis::Z].iter().position(|&a| a == along).unwrap(),
    }
}

/// The axis (0 x, 1 y, 2 z) on which `a` and `b` have a face in one plane, facing the same way and overlapping.
fn shared_face(a: &Part, b: &Part) -> Option<usize> {
    const EPS: f32 = 1e-3;
    let lo = |p: &Part, k: usize| p.at[k] - p.size[k] / 2.0;
    let hi = |p: &Part, k: usize| p.at[k] + p.size[k] / 2.0;
    for axis in 0..3 {
        let same_plane = (lo(a, axis) - lo(b, axis)).abs() < EPS || (hi(a, axis) - hi(b, axis)).abs() < EPS;
        let overlap = (0..3).filter(|&k| k != axis).all(|k| hi(a, k).min(hi(b, k)) - lo(a, k).max(lo(b, k)) > EPS);
        if same_plane && overlap && flat_across(a, axis) && flat_across(b, axis) {
            return Some(axis);
        }
    }
    None
}

/// Extents across and up of a part as (x min, x max, y min, y max), round bodies taken as their boxes. Every dock part
/// is in one cell along the track, so the length does not matter.
fn spans(at: [f32; 3], size: [f32; 3]) -> [f32; 4] {
    [at[0] - size[0] / 2.0, at[0] + size[0] / 2.0, at[1] - size[1] / 2.0, at[1] + size[1] / 2.0]
}

#[test]
fn every_car_fits_through_both_docks() {
    use super::super::docks::{CAR_HALF, CAR_TOP, LOADING_DOCK_SPEC, UNLOADING_DOCK_SPEC};
    use crate::factory::trains::model::{LOCOMOTIVE_BOXES, RAIL_TOP, WAGON_BOXES};
    const EPS: f32 = 1e-4;
    let rail = RAIL_TOP as f32;
    for (x, y, _, size, ..) in LOCOMOTIVE_BOXES.iter().chain(WAGON_BOXES.iter()) {
        let [x0, x1, _, y1] = spans([*x as f32, rail + *y as f32, 0.0], *size);
        assert!(x0 >= -CAR_HALF && x1 <= CAR_HALF, "a car part sticks out sideways: {x}, {y}, {size:?}");
        assert!(y1 <= CAR_TOP, "a car part is taller than a dock lets through: {x}, {y}, {size:?}");
    }
    for spec in [LOADING_DOCK_SPEC, UNLOADING_DOCK_SPEC] {
        assert_eq!(spec.parts.iter().filter(|p| matches!(p.look, Look::Pour)).count(), 1, "{} pours", spec.verb);
        for p in spec.parts.iter().filter(|p| boxed(p)) {
            let [x0, x1, y0, y1] = spans(p.at, p.size);
            let inside = x0 < CAR_HALF - EPS && x1 > -CAR_HALF + EPS && y0 < CAR_TOP - EPS && y1 > rail + EPS;
            assert!(!inside, "{} has a part in the cars' way: {:?} {:?}", spec.verb, p.at, p.size);
        }
    }
}
