use super::*;
use crate::block::IRON_ORE;
use crate::factory::MINER_TIERS;
use crate::world::World;

const P: IVec3 = IVec3::new(0, 0, 0);

#[test]
fn kits_name_their_tier() {
    assert_eq!((kit(0), kit(1), kit(2), kit(3)), (None, Some(GREEN_KIT), Some(BLUE_KIT), Some(VIOLET_KIT)));
    assert_eq!((kit(4), kit(5)), (Some(GOLD_KIT), None));
    assert_eq!(
        (kit_tier(GREEN_KIT), kit_tier(BLUE_KIT), kit_tier(VIOLET_KIT), kit_tier(GOLD_KIT)),
        (Some(1), Some(2), Some(3), Some(4))
    );
    assert_eq!(kit_tier(ItemId::NONE), None);
    assert_eq!(kit_tier(ItemId::block(BELT)), None);
}

#[test]
fn an_upgraded_belt_keeps_its_items_and_each_tier_doubles_its_speed() {
    let mut f = Factory::default();
    f.add_belt(P, 1);
    assert!(f.belts[0].accept(IRON_ORE.into(), 1, false, 0.3));
    assert_eq!(f.next_upgrade(P), Some(Step { block: BELT, tier: 1, kit: GREEN_KIT, kits: 1 }));
    let speed = f.belt_at(P).speed();
    assert!(f.upgrade(P));
    let belt = f.belt_at(P);
    assert_eq!((belt.tier, belt.dir, belt.items.len(), belt.speed()), (1, 1, 1, speed * 2.0));
    assert_eq!(f.next_upgrade(P), Some(Step { block: BELT, tier: 2, kit: BLUE_KIT, kits: 1 }));
    assert!(f.upgrade(P));
    assert_eq!((f.belt_at(P).tier, f.belt_at(P).speed(), f.belt_at(P).items.len()), (2, speed * 4.0, 1));
    assert_eq!(f.next_upgrade(P), Some(Step { block: BELT, tier: 3, kit: VIOLET_KIT, kits: 1 }));
    assert!(f.upgrade(P));
    assert_eq!(f.belt_at(P).speed(), speed * 8.0);
    assert_eq!(f.next_upgrade(P), None, "Mk4 is the top tier for now");
    assert!(!f.upgrade(P));
}

#[test]
fn an_upgraded_miner_keeps_its_buffer_and_takes_four_kits() {
    let mut f = Factory::default();
    f.add_miner(P, 3, None, 0);
    f.miners[0].out.add(IRON_ORE.into(), 5);
    assert_eq!(f.next_upgrade(P).map(|s| s.kits), Some(4));
    assert!(f.upgrade(P));
    let m = f.miner_at(P);
    assert_eq!((m.tier, m.drill, m.out.total()), (1, 3, 5));
    assert_eq!(m.stats().power, MINER_TIERS[1].power);
}

#[test]
fn only_tiered_machines_upgrade() {
    let mut f = Factory::default();
    f.place(&mut World::new(1, 2), crate::block::SPLITTER, P, 0, P, 0);
    assert_eq!((f.tiered_at(P), f.next_upgrade(P)), (None, None));
    assert!(!f.upgrade(P));
    assert!(!f.upgrade(IVec3::new(9, 9, 9)), "nothing there");
}

#[test]
fn every_family_climbs_to_its_top_tier_one_kit_colour_at_a_time() {
    for family in tiers::FAMILIES {
        let mut f = Factory::default();
        match family.block {
            BELT => f.add_belt(P, 1),
            MINER => f.add_miner(P, 3, None, 0),
            block => f.place(&mut World::new(1, 2), block, P, 0, P, 0),
        }
        for tier in 1..family.items.len() as u8 {
            let step = f.next_upgrade(P).unwrap();
            assert_eq!(
                (step.tier, Some(step.kit), step.kits),
                (tier, kit(tier), family.kits),
                "block {}",
                family.block
            );
            assert!(f.upgrade(P));
        }
        assert_eq!(f.tiered_at(P), Some((family.block, family.items.len() as u8 - 1)));
        assert_eq!(f.next_upgrade(P), None, "block {} tops out", family.block);
    }
}
