use super::*;

fn flat(_x: i32, y: i32, _z: i32) -> bool {
    y < 10
}

fn settle(p: &mut Player, seconds: f64) {
    let steps = (seconds * 120.0) as usize;
    for _ in 0..steps {
        p.step(1.0 / 120.0, &mut flat);
    }
}

#[test]
fn lands_and_walks() {
    let mut p = Player::new(Vec3::new(0.5, 14.0, 0.5));
    settle(&mut p, 2.0);
    assert!(p.on_ground);
    assert!((p.pos.y - 10.0).abs() < 1e-6, "y = {}", p.pos.y);
    p.input.forward = 1.0;
    settle(&mut p, 1.0);
    assert!(p.pos.z < -3.0, "should walk towards -Z, z = {}", p.pos.z);
}

#[test]
fn jump_clears_one_block() {
    let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
    settle(&mut p, 0.2);
    p.input.jump = true;
    let mut apex: f64 = 0.0;
    for _ in 0..120 {
        p.step(1.0 / 120.0, &mut flat);
        apex = apex.max(p.pos.y - 10.0);
    }
    assert!(apex > 1.05 && apex < 1.6, "apex {apex}");
}
