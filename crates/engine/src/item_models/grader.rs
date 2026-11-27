//! 3D item & hotbar icon for the Telescoping Bucket-Wheel Site Grader (`SITE_GRADER_ITEM`):
//! miniature crawler undercarriage with steel grouser treads, circular slewing ring, weathered yellow machinery house
//! with glazed cab, rear concrete counterweight with hazard trim, steel A-frame gantry mast, telescoping truss boom,
//! and unmistakable circular burnt-orange digging wheel at the front tip. Back-to-front, resting at y = -0.5, all
//! within [-0.5, 0.5].

use super::shapes::cylinder;
use super::{part, Part};
use crate::block::tex;
use crate::factory::{Axis, Form};
use crate::item::{ItemId, SITE_GRADER_ITEM};

const fn bar_z(center: [f32; 3], length: f32, diameter: f32, layer: u16) -> Part {
    Part { center, size: [diameter, diameter, length], yaw: 0.0, tex: [layer; 3], form: Form::Round(Axis::Z, 1.0) }
}

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    if item == SITE_GRADER_ITEM {
        &GRADER_ICON
    } else {
        &[]
    }
}

const GRADER_ICON: [Part; 16] = [
    // Far crawler track + steel carbody deck + near crawler track (resting flush on y = -0.5).
    part([-0.10, -0.43, -0.24], [0.56, 0.14, 0.14], tex::GRADER_TREAD),
    part([-0.10, -0.40, 0.0], [0.46, 0.12, 0.34], tex::MAT_GUNMETAL),
    part([-0.10, -0.43, 0.24], [0.56, 0.14, 0.14], tex::GRADER_TREAD),
    // Slewing bearing ring.
    cylinder([-0.10, -0.29, 0.0], 0.34, 0.10, tex::MAT_GUNMETAL),
    // Rear concrete ballast counterweight with hazard cap.
    part([-0.37, -0.11, 0.0], [0.20, 0.20, 0.30], tex::CONCRETE),
    part([-0.37, 0.01, 0.0], [0.18, 0.05, 0.28], tex::GRADER_BUCKET),
    // Slewing ochre-yellow machinery house & louvered diesel engine shroud.
    part([-0.12, -0.14, -0.02], [0.36, 0.20, 0.36], tex::GRADER_LOUVRE),
    // Steel A-frame gantry legs & orange apex sheave drum.
    part([-0.14, 0.16, -0.11], [0.06, 0.42, 0.06], tex::STEEL),
    part([-0.14, 0.16, 0.11], [0.06, 0.42, 0.06], tex::STEEL),
    bar_z([-0.14, 0.36, 0.0], 0.26, 0.08, tex::GRADER_BUCKET),
    // Telescoping box-girder truss boom reaching out toward +x.
    Part {
        center: [0.12, 0.04, 0.0],
        size: [0.38, 0.11, 0.12],
        yaw: 0.0,
        tex: [tex::GRADER_TRUSS; 3],
        form: Form::Box,
    },
    // Glazed yellow operator cab on the near (+z) side of the turret.
    part([-0.04, 0.03, 0.15], [0.20, 0.20, 0.18], tex::GRADER_YELLOW),
    part([0.07, 0.04, 0.15], [0.02, 0.11, 0.13], tex::CAB_GLASS),
    // Signature bucket-wheel digging drum at the boom tip (+x): forged orange wheel rim, yellow hub, steel axle.
    bar_z([0.34, 0.07, 0.0], 0.16, 0.30, tex::GRADER_BUCKET),
    bar_z([0.34, 0.07, 0.0], 0.19, 0.16, tex::GRADER_YELLOW),
    bar_z([0.34, 0.07, 0.0], 0.23, 0.07, tex::STEEL),
];
