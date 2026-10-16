use super::*;
use crate::block::IRON_ORE;

#[test]
fn the_tip_waits_for_a_few_blocks_and_adds_them_up() {
    let mut w = HandWaste::default();
    assert_eq!((w.notice(), w.tip()), (String::new(), String::new()));
    for _ in 0..WARN_FROM - 1 {
        w.record(IRON_ORE, 3, 97);
    }
    assert_eq!(w.notice(), "+3 Iron Ore · *97 lost* with the block");
    assert!(w.tip().is_empty(), "no warning before {WARN_FROM} blocks");
    w.record(IRON_ORE, 4, 996);
    assert!(w.tip().contains("*10 ore* from 3 blocks and *1,190 more was lost*"), "{}", w.tip());
}
