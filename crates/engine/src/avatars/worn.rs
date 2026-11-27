//! Worn gear on the Kestrel rig (GEAR-BODY): each equipment slot has a body anchor, and what is worn there is drawn
//! from its own item model (`item_models/wear.rs`, tier stripes included), so a player can tell at a glance what
//! another carries. With nothing worn the robot is bare: no pack, no thruster bells, no antenna. Presentation only.
//! Anchors are in the rig's frame before its yaw (x right, y up, -z ahead), sizes in blocks a model unit (`K_*`).
//! To show a new slot: an anchor and a call from `model.rs`.

use super::grip::{mul, placed_parts, rx_m, rz_m, yaw_m};
use super::model::Rig;
use super::motion::Motion;
use crate::block::tex;
use crate::equipment::{self, BACK, BOOTS, GLOVES, HEAD, JET, SLOTS, TORSO};
use crate::item::{ItemId, HOVER_PACK, JETPACK, SPRING_BOOTS};
use crate::math::Vec3;
use std::f64::consts::PI;

/// Model units to blocks: a pack on the back, a hauler moved down to the waist under a flight pack, the tool belt, the
/// exo frame (shoulder bar to hips), goggles on the helmet.
const K_PACK: f64 = 0.62;
const K_LOW_PACK: f64 = 0.48;
const K_BELT: f64 = 0.46;
const K_EXO: f64 = 0.88;
const K_GOGGLES: f64 = 0.45;

/// What a player holds and wears.
pub(super) struct Outfit {
    /// The right and then the left hand.
    pub hands: [ItemId; 2],
    pub worn: [ItemId; SLOTS],
}

impl Outfit {
    /// Hands only, nothing worn.
    #[cfg(test)]
    pub fn bare(hands: [ItemId; 2]) -> Outfit {
        Outfit { hands, worn: [ItemId::NONE; SLOTS] }
    }
}

/// The back: the flight pack (with its flame while it thrusts), the hauler pack (lower when a flight pack is worn) with
/// its comms antenna.
pub(super) fn back(rig: &mut Rig, worn: &[ItemId; SLOTS], torso: Vec3, waist: Vec3, lean: f64, roll: f64, m: &Motion) {
    let turn = mul(rz_m(roll), rx_m(lean));
    let flight = worn[JET];
    if flight != ItemId::NONE {
        let at = torso + Vec3::new(0.0, 0.0, 0.26);
        rig.model_parts(&placed_parts(flight, at, turn, K_PACK));
        if m.thrust > 0.02 {
            flame(rig, flight, at, m);
        }
    }
    let pack = worn[BACK];
    if pack != ItemId::NONE {
        let (at, k) = if flight == ItemId::NONE {
            (torso + Vec3::new(0.0, -0.02, 0.26), K_PACK)
        } else {
            (waist + Vec3::new(0.0, 0.0, 0.25), K_LOW_PACK)
        };
        rig.model_parts(&placed_parts(pack, at, turn, k));
        let tip = at + Vec3::new(-0.14, 0.2 * k / K_PACK + 0.17, -0.02);
        rig.part(tip, [0.024, 0.25, 0.024], lean, 0.15, 1.0, tex::AVATAR_JOINT);
        rig.part(tip + Vec3::new(-0.02, 0.13, 0.0), [0.052, 0.042, 0.052], 0.0, 0.0, 1.0, tex::AVATAR_TRIM);
    }
}

/// The waist: the exo frame from hips to shoulders (its back to the back) and the tool belt round the front.
pub(super) fn waist(rig: &mut Rig, worn: &[ItemId; SLOTS], waist: Vec3, lean: f64) {
    let exo = worn[TORSO];
    if exo != ItemId::NONE {
        let parts = placed_parts(exo, waist + Vec3::new(0.0, 0.105, 0.0), mul(rx_m(lean * 0.5), yaw_m(PI)), K_EXO);
        // Its leg struts would stand still while the legs walk: the frame shows from the hips up.
        let hips = waist.y - 0.08;
        rig.model_parts(&parts.into_iter().filter(|p| p.center.y > hips).collect::<Vec<_>>());
    }
    let belt = worn[equipment::BELT];
    if belt != ItemId::NONE {
        rig.model_parts(&placed_parts(belt, waist + Vec3::new(0.0, -0.035, -0.18), yaw_m(PI), K_BELT));
    }
}

/// Goggles on the helmet above the visor, turned with the head.
pub(super) fn head(rig: &mut Rig, worn: &[ItemId; SLOTS], head: Vec3, tilt: f64) {
    let goggles = worn[HEAD];
    if goggles == ItemId::NONE {
        return;
    }
    let at = rig.world(head);
    let yaw = rig.head_yaw;
    let mut on_head = Rig { out: rig.out, at, yaw, head_yaw: yaw, muzzle: None };
    let turn = mul(rx_m(tilt), yaw_m(PI));
    let parts = placed_parts(goggles, super::model::rotate(Vec3::new(0.0, 0.1, -0.16), tilt), turn, K_GOGGLES);
    on_head.model_parts(&parts);
}

/// Boots on one foot: spring boots an ember cuff and a coil behind the heel, servo boots a teal cuff and the ankle
/// servo on the outer side (`side` is -1 left, 1 right).
pub(super) fn ankle(rig: &mut Rig, worn: &[ItemId; SLOTS], ankle: Vec3, side: f64) {
    let boots = worn[BOOTS];
    if boots == ItemId::NONE {
        return;
    }
    let spring = boots == SPRING_BOOTS;
    let cuff = if spring { tex::MAT_EMBER } else { tex::MAT_AETHER };
    rig.part(ankle + Vec3::new(0.0, 0.06, 0.0), [0.18, 0.05, 0.19], 0.0, 0.0, 1.0, cuff);
    if spring {
        for (y, turn) in [(-0.12, 0.0), (-0.09, 0.8)] {
            rig.round_part(ankle + Vec3::new(0.0, y, 0.12), [0.09, 0.025, 0.09], 0.0, turn, 1.0, tex::MAT_METAL);
        }
    } else {
        rig.part(ankle + Vec3::new(side * 0.1, -0.02, 0.0), [0.04, 0.08, 0.08], 0.0, 0.0, 1.0, tex::MAT_METAL);
        rig.part(ankle + Vec3::new(side * 0.122, -0.02, 0.0), [0.006, 0.03, 0.03], 0.0, 0.0, 1.0, tex::LAMP_BLUE);
    }
}

/// Gloves on one forearm: a cuff in the gloves' tier stripe just above the wrist.
pub(super) fn wrist(rig: &mut Rig, worn: &[ItemId; SLOTS], elbow: Vec3, wrist: Vec3) {
    let Some(tier) = equipment::ladder_tier(worn[GLOVES]) else { return };
    let cuff = if tier == 0 { tex::MAT_FABRIC } else { tex::stripe(tier) };
    rig.link(wrist + (elbow - wrist) * 0.3, wrist + (elbow - wrist) * 0.08, [0.15, 0.0, 0.16], cuff);
}

/// The flame under a thrusting flight pack at `at`: from both jetpack nozzles, or a blue wash under the hover fans.
fn flame(rig: &mut Rig, flight: ItemId, at: Vec3, m: &Motion) {
    let long = m.thrust * (0.42 + 0.07 * m.flicker.sin() + 0.04 * (m.flicker * 2.7).cos());
    let (spread, low, outer, inner) = match flight {
        JETPACK => (0.12, -0.33, tex::LAMP_YELLOW, tex::LAMP),
        HOVER_PACK => (0.32, -0.12, tex::LAMP_BLUE, tex::LAMP_BLUE),
        _ => return,
    };
    for (side, beat) in [(-1.0, 0.0), (1.0, 1.9)] {
        let nozzle = at + Vec3::new(side * spread * K_PACK, low * K_PACK, 0.0);
        let core = long * (0.55 + 0.06 * (m.flicker + beat).sin());
        rig.part(nozzle - Vec3::new(0.0, long / 2.0, 0.0), [0.06, long as f32, 0.08], 0.0, 0.0, 2.2, outer);
        rig.part(nozzle - Vec3::new(0.0, core / 2.0, 0.0), [0.035, core as f32, 0.05], 0.0, 0.0, 2.0, inner);
    }
}

#[cfg(test)]
mod tests;
