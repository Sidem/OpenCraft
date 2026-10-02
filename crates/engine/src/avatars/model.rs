//! Procedural Kestrel rig: a narrow-waisted survey robot with a wedge helmet and asymmetric pack.
//! All parts use the shared instance draw call. Legs solve a two-link chain to planted/lifting feet;
//! arms pivot at shoulder and elbow, and the equipped item's existing model follows the right hand.

use super::motion::Motion;
use crate::block::tex;
use crate::factory::{push_box, INSTANCE_FLOATS};
use crate::item::{self, ItemId};
use crate::item_models;
use crate::math::Vec3;
use crate::tools::{self, ToolKind};
use std::f64::consts::{FRAC_PI_2, PI};

const NEAR: f64 = 0.9;
const LEG_LINK: f64 = 0.35;

pub(super) fn push_avatar(out: &mut Vec<f32>, at: Vec3, yaw: f64, pitch: f64, m: &Motion, held: ItemId) -> bool {
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
    // Offset field pack and one canister give the back its own recognisable silhouette.
    rig.part(torso + Vec3::new(-0.045, -0.01, 0.23), [0.32, 0.37, 0.17], lean, roll, 0.8, tex::AVATAR_PACK);
    rig.part(torso + Vec3::new(-0.045, -0.03, 0.323), [0.23, 0.06, 0.025], lean, roll, 1.0, tex::AVATAR_HELMET);
    rig.part(torso + Vec3::new(0.2, -0.03, 0.21), [0.1, 0.3, 0.13], lean, roll, 0.85, tex::AVATAR_SKIN);
    rig.part(torso + Vec3::new(-0.15, 0.3, 0.24), [0.026, 0.25, 0.026], lean, 0.15, 1.0, tex::AVATAR_JOINT);
    rig.part(torso + Vec3::new(-0.17, 0.43, 0.24), [0.055, 0.045, 0.055], 0.0, 0.0, 1.0, tex::AVATAR_TRIM);
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
            let work = 1.15 + m.work_phase.cos() * 0.75 + pitch * 0.65;
            upper = upper * (1.0 - m.mining) + work * m.mining;
            elbow += (0.65 + m.work_phase.sin() * 0.45) * m.mining;
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
        if side > 0.0 {
            rig.held_item(wrist, upper + elbow - FRAC_PI_2, held);
        }
    }
    true
}

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

    fn held_item(&mut self, hand: Vec3, pitch: f64, held: ItemId) {
        let parts = item_models::parts(held);
        if parts.is_empty() {
            if let Some(def) = item::def(held).filter(|_| held != ItemId::NONE) {
                self.part(hand + rotate(Vec3::new(0.0, 0.08, -0.1), pitch), [0.21; 3], pitch, 0.0, 1.0, def.tex[1]);
                let i = self.out.len() - INSTANCE_FLOATS;
                self.out[i + 8..i + 11].copy_from_slice(&def.tex.map(f32::from));
            }
        } else {
            // Pick/axe edges run along model +X. Turn them into the arm's YZ swing plane so
            // their working end leads the stroke. Yaw -90 then roll equals wrist pitch then -90.
            let edge_on = tools::tool(held).is_some_and(|t| matches!(t.kind, ToolKind::Pickaxe | ToolKind::Axe));
            let at = self.world(hand);
            let yaw = self.yaw - if edge_on { FRAC_PI_2 } else { 0.0 };
            let mut grip = Rig { out: self.out, at, yaw, head_yaw: yaw };
            for p in parts {
                let offset = Vec3::new(p.center[0] as f64, p.center[1] as f64 + 0.28, p.center[2] as f64) * 0.55;
                let (center, tilt, roll) = if edge_on {
                    let (s, c) = pitch.sin_cos();
                    (Vec3::new(offset.x * c - offset.y * s, offset.x * s + offset.y * c, offset.z), 0.0, pitch)
                } else {
                    (rotate(offset, pitch), pitch, 0.0)
                };
                grip.part(center, p.size.map(|v| v * 0.55), tilt, roll, 1.0, p.tex[1]);
                let i = grip.out.len() - INSTANCE_FLOATS;
                grip.out[i + 8..i + 11].copy_from_slice(&p.tex.map(f32::from));
            }
        }
    }
}

fn rotate(p: Vec3, pitch: f64) -> Vec3 {
    let (s, c) = pitch.sin_cos();
    Vec3::new(p.x, p.y * c - p.z * s, p.y * s + p.z * c)
}
