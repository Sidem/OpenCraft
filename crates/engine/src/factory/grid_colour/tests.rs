use super::*;
use crate::block::POLE;
use crate::math::IVec3;
use crate::world::World;

fn at(x: i32) -> IVec3 {
    IVec3::new(x, 3, 0)
}

/// Poles at the given x positions (wired by hand only), relinked.
fn poles(xs: &[i32]) -> Factory {
    let mut f = Factory { by_hand: true, ..Factory::default() };
    let mut world = World::new(1, 2);
    for &x in xs {
        f.place(&mut world, POLE, at(x), 0, at(x), 0);
    }
    f.update(&mut world, 1, &mut Vec::new());
    f
}

fn layers(f: &Factory) -> Vec<u16> {
    (0..f.poles.len() as u32).map(|i| f.power.pole_layer(i)).collect()
}

#[test]
fn the_biggest_grid_is_main_and_wears_the_first_colour() {
    let mut f = poles(&[0, 4, 8, 40]);
    f.connect(at(0), at(4));
    f.connect(at(4), at(8));
    f.update(&mut World::new(1, 2), 2, &mut Vec::new());
    let main = tex::GRID_FIRST;
    assert_eq!(layers(&f), [main, main, main, main + 1], "three wired poles are one colour, the loner another");
    assert!(f.power.grid_label(0).starts_with("Main grid (blue)"));
    assert!(f.power.grid_label(3).starts_with("Separate grid (orange)"));
}

#[test]
fn wiring_two_grids_together_gives_them_one_colour() {
    let mut f = poles(&[0, 4]);
    assert_eq!(layers(&f), [tex::GRID_FIRST, tex::GRID_FIRST + 1], "equal grids: the lower one is main");
    f.connect(at(0), at(4));
    f.update(&mut World::new(1, 2), 2, &mut Vec::new());
    assert_eq!(layers(&f), [tex::GRID_FIRST; 2]);
    f.disconnect(at(0), at(4));
    f.update(&mut World::new(1, 2), 3, &mut Vec::new());
    assert_eq!(layers(&f), [tex::GRID_FIRST, tex::GRID_FIRST + 1], "cutting the wire splits them again");
}

#[test]
fn more_grids_than_colours_repeat_the_accents_but_never_the_main_colour() {
    let grids: Vec<u32> = (0..8).collect();
    assert_eq!(assign(&grids), [0, 1, 2, 3, 4, 5, 1, 2]);
    assert_eq!(assign(&[]), Vec::<u8>::new());
}

#[test]
fn every_pole_draws_two_accent_boxes_in_its_grid_colour() {
    let f = poles(&[0, 40]);
    let mut out = Vec::new();
    f.write_accents(&mut out, Vec3::ZERO, 100.0);
    assert_eq!(out.len(), 4 * crate::factory::render::INSTANCE_FLOATS, "two poles, two boxes each");
    let layer = |box_: usize| out[box_ * crate::factory::render::INSTANCE_FLOATS + 8] as u16;
    assert_eq!(
        [layer(0), layer(1), layer(2), layer(3)],
        [tex::GRID_FIRST, tex::GRID_FIRST, tex::GRID_FIRST + 1, tex::GRID_FIRST + 1]
    );
}
