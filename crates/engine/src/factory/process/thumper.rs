//! Seismic Thumper (`block::THUMPER`, F-SEIS): a powered 2×2×2 geophysical instrument that sounds a 64-block-wide
//! vertical cross-section and 48×48×H 3D subsurface volume down to bedrock (`seismic/`).
//!
//! - Core behaviour: a pure power sink (`Energy::Seismic`) with three tiers (Mk1 40 kW, Mk2 65 kW, Mk3 90 kW); no
//!   buffers, no saved extra state, no save-format bump. Scans are read-only queries (`seismic/`).
//! - Presentation: static frame, corner struts, anvil, ground-seated telemetry cabinet, sighting bar and receiver masts
//!   live in `THUMPER_PARTS` (cached per chunk); `Look::Thumper` draws the animated hydraulic drop-hammer and impact
//!   shock ring with full clearance from the frame and side cabinet.
//!
//! To tune power draw or cycle period: adjust `THUMPER_TIERS` and `CYCLE_SECS` here.

use std::f32::consts::FRAC_PI_2;

use crate::block::{tex, THUMPER};
use crate::math::{hash3, Vec3};

use super::super::footprint::Footprint;
use super::super::power::FULL_SPEED;
use super::super::render::{push_box, push_round, Axis};
use super::model::{local, part, round, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

/// Seconds per hydraulic thump stroke at full power.
const CYCLE_SECS: f64 = 2.6;
/// Peak lift height of the hydraulic drop-hammer above the ground anvil, in blocks.
const LIFT_HEIGHT: f64 = 0.48;

/// Mk1 (red, 40 kW), Mk2 (violet kits, 65 kW), Mk3 (gold kits, 90 kW).
pub const THUMPER_TIERS: [ProcessTier; 3] = [
    ProcessTier { energy: Energy::Seismic, speed: 1000, fuel: 0, power: 40 },
    ProcessTier { energy: Energy::Seismic, speed: 1000, fuel: 0, power: 65 },
    ProcessTier { energy: Energy::Seismic, speed: 1000, fuel: 0, power: 90 },
];

pub const THUMPER_SPEC: ProcessSpec = ProcessSpec {
    block: THUMPER,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [0, 0, 0],
    side: 0,
    tiers: &THUMPER_TIERS,
    footprint: Footprint { size: [2, 2, 2], ports: &[] },
    verb: "Sounding",
    products: "readings",
    waiting: "Idle",
    map_colour: 0x38bdf8,
    compute: 0,
    parts: &THUMPER_PARTS,
};

/// Heavy ground-coupling plinth with tier band, circular machined impact anvil, four hydraulic corner columns with
/// brass damper collars, ground-seated side telemetry cabinet (clear of the hammer well), upper bridge deck, hydraulic
/// turret, slice-axis sighting pointers, twin geophone masts, status lamp and animated drop-hammer (`Look::Thumper`).
const THUMPER_PARTS: [Part; 25] = [
    // Heavy steel ground-coupling footplate and tier band with Mk studs (y = -1.00 .. -0.84).
    part([0.0, -0.92, 0.0], [1.96, 0.16, 1.96], Look::Band(tex::THUMPER_TOP)),
    // Circular machined ground-coupling anvil pad (top at y = -0.74, radius 0.51).
    round([0.0, -0.79, 0.0], [1.02, 0.10, 1.02], Look::Tex([tex::THUMPER_TOP, tex::MAT_GUNMETAL, tex::MAT_GUNMETAL])),
    // Four reinforced corner anchor footings (y = -0.84 .. -0.76).
    part([-0.76, -0.80, -0.76], [0.32, 0.08, 0.32], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([0.76, -0.80, -0.76], [0.32, 0.08, 0.32], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([-0.76, -0.80, 0.76], [0.32, 0.08, 0.32], Look::Tex([tex::MAT_GUNMETAL; 3])),
    part([0.76, -0.80, 0.76], [0.32, 0.08, 0.32], Look::Tex([tex::MAT_GUNMETAL; 3])),
    // Four sturdy hydraulic corner guide columns (y = -0.76 .. +0.46; inner face at 0.65).
    part([-0.76, -0.15, -0.76], [0.22, 1.22, 0.22], Look::Tex([tex::FRAME; 3])),
    part([0.76, -0.15, -0.76], [0.22, 1.22, 0.22], Look::Tex([tex::FRAME; 3])),
    part([-0.76, -0.15, 0.76], [0.22, 1.22, 0.22], Look::Tex([tex::FRAME; 3])),
    part([0.76, -0.15, 0.76], [0.22, 1.22, 0.22], Look::Tex([tex::FRAME; 3])),
    // Brass hydraulic damper collars near the top of each column.
    part([-0.76, 0.24, -0.76], [0.27, 0.09, 0.27], Look::Tex([tex::MAT_BRASS; 3])),
    part([0.76, 0.24, -0.76], [0.27, 0.09, 0.27], Look::Tex([tex::MAT_BRASS; 3])),
    part([-0.76, 0.24, 0.76], [0.27, 0.09, 0.27], Look::Tex([tex::MAT_BRASS; 3])),
    part([0.76, 0.24, 0.76], [0.27, 0.09, 0.27], Look::Tex([tex::MAT_BRASS; 3])),
    // Ground-seated side electrical & telemetry cabinet on +X (x = 0.70..0.96, y = -0.84..-0.08; 0.30 clear of hammer).
    part([0.83, -0.46, 0.0], [0.26, 0.76, 0.84], Look::Tex([tex::THUMPER_TOP, tex::THUMPER_SIDE, tex::FRAME])),
    // Illuminated cyan oscilloscope screen on the outer face of the cabinet.
    part([0.97, -0.30, 0.0], [0.03, 0.24, 0.44], Look::Tex([tex::LAMP_BLUE; 3])),
    // Upper hydraulic bridge deck (y = +0.46 .. +0.70).
    part([0.0, 0.58, 0.0], [1.84, 0.24, 1.84], Look::Tex([tex::THUMPER_TOP, tex::THUMPER_SIDE, tex::FRAME])),
    // Central hydraulic actuator turret + brass pressure cap above the bridge.
    round([0.0, 0.84, 0.0], [0.76, 0.28, 0.76], Look::Tex([tex::THUMPER_TOP, tex::MAT_GUNMETAL, tex::FRAME])),
    round([0.0, 1.01, 0.0], [0.46, 0.08, 0.46], Look::Tex([tex::MAT_BRASS; 3])),
    // Slice-axis sighting pointers along Z (-z amber forward, +z brass rear).
    part([0.0, 0.74, -0.72], [0.22, 0.08, 0.42], Look::Tex([tex::MAT_EMBER; 3])),
    part([0.0, 0.74, 0.72], [0.16, 0.06, 0.34], Look::Tex([tex::MAT_BRASS; 3])),
    // Twin geophone receiver masts and brass sensor caps on the rear corners.
    round([-0.62, 1.08, -0.62], [0.09, 0.76, 0.09], Look::Tex([tex::STEEL; 3])),
    round([0.62, 1.08, -0.62], [0.09, 0.76, 0.09], Look::Tex([tex::STEEL; 3])),
    // Status lamp cleanly seated on the front-right bridge corner.
    part([0.66, 0.76, 0.66], [0.20, 0.12, 0.20], Look::Lamp),
    // Dynamic drop-hammer piston, forged impact shoe & ground-coupling pulse ring (`draw_dynamic`).
    part([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], Look::Thumper),
];

impl Processor {
    /// The target-card status line of a Seismic Thumper (`None` for other processors).
    pub(super) fn thumper_text(&self) -> Option<String> {
        if self.energy() != Energy::Seismic {
            return None;
        }
        let axis = if self.dir.is_multiple_of(2) { "N–S" } else { "E–W" };
        Some(match self.status {
            Status::Working if self.speed < FULL_SPEED => {
                format!("Low power ({}%) · noisy 64m {axis} slice · right-click to read", self.speed / 10)
            }
            Status::Working => format!("Seismic array active · 64m {axis} slice · right-click to read"),
            _ => "No power: wire to a power pole to sound the ground".to_string(),
        })
    }
}

/// Snapshot of a placed Seismic Thumper for `seismic/`: anchor, facing `dir`, tier (0..=2), power share (0..=1000),
/// tier power draw in kW, status code (0 = unpowered, 1 = working, 2 = switched off).
#[derive(Clone, Copy, Debug)]
pub struct ThumperInfo {
    pub anchor: crate::math::IVec3,
    pub dir: u8,
    pub tier: u8,
    pub power_share: u32,
    pub kw: u32,
    pub status: u8,
}

impl super::super::Factory {
    /// Returns [`ThumperInfo`] if `cell` belongs to a placed Seismic Thumper (any cell of its 2×2×2 footprint).
    pub fn thumper_at(&self, cell: crate::math::IVec3) -> Option<ThumperInfo> {
        let anchor = self.anchor_of(cell)?;
        let super::super::links::Slot::Process(i) = *self.at.get(&anchor)? else { return None };
        let p = &self.processors[i as usize];
        if p.spec.block != THUMPER {
            return None;
        }
        let status = match p.status {
            Status::Working => 1,
            Status::Off => 2,
            _ => 0,
        };
        Some(ThumperInfo {
            anchor,
            dir: p.dir,
            tier: p.tier,
            power_share: if status == 1 { p.speed } else { 0 },
            kw: p.stats().power,
            status,
        })
    }
}

/// Draws the animated hydraulic drop-hammer ram, forged impact shoe, brass collar, central drive rod and expanding
/// impact pulse ring at camera-relative `centre`. Pure presentation: phase is derived from `time` and `p.pos` alone.
pub(super) fn draw_dynamic(p: &Processor, out: &mut Vec<f32>, centre: Vec3, yaw: f32, time: f64) {
    let working = p.status == Status::Working && p.speed > 0;
    let phase = if working {
        let rate = (f64::from(p.speed) / f64::from(FULL_SPEED)).clamp(0.25, 1.0);
        let offset = f64::from(hash3(0x5E15, p.pos.x, p.pos.y, p.pos.z) & 0xFFFF) / 65536.0;
        (time * rate / CYCLE_SECS + offset).fract()
    } else {
        0.0
    };

    // Lift slowly (0..0.64), hold briefly (0.64..0.76), slam sharply down (0.76..0.84), tiny damped recoil (0.84..1.0).
    let lift = if !working {
        0.0
    } else if phase < 0.64 {
        let u = phase / 0.64;
        u * u * (3.0 - 2.0 * u)
    } else if phase < 0.76 {
        1.0
    } else if phase < 0.84 {
        let u = (phase - 0.76) / 0.08;
        1.0 - u * u
    } else {
        let u = (phase - 0.84) / 0.16;
        (u * std::f64::consts::PI).sin() * (1.0 - u) * 0.10
    };

    // At lift = 0, shoe bottom sits flush on the anvil top at y = -0.74; at lift = 1, collar clears bridge bottom +0.46.
    let head_y = -0.41 + lift * LIFT_HEIGHT;
    // Forged steel impact shoe at the bottom of the drop-weight (radius 0.40, 0.30 clear of the side cabinet).
    let shoe_at = local(centre, yaw, [0.0, head_y - 0.27, 0.0]);
    push_round(out, shoe_at, yaw, Axis::Up, [0.80, 0.12, 0.80], 1.0, [tex::MAT_GUNMETAL; 3], false);
    // Heavy hazard-striped cylindrical hydraulic drop-weight (radius 0.36, true straight cylinder taper = 1.0).
    let head_at = local(centre, yaw, [0.0, head_y, 0.0]);
    push_round(
        out,
        head_at,
        yaw,
        Axis::Up,
        [0.72, 0.44, 0.72],
        1.0,
        [tex::MAT_GUNMETAL, tex::THUMPER_PISTON, tex::MAT_GUNMETAL],
        false,
    );
    // Brass hydraulic retention collar crowning the drop-weight.
    let collar_at = local(centre, yaw, [0.0, head_y + 0.25, 0.0]);
    push_round(out, collar_at, yaw, Axis::Up, [0.52, 0.07, 0.52], 1.0, [tex::MAT_BRASS; 3], false);
    // Steel hydraulic drive rod connecting the hammer collar straight up into the bridge cylinder.
    let rod_top = 0.70;
    let rod_bot = head_y + 0.28;
    let rod_h = (rod_top - rod_bot).max(0.08) as f32;
    let rod_at = local(centre, yaw, [0.0, (rod_top + rod_bot) * 0.5, 0.0]);
    push_round(out, rod_at, yaw, Axis::Up, [0.22, rod_h, 0.22], 1.0, [tex::STEEL; 3], false);

    // Expanding luminous ground-coupling shock ring right after impact (phase 0.84..0.99).
    if working && phase >= 0.84 {
        let u = ((phase - 0.84) / 0.16).clamp(0.0, 1.0) as f32;
        let reach = 1.02 + u * 0.65;
        let thick = 0.07 * (1.0 - u * 0.7);
        let span = reach * 2.0;
        let ring_tex = [if u < 0.35 { tex::LAMP_YELLOW } else { tex::LAMP_BLUE }; 3];
        for (dx, dz, sx, sz, turn) in [
            (0.0, -reach, span, thick, 0.0),
            (0.0, reach, span, thick, 0.0),
            (-reach, 0.0, thick, span, 0.0),
            (reach, 0.0, thick, span, 0.0),
        ] {
            let at = local(centre, yaw, [f64::from(dx), -0.95, f64::from(dz)]);
            push_box(out, at, yaw + turn * FRAC_PI_2, [sx, 0.04, sz], 0.0, ring_tex, false);
        }
    }
}
