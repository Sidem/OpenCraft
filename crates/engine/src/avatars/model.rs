//! Procedural Kestrel rig: an articulated survey robot with a sculpted helmet, layered armor and asymmetric pack.
//! All parts use the shared instance draw call. Legs solve a two-link chain to planted/lifting feet; arms pivot at
//! shoulder and elbow, and BOTH the 3rd-person arms and 1st-person POV arm share `hand.rs` so materials, gauntlet
//! armor and the articulated hand match 1:1.

use super::grip::{self, apply, mul, rx_m, rz_m, yaw_m};
use super::hand::{self, HandGrip, HandPart};
use super::motion::Motion;
use super::tool_hold::{self, Pose, Stroke};
use crate::block::tex;
use crate::factory::{make_round, push_box, Axis, INSTANCE_FLOATS};
use crate::item::ItemId;
use crate::math::Vec3;
use std::f64::consts::{FRAC_PI_2, PI};

const NEAR: f64 = 0.9;
const LEG_LINK: f64 = 0.35;

/// `hands` is what the right and then the left hand hold.
pub(super) fn push_avatar(out: &mut Vec<f32>, at: Vec3, yaw: f64, pitch: f64, m: &Motion, hands: [ItemId; 2]) -> bool {
    let hip_y = 0.83 - m.crouch * 0.26;
    if (at + Vec3::new(0.0, hip_y + 0.78, 0.0)).length() < NEAR {
        return false;
    }
    let mut rig = Rig { out, at, yaw, head_yaw: yaw };
    let stroke = tool_hold::stroke_of(hands[0]);
    let work_arm = tool_hold::arm(hands[0], m.work_phase);
    let work_pose = tool_hold::pose(stroke, m.work_phase, m.mining);
    // Subtle torso pitch follow-through when swinging a tool (rocks slightly back on wind-up, leans into strike).
    let work_lean = -work_arm * 0.09 * m.mining;
    let work_roll = work_pose.sweep * 0.14;
    let lean = -0.11 * m.lean - 0.18 * m.crouch + work_lean;
    let roll = -m.strafe * 0.06 + work_roll;

    let v = Vec3::new;
    // Waist, utility belt, buckle and side hip pouches.
    let waist = v(0.0, hip_y + 0.09, 0.03 * m.crouch);
    rig.part(waist, [0.37, 0.18, 0.29], 0.0, 0.0, 1.0, tex::AVATAR_JOINT);
    rig.part(waist + v(0.0, 0.02, 0.0), [0.395, 0.055, 0.315], 0.0, 0.0, 1.0, tex::AVATAR_HELMET);
    rig.part(waist + v(0.0, 0.02, -0.165), [0.085, 0.06, 0.024], 0.0, 0.0, 1.0, tex::AVATAR_TRIM);
    for side in [-1.0, 1.0] {
        rig.part(waist + v(side * 0.215, 0.005, 0.02), [0.055, 0.11, 0.14], 0.0, 0.0, 0.92, tex::AVATAR_PACK);
    }

    // Layered chest chassis, collar yoke, amber harness straps and chest telemetry bar.
    let torso = waist + v(0.0, 0.26, -0.04 * m.crouch);
    rig.part(torso, [0.46, 0.43, 0.3], lean, roll, 1.22, tex::AVATAR_SKIN);
    rig.part(torso + v(0.0, 0.035, -0.162), [0.29, 0.25, 0.03], lean, roll, 0.76, tex::AVATAR_SUIT);
    rig.part(torso + v(0.0, 0.20, -0.01), [0.36, 0.065, 0.27], lean, roll, 0.92, tex::AVATAR_SUIT);
    for side in [-1.0, 1.0] {
        rig.part(torso + v(side * 0.145, 0.06, -0.155), [0.045, 0.26, 0.032], lean, roll, 0.95, tex::AVATAR_HELMET);
    }
    rig.part(torso + v(0.0, 0.10, -0.184), [0.10, 0.035, 0.016], lean, roll, 1.0, tex::AVATAR_TRIM);

    // Offset field pack, cylindrical canister, thruster bells and comms antenna.
    rig.part(torso + v(-0.045, -0.01, 0.23), [0.32, 0.37, 0.17], lean, roll, 0.82, tex::AVATAR_PACK);
    rig.part(torso + v(-0.045, -0.02, 0.323), [0.24, 0.065, 0.025], lean, roll, 1.0, tex::AVATAR_HELMET);
    rig.round_part(torso + v(0.20, -0.03, 0.21), [0.105, 0.29, 0.105], lean, roll, 0.95, tex::AVATAR_SKIN);
    rig.round_part(torso + v(0.20, 0.125, 0.21), [0.085, 0.045, 0.085], lean, roll, 0.9, tex::AVATAR_JOINT);
    for side in [-1.0, 1.0] {
        rig.round_part(
            torso + v(-0.045 + side * 0.09, -0.20, 0.245),
            [0.075, 0.07, 0.075],
            lean,
            roll,
            0.75,
            tex::AVATAR_JOINT,
        );
    }
    rig.part(torso + v(-0.15, 0.30, 0.24), [0.024, 0.25, 0.024], lean, 0.15, 1.0, tex::AVATAR_JOINT);
    rig.part(torso + v(-0.17, 0.43, 0.24), [0.052, 0.042, 0.052], 0.0, 0.0, 1.0, tex::AVATAR_TRIM);
    if m.thrust > 0.02 {
        let long = m.thrust * (0.42 + 0.07 * m.flicker.sin() + 0.04 * (m.flicker * 2.7).cos());
        for (side, beat) in [(-1.0, 0.0), (1.0, 1.9)] {
            let nozzle = torso + v(-0.045 + side * 0.09, -0.22, 0.25);
            let core = long * (0.55 + 0.06 * (m.flicker + beat).sin());
            rig.part(nozzle - v(0.0, long / 2.0, 0.0), [0.06, long as f32, 0.08], 0.0, 0.0, 2.2, tex::LAMP_YELLOW);
            rig.part(nozzle - v(0.0, core / 2.0, 0.0), [0.035, core as f32, 0.05], 0.0, 0.0, 2.0, tex::LAMP);
        }
    }

    // Neck and sculpted survey helmet (visor stays first AVATAR_VISOR part for crosshair-tracking tests).
    let neck = waist + v(0.0, 0.55, -0.05 * m.crouch);
    rig.part(neck, [0.15, 0.1, 0.16], 0.0, 0.0, 1.0, tex::AVATAR_JOINT);
    let head = neck + v(0.0, 0.15, 0.0);
    let (head_yaw, tilt) = m.focus.map_or((yaw, pitch), |focus| {
        let to = focus - rig.world(head);
        (to.x.atan2(-to.z), to.y.atan2((to.x * to.x + to.z * to.z).sqrt()))
    });
    rig.head_yaw = head_yaw;
    rig.head_part(head, Vec3::ZERO, [0.43, 0.25, 0.32], tilt, 0.0, 0.82, tex::AVATAR_SKIN);
    rig.head_part(head, v(0.0, 0.145, -0.01), [0.34, 0.06, 0.29], tilt, 0.0, 0.88, tex::AVATAR_HELMET);
    rig.head_part(head, v(0.0, 0.082, -0.175), [0.38, 0.035, 0.055], tilt, 0.0, 0.92, tex::AVATAR_HELMET);
    rig.head_part(head, v(0.0, 0.005, -0.167), [0.37, 0.115, 0.025], tilt, 0.0, 0.88, tex::AVATAR_VISOR);
    rig.head_part(head, v(0.0, 0.013, -0.184), [0.27, 0.032, 0.012], tilt, 0.0, 1.0, tex::AVATAR_TRIM);
    rig.head_part(head, v(0.0, -0.105, -0.125), [0.26, 0.06, 0.11], tilt, 0.0, 0.84, tex::AVATAR_JOINT);

    for side in [-1.0, 1.0] {
        // Helmet ear pods + amber rim.
        rig.head_part(head, v(side * 0.22, -0.008, 0.0), [0.085, 0.18, 0.29], tilt, side * 0.28, 0.8, tex::AVATAR_SUIT);
        rig.head_part(
            head,
            v(side * 0.26, -0.005, 0.0),
            [0.024, 0.11, 0.13],
            tilt,
            side * 0.28,
            0.9,
            tex::AVATAR_HELMET,
        );

        // Two-link IK legs and grounded treaded boots.
        let phase = m.phase + if side > 0.0 { PI } else { 0.0 };
        let gait = m.stride * (1.0 - m.air * 0.75) * (1.0 - m.swim * 0.4);
        let hip = Vec3::new(side * 0.17, hip_y, 0.035 * m.crouch);
        let ankle = Vec3::new(
            side * 0.18,
            0.17 + phase.cos().max(0.0) * gait * 0.13 + m.air * 0.12,
            -phase.sin() * gait * 0.23 + m.crouch * 0.09 + side * m.air * 0.07,
        );
        let delta = ankle - hip;
        let distance = delta.length().max(0.001);
        let bend = (LEG_LINK * LEG_LINK - distance * distance * 0.25).max(0.0).sqrt();
        let knee = (hip + ankle) * 0.5 + Vec3::new(0.0, delta.z / distance * bend, -delta.y.abs() / distance * bend);
        rig.part(hip, [0.17, 0.15, 0.19], 0.0, 0.0, 1.0, tex::AVATAR_JOINT);
        rig.link(hip, knee, [0.18, 0.0, 0.19], tex::AVATAR_SUIT);
        rig.part(knee, [0.19, 0.13, 0.20], 0.0, 0.0, 0.85, tex::AVATAR_HELMET);
        rig.link(knee, ankle, [0.15, 0.0, 0.17], tex::AVATAR_SKIN);
        rig.part(ankle + Vec3::new(0.0, 0.02, 0.0), [0.155, 0.055, 0.17], 0.0, 0.0, 0.92, tex::AVATAR_JOINT);
        rig.part(ankle + Vec3::new(0.0, -0.07, -0.05), [0.22, 0.19, 0.34], 0.0, 0.0, 0.82, tex::AVATAR_JOINT);
        rig.part(ankle + Vec3::new(0.0, -0.025, -0.19), [0.18, 0.06, 0.035], 0.0, 0.0, 0.9, tex::AVATAR_HELMET);

        // Articulated arms + shared forearm gauntlet + modelled hand.
        let shoulder = waist + Vec3::new(side * 0.32, 0.43, -0.02 * m.crouch);
        let walk = phase.sin() * gait * 0.48;
        let mut upper = walk + m.crouch * 0.28 + m.air * 0.25 + m.swim * 0.8;
        let mut elbow = 0.22 + gait * 0.2 + m.crouch * 0.3;
        let mut abduct = 0.0;
        let mut item_pose = Pose::default();
        if side > 0.0 {
            let work_upper = match stroke {
                Stroke::Pick => 1.25 - work_arm * 0.88 + pitch * 0.65,
                Stroke::Axe => 1.08 - work_arm * 0.65 + pitch * 0.60,
                Stroke::Dig => 0.85 + work_arm * 0.42 + pitch * 0.55,
                Stroke::Sweep => 1.05 + pitch * 0.55,
                Stroke::Punch => 0.85 + work_arm * 0.65 + pitch * 0.60,
            };
            let work_elbow = match stroke {
                Stroke::Pick => 0.85 - work_arm * 0.25,
                Stroke::Axe => 0.72 - work_arm * 0.20,
                Stroke::Dig => 0.55 - work_arm * 0.30,
                Stroke::Sweep => 0.45,
                Stroke::Punch => 0.95 - work_arm * 0.65,
            };
            upper = upper * (1.0 - m.mining) + work_upper * m.mining;
            elbow += work_elbow * m.mining;
            // Lateral shoulder sweep for diagonal axe chops and sickle sweeps.
            if matches!(stroke, Stroke::Axe | Stroke::Sweep) {
                abduct = work_pose.sweep * 0.55;
                item_pose = Pose { roll: work_pose.roll, sweep: work_pose.sweep * 0.5, ..Pose::default() };
            } else if stroke == Stroke::Dig {
                item_pose = Pose { lean: work_pose.lean, ..Pose::default() };
            }
            upper = upper * (1.0 - m.using) + (1.35 + pitch * 0.6 + m.use_phase.sin() * 0.13) * m.using;
            elbow *= 1.0 - m.using * 0.8;
        } else {
            // Left arm counterbalances right-arm strikes naturally.
            upper += m.mining * (0.28 - work_arm * 0.22) + m.using * 0.55;
            elbow += m.mining * 0.35;
        }
        let swing_m = yaw_m(abduct);
        let joint = shoulder + apply(swing_m, rotate(Vec3::new(0.0, -0.27, 0.0), upper));
        let wrist = joint + apply(swing_m, rotate(Vec3::new(0.0, -0.26, 0.0), upper + elbow));
        let pauldron = if side < 0.0 { tex::AVATAR_HELMET } else { tex::AVATAR_SKIN };
        rig.part(shoulder, [0.24, 0.18, 0.26], lean, side * -0.2, 0.75, pauldron);
        rig.part(
            shoulder + Vec3::new(side * 0.06, 0.06, 0.0),
            [0.12, 0.035, 0.22],
            lean,
            side * -0.2,
            0.9,
            tex::AVATAR_HELMET,
        );
        rig.link(shoulder, joint, [0.13, 0.0, 0.15], tex::AVATAR_SUIT);
        rig.part(joint, [0.145, 0.13, 0.155], upper, 0.0, 1.0, tex::AVATAR_JOINT);

        // Shared Kestrel forearm gauntlet (matches POV arm 1:1).
        let dorsal = apply(swing_m, Vec3::new(side, 0.0, 0.0));
        rig.hand_parts(&hand::forearm_parts(joint, wrist, dorsal, 1.0));

        // Shared modelled articulated hand.
        let held = hands[usize::from(side < 0.0)];
        let hand_grip = grip_for(held, if side > 0.0 { m.mining } else { 0.0 });
        let arm_pitch = upper + elbow;
        let base_hand = hand::cols(Vec3::new(0.0, 0.0, side), Vec3::new(side, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let hand_basis = mul(swing_m, mul(rx_m(arm_pitch), mul(base_hand, rz_m(item_pose.roll * 0.5))));
        rig.hand_parts(&hand::hand_parts(wrist, hand_basis, 1.0, side, hand_grip));

        // The right hand (+x, the working arm) holds the selected item; tools seat right inside the curled fingers.
        let anchor = if tool_hold::hold(held).is_some() { wrist + apply(hand_basis, hand::GRIP_POCKET) } else { wrist };
        rig.held_item(anchor, arm_pitch - FRAC_PI_2, 0.0, held, 1.0, false, &item_pose);
    }
    true
}

/// First-person POV right arm & modelled hand: shares the exact Kestrel forearm gauntlet and articulated hand from
/// `hand.rs`, scaled to [`VIEW_SCALE`] so it looks life-size while staying strictly inside the player's body cylinder.
/// Active, tool-appropriate animations drive both the hand and elbow; no walking bob (motion-sickness safe).
pub(super) fn push_view_hand(out: &mut Vec<f32>, yaw: f64, pitch: f64, m: &Motion, held: ItemId) {
    let mut rig = Rig { out, at: Vec3::ZERO, yaw, head_yaw: yaw };
    let tool = tool_hold::hold(held).map(|h| h.stroke);
    let stroke = tool.unwrap_or(Stroke::Punch);
    let sweeping = stroke == Stroke::Sweep && m.using > m.mining;
    let pose = if sweeping {
        tool_hold::pose(Stroke::Sweep, m.use_phase, m.using)
    } else {
        tool_hold::pose(stroke, m.work_phase, m.mining)
    };
    let push = if sweeping { 0.0 } else { m.using * m.use_phase.sin().abs() };
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
    let hand = Vec3::new(
        0.31 + sweep_x + breath_x - 0.03 * push,
        -0.31 + 0.045 * push + eff_lift + 0.04 * eff_thrust + breath_y,
        -0.46 + near - 0.36 * eff_thrust - 0.03 * push,
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
}

fn grip_for(held: ItemId, mining: f64) -> HandGrip {
    if tool_hold::hold(held).is_some() {
        HandGrip::Tool
    } else if held != ItemId::NONE {
        HandGrip::Item
    } else if mining > 0.25 {
        HandGrip::Fist
    } else {
        HandGrip::Open
    }
}

/// How far a held item that isn't a tool tips its top towards the eye in first person, radians.
const VIEW_TIP_BACK: f64 = 0.45;

/// How much smaller and nearer the first-person hand is than an avatar's.
const VIEW_SCALE: f64 = 0.33;

struct Rig<'a> {
    out: &'a mut Vec<f32>,
    at: Vec3,
    yaw: f64,
    head_yaw: f64,
}

impl Rig<'_> {
    fn world(&self, center: Vec3) -> Vec3 {
        let (s, c) = self.yaw.sin_cos();
        self.at + Vec3::new(c * center.x - s * center.z, center.y, s * center.x + c * center.z)
    }

    #[allow(clippy::too_many_arguments)]
    fn part(&mut self, center: Vec3, size: [f32; 3], pitch: f64, roll: f64, taper: f32, layer: u16) {
        let center = self.world(center);
        push_box(self.out, center, self.yaw as f32, size, 0.0, [layer; 3], false);
        let i = self.out.len() - INSTANCE_FLOATS;
        self.out[i + 12] = pitch as f32;
        self.out[i + 13] = roll as f32;
        self.out[i + 14] = taper;
    }

    #[allow(clippy::too_many_arguments)]
    fn round_part(&mut self, center: Vec3, size: [f32; 3], pitch: f64, roll: f64, taper: f32, layer: u16) {
        self.part(center, size, pitch, roll, taper, layer);
        make_round(self.out, Axis::Up, taper);
    }

    #[allow(clippy::too_many_arguments)]
    fn head_part(&mut self, head: Vec3, offset: Vec3, size: [f32; 3], pitch: f64, roll: f64, taper: f32, layer: u16) {
        let at = self.world(head);
        let yaw = self.head_yaw;
        Rig { out: self.out, at, yaw, head_yaw: yaw }.part(rotate(offset, pitch), size, pitch, roll, taper, layer);
    }

    fn link(&mut self, from: Vec3, to: Vec3, mut size: [f32; 3], layer: u16) {
        let delta = to - from;
        size[1] = delta.length() as f32;
        self.part((from + to) * 0.5, size, (-delta.z).atan2(-delta.y), 0.0, 0.85, layer);
    }

    fn hand_parts(&mut self, parts: &[HandPart]) {
        for p in parts {
            self.part(p.center, p.size, p.pitch, p.roll, p.taper, p.layer);
            let i = self.out.len() - INSTANCE_FLOATS;
            self.out[i + 3] += p.yaw as f32;
        }
    }

    /// `held` in the hand at `hand` turned by `pitch`, at `scale` times its size on an avatar.
    #[allow(clippy::too_many_arguments)]
    fn held_item(&mut self, hand: Vec3, pitch: f64, tip: f64, held: ItemId, scale: f64, view: bool, pose: &Pose) {
        for p in grip::held_parts(held, hand, pitch, tip, scale, view, pose) {
            self.part(p.center, p.size, p.pitch, p.roll, 1.0, p.tex[1]);
            let i = self.out.len() - INSTANCE_FLOATS;
            self.out[i + 8..i + 11].copy_from_slice(&p.tex.map(f32::from));
            self.out[i + 3] += p.yaw as f32;
            if let Some(taper) = p.taper {
                make_round(self.out, Axis::Up, taper);
            }
        }
    }
}

fn rotate(p: Vec3, pitch: f64) -> Vec3 {
    let (s, c) = pitch.sin_cos();
    Vec3::new(p.x, p.y * c - p.z * s, p.y * s + p.z * c)
}
