use super::*;

fn floor_at_zero(_x: i32, y: i32, _z: i32) -> bool {
    y < 0
}

#[test]
fn falls_onto_floor() {
    let mut bb = Aabb::from_feet(Vec3::new(0.5, 1.0, 0.5), 0.3, 1.8);
    let moved = move_axis(&mut bb, 1, -5.0, &mut floor_at_zero);
    assert!((moved + 1.0).abs() < 1e-9, "moved {moved}");
    assert!(bb.min.y.abs() < 1e-9);
    // Resting on the floor: further downward motion is blocked entirely.
    assert_eq!(move_axis(&mut bb, 1, -0.1, &mut floor_at_zero), 0.0);
}

#[test]
fn blocked_by_wall() {
    let mut wall = |x: i32, _y: i32, _z: i32| x >= 3;
    let mut bb = Aabb::from_feet(Vec3::new(1.5, 0.0, 0.5), 0.3, 1.8);
    let moved = move_axis(&mut bb, 0, 5.0, &mut wall);
    assert!((bb.max.x - 3.0).abs() < 1e-9);
    assert!((moved - 1.2).abs() < 1e-9);
    let mut wall_neg = |x: i32, _y: i32, _z: i32| x < 0;
    let mut bb = Aabb::from_feet(Vec3::new(1.5, 0.0, 0.5), 0.3, 1.8);
    move_axis(&mut bb, 0, -5.0, &mut wall_neg);
    assert!(bb.min.x.abs() < 1e-9);
}

#[test]
fn can_leave_overlapping_block() {
    let mut solid = |x: i32, y: i32, z: i32| (x, y, z) == (0, 0, 0);
    let mut bb = Aabb::centered(Vec3::new(0.5, 0.5, 0.5), 0.2);
    assert!((move_axis(&mut bb, 1, 1.0, &mut solid) - 1.0).abs() < 1e-9);
}
