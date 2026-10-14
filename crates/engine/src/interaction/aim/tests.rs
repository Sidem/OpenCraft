use super::*;
use crate::block::STONE;
use crate::interaction::REACH;

#[test]
fn shoulder_crosshair_selects_its_surface_with_the_original_hand_reach() {
    let eye = Vec3::new(0.5, 1.5, 0.5);
    let camera = Vec3::new(1.46, 1.5, 4.5);
    let dir = Vec3::new(0.0, 0.0, -1.0);
    let block = IVec3::new(1, 1, -3);
    assert_eq!(target(eye, camera, dir, REACH, |p| (p == block).then_some(STONE), |_| None).unwrap().block, block);
    let far = IVec3::new(1, 1, -6);
    assert!(target(eye, camera, dir, REACH, |p| (p == far).then_some(STONE), |_| None).is_none());
}

#[test]
fn a_slender_block_lets_the_ray_pass_beside_its_model() {
    let eye = Vec3::new(0.5, 1.5, 0.5);
    let dir = Vec3::new(0.0, 0.0, -1.0);
    let (pole, wall) = (IVec3::new(0, 1, -2), IVec3::new(0, 1, -4));
    let probe = |p: IVec3| (p == pole || p == wall).then_some(STONE);
    let post = |p: IVec3| {
        let mut out = Vec::new();
        crate::factory::push_box(&mut out, Vec3::ZERO, 0.0, [0.13, 0.85, 0.13], 0.0, [0; 3], false);
        (p == pole).then(|| Boxes::from_instances(&out)).flatten()
    };
    assert_eq!(target(eye, eye, dir, REACH, probe, post).unwrap().block, pole);
    let aside = Vec3::new(0.85, 1.5, 0.5);
    assert_eq!(target(aside, aside, dir, REACH, probe, post).unwrap().block, wall);
}

#[test]
fn shoulder_view_cannot_reach_through_a_wall_beside_the_player() {
    let eye = Vec3::new(-0.5, 1.5, 0.5);
    let camera = Vec3::new(1.2, 1.5, 4.5);
    let block = IVec3::new(1, 1, -3);
    let wall = IVec3::new(0, 1, -1);
    assert!(target(
        eye,
        camera,
        Vec3::new(0.0, 0.0, -1.0),
        REACH,
        |p| (p == block || p == wall).then_some(STONE),
        |_| None
    )
    .is_none());
}
