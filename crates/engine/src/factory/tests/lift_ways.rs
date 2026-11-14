//! Lift columns carrying items down (`belt_shape::derive_lift_ways`, `Shape::Drop`).

use super::*;
use crate::block::LIFT;

fn v(x: i32, y: i32) -> IVec3 {
    IVec3::new(x, y, 0)
}

#[test]
fn a_lift_column_fed_over_its_top_carries_items_down_and_on_ahead() {
    let mut f = Factory::default();
    stocked_box(&mut f, v(0, 3), IRON_ORE.into(), 6);
    f.add_belt(v(1, 3), EAST);
    for y in 0..3 {
        shaped(&mut f, LIFT, v(2, y), EAST);
    }
    f.add_storage(v(3, 0));
    run(&mut f, 20.0, spacing_ok);
    assert_eq!(f.storage_count_at(v(3, 0), IRON_ORE.into()), 6);
    let (bottom, top) = (f.belt_at(v(2, 0)), f.belt_at(v(2, 2)));
    assert_eq!((bottom.shape, top.shape), (Shape::Drop, Shape::Drop));
    assert_eq!((bottom.lift_below, bottom.lift_above, top.lift_below, top.lift_above), (false, true, true, false));
    assert_eq!(top.item_at(0.0), (-0.5, 1.0, 0.0), "in over the top from behind");
    assert_eq!(bottom.item_at(1.0), (0.5, 0.0, 0.0), "out along the floor ahead");
    assert!(top.fed_by_belt, "the belt behind and above hands onto the top");
    let mut g = round_trip(&f);
    g.relink();
    assert_eq!(g.belt_at(v(2, 1)).shape, Shape::Drop, "a save keeps it a drop");
}

#[test]
fn a_column_fed_at_its_foot_or_by_nothing_stays_a_lift() {
    let mut f = Factory::default();
    f.add_belt(v(1, 3), EAST); // over the top
    f.add_belt(v(1, 0), EAST); // and at the foot: the foot wins, as before drops
    for y in 0..3 {
        shaped(&mut f, LIFT, v(2, y), EAST);
    }
    shaped(&mut f, LIFT, v(6, 0), EAST); // alone
    shaped(&mut f, LIFT, v(9, 0), EAST); // a belt over its top facing another way
    f.add_belt(v(8, 1), SOUTH);
    f.relink();
    for p in [v(2, 0), v(2, 2), v(6, 0), v(9, 0)] {
        assert_eq!(f.belt_at(p).shape, Shape::Lift, "{p:?}");
    }
}

#[test]
fn a_drop_column_can_feed_the_top_of_another_below_it() {
    let mut f = Factory::default();
    stocked_box(&mut f, v(0, 5), IRON_ORE.into(), 5);
    f.add_belt(v(1, 5), EAST);
    for y in 3..5 {
        shaped(&mut f, LIFT, v(2, y), EAST);
    }
    for y in 0..3 {
        shaped(&mut f, LIFT, v(3, y), EAST);
    }
    f.add_storage(v(4, 0));
    run(&mut f, 25.0, spacing_ok);
    assert_eq!(f.belt_at(v(3, 0)).shape, Shape::Drop);
    assert_eq!(f.storage_count_at(v(4, 0), IRON_ORE.into()), 5);
}
