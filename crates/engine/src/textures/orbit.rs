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
        tex::SURVEY_SATELLITE => survey_satellite(x, y),
        tex::COMMS_SATELLITE => comms_satellite(x, y),
        tex::POWER_SATELLITE => power_satellite(x, y),
        tex::RECTENNA_TOP => rectenna_top(x, y),
        tex::RECTENNA_SIDE => rectenna_side(x, y),
        tex::SCIENCE_SATELLITE => science_satellite(x, y),
        tex::GROUND_TOP => ground_top(x, y),
        tex::GROUND_SIDE => ground_side(x, y),
        tex::DROP_POD => drop_pod(x, y),
        tex::POD_SIDE => pod_side(x, y),
        tex::POD_LAUNCH_TOP => pod_launch_top(x, y),
        tex::POD_PAD_TOP => pod_pad_top(x, y),
        _ => hull(x, y),
    }
}

/// A rounded white capsule with a dark window and a heat-scorched base, on black.
fn drop_pod(x: i32, y: i32) -> [u8; 4] {
    let half = if y < 5 { 1.5 + y as f64 * 0.9 } else { 5.5 };
    if (x as f64 - 7.5).abs() > half || y > 14 {
        return rgb([10.0, 12.0, 22.0], 0.9 + 0.2 * n(713, x, y));
    }
    if y >= 12 {
        return rgb([60.0, 50.0, 46.0], 0.9 + 0.2 * n(714, x, y));
    }
    if (6..=8).contains(&x) && (6..=8).contains(&y) {
        return rgb([40.0, 64.0, 100.0], 1.0);
    }
    rgb([232.0, 234.0, 240.0], round_shade(x) * (0.95 + 0.06 * n(715, x, y)))
}

/// Steel plating with chevrons over a cyan hazard band.
fn pod_side(x: i32, y: i32) -> [u8; 4] {
    if y >= 13 {
        let cyan = (x + y) / 2 % 2 == 0;
        return if cyan { rgb([60.0, 190.0, 220.0], 1.0) } else { rgb([30.0, 30.0, 34.0], 1.0) };
    }
    if (x - 8).abs() == y % 4 + 2 && y % 8 < 4 {
        return rgb([170.0, 176.0, 186.0], 0.9 + 0.2 * n(716, x, y));
    }
    rgb([104.0, 110.0, 124.0], 0.88 + 0.24 * n(717, x, y))
}

/// A launcher's top: a round hatch with a dark tube mouth, in a steel frame.
fn pod_launch_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r < 3.0 {
        return rgb([14.0, 16.0, 22.0], 0.9 + 0.2 * n(718, x, y));
    }
    if r < 4.5 {
        return rgb([60.0, 190.0, 220.0], 1.0);
    }
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return frame(x, y);
    }
    rgb([120.0, 126.0, 140.0], 0.9 + 0.2 * n(719, x, y))
}

/// A landing pad's top: a bullseye of cyan and white rings on dark concrete, in a steel frame.
fn pod_pad_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return frame(x, y);
    }
    if r < 1.8 {
        return rgb([250.0, 250.0, 252.0], 1.0);
    }
    if (3.0..4.4).contains(&r) {
        return rgb([60.0, 190.0, 220.0], 1.0);
    }
    if (5.6..6.6).contains(&r) {
        return rgb([236.0, 238.0, 244.0], 1.0);
    }
    rgb([70.0, 72.0, 80.0], 0.9 + 0.2 * n(720, x, y))
}

/// A white body with a green sensor eye, between two small blue solar wings.
fn science_satellite(x: i32, y: i32) -> [u8; 4] {
    if (5..=10).contains(&x) && (4..=11).contains(&y) {
        let eye = (7..=8).contains(&x) && (7..=8).contains(&y);
        let col = if eye { [90.0, 230.0, 140.0] } else { [226.0, 230.0, 238.0] };
        return rgb(col, 0.9 + 0.2 * n(707, x, y));
    }
    if (6..=9).contains(&y) && (x <= 3 || x >= 12) {
        let cell = x % 2 == 0;
        return if cell { rgb([30.0, 56.0, 120.0], 0.9 + 0.2 * n(708, x, y)) } else { rgb([110.0, 130.0, 170.0], 1.0) };
    }
    rgb([10.0, 12.0, 22.0], 0.9 + 0.2 * n(709, x, y))
}

/// A dish seen from above: pale concentric rings round a dark feed in the middle.
fn ground_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r < 1.6 {
        return rgb([40.0, 44.0, 56.0], 1.0);
    }
    if r > 7.6 {
        return frame(x, y);
    }
    let ring = (r as i32) % 2 == 0;
    rgb(if ring { [226.0, 230.0, 238.0] } else { [176.0, 184.0, 198.0] }, 0.92 + 0.12 * n(710, x, y))
}

/// Pale panels with seams, above a blue hazard band.
fn ground_side(x: i32, y: i32) -> [u8; 4] {
    if y >= 13 {
        let blue = (x + y) / 2 % 2 == 0;
        return if blue { rgb([60.0, 130.0, 220.0], 1.0) } else { rgb([30.0, 30.0, 34.0], 1.0) };
    }
    if x % 8 == 0 || y % 8 == 0 {
        return rgb([120.0, 128.0, 142.0], 0.9 + 0.2 * n(711, x, y));
    }
    rgb([200.0, 206.0, 216.0], 0.9 + 0.2 * n(712, x, y))
}

/// A gold body with a red transmitter dot between two wide solar wings.
fn power_satellite(x: i32, y: i32) -> [u8; 4] {
    if (6..=9).contains(&x) && (5..=10).contains(&y) {
        let dot = (7..=8).contains(&x) && (7..=8).contains(&y);
        let col = if dot { [236.0, 70.0, 56.0] } else { [214.0, 170.0, 70.0] };
        return rgb(col, 0.9 + 0.2 * n(700, x, y));
    }
    if (2..=13).contains(&y) && (x <= 4 || x >= 11) {
        let cell = x % 2 == 0 || y % 3 == 0;
        return if cell { rgb([30.0, 56.0, 120.0], 0.9 + 0.2 * n(701, x, y)) } else { rgb([110.0, 130.0, 170.0], 1.0) };
    }
    rgb([10.0, 12.0, 22.0], 0.9 + 0.2 * n(702, x, y))
}

/// A dark panel with a grid of bright diodes, in a steel frame.
fn rectenna_top(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return frame(x, y);
    }
    if x % 4 == 2 && y % 4 == 2 {
        return rgb([250.0, 200.0, 90.0], 1.0);
    }
    if x % 4 == 2 || y % 4 == 2 {
        return rgb([150.0, 90.0, 50.0], 0.9 + 0.2 * n(703, x, y));
    }
    rgb([24.0, 30.0, 48.0], 0.9 + 0.2 * n(704, x, y))
}

/// Steel plating above an orange hazard band.
fn rectenna_side(x: i32, y: i32) -> [u8; 4] {
    if y >= 13 {
        let orange = (x + y) / 2 % 2 == 0;
        return if orange { rgb([236.0, 130.0, 40.0], 1.0) } else { rgb([30.0, 30.0, 34.0], 1.0) };
    }
    if x == 0 || x == 15 || y == 0 {
        return rgb([170.0, 176.0, 186.0], 0.9 + 0.2 * n(705, x, y));
    }
    rgb([96.0, 104.0, 120.0], 0.88 + 0.24 * n(706, x, y))
}

/// A silver box body under a big white dish, with a red aerial on top.
fn comms_satellite(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 5.5);
    if dx * dx + dy * dy <= 12.0 {
        return rgb([240.0, 242.0, 246.0], 0.88 + 0.2 * n(697, x, y));
    }
    if (7..=8).contains(&x) && y <= 1 {
        return rgb([210.0, 60.0, 50.0], 1.0);
    }
    if (5..=10).contains(&x) && (9..=13).contains(&y) {
        return rgb([150.0, 158.0, 170.0], 0.9 + 0.2 * n(698, x, y));
    }
    rgb([10.0, 12.0, 22.0], 0.9 + 0.2 * n(699, x, y))
}

/// A gold box body between two blue solar wings, with a dish dot in the middle.
fn survey_satellite(x: i32, y: i32) -> [u8; 4] {
    if (6..=9).contains(&x) && (4..=11).contains(&y) {
        let dish = (7..=8).contains(&x) && (7..=8).contains(&y);
        let gold = if dish { [250.0, 244.0, 210.0] } else { [214.0, 170.0, 70.0] };
        return rgb(gold, 0.9 + 0.2 * n(694, x, y));
    }
    if (5..=10).contains(&y) && (x <= 4 || x >= 11) {
        let cell = x % 2 == 0 || y % 3 == 0;
        return if cell { rgb([30.0, 56.0, 120.0], 0.9 + 0.2 * n(695, x, y)) } else { rgb([110.0, 130.0, 170.0], 1.0) };
    }
    rgb([10.0, 12.0, 22.0], 0.9 + 0.2 * n(696, x, y))
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
