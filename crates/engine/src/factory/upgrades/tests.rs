use super::*;
use crate::block::IRON_ORE;
use crate::factory::MINER_TIERS;

const P: IVec3 = IVec3::new(0, 0, 0);

#[test]
fn kits_name_their_tier() {
    assert_eq!((kit(0), kit(1), kit(2)), (None, Some(GREEN_KIT), None));
    assert_eq!((kit_tier(GREEN_KIT), kit_tier(ItemId::NONE), kit_tier(ItemId::block(BELT))), (Some(1), None, None));
}

#[test]
fn an_upgraded_belt_keeps_its_items_and_doubles_its_speed() {
    let mut f = Factory::default();
    f.add_belt(P, 1);
    assert!(f.belts[0].accept(IRON_ORE.into(), false, 0.3));
    assert_eq!(f.next_upgrade(P), Some(Step { block: BELT, tier: 1, kit: GREEN_KIT, kits: 1 }));
    let speed = f.belt_at(P).speed();
    assert!(f.upgrade(P));
    let belt = f.belt_at(P);
    assert_eq!((belt.tier, belt.dir, belt.items.len(), belt.speed()), (1, 1, 1, speed * 2.0));
    assert_eq!(f.next_upgrade(P), None, "Mk2 is the top tier for now");
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
    f.add_storage(P);
    assert_eq!((f.tiered_at(P), f.next_upgrade(P)), (None, None));
    assert!(!f.upgrade(P));
    assert!(!f.upgrade(IVec3::new(9, 9, 9)), "nothing there");
}
