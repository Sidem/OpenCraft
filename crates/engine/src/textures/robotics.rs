//! Robotics parts (Milestone 7, the drone chain): the processor (a black package on gold pins), the servo (a
//! steel can with a copper winding and a shaft), the actuator (a piston: cylinder and orange rod), the drone cell
//! (a violet power cell with a teal gauge), the guidance module (a board with a cyan lens) and the drone itself
//! (seen from above: four rotors on a pale body), the jetpack (two tanks over a flame) the personal drone and the terrain planner. Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::PROCESSOR => processor(x, y),
        tex::SERVO => servo(x, y),
        tex::ACTUATOR => actuator(x, y),
        tex::DRONE_CELL => drone_cell(x, y),
        tex::GUIDANCE => guidance(x, y),
        tex::DRONE_PORT_TOP => port_top(x, y),
        tex::DRONE_PORT_SIDE => port_side(x, y),
        tex::JETPACK => jetpack(x, y),
        tex::PERSONAL_DRONE => personal_drone(x, y),
        tex::PLANNER => planner(x, y),
        _ => drone(x, y),
    }
}

/// A black square package with a pale die and gold pins along every edge.
fn processor(x: i32, y: i32) -> [u8; 4] {
    let inside = (3..13).contains(&x) && (3..13).contains(&y);
    if !inside {
        let pin = (x % 3 == 1 && !(3..13).contains(&y)) || (y % 3 == 1 && !(3..13).contains(&x));
        return if pin { rgb([222.0, 184.0, 84.0], 1.0) } else { rgb([30.0, 90.0, 60.0], 0.9 + 0.1 * n(400, x, y)) };
    }
    if (6..10).contains(&x) && (6..10).contains(&y) {
        return rgb([120.0, 130.0, 170.0], 0.9 + 0.2 * n(401, x, y));
    }
    rgb([28.0, 28.0, 34.0], 0.9 + 0.1 * n(402, x, y))
}

/// A steel can with a copper winding band and a shaft out of the right.
fn servo(x: i32, y: i32) -> [u8; 4] {
    if x >= 13 && (7..9).contains(&y) {
        return rgb([210.0, 214.0, 222.0], 1.0);
    }
    if !(2..13).contains(&x) || !(4..12).contains(&y) {
        return rgb([36.0, 38.0, 46.0], 0.9);
    }
    if (5..8).contains(&x) {
        return rgb([204.0, 124.0, 70.0], 0.85 + 0.2 * (y % 2) as f64 * 0.5 + 0.1 * n(403, x, y));
    }
    rgb([112.0, 122.0, 146.0], 0.8 + 0.25 * (1.0 - (y as f64 - 7.5).abs() / 4.0))
}

/// A steel cylinder on the left and an orange piston rod running out to a flange on the right.
fn actuator(x: i32, y: i32) -> [u8; 4] {
    if (1..8).contains(&x) && (4..12).contains(&y) {
        return rgb([104.0, 116.0, 140.0], 0.75 + 0.3 * (1.0 - (y as f64 - 7.5).abs() / 4.0));
    }
    if (8..14).contains(&x) && (7..9).contains(&y) {
        return rgb([236.0, 128.0, 44.0], 0.95 + 0.1 * n(404, x, y));
    }
    if (14..16).contains(&x) && (5..11).contains(&y) {
        return rgb([188.0, 192.0, 202.0], 1.0);
    }
    rgb([36.0, 38.0, 46.0], 0.9)
}

/// A violet cell with two terminals on top and a teal charge gauge.
fn drone_cell(x: i32, y: i32) -> [u8; 4] {
    if (5..11).contains(&x) && y < 3 {
        return rgb([210.0, 214.0, 222.0], 1.0);
    }
    if !(2..14).contains(&x) || !(3..15).contains(&y) {
        return rgb([36.0, 38.0, 46.0], 0.9);
    }
    if (5..11).contains(&x) && (6..13).contains(&y) {
        return rgb([84.0, 232.0, 214.0], if y > 9 { 1.1 } else { 0.7 });
    }
    rgb([104.0, 58.0, 168.0], 0.9 + 0.15 * n(405, x, y))
}

/// A dark board with a cyan lens ringed in steel and two aerial traces.
fn guidance(x: i32, y: i32) -> [u8; 4] {
    let r = ((x as f64 - 7.5).powi(2) + (y as f64 - 8.5).powi(2)).sqrt();
    if r < 2.2 {
        return rgb([90.0, 226.0, 255.0], 1.1 - 0.3 * r / 2.2);
    }
    if r < 3.6 {
        return rgb([188.0, 192.0, 202.0], 1.0);
    }
    if (x == 3 || x == 12) && y < 4 {
        return rgb([222.0, 184.0, 84.0], 1.0);
    }
    rgb([30.0, 46.0, 66.0], 0.9 + 0.12 * n(406, x, y))
}

/// Four rotor discs on the corners, thin arms to a pale body with an orange light.
fn drone(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = ((x as f64 - 7.5).abs(), (y as f64 - 7.5).abs());
    if dx < 2.2 && dy < 2.2 {
        return if dx < 0.9 && dy < 0.9 { rgb([255.0, 140.0, 40.0], 1.1) } else { rgb([226.0, 230.0, 238.0], 1.0) };
    }
    let rotor = ((dx - 5.0).powi(2) + (dy - 5.0).powi(2)).sqrt();
    if rotor < 2.4 {
        return rgb([92.0, 100.0, 120.0], 0.8 + 0.2 * n(407, x, y));
    }
    if (dx - dy).abs() < 0.8 {
        return rgb([60.0, 64.0, 76.0], 1.0);
    }
    rgb([22.0, 24.0, 30.0], 0.9)
}

/// The landing pad: a dark deck with an orange ring, a pale H and a corner light in each corner.
fn port_top(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = (dx * dx + dy * dy).sqrt();
    let h = (dx.abs() < 3.4 && dy.abs() < 3.4) && (dx.abs() > 2.2 || dy.abs() < 0.8);
    if h {
        return rgb([226.0, 230.0, 238.0], 1.0);
    }
    if (5.6..6.8).contains(&r) {
        return rgb([255.0, 140.0, 40.0], 1.0);
    }
    if (x == 0 || x == 15) && (y == 0 || y == 15) {
        return rgb([255.0, 220.0, 120.0], 1.1);
    }
    rgb([44.0, 48.0, 58.0], 0.9 + 0.12 * n(408, x, y))
}

/// The housing: graphite plating with an orange hazard chevron band across the middle.
fn port_side(x: i32, y: i32) -> [u8; 4] {
    if (6..10).contains(&y) {
        let stripe = (x + y) % 6 < 3;
        return if stripe { rgb([255.0, 140.0, 40.0], 0.95) } else { rgb([32.0, 34.0, 40.0], 1.0) };
    }
    rgb([62.0, 66.0, 78.0], 0.9 + 0.12 * n(409, x, y))
}

/// Two steel tanks with a strap, orange nozzles below and a flame under each.
fn jetpack(x: i32, y: i32) -> [u8; 4] {
    let tank = |cx: f64| (x as f64 - cx).abs() < 2.6;
    let (left, right) = (tank(4.5), tank(11.5));
    if (left || right) && (1..11).contains(&y) {
        let cx = if left { 4.5 } else { 11.5 };
        let shade = 1.0 - ((x as f64 - cx).abs() / 2.6) * 0.5;
        return rgb([150.0, 158.0, 176.0], shade * (0.9 + 0.1 * n(410, x, y)));
    }
    if (left || right) && (11..13).contains(&y) {
        return rgb([196.0, 100.0, 40.0], 1.0);
    }
    if (left || right) && y >= 13 {
        return rgb([255.0, 190.0, 60.0], if y == 13 { 1.2 } else { 0.9 });
    }
    if (5..11).contains(&y) && (6..10).contains(&x) {
        return rgb([84.0, 60.0, 40.0], 1.0);
    }
    rgb([22.0, 24.0, 30.0], 0.9)
}

/// A teal sphere seen from the front: one big cyan eye, a ring of pale plating and two little rotors.
fn personal_drone(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 8.5);
    let r = (dx * dx + dy * dy).sqrt();
    if r < 1.9 {
        return rgb([90.0, 226.0, 255.0], 1.15 - 0.3 * r / 1.9);
    }
    if r < 3.0 {
        return rgb([226.0, 230.0, 238.0], 1.0);
    }
    if r < 5.6 {
        return rgb([40.0, 150.0, 150.0], 0.8 + 0.3 * (1.0 - r / 5.6) + 0.1 * n(411, x, y));
    }
    if y < 3 && (x - 3).abs() < 3 || y < 3 && (x - 12).abs() < 3 {
        return rgb([188.0, 192.0, 202.0], 0.9);
    }
    rgb([22.0, 24.0, 30.0], 0.9)
}

/// A dark tablet with a pale survey grid, an orange site rectangle and a cyan corner marker.
fn planner(x: i32, y: i32) -> [u8; 4] {
    if !(1..15).contains(&x) || !(1..15).contains(&y) {
        return rgb([22.0, 24.0, 30.0], 0.9);
    }
    if (4..12).contains(&x) && (5..11).contains(&y) && (x == 4 || x == 11 || y == 5 || y == 10) {
        return rgb([255.0, 140.0, 40.0], 1.1);
    }
    if (x - 4 == 0 || x - 11 == 0) && (y - 5 == 0 || y - 10 == 0) {
        return rgb([90.0, 226.0, 255.0], 1.2);
    }
    if x % 4 == 1 || y % 4 == 1 {
        return rgb([70.0, 92.0, 120.0], 1.0);
    }
    rgb([34.0, 46.0, 66.0], 0.9 + 0.1 * n(412, x, y))
}
