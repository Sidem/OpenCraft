use super::*;
use crate::block::{BlockId, UNDERPASS_IN, UNDERPASS_OUT};
use crate::factory::links::Link;
use crate::factory::tests::run;
use crate::factory::Factory;
use crate::math::IVec3;
use crate::world::World;

const EAST: u8 = 1;

fn place(f: &mut Factory, block: BlockId, pos: IVec3, dir: u8, tier: u8) {
    f.place(&mut World::new(1, 2), block, pos, dir, pos, tier);
}

/// Underpasses of `tier` at `xs` along z = 0, all facing east, and the role each ends up with.
fn roles(f: &mut Factory, xs: &[i32], tier: u8) -> Vec<Shape> {
    for &x in xs {
        place(f, UNDERPASS_IN, IVec3::new(x, 0, 0), EAST, tier);
    }
    run(f, 0.1, |_| {});
    xs.iter().map(|&x| f.belt_at(IVec3::new(x, 0, 0)).shape).collect()
}

#[test]
fn spans_grow_two_blocks_a_tier() {
    assert_eq!(UNDERPASS_SPAN, [4, 6, 8, 10]);
    assert_eq!([0, 1, 2, 3, 9].map(span), [4, 6, 8, 10, 10]);
}

#[test]
fn pieces_in_a_line_pair_up_in_order_and_a_lone_one_dangles() {
    use Shape::{Entry, Exit};
    let mut f = Factory::default();
    // Mk1 reaches an exit at most 5 cells ahead (four blocks under).
    assert_eq!(roles(&mut f, &[0, 5, 6, 11, 20], 0), [Entry, Exit, Entry, Exit, Entry]);
    assert_eq!(f.belt_at(IVec3::new(0, 0, 0)).out, Link::Belt { belt: 1, mid: false });
    assert_eq!(f.belt_at(IVec3::new(20, 0, 0)).out, Link::None, "nothing ahead: a dangling entry");
    assert!(f.describe(IVec3::new(20, 0, 0)).unwrap().contains("needs another underpass"));
}

#[test]
fn a_piece_too_far_for_its_tier_does_not_pair_and_a_higher_tier_reaches_further() {
    use Shape::{Entry, Exit};
    // Six cells apart is five blocks under: past a Mk1's four, within a Mk2's six.
    assert_eq!(roles(&mut Factory::default(), &[0, 6], 0), [Entry, Entry]);
    assert_eq!(roles(&mut Factory::default(), &[0, 6], 1), [Entry, Exit]);
    assert_eq!(roles(&mut Factory::default(), &[0, 11], 3), [Entry, Exit], "a Mk4 passes under ten blocks");
    assert_eq!(roles(&mut Factory::default(), &[0, 12], 3), [Entry, Entry]);
}

#[test]
fn only_pieces_facing_the_same_way_in_the_same_line_pair() {
    use Shape::{Entry, Exit};
    let mut f = Factory::default();
    place(&mut f, UNDERPASS_IN, IVec3::new(0, 0, 0), EAST, 0);
    place(&mut f, UNDERPASS_IN, IVec3::new(3, 0, 1), EAST, 0); // the next row over
    place(&mut f, UNDERPASS_IN, IVec3::new(3, 1, 0), EAST, 0); // a level up
    place(&mut f, UNDERPASS_IN, IVec3::new(4, 0, 0), 3, 0); // facing back
    run(&mut f, 0.1, |_| {});
    let shape = |f: &Factory, x, y, z| f.belt_at(IVec3::new(x, y, z)).shape;
    let alone = [shape(&f, 0, 0, 0), shape(&f, 3, 0, 1), shape(&f, 3, 1, 0), shape(&f, 4, 0, 0)];
    assert_eq!(alone, [Entry; 4]);
    place(&mut f, UNDERPASS_IN, IVec3::new(2, 0, 0), EAST, 0);
    run(&mut f, 0.1, |_| {});
    assert_eq!((shape(&f, 0, 0, 0), shape(&f, 2, 0, 0)), (Entry, Exit));
}

#[test]
fn the_old_exit_block_is_just_an_underpass_and_the_roles_survive_a_save() {
    use crate::bytes::{ByteReader, ByteWriter};
    use Shape::{Entry, Exit};
    let mut f = Factory::default();
    place(&mut f, UNDERPASS_IN, IVec3::new(0, 0, 0), EAST, 0);
    place(&mut f, UNDERPASS_OUT, IVec3::new(3, 0, 0), EAST, 0);
    run(&mut f, 0.1, |_| {});
    assert_eq!((f.belt_at(IVec3::new(0, 0, 0)).shape, f.belt_at(IVec3::new(3, 0, 0)).shape), (Entry, Exit));
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut g = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    run(&mut g, 0.1, |_| {});
    assert_eq!((g.belt_at(IVec3::new(0, 0, 0)).shape, g.belt_at(IVec3::new(3, 0, 0)).shape), (Entry, Exit));
    assert_eq!(g.tiered_at(IVec3::new(3, 0, 0)), Some((UNDERPASS_IN, 0)));
}
