use super::*;

#[test]
fn hits_floor_from_above() {
    let hit = raycast(Vec3::new(0.5, 5.5, 0.5), Vec3::new(0.0, -1.0, 0.0), 10.0, |p| (p.y <= 0).then_some(1)).unwrap();
    assert_eq!(hit.block, IVec3::new(0, 0, 0));
    assert_eq!(hit.normal, IVec3::new(0, 1, 0));
}

#[test]
fn respects_max_distance() {
    assert!(raycast(Vec3::new(0.5, 50.5, 0.5), Vec3::new(0.0, -1.0, 0.0), 5.0, |p| (p.y <= 0).then_some(1)).is_none());
}

#[test]
fn diagonal_hit_reports_entry_face() {
    let dir = Vec3::new(1.0, 0.0, 1.0) * (1.0 / 2f64.sqrt());
    let hit = raycast(Vec3::new(0.5, 0.5, 0.2), dir, 10.0, |p| (p.x >= 3).then_some(1)).unwrap();
    assert_eq!(hit.block.x, 3);
    assert_eq!(hit.normal, IVec3::new(-1, 0, 0));
}
