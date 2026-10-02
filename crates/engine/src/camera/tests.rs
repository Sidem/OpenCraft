use super::*;

#[test]
fn crouching_lowers_the_eye_gradually() {
    let mut glide = CrouchGlide::default();
    let standing = Vec3::new(1.0, 10.0, 2.0);
    let crouched = Vec3::new(1.0, 10.0 - CROUCH_DIP, 2.0);
    // The body's eye drops at once; the camera's first frame has barely moved.
    let first = glide.apply(crouched, true, 1.0 / 60.0);
    assert!(first.y > crouched.y + 0.2 * CROUCH_DIP, "first frame {}", first.y);
    assert!(first.y < standing.y);
    // It settles on the crouched height.
    let mut eye = first;
    for _ in 0..60 {
        eye = glide.apply(crouched, true, 1.0 / 60.0);
    }
    assert!((eye.y - crouched.y).abs() < 0.001);
    // Standing up eases back the same way, and x and z are untouched.
    let up = glide.apply(standing, false, 1.0 / 60.0);
    assert!(up.y < standing.y);
    assert_eq!((up.x, up.z), (1.0, 2.0));
}

#[test]
fn standing_eye_is_never_moved() {
    let mut glide = CrouchGlide::default();
    let eye = Vec3::new(0.0, 5.0, 0.0);
    for dt in [0.0, 1.0 / 144.0, 0.1] {
        assert_eq!(glide.apply(eye, false, dt).y, 5.0);
    }
}

/// Runs the third-person camera for `seconds` at 60 frames a second and returns where it ended.
fn settle(view: &mut ThirdPerson, eye: Vec3, look: Vec3, seconds: f64, solid: impl Fn(IVec3) -> bool + Copy) -> Vec3 {
    let mut cam = eye;
    for _ in 0..(seconds * 60.0) as usize {
        cam = view.camera(eye, eye - Vec3::new(0.0, 1.62, 0.0), look, 1.0 / 60.0, solid);
    }
    cam
}

#[test]
fn first_person_stays_in_the_head_and_draws_no_avatar() {
    let mut view = ThirdPerson::default();
    let eye = Vec3::new(3.0, 70.0, 3.0);
    assert_eq!(settle(&mut view, eye, Vec3::new(0.0, 0.0, -1.0), 1.0, |_| false), eye);
    assert!(view.feet.is_none());
}

#[test]
fn third_person_backs_away_gradually_over_the_right_shoulder() {
    let mut view = ThirdPerson { distance: 4.0, ..Default::default() };
    let eye = Vec3::new(3.0, 70.0, 3.0);
    let look = Vec3::new(0.0, 0.0, -1.0);
    // One frame in, the camera has hardly left the head (no sudden jump of the view).
    let first = view.camera(eye, eye, look, 1.0 / 60.0, |_| false);
    assert!((first - eye).length() < 0.5);
    // Then it settles four blocks behind and slightly right, leaving the player left of the aim.
    let cam = settle(&mut view, eye, look, 4.0, |_| false);
    assert!((cam - Vec3::new(3.96, 70.0, 7.0)).length() < 0.05, "camera at {cam:?}");
    assert!(view.feet.is_some());
}

#[test]
fn shoulder_path_and_new_obstructions_keep_the_camera_out_of_terrain() {
    let eye = Vec3::new(3.5, 70.5, 3.5);
    let look = Vec3::new(0.0, 0.0, -1.0);
    let mut view = ThirdPerson { distance: 6.0, ..Default::default() };
    settle(&mut view, eye, look, 4.0, |_| false);
    // Introduce a wall along the shoulder path after the camera has already backed out.
    let cam = view.camera(eye, eye, look, 1.0 / 60.0, |p| p.x >= 4 && p.z >= 5);
    assert!(cam.x < 4.0 || cam.z < 5.0, "camera is immediately outside the new wall: {cam:?}");
    assert!((view.aim_eye(eye, look) - cam).length() < 0.001);
}

#[test]
fn a_wall_behind_the_player_pulls_the_camera_in() {
    let mut view = ThirdPerson { distance: 6.0, ..Default::default() };
    let eye = Vec3::new(3.5, 70.5, 3.5);
    let look = Vec3::new(0.0, 0.0, -1.0);
    // A wall of blocks two cells behind the eye (z >= 6).
    let cam = settle(&mut view, eye, look, 4.0, |p| p.z >= 6);
    let behind = cam.z - eye.z;
    assert!(behind > 1.0 && behind < 2.5 - CLEARANCE + 0.05, "{behind} blocks behind");
    assert!(cam.z < 6.0, "the camera stays out of the wall");
}
