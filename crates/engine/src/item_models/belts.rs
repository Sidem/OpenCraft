//! Item models of the belt family, so an icon shows what the thing is and not a cube: a belt is a low slab between two
//! rails (as `factory/belt.rs` draws it, wearing its tier's top and stripe), a splitter a low housing with guards, a
//! filter the same with a bridge over it, a lift a cage round a standing belt, an underpass a belt running into a hood,
//! and the ejector a funnel and a bent tube (`factory/process/ejector.rs`). Sizes are those of the placed models; the
//! parts rest on the cell's floor (y = -0.5). Add a const and a `parts` row.

use super::{part, Form, Part};
use crate::block::{tex, BELT, UNDERPASS_IN};
use crate::factory::{tiers::placed_by, BELT_TIERS, EJECTOR_ICON};
use crate::item::ItemId;

const ITEM_SPLITTER: ItemId = ItemId::block(crate::block::SPLITTER);
const ITEM_FILTER: ItemId = ItemId::block(crate::block::FILTER);
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

/// A low housing under two fork guards and the plate that parts the stream.
const SPLITTER_MODEL: [Part; 4] = [
    boxed([0.0, -0.36, 0.0], [0.96, 0.28, 0.96], [tex::SPLITTER_TOP, tex::FRAME, tex::FRAME]),
    part([-0.39, -0.16, 0.0], [0.08, 0.19, 0.85], tex::FRAME),
    part([0.0, -0.16, -0.26], [0.5, 0.1, 0.36], tex::IRON_PLATE),
    part([0.39, -0.16, 0.0], [0.08, 0.19, 0.85], tex::FRAME),
];

/// The same housing with a copper inspection bridge on two posts.
const FILTER_MODEL: [Part; 6] = [
    boxed([0.0, -0.36, 0.0], [0.96, 0.28, 0.96], [tex::FILTER_TOP, tex::FRAME, tex::FRAME]),
    part([-0.39, -0.16, 0.0], [0.08, 0.19, 0.85], tex::FRAME),
    part([-0.35, 0.1, 0.02], [0.1, 0.38, 0.12], tex::FRAME),
    part([0.0, 0.31, 0.02], [0.8, 0.09, 0.18], tex::COPPER_INGOT),
    part([0.39, -0.16, 0.0], [0.08, 0.19, 0.85], tex::FRAME),
    part([0.35, 0.1, 0.02], [0.1, 0.38, 0.12], tex::FRAME),
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
        ITEM_LIFT => &LIFT_MODEL,
        ITEM_EJECTOR => &EJECTOR_ICON,
        _ => &[],
    }
}
