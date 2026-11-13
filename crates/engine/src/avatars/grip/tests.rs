use super::*;
use crate::item::ITEM_COUNT;
use std::f64::consts::TAU;

fn near(a: M3, b: M3) -> bool {
    (0..3).all(|i| (0..3).all(|j| (a[i][j] - b[i][j]).abs() < 1e-6))
}

fn every_held_item() -> impl Iterator<Item = ItemId> {
    (1..ITEM_COUNT as u16).map(ItemId).filter(|&i| !model(i).is_empty())
}

/// Each part's box corners as drawn, relative to the hand.
fn corners(parts: &[HeldPart], hand: Vec3) -> Vec<Vec3> {
    let mut out = Vec::new();
    for p in parts {
        let m = compose(p.yaw, p.roll, p.pitch);
        for c in 0..8 {
            let s = |a: usize| if c >> a & 1 == 0 { -0.5 } else { 0.5 } * f64::from(p.size[a]);
            out.push(p.center - hand + apply(m, Vec3::new(s(0), s(1), s(2))));
        }
    }
    out
}

#[test]
fn a_turn_splits_into_the_shaders_yaw_roll_and_pitch_and_back() {
    let angles = [-3.0, -1.9, -0.7, 0.0, 0.4, 1.3, 2.2, 3.1];
    for a in angles {
        for b in angles {
            for c in angles {
                let m = mul(yaw_m(a), mul(rx_m(b), mul(yaw_m(c), rx_m(a * 0.7 - b))));
                let (y, r, p) = euler(m);
                assert!(near(compose(y, r, p), m), "{m:?}");
            }
        }
    }
    // Rolled a quarter turn exactly.
    let m = compose(0.4, FRAC_PI_2, 0.3);
    let (y, r, p) = euler(m);
    assert!(near(compose(y, r, p), m));
}

#[test]
fn every_item_in_hand_is_one_size() {
    let hand = Vec3::new(1.0, 2.0, 3.0);
    for item in every_held_item() {
        let pts = corners(&held_parts(item, hand, 0.0, 0.0, 1.0, false, &Pose::default()), hand);
        let span = |a: fn(&Vec3) -> f64| {
            let v: Vec<f64> = pts.iter().map(a).collect();
            v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
        };
        let largest = span(|p| p.x).max(span(|p| p.y)).max(span(|p| p.z));
        let tool = tool_hold::hold(item).is_some();
        let want = HELD_SIZE * if tool { TOOL_SIZE } else { 1.0 };
        assert!((largest - want).abs() < 1e-4, "{} is {largest:.3} across", item::name(item));
        if !tool {
            let bottom = pts.iter().map(|p| p.y).fold(f64::MAX, f64::min);
            let middle = bottom + span(|p| p.y) * 0.5;
            assert!((middle - HOLD_AT[1]).abs() < 1e-4, "{}'s middle is where it is held", item::name(item));
        }
    }
}

/// The tools with handles, each with its grip in model units and the part farthest from it (its head or blade).
fn tools() -> Vec<(ItemId, Vec3, usize)> {
    every_held_item()
        .filter_map(|item| {
            let grip = tool_hold::hold(item)?.grip;
            let grip = Vec3::new(grip[0], grip[1], grip[2]);
            let far = |p: &ModelPart| (Vec3::new(p.center[0], p.center[1], p.center[2]) - grip).length();
            let parts = model(item);
            let head = (0..parts.len()).max_by(|&a, &b| far(&parts[a]).total_cmp(&far(&parts[b])))?;
            Some((item, grip, head))
        })
        .collect()
}

#[test]
fn every_tool_is_gripped_on_its_handle_away_from_its_working_end() {
    let found = tools();
    assert!(found.len() >= 6, "picks, axes, shovels and sickles: {}", found.len());
    for (item, grip, head) in found {
        let parts = model(item);
        let name = item::name(item);
        let on = parts.iter().any(|p| {
            let half = p.size.map(|s| f64::from(s) * 0.5 + 0.02);
            let d = Vec3::new(grip.x - p.center[0], grip.y - p.center[1], grip.z - p.center[2]);
            // In the part's own frame (its basis is a turn: the transpose undoes it).
            (0..3).all(|j| (p.basis[0][j] * d.x + p.basis[1][j] * d.y + p.basis[2][j] * d.z).abs() <= half[j])
        });
        assert!(on, "{name} is gripped on a part, not in the air");
        let reach = Vec3::new(parts[head].center[0], parts[head].center[1], parts[head].center[2]) - grip;
        assert!(reach.length() > 0.4, "{name}'s grip is well away from its working end");
        let down = matches!(tool_hold::hold(item).unwrap().stroke, Stroke::Dig);
        assert_eq!(reach.y < 0.0, down, "{name}: a shovel is held at the top, a pick or axe low");
    }
}

#[test]
fn a_tools_head_comes_down_and_forward_onto_the_strike() {
    // First person, level: a pick's or axe's head is lower and further ahead when the stroke lands than when it is
    // drawn back, so the hit reads as its edge driving into the block; a shovel's blade (stabbed in by the hand,
    // `model.rs`) then levers up; a sickle's hook cuts across.
    let hand = Vec3::ZERO;
    for (item, _, head) in tools() {
        let stroke = tool_hold::hold(item).unwrap().stroke;
        let at =
            |t: f64| held_parts(item, hand, 0.0, 0.0, 0.33, true, &tool_hold::pose(stroke, t * TAU, 1.0))[head].center;
        let (back, hit) = match stroke {
            Stroke::Chop => (at(0.55), at(0.75)),
            Stroke::Dig => (at(0.3), at(0.65)),
            Stroke::Sweep => (at(0.5), at(0.72)),
        };
        let name = item::name(item);
        match stroke {
            Stroke::Sweep => assert!(hit.x < back.x - 0.01, "{name} cuts across to the left: {back:?} {hit:?}"),
            Stroke::Dig => assert!(hit.y > back.y + 0.005, "{name} prises its blade up: {back:?} {hit:?}"),
            Stroke::Chop => {
                assert!(hit.y < back.y - 0.02, "{name} comes down onto the strike: {back:?} {hit:?}");
                assert!(hit.z < back.z, "{name} drives forward: {back:?} {hit:?}");
            }
        }
    }
}

#[test]
fn a_held_item_turns_with_the_wrist_as_one_body() {
    // Turned back by the wrist's pitch, every part is where it is with the wrist level: in first person, looking up and
    // down leaves the item still on screen (drones and logs have turned and round parts).
    let hand = Vec3::new(0.2, -0.1, -0.2);
    for item in every_held_item() {
        let pose = Pose { swing: 0.3, lean: -0.2, sweep: 0.25, ..Pose::default() };
        let level = held_parts(item, hand, 0.0, 0.4, 0.33, true, &pose);
        for pitch in [-1.4, -0.6, 0.5, 1.2] {
            let back = rx_m(-pitch);
            for (a, b) in level.iter().zip(held_parts(item, hand, pitch, 0.4, 0.33, true, &pose)) {
                let at = apply(back, b.center - hand);
                assert!((at - (a.center - hand)).length() < 1e-6, "{} moved at {pitch}", item::name(item));
                let turned = mul(back, compose(b.yaw, b.roll, b.pitch));
                assert!(near(turned, compose(a.yaw, a.roll, a.pitch)), "{} turned at {pitch}", item::name(item));
            }
        }
    }
}
