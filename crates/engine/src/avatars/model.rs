//! Procedural Kestrel rig: a narrow-waisted survey robot with a wedge helmet and asymmetric pack.
//! All parts use the shared instance draw call. Legs solve a two-link chain to planted/lifting feet;
//! arms pivot at shoulder and elbow, and each hand's item model (the selected slot right, `offhand.rs` left) follows it.

use super::grip;
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
    let lean = -0.11 * m.lean - 0.18 * m.crouch;
    let roll = -m.strafe * 0.06;
    let waist = Vec3::new(0.0, hip_y + 0.09, 0.03 * m.crouch);
    rig.part(waist, [0.37, 0.18, 0.29], 0.0, 0.0, 1.0, tex::AVATAR_JOINT);
    let torso = waist + Vec3::new(0.0, 0.26, -0.04 * m.crouch);
    rig.part(torso, [0.46, 0.43, 0.3], lean, roll, 1.22, tex::AVATAR_SKIN);
    rig.part(torso + Vec3::new(0.0, 0.04, -0.164), [0.27, 0.23, 0.025], lean, roll, 0.72, tex::AVATAR_SUIT);
    rig.part(torso + Vec3::new(0.0, 0.1, -0.183), [0.09, 0.035, 0.015], lean, roll, 1.0, tex::AVATAR_TRIM);
    // Offset field pack and one barrel give the back its own recognisable silhouette.
    rig.part(torso + Vec3::new(-0.045, -0.01, 0.23), [0.32, 0.37, 0.17], lean, roll, 0.8, tex::AVATAR_PACK);
    rig.part(torso + Vec3::new(-0.045, -0.03, 0.323), [0.23, 0.06, 0.025], lean, roll, 1.0, tex::AVATAR_HELMET);
    rig.part(torso + Vec3::new(0.2, -0.03, 0.21), [0.1, 0.3, 0.13], lean, roll, 0.85, tex::AVATAR_SKIN);
    rig.part(torso + Vec3::new(-0.15, 0.3, 0.24), [0.026, 0.25, 0.026], lean, 0.15, 1.0, tex::AVATAR_JOINT);
    rig.part(torso + Vec3::new(-0.17, 0.43, 0.24), [0.055, 0.045, 0.055], 0.0, 0.0, 1.0, tex::AVATAR_TRIM);
    if m.thrust > 0.02 {
        // The jetpack's two flames under the pack, flickering in length; they glow (factory::render GLOWING).
        let long = m.thrust * (0.42 + 0.07 * m.flicker.sin() + 0.04 * (m.flicker * 2.7).cos());
        for (side, beat) in [(-1.0, 0.0), (1.0, 1.9)] {
            let nozzle = torso + Vec3::new(-0.045 + side * 0.09, -0.2, 0.25);
            let core = long * (0.55 + 0.06 * (m.flicker + beat).sin());
            rig.part(
                nozzle - Vec3::new(0.0, long / 2.0, 0.0),
                [0.06, long as f32, 0.08],
                0.0,
                0.0,
                2.2,
                tex::LAMP_YELLOW,
            );
            rig.part(nozzle - Vec3::new(0.0, core / 2.0, 0.0), [0.035, core as f32, 0.05], 0.0, 0.0, 2.0, tex::LAMP);
        }
    }
    let neck = waist + Vec3::new(0.0, 0.55, -0.05 * m.crouch);
    rig.part(neck, [0.15, 0.1, 0.16], 0.0, 0.0, 1.0, tex::AVATAR_JOINT);
    let head = neck + Vec3::new(0.0, 0.15, 0.0);
    // A local -Z face rotates upward about +X: positive player pitch must stay positive here.
    let (head_yaw, tilt) = m.focus.map_or((yaw, pitch), |focus| {
        let to = focus - rig.world(head);
        (to.x.atan2(-to.z), to.y.atan2((to.x * to.x + to.z * to.z).sqrt()))
    });
    rig.head_yaw = head_yaw;
    rig.head_part(head, Vec3::ZERO, [0.43, 0.25, 0.32], tilt, 0.0, 0.8, tex::AVATAR_SKIN);
    rig.head_part(head, Vec3::new(0.0, 0.15, 0.0), [0.34, 0.055, 0.27], tilt, 0.0, 0.88, tex::AVATAR_HELMET);
    rig.head_part(head, Vec3::new(0.0, 0.005, -0.167), [0.37, 0.115, 0.025], tilt, 0.0, 0.88, tex::AVATAR_VISOR);
    rig.head_part(head, Vec3::new(0.0, 0.013, -0.184), [0.27, 0.032, 0.012], tilt, 0.0, 1.0, tex::AVATAR_TRIM);
    for side in [-1.0, 1.0] {
        rig.head_part(
            head,
            Vec3::new(side * 0.22, -0.008, 0.0),
            [0.085, 0.18, 0.29],
            tilt,
            side * 0.28,
            0.8,
            tex::AVATAR_SUIT,
        );
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
        rig.part(knee, [0.19, 0.13, 0.2], 0.0, 0.0, 0.85, tex::AVATAR_HELMET);
        rig.link(knee, ankle, [0.15, 0.0, 0.17], tex::AVATAR_SKIN);
        rig.part(ankle + Vec3::new(0.0, -0.07, -0.05), [0.22, 0.19, 0.34], 0.0, 0.0, 0.82, tex::AVATAR_JOINT);
        rig.part(ankle + Vec3::new(0.0, -0.025, -0.19), [0.18, 0.06, 0.035], 0.0, 0.0, 0.9, tex::AVATAR_HELMET);

        let shoulder = waist + Vec3::new(side * 0.32, 0.43, -0.02 * m.crouch);
        let walk = phase.sin() * gait * 0.48;
        let mut upper = walk + m.crouch * 0.28 + m.air * 0.25 + m.swim * 0.8;
        let mut elbow = 0.22 + gait * 0.2 + m.crouch * 0.3;
        if side > 0.0 {
            // Raised at -1, struck at 1, on the held tool's stroke (`tool_hold.rs`).
            let arm = tool_hold::arm(hands[0], m.work_phase);
            let work = 1.15 - arm * 0.75 + pitch * 0.65;
            upper = upper * (1.0 - m.mining) + work * m.mining;
            // Bent more when raised, so the head stays behind the hand; opening onto the strike.
            elbow += (0.75 - arm * 0.15) * m.mining;
            upper = upper * (1.0 - m.using) + (1.35 + pitch * 0.6 + m.use_phase.sin() * 0.13) * m.using;
            elbow *= 1.0 - m.using * 0.8;
        } else {
            upper += m.mining * 0.35 + m.using * 0.55;
        }
        let joint = shoulder + rotate(Vec3::new(0.0, -0.27, 0.0), upper);
        let wrist = joint + rotate(Vec3::new(0.0, -0.26, 0.0), upper + elbow);
        rig.part(
            shoulder,
            [0.24, 0.18, 0.26],
            lean,
            side * -0.2,
            0.75,
            if side < 0.0 { tex::AVATAR_HELMET } else { tex::AVATAR_SKIN },
        );
        rig.link(shoulder, joint, [0.13, 0.0, 0.15], tex::AVATAR_SUIT);
        rig.part(joint, [0.15, 0.13, 0.16], upper, 0.0, 1.0, tex::AVATAR_JOINT);
        rig.link(joint, wrist, [0.16, 0.0, 0.18], tex::AVATAR_SKIN);
        rig.part(wrist, [0.16, 0.15, 0.16], upper + elbow, 0.0, 0.9, tex::AVATAR_JOINT);
        // The right hand (+x, the working arm) holds the selected item.
        let held = hands[usize::from(side < 0.0)];
        rig.held_item(wrist, upper + elbow - FRAC_PI_2, 0.0, held, 1.0, false, &Pose::default());
    }
    true
}

/// The first-person view of the right hand: a forearm and glove in the lower right of the view holding `held`, all
/// at [`VIEW_SCALE`] of the avatar's size and as much nearer, so it looks life-size but stays inside the body (a wall
/// in front of the face is never nearer than the body's half-width). It turns with the camera; mining swings it a
/// little and using lifts it (pushing forward would reach into a wall), gently (some players feel sick from a bobbing view: no walking bob).
pub(super) fn push_view_hand(out: &mut Vec<f32>, yaw: f64, pitch: f64, m: &Motion, held: ItemId) {
    let mut rig = Rig { out, at: Vec3::ZERO, yaw, head_yaw: yaw };
    // A tool makes its own stroke (`tool_hold.rs`); anything else bobs evenly with each hit. A sickle cuts with the use
    // button (`sickle.rs`), so its sweep follows that too.
    let tool = tool_hold::hold(held).map(|h| h.stroke);
    let sweeping = tool == Some(Stroke::Sweep) && m.using > m.mining;
    let pose = match tool {
        Some(s) if sweeping => tool_hold::pose(s, m.use_phase, m.using),
        Some(s) => tool_hold::pose(s, m.work_phase, m.mining),
        None => Pose::default(),
    };
    let swing = if tool.is_some() { 0.0 } else { m.mining * (0.5 - 0.5 * m.work_phase.cos()) };
    let push = if sweeping { 0.0 } else { m.using * m.use_phase.sin().abs() };
    // In the camera's frame before its pitch: x right, y up, -z ahead. A tool's hand sits a little nearer, so its head
    // stays inside the body at the far end of its stroke; a shovel's stab drives the hand forward.
    let near = if tool.is_some() { 0.1 } else { 0.0 };
    let hand = Vec3::new(
        0.36 + 0.35 * pose.sweep,
        -0.3 - 0.05 * swing + 0.05 * push + pose.lift + 0.1 * pose.thrust,
        -0.5 + near - 0.35 * pose.thrust,
    ) * VIEW_SCALE;
    let elbow = Vec3::new(0.36, -0.68, -0.2) * VIEW_SCALE;
    let at = |p: Vec3| rotate(p, pitch);
    rig.link(at(elbow), at(hand), [(0.11 * VIEW_SCALE) as f32, 0.0, (0.13 * VIEW_SCALE) as f32], tex::AVATAR_SUIT);
    let glove = [0.11, 0.09, 0.12].map(|s| (s * VIEW_SCALE) as f32);
    rig.part(at(hand), glove, pitch, 0.0, 0.9, tex::AVATAR_JOINT);
    // A tool is posed by its stroke about its grip. Anything else tips its top towards the eye about its own middle, so a
    // flat item (a plate, a drone) shows its face rather than its edge.
    let tip = VIEW_TIP_BACK - 0.6 * swing;
    rig.held_item(at(hand), pitch, tip, held, VIEW_SCALE, true, &pose);
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

    /// `held` in the hand at `hand` turned by `pitch`, at `scale` times its size on an avatar (the first-person view's is
    /// smaller and nearer, so it looks the same size and never reaches into a wall). Every item is one size (`grip.rs`).
    #[allow(clippy::too_many_arguments)]
    fn held_item(&mut self, hand: Vec3, pitch: f64, tip: f64, held: ItemId, scale: f64, view: bool, pose: &Pose) {
        for p in grip::held_parts(held, hand, pitch, tip, scale, view, pose) {
            self.part(p.center, p.size, p.pitch, p.roll, 1.0, p.tex[1]);
            let i = self.out.len() - INSTANCE_FLOATS;
            self.out[i + 8..i + 11].copy_from_slice(&p.tex.map(f32::from));
            self.out[i + 3] += p.yaw as f32;
            if let Some(taper) = p.taper {
                // Already laid along its axis: only marks it round.
                make_round(self.out, Axis::Up, taper);
            }
        }
    }
}

fn rotate(p: Vec3, pitch: f64) -> Vec3 {
    let (s, c) = pitch.sin_cos();
    Vec3::new(p.x, p.y * c - p.z * s, p.y * s + p.z * c)
}
