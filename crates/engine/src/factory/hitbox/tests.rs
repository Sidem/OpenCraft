use super::super::tests::place_block;
use super::*;
use crate::block::{CABLE, POLE, SMELTER};

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn ray(b: &Boxes, x: f64, y: f64) -> bool {
    b.enter(Vec3::new(x, y, 3.0), Vec3::new(0.0, 0.0, -1.0)).is_some()
}

#[test]
fn a_pole_is_hit_on_its_post_but_not_in_the_empty_corner_of_its_cell() {
    let mut f = Factory::default();
    place_block(&mut f, POLE, v(0, 5, 0));
    let pole = f.hit_boxes(v(0, 5, 0)).unwrap();
    assert!(ray(&pole, 0.5, 0.5), "the post");
    assert!(ray(&pole, 0.2, 1.84), "the crossarm, a block up");
    assert!(!ray(&pole, 0.2, 0.84), "no crossarm at the old height");
    assert!(!ray(&pole, 0.1, 0.3), "empty air beside the post");
    assert!(!ray(&pole, 0.95, 0.9));
}

#[test]
fn a_pole_stands_two_blocks_tall_and_its_upper_half_is_hit_from_the_cell_above() {
    let mut f = Factory::default();
    place_block(&mut f, POLE, v(0, 5, 0));
    place_block(&mut f, CABLE, v(4, 5, 0));
    assert_eq!(f.pole_below(v(0, 6, 0)), Some(v(0, 5, 0)));
    assert_eq!((f.pole_below(v(0, 7, 0)), f.pole_below(v(4, 6, 0))), (None, None), "only a pole's own cell, no cable");
    let upper = f.tall_pole_boxes(v(0, 6, 0)).unwrap();
    assert!(ray(&upper, 0.5, 0.2), "the post goes on");
    assert!(ray(&upper, 0.2, 0.84), "the crossarm");
    assert!(!ray(&upper, 0.1, 0.3));
    // The wires hang from the crossarm, in the cell above.
    let wire = f.pole_top(v(0, 5, 0)).unwrap().y - 5.0;
    assert!((1.7..2.1).contains(&wire), "{wire}");
    let [_, lo, _, _, height, _] = f.hit_boxes(v(0, 5, 0)).unwrap().bounds();
    assert!(lo + height > 2.0, "the outline reaches the crossarm: {}", lo + height);
}

#[test]
fn a_belt_is_low_and_its_items_are_not_part_of_it() {
    let mut f = Factory::default();
    f.add_belt(v(0, 5, 0), 0);
    let belt = f.hit_boxes(v(0, 5, 0)).unwrap();
    assert!(ray(&belt, 0.5, 0.1));
    assert!(!ray(&belt, 0.5, 0.8), "above the belt");
    let [_, _, _, _, height, _] = belt.bounds();
    assert!(height < 0.5, "{height}");
}

#[test]
fn a_cable_is_a_small_cube_and_a_big_machine_or_nothing_is_the_whole_cell() {
    let mut f = Factory::default();
    place_block(&mut f, CABLE, v(0, 5, 0));
    place_block(&mut f, SMELTER, v(4, 5, 0));
    let cable = f.hit_boxes(v(0, 5, 0)).unwrap();
    assert!(ray(&cable, 0.5, 0.5) && !ray(&cable, 0.1, 0.1));
    assert!(f.hit_boxes(v(4, 5, 0)).is_none());
    assert!(f.hit_boxes(v(9, 5, 0)).is_none());
}
