//! Item models of the belt family, so an icon shows what the thing is and not a cube: a belt is a low slab between two
//! rails (as `factory/belt.rs` draws it, wearing its tier's top and stripe), the splitter, filter and merger their family's
//! gatehouse, a lift a cage round a standing belt, an underpass a belt running into a hood,
//! and the ejector a funnel and a bent tube (`factory/process/ejector.rs`). Sizes are those of the placed models; the
//! parts rest on the cell's floor (y = -0.5). Add a const and a `parts` row.

use super::{part, Form, Part};
use crate::block::{tex, BELT, UNDERPASS_IN};
use crate::factory::{tiers::placed_by, BELT_TIERS, EJECTOR_ICON};
use crate::item::ItemId;

const ITEM_SPLITTER: ItemId = ItemId::block(crate::block::SPLITTER);
const ITEM_FILTER: ItemId = ItemId::block(crate::block::FILTER);
const ITEM_MERGER: ItemId = ItemId::block(crate::block::MERGER);
const ITEM_LIFT: ItemId = ItemId::block(crate::block::LIFT);
const ITEM_EJECTOR: ItemId = ItemId::block(crate::block::BELT_EJECTOR);

const fn boxed(center: [f32; 3], size: [f32; 3], tex: [u16; 3]) -> Part {
    Part { center, size, yaw: 0.0, tex, form: Form::Box }
}

const fn top(tier: u8) -> u16 {
    BELT_TIERS[tier as usize].top
}

/// The belt's moving surface, 0.84 wide and 0.18 high.
const fn slab(tier: u8) -> Part {
    boxed([0.0, -0.41, 0.0], [0.84, 0.18, 1.0], [top(tier), tex::FRAME, tex::FRAME])
}

/// A side rail wearing the tier stripe.
const fn rail(tier: u8, x: f32) -> Part {
    boxed([x, -0.37, 0.0], [0.08, 0.26, 1.0], [tex::FRAME, tex::stripe(tier), tex::FRAME])
}

/// A cross bar over the belt's end.
const fn bar(z: f32) -> Part {
    boxed([0.0, -0.3, z], [0.83, 0.07, 0.08], [tex::DRILL; 3])
}

const fn belt(tier: u8) -> [Part; 5] {
    [rail(tier, -0.46), slab(tier), bar(-0.45), bar(0.45), rail(tier, 0.46)]
}

/// A wall of the underpass hood.
const fn wall(tier: u8, x: f32) -> Part {
    boxed([x, -0.18, -0.25], [0.14, 0.64, 0.5], [tex::FRAME, tex::stripe(tier), tex::FRAME])
}

/// The belt running into a hood of two walls and a copper roof over its far half.
const fn underpass(tier: u8) -> [Part; 8] {
    let roof = boxed([0.0, 0.12, -0.25], [0.96, 0.1, 0.5], [tex::COPPER_INGOT; 3]);
    [rail(tier, -0.46), slab(tier), bar(-0.45), bar(0.45), wall(tier, -0.42), wall(tier, 0.42), rail(tier, 0.46), roof]
}

const BELTS: [[Part; 5]; 4] = [belt(0), belt(1), belt(2), belt(3)];
const UNDERPASSES: [[Part; 8]; 4] = [underpass(0), underpass(1), underpass(2), underpass(3)];

/// The routers' family (`factory/router_model.rs`, squat: an icon is about 36 pixels, and a low model lets the roof fill
/// its width): a gatehouse of four posts under a roof in the kind's top (which says the kind: its arrows are the ports,
/// cyan in and gold out), an upright gold header over each opening items leave by and a blue hood leaned out over each
/// they come in by (the filter's copper), the front towards -z. Icons paint parts in order with no depth test, so the
/// roof comes last.
const fn corner(x: f32, z: f32) -> Part {
    boxed([x, -0.265, z], [0.12, 0.45, 0.12], [tex::FRAME, tex::stripe(0), tex::FRAME])
}

const fn roof(top: u16) -> Part {
    boxed([0.0, -0.01, 0.0], [0.94, 0.06, 0.94], [top, tex::FRAME, tex::FRAME])
}

const PLATE: Part = boxed([0.0, -0.45, 0.0], [0.9, 0.1, 0.9], [tex::IRON_PLATE, tex::FRAME, tex::FRAME]);
const HEADER: [f32; 3] = [0.72, 0.16, 0.06];
const HEADER_Y: f32 = -0.12;
/// Outlet headers gold and inlet hoods blue, solid (the lit bands of the placed model would be too thin here).
const OUT: [u16; 3] = [tex::FRAME, tex::LAMP_YELLOW, tex::FRAME];
const IN: [u16; 3] = [tex::FRAME, tex::LAMP_BLUE, tex::FRAME];
const QUARTER: f32 = std::f32::consts::FRAC_PI_2;

/// An upright header on the side `quarter` quarter turns from the front (0 front, 1 left, 2 behind, 3 right).
const fn header(quarter: u8, look: [u16; 3]) -> Part {
    let at = [[0.0, -0.4], [-0.4, 0.0], [0.0, 0.4], [0.4, 0.0]][quarter as usize];
    Part { center: [at[0], HEADER_Y, at[1]], size: HEADER, yaw: quarter as f32 * QUARTER, tex: look, form: Form::Box }
}

/// A hood leaned out over the side `quarter` quarter turns from behind (0 behind, 1 left, 3 right): its top leans
/// towards its own +z, which these turns point out of the side.
const fn hood(quarter: u8, look: [u16; 3]) -> Part {
    let at = [[0.0, 0.42], [-0.42, 0.0], [0.0, -0.42], [0.42, 0.0]][quarter as usize];
    Part {
        center: [at[0], HEADER_Y, at[1]],
        size: HEADER,
        yaw: quarter as f32 * QUARTER,
        tex: look,
        form: Form::Lean(0.42),
    }
}

const POSTS: [Part; 4] = [corner(-0.4, -0.4), corner(0.4, -0.4), corner(-0.4, 0.4), corner(0.4, 0.4)];

/// One hood behind, three headers.
const SPLITTER_MODEL: [Part; 10] = [
    PLATE,
    POSTS[0],
    POSTS[1],
    POSTS[2],
    POSTS[3],
    header(0, OUT),
    header(1, OUT),
    header(3, OUT),
    hood(0, IN),
    roof(tex::SPLITTER_TOP),
];

/// The splitter with a copper hood.
const FILTER_MODEL: [Part; 10] = [
    PLATE,
    POSTS[0],
    POSTS[1],
    POSTS[2],
    POSTS[3],
    header(0, OUT),
    header(1, OUT),
    header(3, OUT),
    hood(0, [tex::COPPER_INGOT; 3]),
    roof(tex::FILTER_TOP),
];

/// Three hoods, one header.
const MERGER_MODEL: [Part; 10] = [
    PLATE,
    POSTS[0],
    POSTS[1],
    POSTS[2],
    POSTS[3],
    header(0, OUT),
    hood(0, IN),
    hood(1, IN),
    hood(3, IN),
    roof(tex::MERGER_TOP),
];
/// A corner post of the lift's cage, stopping under the frame.
const fn post(x: f32, z: f32) -> Part {
    part([x, -0.045, z], [0.08, 0.91, 0.08], tex::FRAME)
}

/// A bar of the lift's top frame.
const fn frame(center: [f32; 3], size: [f32; 3]) -> Part {
    boxed(center, size, [tex::FRAME, tex::stripe(0), tex::FRAME])
}

/// A standing belt panel in a cage of four posts and a top frame, over a belt slab.
const LIFT_MODEL: [Part; 10] = [
    slab(0),
    boxed([0.0, -0.02, -0.4], [0.66, 0.86, 0.08], [top(0); 3]),
    post(-0.42, -0.42),
    post(0.42, -0.42),
    post(-0.42, 0.42),
    post(0.42, 0.42),
    frame([0.0, 0.455, -0.42], [0.92, 0.09, 0.08]),
    frame([-0.42, 0.455, 0.0], [0.08, 0.09, 0.76]),
    frame([0.42, 0.455, 0.0], [0.08, 0.09, 0.76]),
    frame([0.0, 0.455, 0.42], [0.92, 0.09, 0.08]),
];

pub(super) fn parts(item: ItemId) -> &'static [Part] {
    if let Some((block, tier)) = placed_by(item) {
        match block {
            BELT => return &BELTS[usize::from(tier)],
            UNDERPASS_IN => return &UNDERPASSES[usize::from(tier)],
            _ => {}
        }
    }
    match item {
        ITEM_SPLITTER => &SPLITTER_MODEL,
        ITEM_FILTER => &FILTER_MODEL,
        ITEM_MERGER => &MERGER_MODEL,
        ITEM_LIFT => &LIFT_MODEL,
        ITEM_EJECTOR => &EJECTOR_ICON,
        _ => &[],
    }
}
