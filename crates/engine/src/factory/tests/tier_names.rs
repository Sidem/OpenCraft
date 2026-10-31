//! A machine upgraded in place is named at its new tier everywhere it is named: its readout title (`name_at`, which the
//! crosshair, the panel, belts and the stall lesson use), its map mark and hover words, and its analytics row.

use crate::analytics::kind_at;
use crate::factory::tiers::{self, FAMILIES};
use crate::factory::Factory;
use crate::math::IVec3;
use crate::minimap::Subject;
use crate::world::World;

#[test]
fn an_upgraded_machine_is_named_at_its_tier_on_every_screen() {
    let pos = IVec3::new(4, 10, 4);
    for family in FAMILIES {
        for tier in 0..family.items.len() as u8 {
            let mut f = Factory::default();
            f.place(&mut World::new(1, 2), family.block, pos, 0, pos - IVec3::new(0, 1, 0), 0);
            assert!(f.tiered_at(pos).is_some(), "a {} stands", crate::block::def(family.block).name);
            f.set_tier(pos, tier); // what an upgrade in place does
            let want = tiers::name(family.block, tier);
            assert!(want.contains(&format!("Mk{}", tier + 1)), "{want} says its Mk");
            assert_eq!(f.name_at(pos).as_deref(), Some(want.as_str()), "the readout title");

            let mut marked = Vec::new();
            f.map_machines((i32::MIN, i32::MIN), (i32::MAX, i32::MAX), &mut |block, t, p| marked.push((block, t, p)));
            for &(block, t, _) in marked.iter().filter(|m| m.2 == pos) {
                assert_eq!(Subject::Machine { block, tier: t, dry: false }.words().0, want, "the map's hover words");
            }

            let kind = tiers::item_of(family.block, tier).expect("tier item").0;
            assert_eq!(kind_at(&f, family.block, pos), kind, "the analytics row of a {want}");
        }
    }
}
