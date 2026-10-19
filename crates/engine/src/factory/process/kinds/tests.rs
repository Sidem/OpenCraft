//! The kind lists follow the processor list through placing, removing (which moves the last processor into the gap)
//! and upgrading, so a pass never visits a machine that is gone or misses one just placed.

use crate::block::{BlockId, ACCUMULATOR, BOILER, LANDING_PAD, RESEARCH_CENTER, SMELTER, SOLAR_PANEL, TURBINE};
use crate::factory::upgrades::Step;
use crate::factory::Factory;
use crate::math::IVec3;
use crate::world::World;

use super::*;

fn place(f: &mut Factory, world: &mut World, block: BlockId, x: i32) -> IVec3 {
    let pos = IVec3::new(x, 0, 0);
    f.place(world, block, pos, 0, pos - IVec3::new(0, 1, 0), 0);
    pos
}

#[test]
fn the_lists_follow_placing_removing_and_upgrading() {
    let (mut world, mut f) = (World::new(1, 2), Factory { by_hand: true, ..Factory::default() });
    let mut events = Vec::new();
    let first = place(&mut f, &mut world, SOLAR_PANEL, 0);
    let smelter = place(&mut f, &mut world, SMELTER, 10);
    for (x, block) in [(20, ACCUMULATOR), (30, BOILER), (40, TURBINE), (50, LANDING_PAD), (60, RESEARCH_CENTER)] {
        place(&mut f, &mut world, block, x);
    }
    place(&mut f, &mut world, SOLAR_PANEL, 70);
    f.update(&mut world, 1, &mut events);
    let k = &f.kinds;
    assert_eq!((&k.solar[..], &k.accumulator[..], &k.turbine[..]), (&[0, 7][..], &[2][..], &[4][..]));
    assert_eq!((&k.pad[..], &k.center[..], &k.water[..]), (&[5][..], &[6][..], &[3][..]));

    // The last panel moves into the first one's place.
    f.remove(first);
    f.update(&mut world, 2, &mut events);
    assert_eq!(f.kinds.solar, [0]);
    assert_eq!(f.processors[0].pos, IVec3::new(70, 0, 0));
    assert_eq!(f.kinds, Kinds::of(&f.processors));

    // A tier can change how a machine is driven (the Mk3 smelter is electric): it marks the factory for a relink, which
    // makes the lists again.
    while let Some(Step { tier, .. }) = f.next_upgrade(smelter) {
        f.set_tier(smelter, tier);
    }
    assert!(f.dirty);
    f.update(&mut world, 3, &mut events);
    assert_eq!(f.kinds, Kinds::of(&f.processors));
}
