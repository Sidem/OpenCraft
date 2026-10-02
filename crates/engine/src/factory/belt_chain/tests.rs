use super::*;
use crate::factory::tests::run;

const EAST: u8 = 1;
const NORTH: u8 = 0;

fn at(x: i32, z: i32) -> IVec3 {
    IVec3::new(x, 0, z)
}

/// Belts east along z = 0 from x = 0 to 6, a side belt at (5, 1) feeding into x = 5, and x = 3 upgraded.
fn line() -> Factory {
    let mut f = Factory::default();
    for x in 0..=6 {
        f.add_belt(at(x, 0), EAST);
    }
    f.add_belt(at(5, 1), NORTH);
    assert!(f.upgrade(at(3, 0)));
    run(&mut f, 0.05, |_| {});
    f
}

#[test]
fn a_chain_follows_belts_of_one_tier_both_ways_and_through_side_joins() {
    let f = line();
    let sorted = |mut v: Vec<IVec3>| {
        v.sort_by_key(|p| (p.x, p.z));
        v
    };
    assert_eq!(sorted(f.belt_chain(at(0, 0), 99)), [at(0, 0), at(1, 0), at(2, 0)], "stops at the Mk2 belt");
    assert_eq!(f.belt_chain(at(3, 0), 99), [at(3, 0)], "a lone Mk2 belt");
    let from_middle = f.belt_chain(at(5, 0), 99);
    assert_eq!(from_middle[0], at(5, 0), "the start comes first");
    assert_eq!(sorted(from_middle), [at(4, 0), at(5, 0), at(5, 1), at(6, 0)], "upstream, downstream and the side belt");
}

#[test]
fn a_chain_is_capped_and_empty_where_no_belt_stands() {
    let f = line();
    assert_eq!(f.belt_chain(at(4, 0), 2).len(), 2);
    assert!(f.belt_chain(at(0, 5), 99).is_empty());
}
