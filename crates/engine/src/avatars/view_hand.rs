//! The first-person POV arms: the right arm works the selected item (strokes from `tool_hold.rs`), the left arm, shown
//! while the left hand holds something, rests its item low on the left and lifts it while `G` works it (GEAR-LEFT,
//! `left_hand.rs`). Both share the Kestrel forearm and hand of `hand.rs` with the avatar (`model.rs`), scaled to
//! [`VIEW_SCALE`] so they look life-size while staying strictly inside the player's body cylinder. No walking bob
//! (motion-sickness safe).

use super::grip::{self, apply, mul, rx_m, rz_m, yaw_m};
use super::hand::{self, HandGrip};
use super::model::{grip_for, rotate, Muzzle, Rig};
use super::motion::Motion;
use super::tool_hold::{self, Pose, Stroke};
use crate::item::ItemId;
use crate::math::Vec3;
use std::f64::consts::FRAC_PI_2;
/// First-person POV right arm & modelled hand: shares the exact Kestrel forearm gauntlet and articulated hand from
/// `hand.rs`, scaled to [`VIEW_SCALE`] so it looks life-size while staying strictly inside the player's body cylinder.
/// Active, tool-appropriate animations drive both the hand and elbow; no walking bob (motion-sickness safe).
pub(super) fn push_view_hand(out: &mut Vec<f32>, yaw: f64, pitch: f64, m: &Motion, held: ItemId) -> Option<Muzzle> {
    let mut rig = Rig { out, at: Vec3::ZERO, yaw, head_yaw: yaw, muzzle: None };
    let tool = tool_hold::hold(held).map(|h| h.stroke);
    let stroke = tool.unwrap_or(Stroke::Punch);
    let sweeping = stroke == Stroke::Sweep && m.using > m.mining;
    let pose = if sweeping {
        tool_hold::pose(Stroke::Sweep, m.use_phase, m.using)
    } else {
        tool_hold::pose(stroke, m.work_phase, m.mining)
    };
    let aiming = stroke == Stroke::Aim;
    let push = if sweeping || aiming { 0.0 } else { m.using * m.use_phase.sin().abs() };
    let brace = if aiming { m.using } else { 0.0 };
    // Subtle living idle breathing (quieted during active strokes; never tied to walking).
    let idle = (1.0 - m.mining).max(0.0) * (1.0 - m.using).max(0.0);
    let breath_y = m.breath.sin() * 0.008 * idle;
    let breath_x = (m.breath * 0.5).cos() * 0.004 * idle;

    // Camera-local coordinates before pitch: x right, y up, -z ahead.
    // Tools sit slightly nearer so their working heads stay inside the body across their full strike arc;
    // bulky non-tool items strike slightly lower and closer so they never poke above the eye while looking up.
    let item_hold = held != ItemId::NONE && tool.is_none();
    let near = if tool.is_some() { 0.09 } else { 0.0 };
    let thrust_scale = if item_hold { 0.45 } else { 1.0 };
    let strike_drop = if item_hold { -0.05 * pose.thrust.max(0.0) / 0.15 } else { 0.0 };
    let sweep_x = match stroke {
        Stroke::Axe => 0.24 * pose.sweep,
        Stroke::Sweep => 0.34 * pose.sweep,
        _ => 0.26 * pose.sweep,
    };
    let eff_thrust = pose.thrust * thrust_scale;
    let eff_lift = if item_hold { pose.lift * 0.3 + strike_drop } else { pose.lift };
    let aim_dx = if aiming { -0.03 } else { 0.0 };
    let aim_dy = if aiming { 0.03 } else { 0.0 };
    let hand = Vec3::new(
        0.31 + aim_dx + sweep_x + breath_x - 0.03 * push,
        -0.31 + aim_dy + 0.045 * push + eff_lift + 0.04 * eff_thrust + breath_y + 0.012 * brace,
        -0.46 + near - 0.36 * eff_thrust - 0.03 * push + 0.025 * brace,
    ) * VIEW_SCALE;
    // Elbow enters diagonally from the lower-right corner (natural shoulder line) while staying strictly ahead (z < 0).
    let elbow = Vec3::new(
        0.52 + sweep_x * 0.35 + breath_x * 0.5,
        -0.60 + eff_lift * 0.40 - 0.03 * eff_thrust,
        -0.21 - 0.12 * eff_thrust,
    ) * VIEW_SCALE;

    let at = |p: Vec3| rotate(p, pitch);
    let hand_grip = grip_for(held, m.mining);
    // Orient the hand: for handled tools, align the grip cylinder (-X_h) co-linearly with the tool handle (+Y) via
    // `rz_m(-FRAC_PI_2)` so the curled fingers and thumb stay wrapped right around the shaft through every stroke.
    let hand_own = match tool {
        Some(s) => {
            let (_, rest_lean, rest_roll) = grip::rest(s, true);
            // A shovel's rest_lean is near PI (blade forward-down); keep the wrist upright around the T-handle.
            let wrist_lean = if s == Stroke::Dig { 0.35 } else { rest_lean };
            let wrist_swing = if matches!(s, Stroke::Pick | Stroke::Axe) { -pose.swing * 0.75 } else { 0.0 };
            mul(
                yaw_m(pose.sweep),
                mul(
                    rx_m(wrist_lean + pose.lean + wrist_swing + 0.2 * push),
                    mul(rz_m(rest_roll + pose.roll), rz_m(-FRAC_PI_2)),
                ),
            )
        }
        None => {
            // Bare hand or block: follow the forearm's gentle inward diagonal so wrist and palm flow as one piece.
            let (rest_yaw, rest_pitch, rest_roll) = match hand_grip {
                HandGrip::Fist => (0.28, 0.24, -0.78),
                HandGrip::Item => (0.22, 0.28, -0.48),
                _ => (0.28, 0.28, -0.45),
            };
            mul(
                yaw_m(rest_yaw + pose.sweep * 0.90),
                mul(rx_m(rest_pitch + pose.lean * 0.85 + 0.25 * push), rz_m(rest_roll + pose.roll * 0.85)),
            )
        }
    };
    let tilt = rx_m(pitch);
    let hand_basis = mul(tilt, hand_own);

    // Forearm dorsal hint aligns the ivory ceramic gauntlet & wrist LED with the back of the hand.
    let dorsal = apply(hand_basis, Vec3::new(0.0, 1.0, 0.0));
    rig.hand_parts(&hand::forearm_parts(at(elbow), at(hand), dorsal, VIEW_SCALE));
    rig.hand_parts(&hand::hand_parts(at(hand), hand_basis, VIEW_SCALE, 1.0, hand_grip));

    // Tools attach directly inside the curled palm/finger pocket; blocks/items sit nicely above the cupped hand.
    let tip = VIEW_TIP_BACK + 0.18 * push;
    let item_anchor =
        if tool.is_some() { at(hand) + apply(hand_basis, hand::GRIP_POCKET * VIEW_SCALE) } else { at(hand) };
    rig.held_item(item_anchor, pitch, tip, held, VIEW_SCALE, true, &pose);
    rig.muzzle
}

/// First-person left arm, tucked low on the left edge so it stays out of the view at rest, drawn only while the left
/// hand holds something; it lifts gently while `G` works it (`raise`, 0 to 1). Light-emitting items are drawn even more
/// compactly so carrying a light never crowds the viewport.
pub(super) fn push_view_left_hand(out: &mut Vec<f32>, yaw: f64, pitch: f64, m: &Motion, held: ItemId, raise: f64) {
    if held == ItemId::NONE {
        return;
    }
    let mut rig = Rig { out, at: Vec3::ZERO, yaw, head_yaw: yaw, muzzle: None };
    let breath_y = m.breath.sin() * 0.006 * (1.0 - raise);
    let hand = Vec3::new(-0.37 + 0.05 * raise, -0.43 + 0.09 * raise + breath_y, -0.46 + 0.04 * raise) * VIEW_SCALE;
    let elbow = Vec3::new(-0.54 + 0.03 * raise, -0.64 + 0.05 * raise, -0.20) * VIEW_SCALE;
    let at = |p: Vec3| rotate(p, pitch);
    let basis = mul(rx_m(pitch), mul(yaw_m(-0.26), mul(rx_m(0.24 + 0.34 * raise), rz_m(0.52))));
    let dorsal = apply(basis, Vec3::new(0.0, 1.0, 0.0));
    rig.hand_parts(&hand::forearm_parts(at(elbow), at(hand), dorsal, VIEW_SCALE));
    rig.hand_parts(&hand::hand_parts(at(hand), basis, VIEW_SCALE, -1.0, HandGrip::Item));
    let is_light = crate::held_light::reach(held) > 0.0;
    let item_scale = VIEW_SCALE * if is_light { LEFT_LIGHT_SCALE } else { LEFT_ITEM_SCALE };
    rig.held_item(at(hand), pitch, VIEW_TIP_BACK * 0.6, held, item_scale, false, &Pose::default());
}
/// Scale of an instrument held in the left hand relative to `VIEW_SCALE` (without `VIEW_GROW`, keeping the view clear).
const LEFT_ITEM_SCALE: f64 = 0.78;
/// Scale of a light-emitting item in the left hand: compact so a lamp or torch never blocks the viewport.
const LEFT_LIGHT_SCALE: f64 = 0.56;
/// How far a held item that isn't a tool tips its top towards the eye in first person, radians.
pub(super) const VIEW_TIP_BACK: f64 = 0.45;

/// How much smaller and nearer the first-person hand is than an avatar's.
pub(super) const VIEW_SCALE: f64 = 0.33;
