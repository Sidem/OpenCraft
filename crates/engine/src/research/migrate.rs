//! Save migration from pre-86 research tables (95 techs, indices 0..94) to the frontier-map tree (76 techs,
//! `SAVE_VERSION >= 86`). Maps old progress, current tech, queue and in-progress lab units without losing unlocks.

use crate::bytes::ByteReader;

use super::bonus::LEVELS;
use super::ids::*;
use super::queue::MAX_QUEUE;
use super::rings::Ring;
use super::{Research, TECHS};

/// Number of techs in saves before version 86.
pub const OLD_TECH_COUNT: usize = 95;

/// Units required to finish each tech in saves before version 86 (indices 0..94).
const OLD_UNITS: [u32; OLD_TECH_COUNT] = [
    10, 20, 30, 15, 30, 20, 25, 25, 40, 50, 50, 60, 80, 30, 60, 60, 40, // 0..17 MAIN
    80, 100, 100, 120, 80, 60, 80, 100, 140, 160, 180, 240, 200, // 17..30 ELECTRONICS
    120, 160, 140, 50, 60, 140, // 30..36 PERSONAL
    120, 100, 150, 120, 160, 200, // 36..42 DISTANCE
    140, 160, 120, 120, 140, 160, 100, 140, 40, 100, 200, 240, 200, // 42..55 CHEMISTRY
    60,  // 55 RECYCLING
    220, 260, 150, 300, 200, 200, 100, // 56..63 COMPUTE
    LEVELS, LEVELS, LEVELS, // 63..66 BONUS
    250, 150, 60, 200, 300, 250, 150, 100, // 66..74 AI
    40, 60, 100, 150, // 74..78 HANDS
    300, 250, 300, 300, // 78..82 ORBIT
    150, 250, 100, 40, 15, 120, 80, 120, 140, 160, 120, 120, 25, // 82..95 LATER
];

/// Maps a pre-86 tech index to its primary new tech index (for `current`, queue, and in-progress lab units).
/// Returns `None` for the four removed Science gate techs (2, 10, 18, 53) or out-of-range indices.
pub fn map_tech_id(old: u8) -> Option<u8> {
    Some(match old {
        0 => BELT_ROUTING,
        1 => BELT_LIFTS,
        2 => return None, // Green Science
        3 => UNDERPASSES,
        4 | 5 => MECHANICS,
        6 => FLUID_HANDLING,
        7 => MASONRY,
        8 => ASSEMBLY,
        9 => STEELMAKING,
        10 => return None, // Blue Science
        11 | 12 | 68 => MK3_KITS,
        13 => STEEL_TOOLS,
        14 => STEAM_POWER,
        15 => ORE_CRUSHING,
        16 => BULK_STORAGE,
        17 => ELECTRONICS,
        18 => return None, // Violet Science
        19 | 20 | 69 => MK4_KITS,
        21 => SOLAR_POWER,
        22 => ADVANCED_SCANNING,
        23 => LOGIC,
        24 => PROCESSORS,
        25 => ROBOTICS,
        26..=28 => CONSTRUCTION_DRONES,
        29 => SWARM_LOGISTICS,
        30 => JETPACK,
        31 => PERSONAL_DRONE,
        32 => EARTHWORKS,
        33 => HAULER_GEAR,
        34 => FIELD_GEAR,
        35 => EXOSUIT,
        36 => BAUXITE_PROCESSING,
        37..=39 => RAILWAYS,
        40 => HOVER_PACK,
        41 => CARGO_DRONES,
        42 | 43 => OIL_REFINING,
        44 => PLASTICS,
        45 => ACIDS_AND_LUBRICANTS,
        46 => DIESEL_POWER,
        47 => ELECTROLYSIS,
        48 => ORE_WASHING,
        49 => RESEARCH_CENTER,
        50 => HYDROPOWER,
        51 => HOISTS,
        52 => NUCLEAR_POWER,
        53 => return None, // Gold Science
        54 | 70 | 77 => MK5_KITS,
        55 => RECYCLING,
        56 | 57 => CHIP_FAB,
        58 => DATA_NETWORK,
        59 | 60 => DATACENTERS,
        61 | 62 => AI_LABS,
        63 => MINING_PRODUCTIVITY,
        64 => MACHINE_SPEED,
        65 => DRONE_SPEED,
        66 => OPTIMIZER,
        67 => PHOTONICS,
        71 => DRONE_SWARMS,
        72 => AUTO_ROUTING,
        73 => AI_SURVEY,
        74..=76 => WORK_GLOVES,
        78 => ROCKETRY,
        79 => ORBITAL_COMMS,
        80 => ORBITAL_POWER,
        81 => ORBITAL_SCIENCE,
        82 => LONG_DISTANCE_POWER,
        83 => DROP_PODS,
        84 => SETTLEMENTS,
        85 => STONE_STYLES, // Architecture also unlocks Glass, Metalwork, Paint below
        86 => FORESTRY,
        87 => XRAY_GOGGLES,
        88 => LED_LAMPS,
        89 => PIGMENTS,
        90 | 91 => NATURAL_GAS,
        92 => FLAMETHROWER,
        93 => SEISMIC_PROSPECTING,
        94 => ORE_SCANNING,
        _ => return None,
    })
}

/// Reads pre-86 `Research` bytes and migrates them to the 76-tech frontier-map tree.
pub(super) fn read_legacy(r: &mut ByteReader) -> Option<Research> {
    let old_current = r.u8()?;
    let n = r.count()?;
    if n > OLD_TECH_COUNT {
        return None;
    }
    let mut old_progress = Vec::with_capacity(n);
    for _ in 0..n {
        old_progress.push(r.u32()?);
    }
    let mut old_queue = Vec::new();
    if r.version >= 41 {
        let qn = r.count()?;
        if qn > MAX_QUEUE {
            return None;
        }
        for _ in 0..qn {
            old_queue.push(r.u8()?);
        }
    }

    let mut res = Research::default();
    for (old_i, &p) in old_progress.iter().enumerate() {
        if p == 0 {
            continue;
        }
        let old_u = OLD_UNITS[old_i];
        let done = p >= old_u;
        // Science gate techs open their ring when done.
        match old_i {
            2 if done => res.open_ring(Ring::Green),
            10 if done => res.open_ring(Ring::Blue),
            18 if done => res.open_ring(Ring::Violet),
            53 if done => res.open_ring(Ring::Gold),
            _ => {}
        }
        // Architecture (85) split into Stone Styles, Glass, Metalwork, Paint.
        let targets: &[u8] = if old_i == 85 {
            &[STONE_STYLES, GLASS, METALWORK, PAINT]
        } else if let Some(new_i) = map_tech_id(old_i as u8) {
            &[new_i]
        } else {
            continue;
        };
        for &new_i in targets {
            let t = &TECHS[new_i as usize];
            if super::is_bonus(new_i) {
                res.progress[new_i as usize] = res.progress[new_i as usize].max(p.min(LEVELS));
            } else if done {
                res.progress[new_i as usize] = t.units;
            } else {
                let partial = p.min(t.units.saturating_sub(1));
                res.progress[new_i as usize] = res.progress[new_i as usize].max(partial);
            }
            res.open_ring(t.ring);
        }
    }
    res.refresh();
    if old_current != u8::MAX {
        if let Some(new_c) = map_tech_id(old_current) {
            res.set_current(Some(new_c));
        }
    }
    for old_q in old_queue {
        if let Some(new_q) = map_tech_id(old_q) {
            if !res.queue.contains(&new_q) {
                res.queue.push(new_q);
            }
        }
    }
    res.prune();
    res.advance();
    Some(res)
}
