use super::*;
use crate::action::Action;
use crate::Game;

fn ingots(g: &Game, o: &Opening) -> (u32, u32) {
    let slots = g.sim.factory.box_slots(o.out).unwrap();
    let n = |item| slots.iter().filter(|s| s.item == item).map(|s| s.count).sum();
    (n(IRON_INGOT), n(COPPER_INGOT))
}

fn step(g: &Game) -> Step {
    g.sim.opening.unwrap().step(g.inventory(), &g.sim.factory)
}

#[test]
fn putting_the_loose_belt_back_runs_the_line_and_pays_for_a_miner() {
    let mut g = Game::new_normal(2024, 2);
    let o = g.sim.opening.expect("a new normal world has an opening");
    assert_eq!(step(&g), Step::Pick);
    assert_eq!(g.sim.world.block_anywhere_or_generate(o.gap), AIR);
    g.skip_time(20.0);
    assert_eq!(ingots(&g, &o), (0, 0), "nothing crosses the gap");

    g.act(Action::BreakBlock { pos: o.loose });
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Collect, "the loose belt is a drop now");
    g.give(BELT as u16, 1);
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Place);
    let slot = g.inventory().slots.iter().position(|s| s.item == BELT.into()).unwrap() as u8;
    // Placed facing away from the player, as a click from the spawn does: it keeps that facing.
    let away = (o.run() + 3) % 4;
    g.act(Action::PlaceBlock { pos: o.gap, slot, facing: away, against: o.gap - IVec3::new(0, 1, 0) });
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Rotate);
    assert_eq!(o.marker(Step::Rotate), Some((o.gap, PROMPT_KEY_R)));
    g.skip_time(10.0);
    assert_eq!(ingots(&g, &o), (0, 0), "a belt facing the wrong way carries nothing to the furnace");
    g.act(Action::Rotate { pos: o.gap });
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Take, "one turn clockwise is the line's way");

    g.skip_time(60.0);
    assert_eq!(ingots(&g, &o), (14, 4), "the whole load is smelted");
    assert!(g.sim.factory.box_slots(o.source).unwrap().iter().all(|s| s.count == 0));
    assert_eq!(step(&g), Step::Take);
    g.give(IRON_INGOT.0, 1);
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Done, "holding ingots");
}

#[test]
fn the_scene_is_saved_and_the_same_for_a_seed() {
    let (a, b) = (Game::new_normal(7, 2), Game::new_normal(7, 2));
    assert_eq!(a.sim.state_hash(), b.sim.state_hash());
    let loaded = Game::load(&a.save(), 2).unwrap();
    assert_eq!(loaded.sim.opening, a.sim.opening);
    assert_eq!(loaded.sim.state_hash(), a.sim.state_hash());
    assert!(Game::new(7, 2).sim.opening.is_none(), "only new normal worlds get one");
}

#[test]
fn the_player_starts_looking_at_the_gap_within_reach() {
    for seed in [1, 7, 1337, 2024, 99_999] {
        let g = Game::new_normal(seed, 2);
        let o = g.sim.opening.unwrap();
        let eye = g.body().eye();
        let to_gap = o.gap.as_vec3() + crate::math::Vec3::new(0.5, 0.5, 0.5) - eye;
        let to_loose = o.loose.as_vec3() + crate::math::Vec3::new(0.5, 0.5, 0.5) - eye;
        let look = g.body().look_dir();
        assert!(
            to_gap.x * look.x + to_gap.y * look.y + to_gap.z * look.z > 0.85 * to_gap.length(),
            "seed {seed}: the gap is in front"
        );
        assert!(to_loose.length() < crate::interaction::REACH + 1.0, "seed {seed}: the loose belt is close");
        assert!(g.sim.events.is_empty(), "seed {seed}: building it makes no sounds");
    }
}
