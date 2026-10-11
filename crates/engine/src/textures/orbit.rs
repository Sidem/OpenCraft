//! Orbit (Milestone 12): the launch site's pad and gantry, the rocket parts' icons, the rocket fuel canister and the
//! rocket's hull. Placeholders until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::chemistry::canister;
use super::paint::round_shade;
use super::{frame, n, rgb};

const FUEL_BAND: [f64; 3] = [232.0, 84.0, 40.0];

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::LAUNCH_SIDE => pad_side(x, y),
        tex::LAUNCH_TOP => pad_top(x, y),
        tex::LIGHT_FRAME => light_frame(x, y),
        tex::ROCKET_ENGINE => engine(x, y),
        tex::FUEL_TANK => tank(x, y),
        tex::AVIONICS => avionics(x, y),
        tex::ROCKET_FUEL_CANISTER => canister(x, y, FUEL_BAND),
        _ => hull(x, y),
    }
}

/// Grey concrete with steel cross-bracing above a yellow and black hazard band.
fn pad_side(x: i32, y: i32) -> [u8; 4] {
    if y >= 13 {
        let yellow = (x + y) / 2 % 2 == 0;
        return if yellow { rgb([232.0, 190.0, 40.0], 1.0) } else { rgb([30.0, 30.0, 34.0], 1.0) };
    }
    if x == 0 || x == 15 || y == 0 || x == y || x + y == 12 {
        return rgb([170.0, 176.0, 186.0], 0.9 + 0.2 * n(681, x, y));
    }
    rgb([112.0, 116.0, 122.0], 0.88 + 0.24 * n(682, x, y))
}

/// A slab scorched round a blast ring: a dark centre, an orange ring, pale concrete outside.
fn pad_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r2 = dx * dx + dy * dy;
    if (30.0..44.0).contains(&r2) {
        return rgb([226.0, 120.0, 40.0], 1.0);
    }
    if r2 < 20.0 {
        return rgb([40.0, 38.0, 38.0], 0.9 + 0.2 * n(683, x, y));
    }
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return frame(x, y);
    }
    rgb([150.0, 152.0, 156.0], 0.9 + 0.2 * n(684, x, y))
}

/// A square aluminium frame braced with a diagonal and open in the middle.
fn light_frame(x: i32, y: i32) -> [u8; 4] {
    let beam = !(2..=13).contains(&x) || !(2..=13).contains(&y) || x == y || x + y == 15;
    if beam {
        rgb([206.0, 212.0, 222.0], 0.9 + 0.15 * n(685, x, y))
    } else {
        rgb([34.0, 38.0, 46.0], 0.9 + 0.2 * n(686, x, y))
    }
}

/// A bell nozzle seen from the side: a narrow throat flaring to a wide mouth, hot at the lip.
fn engine(x: i32, y: i32) -> [u8; 4] {
    let half = 2.0 + y as f64 * 0.42;
    let inside = (x as f64 - 7.5).abs() <= half;
    if !inside {
        return rgb([24.0, 26.0, 32.0], 0.9 + 0.2 * n(687, x, y));
    }
    if y >= 14 {
        return rgb([250.0, 150.0, 50.0], 1.0);
    }
    rgb([86.0, 92.0, 104.0], round_shade(x) * (0.9 + 0.15 * n(688, x, y)))
}

/// A pale cylinder with a dark ring at each end and a red stripe, shaded round.
fn tank(x: i32, y: i32) -> [u8; 4] {
    let shade = round_shade(x);
    if !(2..=13).contains(&y) {
        return rgb([60.0, 64.0, 74.0], shade);
    }
    if (7..=8).contains(&y) {
        return rgb([200.0, 60.0, 50.0], shade);
    }
    rgb([226.0, 230.0, 236.0], shade * (0.94 + 0.08 * n(689, x, y)))
}

/// A dark board with green traces, a gold chip in the middle and a light at one corner.
fn avionics(x: i32, y: i32) -> [u8; 4] {
    if (5..=10).contains(&x) && (5..=10).contains(&y) {
        return rgb([226.0, 190.0, 80.0], 0.9 + 0.2 * n(690, x, y));
    }
    if x == 2 && y == 2 {
        return rgb([120.0, 255.0, 140.0], 1.0);
    }
    if x % 4 == 1 || y % 4 == 1 {
        return rgb([70.0, 190.0, 100.0], 0.9 + 0.2 * n(691, x, y));
    }
    rgb([26.0, 42.0, 34.0], 0.9 + 0.2 * n(692, x, y))
}

/// White paint shaded round, a red band near the top and black rivet seams.
fn hull(x: i32, y: i32) -> [u8; 4] {
    let shade = round_shade(x);
    if y == 6 || y == 11 {
        return rgb([40.0, 42.0, 48.0], shade);
    }
    if (3..=5).contains(&y) {
        return rgb([204.0, 52.0, 44.0], shade);
    }
    rgb([238.0, 240.0, 244.0], shade * (0.95 + 0.06 * n(693, x, y)))
}
