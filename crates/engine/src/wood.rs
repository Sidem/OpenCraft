//! The woods: which tree gives which log, and each wood's planks, floor and door (`block/woods.rs`). Wood 0 is the
//! oak (`LOG`, `PLANKS`, `WOOD_FLOOR`, `WOOD_DOOR`); woods 1–9 follow in `block::OTHER_WOODS` order (birch, pine,
//! palm, cherry, baobab, acacia, redwood, eucalyptus, bamboo).
//!
//! Invariants: every wood's log, planks, floor and door are interchangeable in recipes. A recipe names the oak's,
//! and `like` / `standard` let any wood's stand in (crafting, machines, towns). A recipe whose output is a wood item
//! made from a wood item makes the same wood's (`in_wood_of`), so pine logs saw into pine planks.
//!
//! To add a wood: its blocks in `block/woods.rs`, its log item in `item.rs`, a row in `TREE_WOODS`.

use crate::block::{self, BlockId, LOG, PLANKS, WOOD_DOOR, WOOD_FLOOR};
use crate::item::{ItemId, FIRST_WOOD_LOG};

/// The oak and the woods after it.
pub const WOODS: usize = 1 + block::OTHER_WOODS;

/// What a wood item is, whatever its wood.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    Log,
    Planks,
    Floor,
    Door,
}

const FORMS: [Form; 4] = [Form::Log, Form::Planks, Form::Floor, Form::Door];

/// Wood `w`'s item of `form`.
pub fn item(w: usize, form: Form) -> ItemId {
    debug_assert!(w < WOODS);
    if w == 0 {
        return match form {
            Form::Log => ItemId::block(LOG),
            Form::Planks => ItemId::block(PLANKS),
            Form::Floor => ItemId::block(WOOD_FLOOR),
            Form::Door => ItemId::block(WOOD_DOOR),
        };
    }
    let o = w - 1;
    match form {
        Form::Log => ItemId(FIRST_WOOD_LOG.0 + o as u16),
        Form::Planks => ItemId::block(block::planks_of(o)),
        Form::Floor => ItemId::block(block::floor_of(o)),
        Form::Door => ItemId::block(block::door_of(o)),
    }
}

/// The wood and form of a wood item; `None` for anything else.
pub fn of(id: ItemId) -> Option<(usize, Form)> {
    let i = id.0;
    if (FIRST_WOOD_LOG.0..FIRST_WOOD_LOG.0 + block::OTHER_WOODS as u16).contains(&i) {
        return Some((1 + (i - FIRST_WOOD_LOG.0) as usize, Form::Log));
    }
    let b = id.places()?;
    match b {
        LOG => Some((0, Form::Log)),
        PLANKS => Some((0, Form::Planks)),
        WOOD_FLOOR => Some((0, Form::Floor)),
        WOOD_DOOR => Some((0, Form::Door)),
        _ => match block::wood_block(b) {
            Some((o, f)) if f < 3 => Some((1 + o, FORMS[f + 1])),
            _ => None,
        },
    }
}

/// The oak's item of the same form: what recipes name. Anything that is no wood item stays itself.
pub fn standard(id: ItemId) -> ItemId {
    of(id).map_or(id, |(_, f)| item(0, f))
}

/// Whether `have` will do where a recipe asks for `want`: the same item, or the same form of another wood.
pub fn like(have: ItemId, want: ItemId) -> bool {
    have == want || standard(have) == want && standard(want) == want
}

/// `out` in the wood of `input`, when both are wood items; else `out` as it is.
pub fn in_wood_of(out: ItemId, input: ItemId) -> ItemId {
    match (of(out), of(input)) {
        (Some((_, f)), Some((w, _))) => item(w, f),
        _ => out,
    }
}

/// What can stand in for `want`, in the order stock is used up: for an oak item (what recipes name) the same form of
/// every wood, oak's first, then by wood number; any other item only itself.
pub fn kinds(want: ItemId) -> impl Iterator<Item = ItemId> {
    let form = of(want).filter(|&(w, _)| w == 0).map(|(_, f)| f);
    (0..if form.is_some() { WOODS } else { 1 }).map(move |w| form.map_or(want, |f| item(w, f)))
}

/// What a craft or batch makes of `out`: in the wood of the first wood item among `used` (so pine planks make pine
/// floor), else `out` itself.
pub fn made_from(out: ItemId, used: impl IntoIterator<Item = ItemId>) -> ItemId {
    used.into_iter().find(|&i| of(i).is_some()).map_or(out, |i| in_wood_of(out, i))
}

/// Which wood a grown tree's log block is: the log a felled tree drops (`sim/felling.rs`). `None`: no tree wood.
pub fn of_tree(b: BlockId) -> Option<usize> {
    TREE_WOODS.iter().position(|ids| ids.contains(&b))
}

/// The tree blocks of each wood. The snag (dead wood) gives the oak's logs.
const TREE_WOODS: [&[BlockId]; WOODS] = [
    &[block::LOG, block::OAK_TREE_LOG, block::SNAG_LOG],
    &[block::BIRCH_LOG],
    &[block::PINE_LOG],
    &[block::PALM_LOG],
    &[block::CHERRY_LOG],
    &[block::BAOBAB_LOG],
    &[block::ACACIA_LOG],
    &[block::REDWOOD_LOG],
    &[block::EUCALYPTUS_LOG],
    &[block::BAMBOO],
];

#[cfg(test)]
mod tests;
