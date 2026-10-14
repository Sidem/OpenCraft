use super::*;
use crate::factory::push_box;

/// A pole-like post: 0.13 wide, standing in the middle of the cell.
fn post() -> Boxes {
    let mut out = Vec::new();
    push_box(&mut out, Vec3::ZERO, 0.0, [0.13, 0.85, 0.13], 0.0, [0; 3], false);
    Boxes::from_instances(&out).unwrap()
}

#[test]
fn a_ray_through_the_post_hits_and_one_beside_it_misses() {
    let d = Vec3::new(0.0, 0.0, -1.0);
    let hit = post().enter(Vec3::new(0.5, 0.5, 3.0), d).unwrap();
    assert!((hit - (3.0 - 0.5 - 0.065 - PAD)).abs() < 1e-6, "{hit}");
    assert!(post().enter(Vec3::new(0.85, 0.5, 3.0), d).is_none());
    assert!(post().enter(Vec3::new(0.5, 0.97, 3.0), d).is_none(), "over the top of the post");
}

#[test]
fn a_ray_starting_inside_a_box_enters_at_zero() {
    assert_eq!(post().enter(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0)), Some(0.0));
}

#[test]
fn a_turned_box_is_hit_where_it_lies() {
    // A long thin bar along x, turned a quarter to lie along z.
    let mut out = Vec::new();
    push_box(&mut out, Vec3::ZERO, std::f32::consts::FRAC_PI_2, [0.8, 0.1, 0.1], 0.0, [0; 3], false);
    let bar = Boxes::from_instances(&out).unwrap();
    let up = Vec3::new(0.0, -1.0, 0.0);
    assert!(bar.enter(Vec3::new(0.5, 3.0, 0.15), up).is_some());
    assert!(bar.enter(Vec3::new(0.15, 3.0, 0.5), up).is_none());
}

#[test]
fn bounds_wrap_the_model() {
    let [x, y, z, w, h, d] = post().bounds();
    assert!((x - (0.5 - 0.065 - PAD as f32)).abs() < 1e-5 && (z - x).abs() < 1e-6);
    assert!((w - (0.13 + 2.0 * PAD as f32)).abs() < 1e-5 && (d - w).abs() < 1e-6);
    assert!((y - (0.5 - 0.425 - PAD as f32)).abs() < 1e-5 && (h - (0.85 + 2.0 * PAD as f32)).abs() < 1e-5);
}
