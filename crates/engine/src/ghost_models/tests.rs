use super::*;
use crate::block;
use crate::blueprint::Entry;
use crate::factory::INSTANCE_FLOATS;

const EYE: Vec3 = Vec3::new(0.0, 40.0, 0.0);

fn ghost(block: BlockId, x: i32, z: i32) -> Ghost {
    Ghost { pos: IVec3::new(x, 40, z), block, facing: 1, tier: 0 }
}

fn planted(list: &[Ghost]) -> Ghosts {
    let mut g = Ghosts::default();
    list.iter().for_each(|&x| assert!(g.set(x)));
    g
}

fn drawn(m: &mut GhostModels, g: &Ghosts, stamp: Option<(StampKey, &Blueprint)>) -> Vec<f32> {
    let mut out = Vec::new();
    m.write(g, stamp, EYE, 0.0, &mut out);
    out
}

#[test]
fn a_planted_machine_shows_as_its_model_not_a_single_box() {
    let mut m = GhostModels::default();
    let out = drawn(&mut m, &planted(&[ghost(block::SMELTER, 6, 4)]), None);
    assert!(out.len() / INSTANCE_FLOATS > 1, "a smelter is several parts: {}", out.len() / INSTANCE_FLOATS);
}

#[test]
fn a_plain_block_is_a_cube_and_a_tear_down_mark_nothing() {
    let mut m = GhostModels::default();
    let out = drawn(&mut m, &planted(&[ghost(block::STONE, 6, 4), ghost(block::AIR, 8, 4)]), None);
    assert_eq!(out.len() / INSTANCE_FLOATS, 1, "one cube for the stone, none for the mark");
}

#[test]
fn ghosts_out_of_range_are_not_drawn() {
    let mut m = GhostModels::default();
    let far = ghost(block::SMELTER, 500, 0);
    assert!(drawn(&mut m, &planted(&[far]), None).is_empty());
}

#[test]
fn the_set_follows_the_ghosts() {
    let mut m = GhostModels::default();
    let a = planted(&[ghost(block::SMELTER, 6, 4)]);
    let one = drawn(&mut m, &a, None).len();
    let two = drawn(&mut m, &planted(&[ghost(block::SMELTER, 6, 4), ghost(block::SMELTER, 6, 10)]), None).len();
    assert_eq!(two, one * 2);
    assert_eq!(drawn(&mut m, &Ghosts::default(), None).len(), 0, "all removed");
}

#[test]
fn a_blueprint_stamp_shows_its_machines_where_it_would_stand_and_follows_the_aim() {
    let bp = Blueprint {
        name: "Line".into(),
        size: IVec3::new(5, 1, 1),
        entries: vec![
            Entry { off: IVec3::new(0, 0, 0), block: block::SMELTER, facing: 1, tier: 0 },
            Entry { off: IVec3::new(4, 0, 0), block: block::SMELTER, facing: 1, tier: 0 },
        ],
    };
    let mut m = GhostModels::default();
    let none = Ghosts::default();
    let at = |x| Some(((0, 0, IVec3::new(x, 40, 4), 1), &bp));
    let near = drawn(&mut m, &none, at(5));
    assert!(near.len() / INSTANCE_FLOATS > 2, "both smelters are models");
    let moved = drawn(&mut m, &none, at(6));
    assert_eq!(moved.len(), near.len());
    assert_ne!(moved, near, "aiming one block over moves the models");
    assert!(drawn(&mut m, &none, None).is_empty(), "no stamp, no models");
}
