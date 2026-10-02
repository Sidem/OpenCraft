use super::*;
use crate::block::STONE;

#[test]
fn shoulder_crosshair_selects_its_surface_with_the_original_hand_reach() {
    let eye = Vec3::new(0.5, 1.5, 0.5);
    let camera = Vec3::new(1.46, 1.5, 4.5);
    let dir = Vec3::new(0.0, 0.0, -1.0);
    let block = IVec3::new(1, 1, -3);
    assert_eq!(target(eye, camera, dir, |p| (p == block).then_some(STONE)).unwrap().block, block);
    let far = IVec3::new(1, 1, -6);
    assert!(target(eye, camera, dir, |p| (p == far).then_some(STONE)).is_none());
}

#[test]
fn shoulder_view_cannot_reach_through_a_wall_beside_the_player() {
    let eye = Vec3::new(-0.5, 1.5, 0.5);
    let camera = Vec3::new(1.2, 1.5, 4.5);
    let block = IVec3::new(1, 1, -3);
    let wall = IVec3::new(0, 1, -1);
    assert!(target(eye, camera, Vec3::new(0.0, 0.0, -1.0), |p| (p == block || p == wall).then_some(STONE)).is_none());
}
