use crate::action::Action;
use crate::block;
use crate::item::{self, ItemId};
use crate::lessons::{mask, LOW_POWER_BIT, NO_RECIPE_BIT, UPGRADE_BIT};
use crate::math::{IVec3, Vec3};
use crate::opening::PROMPT_OPEN;
use crate::raycast::RayHit;
use crate::recipes::MACHINE_RECIPES;
use crate::research::{TechState, TECHS};
use crate::tests::{find_outcrop_block, run_until_ready};
use crate::Game;

use super::NEED_BASE;

/// The scenes stand on a stone pad high in the air, out of the terrain's way.
const PAD_Y: i32 = 200;

fn ready() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.fill_blocks(-16, PAD_Y - 1, -16, 24, PAD_Y - 1, 16, block::STONE);
    g
}

fn at(x: i32, z: i32) -> IVec3 {
    IVec3::new(x, PAD_Y, z)
}

/// Puts the player at `at` and points their hands at `cell`, as the camera would.
fn look(g: &mut Game, from: IVec3, cell: IVec3) {
    g.body_mut().pos = Vec3::new(from.x as f64 + 0.5, from.y as f64, from.z as f64 + 0.5);
    g.render_eye = g.body().pos;
    let id = g.sim.world.block_anywhere_or_generate(cell);
    g.target = Some(RayHit { block: cell, normal: IVec3::new(0, 1, 0), id });
}

/// The player stands at `from` looking at `cell` for `secs` seconds.
fn stay(g: &mut Game, from: IVec3, cell: IVec3, secs: f64) {
    look(g, from, cell);
    g.skip_time(secs);
    look(g, from, cell);
}

/// Finishes the tech called `name` and every tech it needs.
fn finish(g: &mut Game, name: &str) {
    let t = TECHS.iter().position(|t| t.name == name).unwrap_or_else(|| panic!("no tech {name}"));
    for &need in TECHS[t].needs {
        finish(g, TECHS[need as usize].name);
    }
    while g.sim.factory.research.state(t as u8) != TechState::Done {
        g.sim.factory.research.add_unit(t as u8);
    }
}

/// A coal generator with fuel and a pole beside it, wired to `machines`.
fn power(g: &mut Game, pole: IVec3, machines: &[IVec3]) {
    let gen = pole + IVec3::new(-1, 0, -1);
    g.place_at(gen.x, gen.y, gen.z, block::GENERATOR, 0);
    g.stock_at(gen.x, gen.y, gen.z, block::COAL_ORE, 64);
    g.place_at(pole.x, pole.y, pole.z, block::POLE, 0);
    for &m in std::iter::once(&gen).chain(machines) {
        g.connect_at(pole.x, pole.y, pole.z, m.x, m.y, m.z);
    }
}

fn recipe_making(out: ItemId) -> u16 {
    MACHINE_RECIPES.iter().position(|r| r.outputs[0].0 == out).unwrap() as u16
}

fn stock(g: &mut Game, cell: IVec3, item: ItemId, n: u32) {
    g.stock_at(cell.x, cell.y, cell.z, item.0, n);
}

/// What the screen says now: the card on the left (as `web/src/ui/hints.ts` picks it, the hand-mining warning and
/// the blueprint prompt aside) and the readout of what the player looks at.
fn screen(g: &mut Game) -> String {
    let card = if g.opening_step() > 0 {
        format!("[Getting started] {}", g.opening_text(g.opening_step()))
    } else if g.lesson_now() > 0 {
        format!("[{}] {}", g.lesson_head(), g.lesson_text())
    } else if let Some(&s) = g.hint_side().first() {
        format!("[New to try] {}", g.hint_text(s))
    } else {
        let p = g.hint_progress();
        format!("[Tip {} of {}] {}", p + 1, g.hint_count(), g.hint_text(p))
    };
    let readout = format!("{}\n{}", g.target_name(), g.target_detail());
    format!("CARD: {card}\nLOOKING AT: {}", readout.trim_end())
}

/// Labs that ask more of one coal generator than it gives: they research, slowly.
fn power_short() -> Game {
    short_grid(ready())
}

fn short_grid(mut g: Game) -> Game {
    g.sim.factory.research.open_ring(crate::research::Ring::Green);
    let tech = crate::research::ids::MECHANICS;
    let labs: Vec<IVec3> = (0..8).map(|i| at(i * 2, 3)).collect();
    for &l in &labs {
        g.place_at(l.x, l.y, l.z, block::LAB, 0);
        stock(&mut g, l, item::RED_PACK, 50);
        stock(&mut g, l, item::GREEN_PACK, 50);
    }
    let poles: Vec<IVec3> = (0..4).map(|i| at(2 + i * 4, 1)).collect();
    power(&mut g, poles[0], &labs[0..2]);
    for (i, p) in poles.iter().enumerate().skip(1) {
        g.place_at(p.x, p.y, p.z, block::POLE, 0);
        let prev = poles[i - 1];
        g.connect_at(prev.x, prev.y, prev.z, p.x, p.y, p.z);
        for l in &labs[i * 2..i * 2 + 2] {
            g.connect_at(p.x, p.y, p.z, l.x, l.y, l.z);
        }
    }
    g.sim.factory.research.current = Some(tech);
    for t in [crate::research::ids::UNDERPASSES, crate::research::ids::MASONRY, crate::research::ids::FLUID_HANDLING] {
        g.act(Action::QueueResearch { tech: t });
    }
    stay(&mut g, at(14, 6), labs[7], 40.0);
    g
}

/// A blast furnace stopped because its slag has nowhere to go, Ore Crushing researched.
fn slag_blocked() -> Game {
    let mut g = ready();
    finish(&mut g, "Ore Crushing");
    let f = at(0, 0);
    g.place_at(f.x, f.y, f.z, block::BLAST_FURNACE, 0);
    for (item, n) in [(block::IRON_ORE.into(), 200), (block::COAL_ORE.into(), 100), (item::QUICKLIME, 100)] {
        stock(&mut g, f, item, n);
    }
    stay(&mut g, at(4, -4), f, 400.0);
    g
}

/// An assembler making concrete, powered, with every input but `missing`.
fn concrete_without(missing: ItemId) -> Game {
    let mut g = ready();
    finish(&mut g, "Assembly");
    let a = at(0, 0);
    g.place_at(a.x, a.y, a.z, block::ASSEMBLER, 0);
    g.act(Action::SetRecipe { pos: a, recipe: recipe_making(block::CONCRETE.into()) });
    g.run_ticks(2);
    for (item, n) in [(item::QUICKLIME, 20), (block::SAND.into(), 40), (block::STONE.into(), 40)] {
        if item != missing {
            stock(&mut g, a, item, n);
        }
    }
    power(&mut g, at(3, -2), &[a]);
    stay(&mut g, at(4, 4), a, 45.0);
    g
}

/// A powered lab with red packs and nothing chosen to research.
fn lab_idle() -> Game {
    let mut g = ready();
    let l = at(0, 0);
    g.place_at(l.x, l.y, l.z, block::LAB, 0);
    stock(&mut g, l, item::RED_PACK, 20);
    power(&mut g, at(2, -2), &[l]);
    stay(&mut g, at(3, 3), l, 45.0);
    g
}

/// The first powered line: coal miner, smelter, belts, a box, a generator and a working constructor.
fn first_line() -> Game {
    let mut g = ready();
    let c = at(0, 0);
    g.place_at(c.x, c.y, c.z, block::CONSTRUCTOR, 0);
    g.act(Action::SetRecipe { pos: c, recipe: recipe_making(item::IRON_PLATE) });
    g.run_ticks(2);
    stock(&mut g, c, item::IRON_INGOT, 50);
    for (b, x) in [(block::SMELTER, 6), (block::STORAGE, 8), (block::BELT, 10)] {
        g.place_at(x, PAD_Y, 4, b, 0);
    }
    power(&mut g, at(3, -2), &[c]);
    let (ore, _) = find_outcrop_block(&mut g, 4);
    let m = ore + IVec3::new(0, 1, 0);
    g.place_at(m.x, m.y, m.z, block::MINER, 0);
    stay(&mut g, at(3, 3), c, 10.0);
    g
}

/// Two coal miners burning on an outcrop near spawn: the opening's end.
fn coal_miners_burn() -> Game {
    let mut g = ready();
    let (ore, _) = find_outcrop_block(&mut g, 4);
    let m = ore + IVec3::new(0, 1, 0);
    g.place_at(m.x, m.y, m.z, block::COAL_MINER, 0);
    stock(&mut g, m, block::COAL_ORE.into(), 5);
    stay(&mut g, m + IVec3::new(3, 0, 3), m, 10.0);
    g
}

/// A coal miner whose deposit is dug out.
fn deposit_dry() -> Game {
    let mut g = ready();
    let (ore, key) = find_outcrop_block(&mut g, 4);
    let m = ore + IVec3::new(0, 1, 0);
    g.place_at(m.x, m.y, m.z, block::COAL_MINER, 0);
    stock(&mut g, m, block::COAL_ORE.into(), 5);
    g.sim.factory.deposits.exhaust(&key);
    stay(&mut g, m + IVec3::new(3, 0, 3), m, 45.0);
    g
}

const LOW_POWER: u32 = NEED_BASE + LOW_POWER_BIT as u32;

fn taught(g: &Game, bit: u8) -> bool {
    g.sim.taught & mask(bit) != 0
}

fn words(text: &str) -> usize {
    text.split_whitespace().count()
}

#[test]
fn labs_short_of_power_bring_the_power_screen_card_and_opening_it_teaches_it() {
    let mut g = power_short();
    assert_eq!(g.lesson_now(), LOW_POWER, "{}", screen(&mut g));
    assert_eq!(g.lesson_head(), "Good to know");
    let text = g.lesson_text();
    assert!(text.contains("[P]") && text.contains("75%"), "{text}");
    assert!(g.lesson_marker().is_empty(), "a card only: nothing to right-click");
    g.power_screen_opened();
    g.run_ticks(2);
    assert!(taught(&g, LOW_POWER_BIT));
    assert_eq!(g.lesson_now(), 0);
    g.skip_time(120.0);
    assert_eq!(g.lesson_now(), 0, "taught for good");
}

#[test]
fn h_on_the_card_teaches_it_too() {
    let mut g = power_short();
    assert_eq!(g.lesson_now(), LOW_POWER);
    g.lesson_dismiss();
    g.run_ticks(2);
    assert!(taught(&g, LOW_POWER_BIT));
}

#[test]
fn a_second_generator_mends_the_grid_and_that_teaches_it() {
    let mut g = power_short();
    assert_eq!(g.lesson_now(), LOW_POWER);
    let (pole, gen) = (at(2, 1), at(3, 0));
    g.place_at(gen.x, gen.y, gen.z, block::GENERATOR, 0);
    g.stock_at(gen.x, gen.y, gen.z, block::COAL_ORE, 64);
    g.connect_at(pole.x, pole.y, pole.z, gen.x, gen.y, gen.z);
    stay(&mut g, at(14, 6), at(14, 3), 20.0);
    assert!(taught(&g, LOW_POWER_BIT));
    assert_eq!(g.lesson_now(), 0);
}

#[test]
fn labs_gone_idle_take_the_card_away_without_teaching_it() {
    let mut g = power_short();
    assert_eq!(g.lesson_now(), LOW_POWER);
    for t in 0..TECHS.len() as u8 {
        g.act(Action::UnqueueResearch { tech: t });
    }
    g.run_ticks(2);
    g.sim.factory.research.current = None;
    // The reading keeps the last 10 s, then the idle grace runs out.
    stay(&mut g, at(14, 6), at(14, 3), 25.0);
    assert_eq!(g.lesson_now(), 0, "{}", screen(&mut g));
    assert!(!taught(&g, LOW_POWER_BIT));
}

#[test]
fn a_world_that_was_taught_never_shows_it() {
    let mut g = ready();
    g.act(Action::LessonTaught { bit: LOW_POWER_BIT });
    let mut g = short_grid(g);
    assert_eq!(g.lesson_now(), 0, "{}", screen(&mut g));
}

#[test]
fn a_machine_waiting_for_sand_names_the_quarry_and_the_crusher() {
    let mut g = concrete_without(block::SAND.into());
    let text = g.lesson_text();
    assert!(text.starts_with("Waiting for Sand: "), "{text}");
    assert!(text.contains("quarry") && text.contains("Crusher makes it from Slag"), "{text}");
    assert_eq!(text.matches("Crusher").count(), 1, "one line per machine: {text}");
    assert!(words(&text) <= 60, "{} words: {text}", words(&text));
    let readout = g.target_detail();
    assert!(readout.contains("Sand: Dig it on beaches"), "{readout}");
}

#[test]
fn a_machine_waiting_for_quicklime_names_limestone_and_the_scanner() {
    let g = concrete_without(item::QUICKLIME);
    let text = g.lesson_text();
    assert!(text.starts_with("Waiting for Quicklime: "), "{text}");
    assert!(text.contains("Limestone") && text.to_lowercase().contains("scanner"), "{text}");
    assert!(words(&text) <= 60, "{} words: {text}", words(&text));
}

#[test]
fn a_blast_furnace_full_of_slag_names_the_crusher_and_sand() {
    let g = slag_blocked();
    let text = g.lesson_text();
    assert!(text.starts_with("Full of Slag: "), "{text}");
    assert!(text.contains("Crusher makes Sand from it"), "{text}");
    assert!(words(&text) <= 50, "{} words: {text}", words(&text));
}

#[test]
fn the_slag_side_tip_shows_after_ore_crushing_and_ends_with_a_fed_crusher() {
    let mut g = ready();
    finish(&mut g, "Ore Crushing");
    g.run_ticks(2);
    let side = g.hint_side();
    let slag = side.iter().copied().find(|&s| g.hint_text(s).starts_with("Slag into sand"));
    let slag = slag.unwrap_or_else(|| panic!("no slag tip in {side:?}"));
    let c = at(0, 0);
    g.place_at(c.x, c.y, c.z, block::CRUSHER, 0);
    stock(&mut g, c, block::SLAG.into(), 5);
    g.run_ticks(2);
    assert!(!g.hint_side().contains(&slag), "done once a crusher has slag");
}

const NO_RECIPE: u32 = NEED_BASE + NO_RECIPE_BIT as u32;

/// An assembler with no recipe chosen, the player `secs` seconds near it.
fn bare_assembler(secs: f64) -> Game {
    let mut g = ready();
    finish(&mut g, "Assembly");
    let a = at(0, 0);
    g.place_at(a.x, a.y, a.z, block::ASSEMBLER, 0);
    stay(&mut g, at(4, 4), a, secs);
    g
}

#[test]
fn an_assembler_with_no_recipe_brings_a_card_and_an_open_prompt_and_choosing_one_teaches_it() {
    let mut g = bare_assembler(45.0);
    assert_eq!(g.lesson_now(), NO_RECIPE, "{}", screen(&mut g));
    assert_eq!(g.lesson_head(), "Good to know");
    let text = g.lesson_text();
    assert!(text.contains("Assembler") && text.contains("[Right-click]"), "{text}");
    assert!(words(&text) <= 25, "{} words: {text}", words(&text));
    let marker = g.lesson_marker();
    assert_eq!(marker.get(3), Some(&(PROMPT_OPEN as f32)), "the Open prompt floats over it: {marker:?}");
    let a = at(0, 0);
    g.act(Action::SetRecipe { pos: a, recipe: recipe_making(block::CONCRETE.into()) });
    stay(&mut g, at(4, 4), a, 3.0);
    assert!(taught(&g, NO_RECIPE_BIT));
    assert_eq!(g.lesson_now(), 0);
    assert!(g.lesson_marker().is_empty());
    g.skip_time(120.0);
    assert_ne!(g.lesson_now(), NO_RECIPE, "taught for good (a machine waiting for inputs is a stall lesson now)");
}

#[test]
fn h_on_the_no_recipe_card_teaches_it_and_it_never_comes_back() {
    let mut g = bare_assembler(45.0);
    assert_eq!(g.lesson_now(), NO_RECIPE);
    g.lesson_dismiss();
    g.run_ticks(2);
    assert!(taught(&g, NO_RECIPE_BIT));
    stay(&mut g, at(4, 4), at(0, 0), 120.0);
    assert_eq!(g.lesson_now(), 0, "{}", screen(&mut g));
}

#[test]
fn a_recipe_chosen_before_the_card_shows_teaches_it_without_ever_showing_it() {
    let mut g = bare_assembler(10.0);
    assert_eq!(g.lesson_now(), 0, "not yet: the player may be about to open it");
    let a = at(0, 0);
    g.act(Action::SetRecipe { pos: a, recipe: recipe_making(block::CONCRETE.into()) });
    stay(&mut g, at(4, 4), a, 3.0);
    assert!(taught(&g, NO_RECIPE_BIT), "they know how");
    assert_eq!(g.stall.quiet_until, 0, "no card was seen, so no quiet spell follows");
    stay(&mut g, at(4, 4), a, 60.0);
    assert_ne!(g.lesson_now(), NO_RECIPE);
}

#[test]
fn a_removed_or_distant_assembler_drops_the_card_without_teaching() {
    let mut g = bare_assembler(45.0);
    assert_eq!(g.lesson_now(), NO_RECIPE);
    let a = at(0, 0);
    g.act(Action::BreakBlock { pos: a });
    stay(&mut g, at(4, 4), a, 3.0);
    assert_eq!(g.lesson_now(), 0);
    assert!(!taught(&g, NO_RECIPE_BIT), "the machine just went away");

    let mut g = bare_assembler(0.0);
    stay(&mut g, at(-40, 40), at(0, 0), 60.0);
    assert_eq!(g.lesson_now(), 0, "far from the machine: nothing to teach");
}

#[test]
fn a_world_that_was_taught_the_no_recipe_lesson_never_shows_it() {
    let mut g = ready();
    g.act(Action::LessonTaught { bit: NO_RECIPE_BIT });
    finish(&mut g, "Assembly");
    let a = at(0, 0);
    g.place_at(a.x, a.y, a.z, block::ASSEMBLER, 0);
    stay(&mut g, at(4, 4), a, 120.0);
    assert_eq!(g.lesson_now(), 0, "{}", screen(&mut g));
}

#[test]
fn a_stall_lesson_goes_first_and_the_recipe_card_waits_for_it() {
    // A blast furnace stalled on its slag (a stall lesson on show), and beside it an assembler with no recipe.
    let mut g = slag_blocked();
    finish(&mut g, "Assembly");
    let a = at(8, 0);
    g.place_at(a.x, a.y, a.z, block::ASSEMBLER, 0);
    stay(&mut g, at(4, -4), a, 45.0);
    assert!(g.lesson_now() >= 1 && g.lesson_now() < NEED_BASE, "the stall lesson first: {}", screen(&mut g));
    g.lesson_dismiss();
    stay(&mut g, at(4, -4), a, 45.0);
    assert_eq!(g.lesson_now(), 0, "quiet after a lesson: {}", screen(&mut g));
    stay(&mut g, at(4, -4), a, 60.0);
    assert!(!taught(&g, NO_RECIPE_BIT));
    assert_eq!(g.lesson_now(), NO_RECIPE, "after the quiet spell: {}", screen(&mut g));
}

const UPGRADE: u32 = NEED_BASE + UPGRADE_BIT as u32;

/// Puts the player at `from` with the camera turned at the middle of `cell`: the ticks aim there for real.
fn aim(g: &mut Game, from: IVec3, cell: IVec3) {
    g.body_mut().pos = Vec3::new(from.x as f64 + 0.5, from.y as f64, from.z as f64 + 0.5);
    g.render_eye = g.body().pos;
    let to = Vec3::new(cell.x as f64 + 0.5, cell.y as f64 + 0.5, cell.z as f64 + 0.5) - g.body().eye();
    let body = g.body_mut();
    body.yaw = to.x.atan2(-to.z).rem_euclid(std::f64::consts::TAU);
    body.pitch = to.y.atan2(to.x.hypot(to.z));
}

/// A powered constructor making iron plates, `ingots` iron ingots in it, every tech done, `kits` green kits held and the
/// player `secs` seconds away from it with the camera on it. With enough ingots it runs flat out with inputs piled up.
fn plate_maker(ingots: u32, kits: u32, secs: f64) -> Game {
    let mut g = ready();
    for (t, tech) in TECHS.iter().enumerate() {
        (0..tech.units).for_each(|_| g.sim.factory.research.add_unit(t as u8));
    }
    let c = at(0, 0);
    g.place_at(c.x, c.y, c.z, block::CONSTRUCTOR, 0);
    g.act(Action::SetRecipe { pos: c, recipe: recipe_making(item::IRON_PLATE) });
    g.run_ticks(2);
    stock(&mut g, c, item::IRON_INGOT, ingots);
    power(&mut g, at(3, -2), &[c]);
    g.give(item::GREEN_KIT.0, kits);
    aim(&mut g, at(2, 2), c); // within reach
    g.skip_time(secs);
    g
}

#[test]
fn a_bottleneck_pointed_at_with_kits_in_the_pack_brings_the_upgrade_card_and_upgrading_teaches_it() {
    let mut g = plate_maker(60, 20, 8.0);
    assert_eq!(g.lesson_now(), UPGRADE, "{}", screen(&mut g));
    assert_eq!(g.lesson_head(), "Good to know");
    let text = g.lesson_text();
    assert!(text.contains("Constructor") && text.contains("Green Kit") && text.contains("[Right-click]"), "{text}");
    assert!(words(&text) <= 30, "{} words: {text}", words(&text));
    assert_eq!(g.lesson_marker().get(3), Some(&(PROMPT_OPEN as f32)), "the Open prompt floats over it");
    let c = at(0, 0);
    g.upgrade_machine(c.x, c.y, c.z);
    stay(&mut g, at(4, 4), c, 3.0);
    assert_eq!(g.sim.factory.tiered_at(c).map(|t| t.1), Some(1), "upgraded to Mk2");
    assert!(taught(&g, UPGRADE_BIT));
    assert_ne!(g.lesson_now(), UPGRADE);
}

#[test]
fn h_on_the_upgrade_card_teaches_it() {
    let mut g = plate_maker(60, 20, 8.0);
    assert_eq!(g.lesson_now(), UPGRADE);
    g.lesson_dismiss();
    g.run_ticks(2);
    assert!(taught(&g, UPGRADE_BIT));
    g.skip_time(120.0);
    assert_ne!(g.lesson_now(), UPGRADE, "taught for good (the machine ran dry: that is a stall lesson now)");
}

#[test]
fn the_upgrade_card_needs_the_kits_a_busy_full_pace_machine_and_a_look() {
    let g = plate_maker(60, 0, 20.0);
    assert_eq!(g.lesson_now(), 0, "no kits held");
    let mut g = plate_maker(3, 20, 20.0);
    assert_eq!(g.lesson_now(), 0, "{}", screen(&mut g));
    assert!(!taught(&g, UPGRADE_BIT), "inputs are not piled up: an upgrade would not help");
    let mut g = plate_maker(60, 20, 0.0);
    g.body_mut().yaw += std::f64::consts::PI;
    g.skip_time(20.0);
    assert_eq!(g.lesson_now(), 0, "not pointing at it");
    // One kit short of the step.
    let mut g = plate_maker(60, 3, 20.0);
    assert_eq!(g.lesson_now(), 0, "{}", screen(&mut g));
}

#[test]
fn a_world_that_was_taught_the_upgrade_lesson_never_shows_it() {
    let mut g = ready();
    g.act(Action::LessonTaught { bit: UPGRADE_BIT });
    g.run_ticks(2);
    let c = at(0, 0);
    for (t, tech) in TECHS.iter().enumerate() {
        (0..tech.units).for_each(|_| g.sim.factory.research.add_unit(t as u8));
    }
    g.place_at(c.x, c.y, c.z, block::CONSTRUCTOR, 0);
    g.act(Action::SetRecipe { pos: c, recipe: recipe_making(item::IRON_PLATE) });
    g.run_ticks(2);
    stock(&mut g, c, item::IRON_INGOT, 60);
    power(&mut g, at(3, -2), &[c]);
    g.give(item::GREEN_KIT.0, 20);
    stay(&mut g, at(4, 4), c, 30.0);
    assert_eq!(g.lesson_now(), 0, "{}", screen(&mut g));
}

/// The cold-player benchmark's scenes: what the screen says in each. `cargo test --release -q
/// benchmark_screens -- --ignored --nocapture`
#[test]
#[ignore]
fn benchmark_screens() {
    type Scene = (&'static str, fn() -> Game);
    let scenes: [Scene; 10] = [
        ("power_short", power_short),
        ("slag_blocked", slag_blocked),
        ("concrete_no_sand", || concrete_without(block::SAND.into())),
        ("concrete_no_quicklime", || concrete_without(item::QUICKLIME)),
        ("lab_idle", lab_idle),
        ("first_line", first_line),
        ("deposit_dry", deposit_dry),
        ("concrete_no_stone", || concrete_without(block::STONE.into())),
        ("coal_miners_burn", coal_miners_burn),
        ("first_line_again", first_line),
    ];
    for (name, build) in scenes {
        let mut g = build();
        println!("=== {name}\n{}\n", screen(&mut g));
    }
}

/// How far a new world's nearest sand lies from spawn, over many seeds: on dry ground, and anywhere (under water too).
/// `cargo test --release -q sand_near_spawn -- --ignored --nocapture`
#[test]
#[ignore]
fn sand_near_spawn() {
    const SEEDS: u32 = 200;
    const REACH: i32 = 480;
    const STEP: i32 = 4;
    let (mut dry, mut any) = (Vec::new(), Vec::new());
    for seed in 1..=SEEDS {
        let mut g = Game::new_normal(seed, 2);
        let (mut d, mut a) = (None::<i32>, None::<i32>);
        'rings: for r in (0..=REACH).step_by(STEP as usize) {
            for i in (-r..=r).step_by(STEP as usize) {
                for (x, z) in [(i, -r), (i, r), (-r, i), (r, i)] {
                    let h = g.sim.world.generator().height_at(x, z);
                    let top = IVec3::new(x, h, z);
                    if g.sim.world.block_anywhere_or_generate(top) != block::SAND {
                        continue;
                    }
                    a.get_or_insert(r);
                    if g.sim.world.block_anywhere_or_generate(top + IVec3::new(0, 1, 0)) == block::AIR {
                        d = Some(r);
                        break 'rings;
                    }
                }
            }
        }
        dry.push(d.unwrap_or(i32::MAX));
        any.push(a.unwrap_or(i32::MAX));
    }
    let spread = |v: &mut Vec<i32>| {
        v.sort_unstable();
        let pct = |p: usize| v[(v.len() * p / 100).min(v.len() - 1)];
        let within = |b: i32| v.iter().filter(|&&x| x <= b).count();
        format!(
            "median {} p75 {} p90 {} max {} - within 64: {} - 128: {} - 256: {} - none in {REACH}: {}",
            pct(50),
            pct(75),
            pct(90),
            v[v.len() - 1],
            within(64),
            within(128),
            within(256),
            v.iter().filter(|&&x| x == i32::MAX).count()
        )
    };
    println!("{SEEDS} seeds, square rings every {STEP} blocks (Chebyshev distance)");
    println!("dry sand: {}", spread(&mut dry));
    println!("any sand: {}", spread(&mut any));
}
