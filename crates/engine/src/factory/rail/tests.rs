use super::*;
use crate::block::RAIL;
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::render::INSTANCE_FLOATS;
use crate::world::World;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn track(cells: &[IVec3]) -> Factory {
    let mut f = Factory::default();
    for &c in cells {
        f.place(&mut World::new(1, 2), RAIL, c, 0, c, 0);
    }
    f.relink();
    f
}

fn arms_at(f: &Factory, pos: IVec3) -> u16 {
    let Some(&Slot::Rail(i)) = f.at.get(&pos) else { panic!("no rail at {pos:?}") };
    f.rails[i as usize].arms
}

fn bit(d: u8, k: u8) -> u16 {
    1 << (d * 3 + k)
}

#[test]
fn rails_join_level_and_a_block_up_or_down() {
    // East along y 0, then a rise to y 1 and a drop to y 0 again, with a rail north of the first that isn't beside it.
    let f = track(&[v(0, 0, 0), v(1, 0, 0), v(2, 1, 0), v(3, 0, 0), v(0, 0, -2)]);
    assert_eq!(arms_at(&f, v(0, 0, 0)), bit(1, LEVEL), "joins the one beside it, not the one two away");
    assert_eq!(arms_at(&f, v(1, 0, 0)), bit(3, LEVEL) | bit(1, RISE));
    assert_eq!(arms_at(&f, v(2, 1, 0)), bit(3, DROP) | bit(1, DROP));
    assert_eq!(arms_at(&f, v(3, 0, 0)), bit(3, RISE));
    assert_eq!(arms_at(&f, v(0, 0, -2)), 0, "a lone rail joins nothing");
}

#[test]
fn a_rail_two_levels_off_or_diagonal_does_not_join() {
    let f = track(&[v(0, 0, 0), v(1, 2, 0), v(1, 0, 1), v(-1, 0, -1)]);
    assert!((0..4).all(|i| arms_at(&f, [v(0, 0, 0), v(1, 2, 0), v(1, 0, 1), v(-1, 0, -1)][i]) == 0));
}

#[test]
fn removing_a_rail_cuts_the_joins_and_the_readout_counts_them() {
    let mut f = track(&[v(0, 0, 0), v(1, 0, 0), v(2, 0, 0)]);
    assert_eq!(f.describe(v(1, 0, 0)).unwrap(), "Rail\nJoined to 2 rails");
    assert!(f.describe(v(0, 0, 0)).unwrap().contains("End of the track"));
    assert_eq!(f.rail_joins(v(1, 0, 0)).len(), 2);
    f.remove(v(1, 0, 0));
    f.relink();
    assert_eq!((arms_at(&f, v(0, 0, 0)), arms_at(&f, v(2, 0, 0))), (0, 0));
    assert!(f.rail_joins(v(1, 0, 0)).is_empty(), "nothing is there");
}

#[test]
fn a_track_saves_and_loads() {
    let f = track(&[v(0, 0, 0), v(1, 0, 0), v(1, 0, 1), v(2, 1, 1)]);
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).expect("reads back");
    back.relink();
    assert_eq!(back.count(crate::factory::Kind::Rail), 4);
    for c in [v(0, 0, 0), v(1, 0, 0), v(1, 0, 1), v(2, 1, 1)] {
        assert_eq!(arms_at(&back, c), arms_at(&f, c));
    }
}

#[test]
fn ghosts_and_models_draw_whole_boxes_and_tilt_the_rises() {
    assert_eq!(ghost_arms(1, Shape::Flat), bit(1, LEVEL) | bit(3, LEVEL));
    assert_eq!(ghost_arms(1, Shape::Up), bit(1, RISE) | bit(3, LEVEL));
    assert_eq!(ghost_arms(0, Shape::Down), bit(0, LEVEL) | bit(2, RISE));
    let (mut flat, mut sloped) = (Vec::new(), Vec::new());
    write_rail(&mut flat, Vec3::ZERO, ghost_arms(0, Shape::Flat));
    write_rail(&mut sloped, Vec3::ZERO, ghost_arms(0, Shape::Up));
    assert_eq!(flat.len() % INSTANCE_FLOATS, 0);
    assert_eq!(flat.chunks_exact(INSTANCE_FLOATS).count(), 1 + 2 * 3, "a sleeper, then two rails and a sleeper a side");
    assert!(flat.chunks_exact(INSTANCE_FLOATS).all(|b| b[12] == 0.0), "level track is not tilted");
    assert!(sloped.chunks_exact(INSTANCE_FLOATS).any(|b| b[12] > 0.7), "a rise is tilted by 45°");
    // A bare rail still draws a straight piece.
    let mut bare = Vec::new();
    write_rail(&mut bare, Vec3::ZERO, 0);
    assert_eq!(bare.len(), flat.len());
}

#[test]
fn rails_are_laid_in_lines_like_belts() {
    use crate::belt_line::is_laid_in_lines;
    assert!(is_laid_in_lines(RAIL) && is_laid_in_lines(crate::block::BELT) && !is_laid_in_lines(crate::block::STORAGE));
    assert!(crate::block::def(RAIL).placeable && !crate::block::SOLID[RAIL as usize], "a thin block to walk over");
}
