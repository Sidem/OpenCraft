//! Worn gear shows on the robot where it is worn, in its tier's colours, and a bare robot carries nothing.

use super::*;
use crate::avatars::model::push_avatar_muzzle;
use crate::factory::INSTANCE_FLOATS;
use crate::item::{EXO_FRAME, HAULER_PACK_MK3, TOOL_BELT_MK2, WORK_GLOVES_MK4, XRAY_GOGGLES};

const AT: Vec3 = Vec3::new(0.0, 0.0, -4.0);

fn draw(worn: [ItemId; SLOTS], m: &Motion) -> Vec<f32> {
    let mut out = Vec::new();
    push_avatar_muzzle(&mut out, AT, 0.0, 0.0, m, &Outfit { hands: [ItemId::NONE; 2], worn }, &mut None);
    out
}

/// Boxes drawn with `layer` on any face.
fn with_layer(boxes: &[f32], layer: u16) -> Vec<&[f32]> {
    boxes.chunks_exact(INSTANCE_FLOATS).filter(|b| b[8..11].contains(&f32::from(layer))).collect()
}

fn dressed() -> [ItemId; SLOTS] {
    let mut worn = [ItemId::NONE; SLOTS];
    (worn[BACK], worn[BOOTS], worn[TORSO], worn[equipment::BELT]) =
        (HAULER_PACK_MK3, SPRING_BOOTS, EXO_FRAME, TOOL_BELT_MK2);
    (worn[JET], worn[GLOVES], worn[HEAD]) = (JETPACK, WORK_GLOVES_MK4, XRAY_GOGGLES);
    worn
}

#[test]
fn every_worn_item_is_drawn_on_the_body_and_a_bare_robot_carries_no_pack() {
    let still = Motion::default();
    let bare = draw([ItemId::NONE; SLOTS], &still);
    let full = draw(dressed(), &still);
    assert!(
        with_layer(&bare, tex::AVATAR_PACK).len() <= 2,
        "only the hip pouches: the field pack belongs to a worn pack"
    );
    for slot in 0..SLOTS {
        let mut one = [ItemId::NONE; SLOTS];
        one[slot] = dressed()[slot];
        let more = draw(one, &still).len() - bare.len();
        assert!(more >= INSTANCE_FLOATS, "{} shows", equipment::SLOT_NAMES[slot]);
    }
    for b in full.chunks_exact(INSTANCE_FLOATS) {
        let (x, y, z) = (f64::from(b[0]) - AT.x, f64::from(b[1]) - AT.y, f64::from(b[2]) - AT.z);
        assert!(x.hypot(z) < 0.75 && (-0.05..2.2).contains(&y), "a part strays from the body: {x:.2} {y:.2} {z:.2}");
    }
    for tier in [2, 1, 3] {
        assert!(!with_layer(&full, tex::stripe(tier)).is_empty(), "the Mk{} stripe shows", tier + 1);
    }
}

#[test]
fn packs_sit_on_the_back_the_belt_in_front_and_a_thrusting_jetpack_flames_from_its_nozzles() {
    let still = Motion::default();
    let full = draw(dressed(), &still);
    let mean_z = |layer: u16| {
        let parts = with_layer(&full, layer);
        parts.iter().map(|b| f64::from(b[2]) - AT.z).sum::<f64>() / parts.len() as f64
    };
    assert!(mean_z(tex::stripe(2)) > 0.1, "the hauler's band is on the back (+z)");
    assert!(mean_z(tex::stripe(1)) < -0.1, "the tool belt's band is in front (-z)");
    let flying = Motion { thrust: 1.0, ..Default::default() };
    assert!(!with_layer(&draw(dressed(), &flying), tex::LAMP_YELLOW).is_empty(), "the jetpack flames");
    let mut no_flight = dressed();
    no_flight[JET] = ItemId::NONE;
    assert!(with_layer(&draw(no_flight, &flying), tex::LAMP_YELLOW).is_empty(), "no pack, no flame");
}
