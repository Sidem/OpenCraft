use super::*;
use crate::block::tex;
use crate::factory::INSTANCE_FLOATS;
use crate::item::{IRON_PICKAXE, STEEL_PICKAXE, STONE_PICKAXE};
use crate::player::Player;

#[test]
fn pickaxe_point_leads_the_downstroke_in_the_arm_swing_plane_for_every_tier() {
    // The moment of the pickaxe strike (`tool_hold.rs`).
    let pose = Motion { mining: 1.0, work_phase: 0.72 * std::f64::consts::TAU, ..Default::default() };
    for held in [STONE_PICKAXE, IRON_PICKAXE, STEEL_PICKAXE] {
        for yaw in [0.0, 0.8] {
            let mut boxes = Vec::new();
            model::push_avatar(&mut boxes, Vec3::new(0.0, 0.0, -4.0), yaw, 0.0, &pose, [held, ItemId::NONE]);
            let tool = &boxes[boxes.len() - 6 * INSTANCE_FLOATS..];
            let socket = &tool[INSTANCE_FLOATS..];
            let point = &tool[5 * INSTANCE_FLOATS..];
            let to =
                Vec3::new((point[0] - socket[0]) as f64, (point[1] - socket[1]) as f64, (point[2] - socket[2]) as f64);
            let (s, c) = yaw.sin_cos();
            // In the right hand (+x), the working arm, not the off hand.
            assert!(socket[0] as f64 * c + (socket[2] as f64 + 4.0) * s > 0.1, "the pick is in the right hand");
            assert!(to.x * s - to.z * c > 0.1, "the pick's point faces forward into the strike: {to:?}");
            assert!((to.x * c + to.z * s).abs() < 0.001, "the head stays in the arm's swing plane");
            assert!(to.y < 0.0, "the point leads downward, not the flat face");
        }
    }
}

#[test]
fn head_looks_up_and_down_with_the_view_and_at_the_offset_crosshair_surface() {
    let at = Vec3::new(0.0, 0.0, -4.0);
    let visor = |boxes: &[f32]| {
        boxes.chunks_exact(INSTANCE_FLOATS).find(|p| p[9] == tex::AVATAR_VISOR as f32).unwrap().to_vec()
    };
    for pitch in [-0.7, 0.7] {
        let mut boxes = Vec::new();
        model::push_avatar(&mut boxes, at, 0.4, pitch, &Motion::default(), [ItemId::NONE; 2]);
        let part = visor(&boxes);
        assert!((part[12] as f64 - pitch).abs() < 0.001);
        assert_eq!(part[12].is_sign_positive(), pitch.is_sign_positive());
    }
    let focus = Vec3::new(2.0, 4.0, -9.0);
    let pose = Motion { focus: Some(focus), ..Default::default() };
    let mut boxes = Vec::new();
    model::push_avatar(&mut boxes, at, 0.0, 0.0, &pose, [ItemId::NONE; 2]);
    let part = visor(&boxes);
    let (s, c) = (part[3] as f64).sin_cos();
    let (sp, cp) = (part[12] as f64).sin_cos();
    let facing = Vec3::new(s * cp, sp, -c * cp);
    let to = focus - (at + Vec3::new(0.0, 1.62, 0.0));
    assert!((facing - to * (1.0 / to.length())).length() < 0.001, "visor faces the crosshair surface");
}

#[test]
fn gait_is_driven_by_actual_movement_and_blends_out_when_stopped() {
    let mut body = Player::new(Vec3::ZERO);
    body.on_ground = true;
    let mut pose = Motion::default();
    body.input.forward = 1.0; // Pushing against a wall must not walk in place.
    pose.advance(&body, 0.1);
    assert_eq!(pose.stride, 0.0);
    body.vel.z = -4.3;
    for _ in 0..60 {
        pose.advance(&body, 1.0 / 60.0);
    }
    assert!(pose.stride > 0.95 && pose.phase > 0.0);
    body.vel = Vec3::ZERO;
    let phase = pose.phase;
    for _ in 0..60 {
        pose.advance(&body, 1.0 / 60.0);
    }
    assert!(pose.stride < 0.001);
    assert_eq!(pose.phase, phase);
}

#[test]
fn crouch_folds_the_rig_and_keeps_the_boots_on_the_ground() {
    let mut standing = Vec::new();
    let mut crouched = Vec::new();
    let at = Vec3::new(0.0, 0.0, -4.0);
    model::push_avatar(&mut standing, at, 0.0, 0.0, &Motion::default(), [ItemId::NONE; 2]);
    let pose = Motion { crouch: 1.0, ..Default::default() };
    model::push_avatar(&mut crouched, at, 0.0, 0.0, &pose, [ItemId::NONE; 2]);
    let top = |v: &[f32]| v.chunks_exact(INSTANCE_FLOATS).map(|p| p[1] + p[5] * 0.5).fold(0.0, f32::max);
    assert!(top(&standing) - top(&crouched) > 0.25);
    for v in [&standing, &crouched] {
        assert!(v.iter().all(|f| f.is_finite()));
        let boots: Vec<_> =
            v.chunks_exact(INSTANCE_FLOATS).filter(|p| p[9] == tex::AVATAR_JOINT as f32 && p[6] > 0.3).collect();
        assert_eq!(boots.len(), 2);
        assert!(boots.iter().all(|p| (p[1] - p[5] * 0.5).abs() < 0.01));
    }
}

#[test]
fn work_gestures_articulate_the_arm_and_the_equipped_tool_follows_it() {
    let at = Vec3::new(0.0, 0.0, -4.0);
    let mut body = Player::new(Vec3::ZERO);
    body.on_ground = true;
    body.gesture = 1;
    let mut pose = Motion::default();
    for _ in 0..30 {
        pose.advance(&body, 1.0 / 60.0);
    }
    let mut first = Vec::new();
    model::push_avatar(&mut first, at, 0.0, 0.0, &pose, [IRON_PICKAXE, ItemId::NONE]);
    for _ in 0..10 {
        pose.advance(&body, 1.0 / 60.0);
    }
    let mut second = Vec::new();
    model::push_avatar(&mut second, at, 0.0, 0.0, &pose, [IRON_PICKAXE, ItemId::NONE]);
    assert_ne!(first, second);
    assert_ne!(&first[first.len() - INSTANCE_FLOATS..], &second[second.len() - INSTANCE_FLOATS..]);
    body.gesture = 2;
    pose.advance(&body, 0.05);
    assert!(pose.using > 0.5);
    body.gesture = 0;
    pose.advance(&body, 0.05);
    assert!(pose.using > 0.2, "a released placement keeps its follow-through");
}

#[test]
fn the_first_person_hand_holds_the_item_low_right_ahead_and_inside_the_body() {
    for (yaw, pitch) in [(0.0, 0.0), (1.1, 0.9), (-2.4, -1.2)] {
        let mut empty = Vec::new();
        view_hand::push_view_hand(&mut empty, yaw, pitch, &Motion::default(), ItemId::NONE);
        let mut boxes = Vec::new();
        let swing = Motion { mining: 1.0, work_phase: PI, ..Default::default() };
        view_hand::push_view_hand(&mut boxes, yaw, pitch, &swing, IRON_PICKAXE);
        assert!(boxes.len() > empty.len(), "the pick is drawn as well as the hand");
        let (s, c) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let ahead = Vec3::new(s * cp, sp, -c * cp);
        let right = Vec3::new(c, 0.0, s);
        // Every item, at rest and through its whole stroke.
        let strokes: Vec<Motion> = (0..12)
            .map(|i| i as f64 / 12.0 * std::f64::consts::TAU)
            .flat_map(|a| {
                [
                    Motion { mining: 1.0, work_phase: a, ..Default::default() },
                    Motion { using: 1.0, use_phase: a, ..Default::default() },
                ]
            })
            .chain([Motion::default()])
            .collect();
        for item in (1..crate::item::ITEM_COUNT as u16).map(ItemId) {
            for motion in &strokes {
                boxes.clear();
                view_hand::push_view_hand(&mut boxes, yaw, pitch, motion, item);
                for b in boxes.chunks_exact(INSTANCE_FLOATS) {
                    let at = Vec3::new(b[0] as f64, b[1] as f64, b[2] as f64);
                    let name = crate::item::name(item);
                    // Within the body's half-width all round, so inside its box whichever way it faces.
                    let inside = (at.x * at.x + at.z * at.z).sqrt() < crate::player::HALF_WIDTH;
                    // The body's box reaches 0.18 above the eye (1.8 tall, the eye at 1.62).
                    let phase = motion.work_phase + motion.use_phase;
                    assert!(
                        inside && at.y < 0.18,
                        "{name} stays inside the body, so never in a wall: {at:?} at {phase}"
                    );
                    assert!(
                        at.x * ahead.x + at.y * ahead.y + at.z * ahead.z > 0.0,
                        "{name} in front of the eye: {at:?}"
                    );
                }
            }
        }
        let hand = &empty[INSTANCE_FLOATS..];
        let hand = Vec3::new(hand[0] as f64, hand[1] as f64, hand[2] as f64);
        assert!(hand.x * right.x + hand.z * right.z > 0.05, "on the right of the view");
    }
}

#[test]
fn pov_arm_and_hand_match_the_characters_body_materials_and_have_articulated_fingers() {
    let mut view = Vec::new();
    view_hand::push_view_hand(&mut view, 0.0, 0.0, &Motion::default(), ItemId::NONE);
    let parts: Vec<&[f32]> = view.chunks_exact(INSTANCE_FLOATS).collect();
    // Forearm (5) + palm/dorsal/knuckle (3) + thumb (2) + 4 articulated fingers (8) = 18 parts.
    assert!(parts.len() >= 18, "modelled forearm and articulated hand have 18+ parts, got {}", parts.len());
    for layer in [tex::AVATAR_SKIN, tex::AVATAR_SUIT, tex::AVATAR_HELMET, tex::AVATAR_JOINT, tex::AVATAR_TRIM] {
        assert!(parts.iter().any(|p| p[9] == layer as f32), "POV arm/hand includes Kestrel body layer {layer}");
    }
}

#[test]
fn bare_hand_mining_clenches_into_a_fist_and_drives_forward() {
    let mut idle = Vec::new();
    let mut strike = Vec::new();
    view_hand::push_view_hand(&mut idle, 0.0, 0.0, &Motion::default(), ItemId::NONE);
    let punch = Motion { mining: 1.0, work_phase: 0.58 * std::f64::consts::TAU, ..Default::default() };
    view_hand::push_view_hand(&mut strike, 0.0, 0.0, &punch, ItemId::NONE);
    let palm_z = |v: &[f32]| v[6 * INSTANCE_FLOATS + 2] as f64;
    assert!(palm_z(&strike) < palm_z(&idle) - 0.015, "punch drives the fist forward toward the target");
    // Fingers curl tighter into a fist: their bounding span along local reach shrinks compared to open fingers.
    let finger_span = |v: &[f32]| {
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for p in v.chunks_exact(INSTANCE_FLOATS).skip(9) {
            lo = lo.min(p[2]);
            hi = hi.max(p[2]);
        }
        hi - lo
    };
    assert!(finger_span(&strike) < finger_span(&idle), "clenched fist curls fingers tighter than idle hand");
}

#[test]
fn axe_and_pickaxe_have_distinct_mining_animations_in_pov_and_third_person() {
    use crate::item::IRON_AXE;
    let at_phase = |item: ItemId, t: f64| {
        let mut out = Vec::new();
        let m = Motion { mining: 1.0, work_phase: t * std::f64::consts::TAU, ..Default::default() };
        view_hand::push_view_hand(&mut out, 0.0, 0.0, &m, item);
        // The tool head is the last emitted part.
        let p = &out[out.len() - INSTANCE_FLOATS..];
        Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64)
    };
    let axe_dx = (at_phase(IRON_AXE, 0.50).x - at_phase(IRON_AXE, 0.70).x).abs();
    let pick_dx = (at_phase(IRON_PICKAXE, 0.52).x - at_phase(IRON_PICKAXE, 0.72).x).abs();
    assert!(axe_dx > pick_dx * 1.4, "axe sweeps wider horizontally ({axe_dx:.3}) than pickaxe ({pick_dx:.3})");

    // In 3rd-person too, an axe chop swings the avatar's hand laterally across the body unlike a vertical pick.
    let mut avatar_axe_a = Vec::new();
    let mut avatar_axe_b = Vec::new();
    let ma = Motion { mining: 1.0, work_phase: 0.50 * std::f64::consts::TAU, ..Default::default() };
    let mb = Motion { mining: 1.0, work_phase: 0.70 * std::f64::consts::TAU, ..Default::default() };
    model::push_avatar(&mut avatar_axe_a, Vec3::new(0.0, 0.0, -4.0), 0.0, 0.0, &ma, [IRON_AXE, ItemId::NONE]);
    model::push_avatar(&mut avatar_axe_b, Vec3::new(0.0, 0.0, -4.0), 0.0, 0.0, &mb, [IRON_AXE, ItemId::NONE]);
    let ax_a = avatar_axe_a[avatar_axe_a.len() - INSTANCE_FLOATS] as f64;
    let ax_b = avatar_axe_b[avatar_axe_b.len() - INSTANCE_FLOATS] as f64;
    assert!(ax_a - ax_b > 0.15, "3rd-person axe head sweeps right-to-left: {ax_a:.3} -> {ax_b:.3}");
}

#[test]
fn the_first_person_left_hand_holds_its_item_low_left_ahead_and_inside_the_body_and_hides_when_empty() {
    let mut none = Vec::new();
    view_hand::push_view_left_hand(&mut none, 0.0, 0.0, &Motion::default(), ItemId::NONE, 0.0);
    assert!(none.is_empty(), "an empty left hand is not drawn");
    let mut boxes = Vec::new();
    for (yaw, pitch) in [(0.0f64, 0.0f64), (1.1, 0.9), (-2.4, -1.2)] {
        let (s, c) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let ahead = Vec3::new(s * cp, sp, -c * cp);
        let right = Vec3::new(c, 0.0, s);
        for item in (1..crate::item::ITEM_COUNT as u16).map(ItemId) {
            for raise in [0.0, 1.0] {
                boxes.clear();
                view_hand::push_view_left_hand(&mut boxes, yaw, pitch, &Motion::default(), item, raise);
                let name = crate::item::name(item);
                for b in boxes.chunks_exact(INSTANCE_FLOATS) {
                    let at = Vec3::new(b[0] as f64, b[1] as f64, b[2] as f64);
                    let inside = (at.x * at.x + at.z * at.z).sqrt() < crate::player::HALF_WIDTH;
                    assert!(inside && at.y < 0.18, "{name} stays inside the body: {at:?}");
                    assert!(
                        at.x * ahead.x + at.y * ahead.y + at.z * ahead.z > 0.0,
                        "{name} in front of the eye: {at:?}"
                    );
                }
                let arm = &boxes[INSTANCE_FLOATS..];
                let arm = Vec3::new(arm[0] as f64, arm[1] as f64, arm[2] as f64);
                assert!(arm.x * right.x + arm.z * right.z < -0.05, "on the left of the view");
            }
        }
    }
}
