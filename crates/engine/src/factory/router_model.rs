//! How the belt routers look (presentation; `router.rs` owns what they do). One family, a gatehouse taller than the
//! items riding past it: a low floor plate, four corner posts and a roof in the kind's top (`tex::SPLITTER_TOP`,
//! `FILTER_TOP`, `MERGER_TOP`: an arrow per port the way items move, cyan in and gold out, as on machines' port
//! hatches), and on each side an opening for the belt under a plain header. An outlet's header stands upright, an
//! inlet's is a hood leaned out like a funnel, and a lit band along the roof's edge above each opening is the port's
//! colour (`port_marks.rs`: blue in, orange out). So a splitter shows one blue side and three orange, a merger three
//! blue and one orange, from any side, at belt-view distance and at night.
//! - Over the side the last item went by (splitter, filter: the outlet served; merger: the inlet taken from) a solid lit
//!   chevron points the way it moved, so round robin is a light walking round the roof. The held item sits inside,
//!   seen through the openings.
//! - The filter's hood is copper with an amber scanner lamp under it, and each outlet header carries what that outlet
//!   takes: its picked items in turn (one at a time, `ICON_SECONDS` each), or the lit white wildcard star where
//!   everything else goes. An outlet that takes nothing carries nothing.
//!
//! Beyond `ITEM_DETAIL_RANGE` only the posts, headers and roof are drawn. Hit boxes (`hitbox.rs`) come from [`hull`].
//! Sizes keep faces apart (`tests/face_fight.rs`). To give a new router kind a look: its ports in `ports`, its extras in
//! `draw`; the icons (`item_models/belts.rs`) copy these sizes.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

use crate::block::tex;
use crate::entities::push_item_model;
use crate::item::{self, ItemId};
use crate::math::Vec3;

use super::belt::{ride_unit, ITEM_DETAIL_RANGE};
use super::filter_rules::FilterRules;
use super::port_marks::{arrow_look, chevron, Flow};
use super::render::{mark_beacon_tinted, push_box, push_shape, Form};
use super::router::{Router, RouterKind};
use super::DIRS;

/// Heights are from the cell's floor. The floor plate, under a belt's top so items glide in over it.
const PLATE: [f32; 3] = [0.9, 0.14, 0.9];
/// How far the posts and headers stand from the middle (their centres).
const EDGE: f32 = 0.4;
/// A corner post: across, high (from just off the floor, so its foot shares no plane with the plate's), deep.
const POST: [f32; 3] = [0.12, 0.86, 0.12];
const POST_FLOOR: f32 = 0.01;
/// A header over an opening: across, high, thick; its foot's height (the opening below is taller than a riding item).
const HEADER: [f32; 3] = [0.72, 0.28, 0.06];
const HEADER_FOOT: f32 = 0.58;
/// An inlet's hood leans its top out by this much (radians), and its centre stands this much further out.
const HOOD_LEAN: f32 = 0.42;
const HOOD_OUT: f32 = 0.02;
/// The roof: across, thick; its foot is the headers' top.
const ROOF: [f32; 3] = [0.94, 0.06, 0.94];
const ROOF_FOOT: f32 = HEADER_FOOT + HEADER[1];
/// A lit band along the roof's edge over each opening, in the port's colour, so the kind reads from far and at night:
/// across, high, thick; its centre stands just off the roof's side.
const BAND: [f32; 3] = [0.7, 0.03, 0.02];
const BAND_OUT: f32 = ROOF[2] / 2.0 + BAND[2] / 2.0 + 0.002;
/// A roof chevron's size, and how far from the middle its tip sits (an inlet's points in, so its tip is nearer).
const CHEVRON_SCALE: f32 = 0.6;
const CHEVRON_OUT: f32 = 0.36;
const CHEVRON_IN: f32 = 0.2;
/// How big a filter's pick is on its header (a full cube's width), and how long each picked item shows.
const ICON_UNIT: f64 = 0.2;
const ICON_SECONDS: f64 = 1.2;
/// The filter's wildcard star: lit white, three crossed bars standing on the header's face.
const WILDCARD: u32 = 0xf2f2f2;
const STAR_BAR: [f32; 3] = [0.04, 0.2, 0.03];
/// The filter's scanner lamp under its hood.
const SCANNER: u32 = 0xffb030;

/// What a router does with a side.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Port {
    In,
    Out,
}

/// The ports of `kind`: front, left, right, behind.
fn ports(kind: RouterKind) -> [Port; 4] {
    match kind {
        RouterKind::Splitter | RouterKind::Filter => [Port::Out, Port::Out, Port::Out, Port::In],
        RouterKind::Merger => [Port::Out, Port::In, Port::In, Port::In],
    }
}

/// The router's model at camera-relative `rel` (its cell's centre).
pub fn draw(r: &Router, out: &mut Vec<f32>, rel: Vec3, time: f64) {
    let [front, left, right] = r.out_dirs();
    let sides = [front, left, right, (front + 2) % 4];
    // The roof's top layer points its arrow at row 0 when turned the router's own way.
    let body = f32::from(r.dir) * FRAC_PI_2;
    let top = [tex::SPLITTER_TOP, tex::FILTER_TOP, tex::MERGER_TOP][r.kind as usize];
    let near = rel.length() <= ITEM_DETAIL_RANGE;
    let up = |h: f32| rel + Vec3::new(0.0, f64::from(h - 0.5), 0.0);
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let at = up(POST_FLOOR + POST[1] / 2.0) + Vec3::new(sx, 0.0, sz) * f64::from(EDGE);
        push_box(out, at, 0.0, POST, 0.0, [tex::FRAME, tex::stripe(0), tex::FRAME], false);
    }
    push_box(out, up(ROOF_FOOT + ROOF[1] / 2.0), body, ROOF, 0.0, [top, tex::FRAME, tex::FRAME], false);
    let served = served_side(r, sides);
    for (k, (&side, port)) in sides.iter().zip(ports(r.kind)).enumerate() {
        let away = DIRS[side as usize].as_vec3();
        let header = up(HEADER_FOOT + HEADER[1] / 2.0) + away * f64::from(EDGE);
        let band = up(ROOF_FOOT + ROOF[1] / 2.0) + away * f64::from(BAND_OUT);
        push_box(out, band, yaw(side), BAND, 0.0, [tex::FRAME; 3], false);
        mark_beacon_tinted(out, arrow_look(if port == Port::In { Flow::In } else { Flow::Out }));
        match port {
            Port::Out => {
                push_box(out, header, yaw(side), HEADER, 0.0, [tex::FRAME; 3], false);
                if near && r.kind == RouterKind::Filter && k < 3 {
                    sign(out, &r.rules, k, header + away * f64::from(HEADER[2] / 2.0), side, time);
                }
            }
            Port::In => {
                let filter = r.kind == RouterKind::Filter;
                let look = [if filter { tex::COPPER_INGOT } else { tex::FRAME }; 3];
                let hood = header + away * f64::from(HOOD_OUT);
                push_shape(out, Form::Lean(HOOD_LEAN), hood, yaw(side), HEADER, look, false);
                if near && filter {
                    let lamp = up(HEADER_FOOT - 0.03) + away * f64::from(EDGE - 0.04);
                    push_box(out, lamp, yaw(side), [0.5, 0.03, 0.05], 0.0, [tex::FRAME; 3], false);
                    mark_beacon_tinted(out, SCANNER);
                }
            }
        }
        if near && side == served {
            let (flow, along, tip) = match port {
                Port::In => (Flow::In, (side + 2) % 4, CHEVRON_IN),
                Port::Out => (Flow::Out, side, CHEVRON_OUT),
            };
            let at = up(ROOF_FOOT + ROOF[1] + 0.025) + away * f64::from(tip);
            chevron(out, at, along, CHEVRON_SCALE, arrow_look(flow));
        }
    }
    if !near {
        return;
    }
    push_box(out, up(PLATE[1] / 2.0), body, PLATE, 0.0, [tex::IRON_PLATE, tex::FRAME, tex::FRAME], false);
    if let Some(def) = item::def(r.held).filter(|_| r.held != ItemId::NONE) {
        let unit = ride_unit(r.held);
        let at = up(PLATE[1] + def.size[1] * unit / 2.0);
        if !push_item_model(out, r.held, at, body, f64::from(unit)) {
            push_box(out, at, body, def.size.map(|s| s * unit), 0.0, def.tex, false);
        }
    }
}

/// The router's solid volume for the crosshair: posts to roof, still whatever moves.
pub fn hull(out: &mut Vec<f32>, rel: Vec3) {
    let high = ROOF_FOOT + ROOF[1];
    let size = [ROOF[0], high, ROOF[2]];
    push_box(out, rel + Vec3::new(0.0, f64::from(high / 2.0 - 0.5), 0.0), 0.0, size, 0.0, [tex::FRAME; 3], false);
}

/// The yaw that turns a box's own +z towards `DIRS[side]` (a box's +z looks along (-sin yaw, 0, cos yaw)).
fn yaw(side: u8) -> f32 {
    f32::from((side + 2) % 4) * FRAC_PI_2
}

/// The side the last item went by: an outlet of a splitter or filter, an inlet of a merger.
fn served_side(r: &Router, sides: [u8; 4]) -> u8 {
    match r.kind {
        // `ins` is behind, left, right; `sides` is front, left, right, behind.
        RouterKind::Merger => [sides[3], sides[1], sides[2]][r.last as usize % 3],
        _ => sides[r.last as usize % 3],
    }
}

/// What outlet `k` (front, left, right) of a filter takes, on the outer face of its header (centred at `face`, on
/// `side`): its picked items in turn, the wildcard star for everything else, nothing for an outlet that takes nothing.
fn sign(out: &mut Vec<f32>, rules: &FilterRules, k: usize, face: Vec3, side: u8, time: f64) {
    let away = DIRS[side as usize].as_vec3();
    let picks = &rules.picks[k];
    if !picks.is_empty() {
        let item = picks[(time / ICON_SECONDS) as usize % picks.len()];
        let at = face + away * (ICON_UNIT / 2.0 + 0.01);
        if !push_item_model(out, item, at, yaw(side), ICON_UNIT) {
            if let Some(def) = item::def(item) {
                push_box(out, at, yaw(side), def.size.map(|s| s * ICON_UNIT as f32), 0.0, def.tex, false);
            }
        }
    } else if rules.rest & 1 << k != 0 {
        let at = face + away * f64::from(STAR_BAR[2] / 2.0 + 0.005);
        // Each bar's own x looks out of the side, so leaning it about x keeps it on the header's face.
        let turn = yaw(side) + FRAC_PI_2;
        for lean in [0.0, FRAC_PI_3, -FRAC_PI_3] {
            push_shape(
                out,
                Form::Lean(lean),
                at,
                turn,
                [STAR_BAR[2], STAR_BAR[1], STAR_BAR[0]],
                [tex::FRAME; 3],
                false,
            );
            mark_beacon_tinted(out, WILDCARD);
        }
    }
}
