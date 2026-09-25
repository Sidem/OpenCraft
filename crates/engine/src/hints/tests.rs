use super::*;
use crate::block::{BELT, CONSTRUCTOR, GENERATOR, SMELTER, STORAGE};
use crate::math::IVec3;
use crate::world::World;

fn place(f: &mut Factory, block: crate::block::BlockId, x: i32) {
    let pos = IVec3::new(x, 0, 0);
    f.place(&mut World::new(1, 2), block, pos, 0, pos);
}

#[test]
fn hints_follow_what_the_player_has_done() {
    let (mut inv, mut f) = (Inventory::default(), Factory::default());
    assert_eq!(progress(&inv, &f), 0);
    inv.add(IRON_ORE.into(), 3);
    assert_eq!(progress(&inv, &f), 1, "found ore");
    inv.add(MINER.into(), 1);
    assert_eq!(progress(&inv, &f), 2, "crafted a miner");
    place(&mut f, MINER, 0);
    assert_eq!(progress(&inv, &f), 3, "placed it");
    place(&mut f, BELT, 1);
    assert_eq!(progress(&inv, &f), 3, "a belt but no box yet");
    place(&mut f, STORAGE, 2);
    assert_eq!(progress(&inv, &f), 4);
    place(&mut f, SMELTER, 3);
    assert_eq!(progress(&inv, &f), 5);
    place(&mut f, CONSTRUCTOR, 4);
    place(&mut f, GENERATOR, 5);
    assert_eq!(progress(&inv, &f), 6);
    f.research.add_unit(0);
    assert_eq!(progress(&inv, &f), HINTS.len(), "all done");
    assert!(HINTS.iter().all(|h| !h.text.is_empty()));
}

#[test]
fn a_later_step_skips_the_earlier_hints() {
    let (inv, mut f) = (Inventory::default(), Factory::default());
    place(&mut f, SMELTER, 0);
    assert_eq!(progress(&inv, &f), 5);
}
