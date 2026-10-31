//! Trains (Milestone 9): the rail icon (two steel rails over wooden sleepers on gravel), the locomotive and wagon
//! and signal icons, and the docks' faces (a hazard-striped steel side; an amber top with a down arrow to load, a blue one
//! with an up arrow to unload). Placeholder looks until the art pass (`docs/ART_HANDOVER.md`).

use crate::block::tex;

use super::{n, rgb};

pub fn pixel(layer: u16, x: i32, y: i32) -> [u8; 4] {
    match layer {
        tex::RAIL => rail(x, y),
        tex::LOCOMOTIVE => locomotive(x, y),
        tex::WAGON => wagon(x, y),
        tex::DOCK_SIDE => dock_side(x, y),
        tex::DOCK_LOAD_TOP => dock_top(x, y, [224.0, 160.0, 48.0], true),
        tex::DOCK_UNLOAD_TOP => dock_top(x, y, [48.0, 160.0, 224.0], false),
        tex::SIGNAL => signal(x, y),
        tex::LOCO_BODY => loco_body(x, y),
        tex::WHEEL => wheel(x, y),
        tex::CAB_GLASS => cab_glass(x, y),
        _ => [255, 0, 255, 255],
    }
}

/// The train's livery, locomotive and wagons alike: deep blue with an amber band low on every side and dark seams at
/// the edges.
fn loco_body(x: i32, y: i32) -> [u8; 4] {
    if y == 0 || y == 15 {
        return rgb([24.0, 28.0, 36.0], 1.0);
    }
    if y == 10 || y == 11 {
        return rgb([236.0, 168.0, 40.0], 0.95 + 0.1 * n(436, x, y));
    }
    if y == 12 {
        return rgb([236.0, 236.0, 230.0], 0.95 + 0.05 * n(438, x, y));
    }
    rgb([44.0, 84.0, 156.0], 0.92 + 0.1 * n(437, x, y))
}

/// A wheel's face: a bright tyre round a dark disc, five spokes and a hub, so its turning shows.
fn wheel(x: i32, y: i32) -> [u8; 4] {
    let (dx, dy) = (x as f64 - 7.5, y as f64 - 7.5);
    let r = dx.hypot(dy);
    let spoke = (dy.atan2(dx) * 5.0 / std::f64::consts::TAU).rem_euclid(1.0);
    if r > 6.5 {
        rgb([168.0, 172.0, 180.0], 0.9 + 0.1 * n(470, x, y))
    } else if r < 1.8 {
        rgb([200.0, 160.0, 60.0], 1.0)
    } else if !(0.12..=0.88).contains(&spoke) {
        rgb([120.0, 124.0, 132.0], 0.95 + 0.1 * n(471, x, y))
    } else {
        rgb([34.0, 36.0, 42.0], 0.9 + 0.2 * n(472, x, y))
    }
}

/// Tinted cab glass: dark blue-grey with a pale streak where it catches the sky, and a dark frame.
fn cab_glass(x: i32, y: i32) -> [u8; 4] {
    if x == 0 || x == 15 || y == 0 || y == 15 {
        return rgb([24.0, 26.0, 30.0], 1.0);
    }
    let streak = (x + y - 9).abs() <= 1 || (x + y - 14).abs() == 0;
    if streak {
        rgb([150.0, 176.0, 196.0], 1.0)
    } else {
        rgb([44.0, 62.0, 82.0], 0.92 + 0.1 * n(473, x, y))
    }
}

/// A signal: a steel post under an amber lamp, on a dark ground.
fn signal(x: i32, y: i32) -> [u8; 4] {
    let lamp = (5..=10).contains(&x) && (1..=6).contains(&y);
    let post = (7..=8).contains(&x) && y > 6;
    if lamp {
        rgb([240.0, 170.0, 40.0], 0.9 + 0.2 * n(460, x, y))
    } else if post {
        rgb([150.0, 158.0, 170.0], 0.9 + 0.2 * n(461, x, y))
    } else {
        rgb([58.0, 62.0, 70.0], 0.85 + 0.3 * n(462, x, y))
    }
}

/// The wagon from the side: an open steel box with a violet stripe on a flat bed over dark wheels.
fn wagon(x: i32, y: i32) -> [u8; 4] {
    let wheel = (y == 13 || y == 14) && matches!(x, 3..=5 | 10..=12);
    let rail = y == 15;
    let body = (1..=14).contains(&x) && (4..=11).contains(&y);
    let stripe = body && y == 7;
    let bed = (0..=15).contains(&x) && y == 12;
    if rail {
        rgb([190.0, 196.0, 206.0], 0.95 + 0.1 * n(440, x, y))
    } else if wheel {
        rgb([44.0, 44.0, 50.0], 0.9 + 0.2 * n(441, x, y))
    } else if stripe {
        rgb([140.0, 80.0, 200.0], 1.0)
    } else if body {
        rgb([138.0, 146.0, 158.0], 0.9 + 0.2 * n(442, x, y))
    } else if bed {
        rgb([58.0, 62.0, 70.0], 0.9 + 0.2 * n(443, x, y))
    } else {
        rgb([58.0, 62.0, 70.0], 0.85 + 0.3 * n(444, x, y))
    }
}

/// Steel plate with a hazard band (amber diagonals) along the bottom.
fn dock_side(x: i32, y: i32) -> [u8; 4] {
    if y >= 12 {
        let dark = (x + y) % 4 < 2;
        return rgb(if dark { [40.0, 40.0, 44.0] } else { [226.0, 176.0, 40.0] }, 0.95 + 0.1 * n(450, x, y));
    }
    rgb([118.0, 126.0, 138.0], 0.85 + 0.3 * n(451, x, y))
}

/// A framed deck in the dock's colour with an arrow down (loading: items arrive) or up (unloading).
fn dock_top(x: i32, y: i32, colour: [f64; 3], down: bool) -> [u8; 4] {
    let edge = x == 0 || y == 0 || x == 15 || y == 15;
    let (dx, dy) = ((x - 8).abs(), if down { y - 4 } else { 11 - y });
    let arrow = (0..=7).contains(&dy) && dx <= (if dy < 4 { 1 } else { 7 - dy });
    if edge {
        rgb([60.0, 64.0, 72.0], 1.0)
    } else if arrow {
        rgb([240.0, 240.0, 236.0], 1.0)
    } else {
        rgb(colour, 0.85 + 0.2 * n(452, x, y))
    }
}

/// The locomotive from the side: a steel boiler and a violet-roofed cab on two rails, over dark wheels.
fn locomotive(x: i32, y: i32) -> [u8; 4] {
    let wheel = (y == 13 || y == 14) && matches!(x, 3..=5 | 7..=9 | 11..=13);
    let rail = y == 15;
    let roof = y == 4 && (9..=14).contains(&x);
    let cab = (9..=14).contains(&x) && (5..=12).contains(&y);
    let boiler = (2..=8).contains(&x) && (7..=12).contains(&y);
    let stripe = y == 10 && (2..=14).contains(&x);
    if rail {
        rgb([190.0, 196.0, 206.0], 0.95 + 0.1 * n(430, x, y))
    } else if wheel {
        rgb([44.0, 44.0, 50.0], 0.9 + 0.2 * n(431, x, y))
    } else if roof {
        rgb([140.0, 80.0, 200.0], 0.95 + 0.1 * n(432, x, y))
    } else if stripe {
        rgb([226.0, 190.0, 60.0], 1.0)
    } else if cab {
        rgb([76.0, 86.0, 104.0], 0.9 + 0.2 * n(433, x, y))
    } else if boiler {
        rgb([150.0, 158.0, 170.0], 0.9 + 0.2 * n(434, x, y))
    } else {
        rgb([58.0, 62.0, 70.0], 0.85 + 0.3 * n(435, x, y))
    }
}

/// Two rails run up the picture over sleepers, on grey gravel.
fn rail(x: i32, y: i32) -> [u8; 4] {
    if x == 4 || x == 11 {
        return rgb([190.0, 196.0, 206.0], 0.95 + 0.1 * n(420, x, y));
    }
    if (3..=12).contains(&x) && y % 5 < 2 {
        return rgb([116.0, 84.0, 52.0], 0.9 + 0.15 * n(421, x, y));
    }
    rgb([104.0, 102.0, 98.0], 0.8 + 0.4 * n(422, x, y))
}
