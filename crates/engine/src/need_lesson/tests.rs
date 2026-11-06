use crate::action::Action;
use crate::block;
use crate::item::{self, ItemId};
use crate::lessons::{mask, LOW_POWER_BIT};
use crate::math::{IVec3, Vec3};
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
    g.place_at(gen.x, gen.y, gen.z, block::GENERATOR as u16, 0);
    g.stock_at(gen.x, gen.y, gen.z, block::COAL_ORE as u16, 64);
    g.place_at(pole.x, pole.y, pole.z, block::POLE as u16, 0);
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
    let tech = TECHS.iter().position(|t| t.needs.is_empty() && t.packs == [item::RED_PACK]).unwrap() as u8;
    let labs: Vec<IVec3> = (0..8).map(|i| at(i * 2, 3)).collect();
    for &l in &labs {
        g.place_at(l.x, l.y, l.z, block::LAB as u16, 0);
        stock(&mut g, l, item::RED_PACK, 50);
    }
    let poles: Vec<IVec3> = (0..4).map(|i| at(2 + i * 4, 1)).collect();
    power(&mut g, poles[0], &labs[0..2]);
    for (i, p) in poles.iter().enumerate().skip(1) {
        g.place_at(p.x, p.y, p.z, block::POLE as u16, 0);
        let prev = poles[i - 1];
        g.connect_at(prev.x, prev.y, prev.z, p.x, p.y, p.z);
        for l in &labs[i * 2..i * 2 + 2] {
            g.connect_at(p.x, p.y, p.z, l.x, l.y, l.z);
        }
    }
    g.sim.factory.research.current = Some(tech);
    for t in 0..TECHS.len() as u8 {
        if TECHS[t as usize].packs == [item::RED_PACK] {
            g.act(Action::QueueResearch { tech: t });
        }
    }
    stay(&mut g, at(14, 6), labs[7], 40.0);
    g
}

/// A blast furnace stopped because its slag has nowhere to go, Ore Crushing researched.
fn slag_blocked() -> Game {
    let mut g = ready();
    finish(&mut g, "Ore Crushing");
    let f = at(0, 0);
    g.place_at(f.x, f.y, f.z, block::BLAST_FURNACE as u16, 0);
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
    g.place_at(a.x, a.y, a.z, block::ASSEMBLER as u16, 0);
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
    g.place_at(l.x, l.y, l.z, block::LAB as u16, 0);
    stock(&mut g, l, item::RED_PACK, 20);
    power(&mut g, at(2, -2), &[l]);
    stay(&mut g, at(3, 3), l, 45.0);
    g
}

/// The first powered line: coal miner, smelter, belts, a box, a generator and a working constructor.
fn first_line() -> Game {
    let mut g = ready();
    let c = at(0, 0);
    g.place_at(c.x, c.y, c.z, block::CONSTRUCTOR as u16, 0);
    g.act(Action::SetRecipe { pos: c, recipe: recipe_making(item::IRON_PLATE) });
    g.run_ticks(2);
    stock(&mut g, c, item::IRON_INGOT, 50);
    for (b, x) in [(block::SMELTER, 6), (block::STORAGE, 8), (block::BELT, 10)] {
        g.place_at(x, PAD_Y, 4, b as u16, 0);
    }
    power(&mut g, at(3, -2), &[c]);
    let (ore, _) = find_outcrop_block(&mut g, 4);
    let m = ore + IVec3::new(0, 1, 0);
    g.place_at(m.x, m.y, m.z, block::MINER as u16, 0);
    stay(&mut g, at(3, 3), c, 10.0);
    g
}

/// Two coal miners burning on an outcrop near spawn: the opening's end.
fn coal_miners_burn() -> Game {
    let mut g = ready();
    let (ore, _) = find_outcrop_block(&mut g, 4);
    let m = ore + IVec3::new(0, 1, 0);
    g.place_at(m.x, m.y, m.z, block::COAL_MINER as u16, 0);
    stock(&mut g, m, block::COAL_ORE.into(), 5);
    stay(&mut g, m + IVec3::new(3, 0, 3), m, 10.0);
    g
}

/// A coal miner whose deposit is dug out.
fn deposit_dry() -> Game {
    let mut g = ready();
    let (ore, key) = find_outcrop_block(&mut g, 4);
    let m = ore + IVec3::new(0, 1, 0);
    g.place_at(m.x, m.y, m.z, block::COAL_MINER as u16, 0);
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
    g.place_at(gen.x, gen.y, gen.z, block::GENERATOR as u16, 0);
    g.stock_at(gen.x, gen.y, gen.z, block::COAL_ORE as u16, 64);
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
    g.place_at(c.x, c.y, c.z, block::CRUSHER as u16, 0);
    stock(&mut g, c, block::SLAG.into(), 5);
    g.run_ticks(2);
    assert!(!g.hint_side().contains(&slag), "done once a crusher has slag");
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
