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
    assert_eq!(o.marker(Step::Rotate, &g.sim.factory), Some((o.gap, PROMPT_KEY_R)));
    g.skip_time(10.0);
    assert_eq!(ingots(&g, &o), (0, 0), "a belt facing the wrong way carries nothing to the furnace");
    g.act(Action::Rotate { pos: o.gap });
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Take, "one turn clockwise is the line's way");

    g.skip_time(SMELT_SECONDS);
    assert_eq!(ingots(&g, &o), (2 * coal_miner_iron(), 0), "the whole load is smelted: no copper");
    assert!(g.sim.factory.box_slots(o.source).unwrap().iter().all(|s| s.count == 0));
    assert_eq!(step(&g), Step::Take);
    g.give(IRON_INGOT.0, 1);
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Craft, "holding ingots: on to the coal miners");
}

/// Long enough for the furnace to smelt the whole load.
const SMELT_SECONDS: f64 = 120.0;

/// Iron ingots one coal miner takes, from its recipe and its parts' recipes.
fn coal_miner_iron() -> u32 {
    use crate::recipes::RECIPES;
    let recipe = |item| RECIPES.iter().find(|r| r.output == item).unwrap();
    let ingots_of = |item| recipe(item).inputs[0].1 / recipe(item).count;
    let parts = recipe(block::COAL_MINER.into()).inputs.iter().filter(|i| i.0 != block::STONE.into());
    parts.map(|&(item, n)| n * ingots_of(item)).sum()
}

/// Sets the held coal miner on top of the ore block at `ore`, as a right-click on it does.
fn place_on(g: &mut Game, ore: IVec3) {
    let slot = g.inventory().slots.iter().position(|s| s.item == block::COAL_MINER.into()).unwrap() as u8;
    g.act(Action::PlaceBlock { pos: ore + IVec3::new(0, 1, 0), slot, facing: 0, against: ore });
    g.run_ticks(1);
}

/// Gives the coal miner the step's prompt points at a coal.
fn fuel(g: &mut Game) {
    let o = g.sim.opening.unwrap();
    let (at, kind) = o.marker(step(g), &g.sim.factory).expect("a fuel step points at its miner");
    assert_eq!(kind, PROMPT_OPEN);
    g.act(Action::Insert { pos: at, item: COAL_ORE.into() });
    g.run_ticks(1);
}

#[test]
fn the_guide_walks_from_the_line_to_two_burning_coal_miners() {
    // Seed 2 starts on granite high ground, where digging down finds no plain stone; seed 7 has a tree over its coal.
    for seed in [2, 7, 1337, 2024] {
        play_the_guide(seed);
    }
}

/// Every seed from 1 to `SWEEP_SEEDS` plays the guide through; prints the walks it asks for (blocks, straight
/// line): spawn to the coal patch, on to the iron patch, and the longest. `cargo test --release -q
/// guide_on_many_seeds -- --ignored --nocapture`
#[test]
#[ignore]
fn guide_on_many_seeds() {
    let walks: Vec<(i32, i32)> = (1..=SWEEP_SEEDS).map(play_the_guide).collect();
    let spread = |mut v: Vec<i32>| {
        v.sort_unstable();
        format!("median {} max {}", v[v.len() / 2], v[v.len() - 1])
    };
    let (coal, iron) = (spread(walks.iter().map(|w| w.0).collect()), spread(walks.iter().map(|w| w.1).collect()));
    println!("{SWEEP_SEEDS} seeds: spawn to coal {coal}; coal to iron {iron}");
}

const SWEEP_SEEDS: u32 = 200;

/// Plays the opening as the tips ask, through actions, and returns the walks (spawn to coal, coal to iron).
fn play_the_guide(seed: u32) -> (i32, i32) {
    let mut g = Game::new_normal(seed, 2);
    let o = g.sim.opening.unwrap();
    let (coal, iron) = (o.coal.expect("a coal patch"), o.iron.expect("an iron patch"));
    for (cell, ore) in [(coal, COAL_ORE), (iron, IRON_ORE)] {
        let mut at = |p| g.sim.world.block_anywhere_or_generate(p);
        assert_eq!(at(cell), ore, "seed {seed}: the marker is on the ore");
        assert_eq!(at(cell + IVec3::new(0, 1, 0)), AIR, "seed {seed}: in the open, not under a tree");
    }
    mend(&mut g, &o);
    g.skip_time(SMELT_SECONDS);
    assert_eq!(step(&g), Step::Take);
    g.act(Action::TakeContents { pos: o.out });
    g.run_ticks(1);
    assert_eq!(g.sim.opening.unwrap().progress, Progress::Mining);
    assert_eq!(step(&g), Step::Craft);
    assert_eq!(g.inventory().count(block::STONE.into()), 16, "seed {seed}: stone came with the ingots");

    let recipe = crate::recipes::RECIPES.iter().position(|r| r.output == block::COAL_MINER.into()).unwrap();
    g.act(Action::Craft { recipe: recipe as u16, times: 2 });
    g.skip_time(60.0);
    assert_eq!(g.inventory().count(block::COAL_MINER.into()), 2, "seed {seed}: the scene paid for two");
    assert_eq!(g.inventory().count(block::MINER.into()), 0);

    assert_eq!(step(&g), Step::PlaceCoal);
    assert_eq!(g.sim.opening.unwrap().marker(Step::PlaceCoal, &g.sim.factory), Some((coal, PROMPT_COAL)));
    place_on(&mut g, coal);
    assert_eq!(step(&g), Step::FuelCoal, "seed {seed}");
    fuel(&mut g);
    assert_eq!(step(&g), Step::PlaceIron, "seed {seed}");
    assert_eq!(g.sim.opening.unwrap().marker(Step::PlaceIron, &g.sim.factory), Some((iron, PROMPT_IRON)));
    place_on(&mut g, iron);
    assert_eq!(step(&g), Step::FuelIron, "seed {seed}");
    // All the coal went into the first miner (a shift-click does that): take some back from it, as the tip says.
    assert_eq!(g.inventory().count(COAL_ORE.into()), 0);
    g.skip_time(30.0);
    g.act(Action::TakeContents { pos: coal + IVec3::new(0, 1, 0) });
    g.run_ticks(1);
    assert!(g.inventory().count(COAL_ORE.into()) > 0, "seed {seed}: the miner on coal had coal to take");
    assert_eq!(step(&g), Step::FuelIron, "seed {seed}: taking from it leaves it lit");
    fuel(&mut g);
    assert_eq!(step(&g), Step::Done, "seed {seed}");
    assert_eq!(g.sim.opening.unwrap().progress, Progress::Over);

    g.skip_time(60.0);
    let dug = |ore: BlockId| g.sim.factory.coal_miners().any(|m| m.ore == ore.into() && m.lit);
    assert!(dug(COAL_ORE) && dug(IRON_ORE), "seed {seed}: both still burn or have dug");
    let panel = g.sim.factory.panel(iron + IVec3::new(0, 1, 0)).expect("the coal miner on iron");
    let out = panel.slots.iter().find(|s| s.0 == crate::factory::ROLE_OUTPUT).unwrap().1;
    assert!(out.item == IRON_ORE.into() && out.count > 0, "seed {seed}: the miner on iron dug iron: {out:?}");
    let flat = |a: IVec3, b: IVec3| (((a.x - b.x).pow(2) + (a.z - b.z).pow(2)) as f64).sqrt() as i32;
    (flat(IVec3::new(0, 0, 0), coal), flat(coal, iron))
}

#[test]
fn the_guide_takes_the_miners_in_any_order() {
    let mut g = Game::new_normal(2024, 2);
    let o = g.sim.opening.unwrap();
    mend(&mut g, &o);
    g.give(IRON_INGOT.0, 1);
    g.run_ticks(2);
    assert_eq!(step(&g), Step::Craft);
    g.give(block::COAL_MINER as u16, 2);
    g.run_ticks(1);
    place_on(&mut g, o.iron.unwrap());
    assert_eq!(step(&g), Step::PlaceCoal, "iron first: still asked for coal, the one on iron counts");
    place_on(&mut g, o.coal.unwrap());
    assert_eq!(step(&g), Step::FuelCoal, "the one on coal is lit first");
    g.give(COAL_ORE as u16, 1);
    g.run_ticks(1);
    fuel(&mut g);
    assert_eq!(step(&g), Step::FuelIron);
    g.act(Action::BreakBlock { pos: o.iron.unwrap() + IVec3::new(0, 1, 0) });
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Craft, "the one on iron was taken back and lies on the ground");
    g.give(block::COAL_MINER as u16, 1);
    g.run_ticks(1);
    assert_eq!(step(&g), Step::PlaceIron, "picked up: asked to set it again");
    assert_eq!(g.sim.opening.unwrap().progress, Progress::Mining, "not over before both burn");
}

#[test]
fn a_scene_from_before_the_guide_ends_at_the_ingots() {
    let mut g = Game::new_normal(2024, 2);
    let o = g.sim.opening.as_mut().unwrap();
    (o.coal, o.iron) = (None, None);
    let o = *o;
    mend(&mut g, &o);
    g.give(IRON_INGOT.0, 1);
    g.run_ticks(2);
    assert_eq!(g.sim.opening.unwrap().progress, Progress::Over);
    assert_eq!(step(&g), Step::Done);
}

#[test]
fn version_66_scenes_read_without_patches() {
    let g = Game::new_normal(2024, 2);
    let mut o = g.sim.opening.unwrap();
    o.progress = Progress::Mining;
    let mut w = ByteWriter::default();
    Opening::write_state(&Some(o), &mut w);
    let mut r = ByteReader::new(&w.bytes);
    assert_eq!(Opening::read_state(&mut r), Some(Some(o)));
    assert!(r.is_done());

    // Version 66 wrote the same bytes without the two patches at the end (and no Mining).
    (o.coal, o.iron, o.progress) = (None, None, Progress::Over);
    let mut w = ByteWriter::default();
    Opening::write_state(&Some(o), &mut w);
    let old = &w.bytes[..w.bytes.len() - 2];
    let mut r = ByteReader::new(old);
    r.version = 66;
    assert_eq!(Opening::read_state(&mut r), Some(Some(o)));
    assert!(r.is_done(), "all of it was read, and no more");
}

/// Mends the line the way the tutorial asks: pick the loose belt up, put it in the gap, turn it.
fn mend(g: &mut Game, o: &Opening) {
    g.act(Action::BreakBlock { pos: o.loose });
    g.run_ticks(1);
    g.give(BELT as u16, 1);
    g.run_ticks(1);
    let slot = g.inventory().slots.iter().position(|s| s.item == BELT.into()).unwrap() as u8;
    let away = (o.run() + 3) % 4;
    g.act(Action::PlaceBlock { pos: o.gap, slot, facing: away, against: o.gap - IVec3::new(0, 1, 0) });
    g.run_ticks(1);
    g.act(Action::Rotate { pos: o.gap });
    g.run_ticks(1);
}

#[test]
fn taking_the_belts_away_later_does_not_bring_the_belt_steps_back() {
    let mut g = Game::new_normal(2024, 2);
    let o = g.sim.opening.unwrap();
    mend(&mut g, &o);
    assert_eq!(g.sim.opening.unwrap().progress, Progress::Fixed, "the line was mended");

    // Reusing the belts before taking the ingots: the gap is empty again and the pack holds a belt.
    g.act(Action::BreakBlock { pos: o.gap });
    g.run_ticks(2);
    g.give(BELT as u16, 1);
    g.run_ticks(1);
    assert!(!matches!(step(&g), Step::Pick | Step::Collect | Step::Place | Step::Rotate), "{:?}", step(&g));

    g.give(IRON_INGOT.0, 1);
    g.run_ticks(1);
    assert_eq!(g.sim.opening.unwrap().progress, Progress::Mining, "someone holds ingots");
    g.act(Action::BreakBlock { pos: o.loose });
    g.run_ticks(1);
    assert_eq!(step(&g), Step::Craft, "on to the coal miners, never back to the belt");

    let loaded = Game::load(&g.save(), 2).unwrap();
    assert_eq!(loaded.sim.opening, g.sim.opening, "progress and patches are saved");
}

#[test]
fn a_miner_ends_the_opening_for_good() {
    let mut g = Game::new_normal(2024, 2);
    g.give(block::MINER as u16, 1);
    g.run_ticks(1);
    assert_eq!(g.sim.opening.unwrap().progress, Progress::Over);
    g.sim.players[0].as_mut().unwrap().inventory = Default::default(); // the miner is used up elsewhere
    g.run_ticks(2);
    assert_eq!(step(&g), Step::Done, "the miner is gone and the scene untouched, and the tips stay done");
}

#[test]
fn a_save_from_before_progress_was_kept_reads_it_from_the_scene() {
    let mut g = Game::new_normal(2024, 2);
    let mut o = g.sim.opening.unwrap();
    o.migrate(&g.sim.factory);
    assert_eq!(o.progress, Progress::Start, "an untouched scene still teaches");
    // Both belts taken away: the player is past the belt steps.
    g.act(Action::BreakBlock { pos: o.loose });
    g.run_ticks(1);
    o.migrate(&g.sim.factory);
    assert_eq!(o.progress, Progress::Over);
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
