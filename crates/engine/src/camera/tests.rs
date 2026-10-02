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
