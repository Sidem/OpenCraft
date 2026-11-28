use super::*;

const FOV: f64 = 70.0 * std::f64::consts::PI / 180.0;
const ASPECT: f64 = 16.0 / 9.0;

/// Where a point at normalised screen position (`sx`, `sy` in -1..=1) and `depth` along the view lies, for a camera
/// built the host's way (`render/mat4.ts` `viewRotation`: right = (cy, 0, sy), up = right × forward).
fn on_screen(yaw: f64, pitch: f64, sx: f64, sy: f64, depth: f64) -> Vec3 {
    let (s, c) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    let forward = Vec3::new(s * cp, sp, -c * cp);
    let right = Vec3::new(c, 0.0, s);
    let up = Vec3::new(-s * sp, cp, c * sp);
    let t = (FOV * 0.5).tan();
    (forward + right * (sx * t * ASPECT) + up * (sy * t)) * depth
}

#[test]
fn everything_on_screen_is_seen_even_its_corners() {
    for (yaw, pitch) in [(0.0, 0.0), (1.0, 0.4), (-2.5, -1.2), (3.0, 1.5)] {
        let cone = ViewCone::new(yaw, pitch, FOV, ASPECT);
        for (sx, sy) in [(0.0, 0.0), (1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (0.3, -0.9)] {
            for depth in [0.5, 10.0, 300.0] {
                // A corner lies exactly on the cone: a hair of radius absorbs float rounding.
                assert!(cone.sees(on_screen(yaw, pitch, sx, sy, depth), 1e-6), "yaw {yaw} pitch {pitch} at {sx},{sy}");
            }
        }
    }
}

#[test]
fn what_is_behind_or_beside_the_camera_is_not_seen_unless_its_ball_reaches_in() {
    let cone = ViewCone::new(0.0, 0.0, FOV, ASPECT);
    assert!(!cone.sees(Vec3::new(0.0, 0.0, 50.0), 10.0), "behind");
    assert!(!cone.sees(Vec3::new(80.0, 0.0, -5.0), 10.0), "far to the side");
    assert!(cone.sees(Vec3::new(0.0, 0.0, 5.0), 10.0), "a ball round the camera");
    assert!(cone.sees(Vec3::new(80.0, 0.0, -5.0), 80.0), "a ball big enough to reach the view");
}

#[test]
fn no_field_of_view_sees_everything() {
    let cone = ViewCone::new(0.0, 0.0, 0.0, ASPECT);
    assert!(cone.sees(Vec3::new(0.0, 0.0, 50.0), 0.0));
    assert!(ViewCone::ALL.sees(Vec3::new(0.0, -9.0, 1.0), 0.0));
}
