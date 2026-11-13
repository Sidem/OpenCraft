use super::*;
use crate::helpers::drone::trip::Stop;
use crate::item::{COPPER_WIRE, IRON_PLATE};

fn trip(store: bool) -> Trip {
    let stop =
        |x, item, tick| Stop { at: IVec3::new(x, 70, 0), item, count: 10, tick, slot: crate::helpers::drone::ANY_SLOT };
    Trip {
        store,
        from: IVec3::new(0, 70, 0),
        stops: vec![stop(20, IRON_PLATE, 120), stop(30, COPPER_WIRE, 200)],
        hold: Vec::new(),
        ticks: 0,
        total: 320,
    }
}

#[test]
fn the_flight_starts_and_ends_at_home_and_hovers_over_each_stop_at_its_tick() {
    let home = Vec3::new(0.5, 72.0, 0.5);
    let t = trip(false);
    assert!((flight(&t, 0.0, home).0 - home).length() < 1e-9);
    assert!((flight(&t, 320.0, home).0 - home).length() < 1e-9);
    for s in &t.stops {
        let over = s.at.as_vec3() + Vec3::new(0.5, 1.0 + HOVER - DIP, 0.5);
        assert!((flight(&t, f64::from(s.tick), home).0 - over).length() < 1e-6, "at its lowest over the box");
    }
}

#[test]
fn the_flight_never_jumps_between_ticks() {
    let home = Vec3::new(0.5, 72.0, 0.5);
    let t = trip(true);
    let mut last = flight(&t, 0.0, home).0;
    for i in 1..=3200 {
        let p = flight(&t, f64::from(i) * 0.1, home).0;
        assert!((p - last).length() < 0.5, "a smooth path at tick {}", f64::from(i) * 0.1);
        last = p;
    }
}

#[test]
fn it_carries_what_it_fetched_after_the_stop_and_what_it_stores_before() {
    let home = Vec3::new(0.5, 72.0, 0.5);
    let (fetch, store) = (trip(false), trip(true));
    assert_eq!(flight(&fetch, 60.0, home).1, None);
    assert_eq!(flight(&fetch, 150.0, home).1, Some(IRON_PLATE));
    assert_eq!(flight(&fetch, 300.0, home).1, Some(COPPER_WIRE));
    assert_eq!(flight(&store, 60.0, home).1, Some(IRON_PLATE));
    assert_eq!(flight(&store, 150.0, home).1, Some(COPPER_WIRE));
    assert_eq!(flight(&store, 300.0, home).1, None);
}

#[test]
fn it_leans_into_speed_and_push_but_never_past_its_limit() {
    let still = tilt(Vec3::ZERO, Vec3::ZERO);
    assert!((still - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-9, "level at rest");
    let cruise = tilt(Vec3::ZERO, Vec3::new(16.0, 0.0, 0.0));
    assert!(cruise.x > 0.1, "drag tilts it forward at cruise");
    let hard = tilt(Vec3::new(500.0, 0.0, -300.0), Vec3::ZERO);
    let lean = hard.y.acos();
    assert!(lean <= MAX_LEAN + 1e-9 && lean > MAX_LEAN - 1e-3, "clamped: {lean}");
}

#[test]
fn the_spring_follows_a_moving_target_without_lagging_at_cruise() {
    let mut f = Flyer::default();
    let target = |t: f64| (Vec3::new(10.0 * t, 70.0, 0.0), None);
    f.fly(0.0, target, true);
    for i in 1..=180 {
        f.fly(f64::from(i) / 60.0, target, true);
    }
    assert!((f.pos - Vec3::new(30.0, 70.0, 0.0)).length() < 0.05, "on the target: {:?}", f.pos);
    assert!((f.vel.x - 10.0).abs() < 0.1);
    assert!(f.up.x > 0.0, "leaning into its flight");
}

#[test]
fn it_hovers_beside_and_behind_the_right_shoulder_whichever_way_the_player_faces() {
    for yaw in [0.0, 1.0, 2.5, -2.0] {
        let mut body = Player::new(Vec3::new(0.0, 70.0, 0.0));
        body.yaw = yaw;
        let head = body.eye();
        let at = shoulder(&body, head, 0.0) - head;
        let look = body.look_dir();
        let ahead = unit(Vec3::new(look.x, 0.0, look.z));
        let right = Vec3::new(-ahead.z, 0.0, ahead.x);
        assert!(dot(at, ahead) < -0.2 && dot(at, right) > 0.8, "yaw {yaw}: {at:?}");
    }
}
