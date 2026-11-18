use super::*;
use crate::action::Action;
use crate::block::{AIR, LAMP, LED, LED_STEPS, STONE};
use crate::sim::PlayerId;
use crate::tests::run_until_ready;
use crate::Game;

const LED_DIM: BlockId = LED_STEPS[0];
const LED_MEDIUM: BlockId = LED_STEPS[1];

const P: PlayerId = PlayerId(0);
const AT: IVec3 = IVec3::new(40, 200, 40);
const BLUE: u32 = 0x3366cc;

/// A bare sim with an LED lamp at `AT`.
fn lit() -> Sim {
    let mut sim = Sim::new(7, 2);
    assert!(sim.world.set_block_anywhere(AT, LED));
    sim
}

fn block(sim: &mut Sim) -> BlockId {
    sim.block(AT)
}

#[test]
fn the_panel_sets_the_brightness_step_in_place_and_the_colour() {
    let mut sim = lit();
    assert_eq!((sim.leds.picked(AT), sim.leds.colour(AT)), (None, DEFAULT_COLOUR), "an untouched lamp is warm white");
    sim.apply(P, Action::SetLed { pos: AT, step: 2, rgb: BLUE });
    assert_eq!((block(&mut sim), sim.leds.picked(AT)), (LED, Some(BLUE)));
    sim.apply(P, Action::SetLed { pos: AT, step: 1, rgb: BLUE });
    assert_eq!(block(&mut sim), LED_MEDIUM, "down a step, same colour");
    sim.apply(P, Action::SetLed { pos: AT, step: 0, rgb: DEFAULT_COLOUR });
    assert_eq!(
        (block(&mut sim), sim.leds.is_empty()),
        (LED_DIM, true),
        "the dimmest step; the default colour keeps no record"
    );
}

#[test]
fn nothing_else_is_a_lamp_and_a_step_has_to_exist() {
    let mut sim = lit();
    let stone = AT + IVec3::new(2, 0, 0);
    sim.world.set_block_anywhere(stone, STONE);
    sim.apply(P, Action::SetLed { pos: stone, step: 1, rgb: BLUE });
    sim.apply(P, Action::SetLed { pos: AT + IVec3::new(0, 3, 0), step: 0, rgb: BLUE });
    sim.apply(P, Action::SetLed { pos: AT, step: 3, rgb: BLUE });
    assert_eq!((sim.world.block_anywhere_or_generate(stone), block(&mut sim)), (STONE, LED));
    assert!(sim.leds.is_empty(), "no record for a stone, for air or for a step that is not there");
    sim.world.set_block_anywhere(AT, LAMP);
    sim.apply(P, Action::SetLed { pos: AT, step: 0, rgb: BLUE });
    assert_eq!(block(&mut sim), LAMP, "a plain lamp has no panel");
}

#[test]
fn breaking_a_lamp_forgets_its_colour_at_any_step() {
    let mut sim = lit();
    sim.apply(P, Action::SetLed { pos: AT, step: 2, rgb: BLUE });
    sim.apply(P, Action::BreakBlock { pos: AT });
    assert_eq!(block(&mut sim), AIR);
    assert!(sim.leds.is_empty());
    // Built over, not broken: a block placed where it stood does not inherit the colour either.
    sim.world.set_block_anywhere(AT, LED);
    assert_eq!(sim.leds.colour(AT), DEFAULT_COLOUR);
}

#[test]
fn what_a_lamp_drops_is_the_plain_led_whatever_its_step() {
    for step in crate::block::LED_STEPS {
        assert_eq!(crate::block::def(step).drop, LED);
    }
    assert!(crate::block::def(LED).placeable && !crate::block::def(LED_DIM).placeable);
}

#[test]
fn colours_are_written_sorted_and_read_back_and_bad_bytes_are_refused() {
    let mut sim = Sim::new(7, 2);
    // Set out of order: the bytes must come out the same either way.
    let cells = [IVec3::new(9, 5, 1), IVec3::new(-3, 7, 2), IVec3::new(9, 4, 8)];
    let mut a = Leds::default();
    let mut b = Leds::default();
    for (i, &c) in cells.iter().enumerate() {
        a.set(c, 0x100 + i as u32);
        b.set(cells[2 - i], 0x100 + (2 - i) as u32);
    }
    let bytes = |l: &Leds| {
        let mut w = ByteWriter::default();
        l.write_state(&mut w);
        w.bytes
    };
    assert_eq!(bytes(&a), bytes(&b));
    let back = Leds::read_state(&mut ByteReader::new(&bytes(&a))).expect("reads");
    assert_eq!(back, a);
    // A default-coloured record, a colour past 24 bits and an unsorted list are damage.
    for bad in [[(cells[0], DEFAULT_COLOUR)], [(cells[0], 0x0100_0000)]] {
        let mut w = ByteWriter::default();
        w.count(1);
        w.ivec3(bad[0].0);
        w.u32(bad[0].1);
        assert!(Leds::read_state(&mut ByteReader::new(&w.bytes)).is_none());
    }
    let mut w = ByteWriter::default();
    w.count(2);
    for c in [cells[0], cells[1]] {
        w.ivec3(c);
        w.u32(5);
    }
    assert!(Leds::read_state(&mut ByteReader::new(&w.bytes)).is_none(), "unsorted");
    // And a colour moves the state hash.
    let before = sim.state_hash();
    sim.leds.set(cells[0], BLUE);
    assert_ne!(sim.state_hash(), before);
}

#[test]
fn a_lamps_step_and_colour_survive_a_save_and_reach_a_joining_client() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let pos = IVec3::new(3, g.sim.world.generator().height_at(3, 3) + 3, 3);
    g.sim.world.set_block_anywhere(pos, LED);
    g.set_led(pos.x, pos.y, pos.z, 1, BLUE);
    g.run_ticks(2);
    assert_eq!(g.led_state(pos.x, pos.y, pos.z), vec![1, BLUE]);
    let mut back = Game::load(&g.save(), 3).expect("loads");
    assert_eq!(back.sim.leds.picked(pos), Some(BLUE));
    assert_eq!(back.sim.block(pos), LED_MEDIUM);
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());
}

/// Steps frames and streaming until the remeshed chunks are in.
fn settle(g: &mut Game) {
    for _ in 0..4 {
        g.update(1.0 / 60.0);
        g.begin_work();
        while g.work_step() {}
        while g.next_event() != 0 {}
    }
}

#[test]
fn a_coloured_lamp_gets_a_tint_box_and_a_halo_in_its_colour_and_an_uncoloured_one_only_the_halo() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    settle(&mut g);
    let y = g.sim.world.generator().height_at(3, 3) + 3;
    let (plain, coloured) = (IVec3::new(3, y, 3), IVec3::new(6, y, 3));
    for p in [plain, coloured] {
        g.sim.world.set_block_anywhere(p, LED_MEDIUM);
    }
    settle(&mut g);
    let boxes = |g: &Game| g.instances.chunks_exact(crate::factory::INSTANCE_FLOATS).count();
    let led_halos = |g: &Game| {
        g.halos
            .chunks_exact(crate::halos::HALO_FLOATS)
            .filter(|h| h[3] == 2.4)
            .map(|h| [h[4], h[5], h[6]])
            .collect::<Vec<_>>()
    };
    let base = boxes(&g);
    assert_eq!(led_halos(&g).len(), 2, "a halo each, both warm white");
    g.sim.leds.set(coloured, BLUE);
    settle(&mut g);
    assert_eq!(boxes(&g), base + 1, "one tint box, for the lamp a player coloured");
    let blue = [0x33 as f32 / 255.0, 0x66 as f32 / 255.0, 0xcc as f32 / 255.0];
    assert!(led_halos(&g).contains(&blue), "a halo in the picked colour");
    assert_eq!(led_halos(&g).len(), 2);
}

#[test]
fn a_standing_scene_with_coloured_lamps_remeshes_nothing() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(3, 3) + 3;
    for i in 0..12 {
        let p = IVec3::new(3 + 2 * i, y, 3);
        g.sim.world.set_block_anywhere(p, LED_STEPS[i as usize % 3]);
        g.sim.leds.set(p, 0x100 * (i as u32 + 1));
    }
    settle(&mut g);
    let mut events = 0;
    for _ in 0..120 {
        g.update(1.0 / 60.0);
        g.begin_work();
        while g.work_step() {}
        while g.next_event() != 0 {
            events += 1;
        }
    }
    assert_eq!(events, 0, "frames after the lamps settled remeshed {events} chunks");
}

/// `cargo test --release -q bench_led -- --ignored --nocapture`: what the lamps cost a frame and a mesh.
#[test]
#[ignore]
fn bench_led_frame_and_light_cost() {
    use std::time::Instant;
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(3, 3) + 3;
    for i in 0..256 {
        let p = IVec3::new(-30 + 4 * (i % 16), y, -30 + 4 * (i / 16));
        g.sim.world.set_block_anywhere(p, LED_STEPS[i as usize % 3]);
        g.sim.leds.set(p, 0x100 * (i as u32 + 1));
    }
    settle(&mut g);
    let eye = g.render_eye;
    let time = |name: &str, f: &mut dyn FnMut()| {
        let t = Instant::now();
        for _ in 0..2000 {
            f();
        }
        println!("{name}: {:.1} us a frame", t.elapsed().as_secs_f64() * 1e6 / 2000.0);
    };
    g.write_halos(eye);
    assert!(g.halos.len() >= 256 * crate::halos::HALO_FLOATS, "all the lamps have halos: {}", g.halos.len() / 8);
    g.instances.clear();
    g.write_leds(eye);
    let boxes = g.instances.len() / crate::factory::INSTANCE_FLOATS;
    assert_eq!(boxes, 256, "a tint box each");
    time("write_halos, 256 LEDs", &mut || g.write_halos(eye));
    time("write_leds, 256 coloured LEDs", &mut || {
        g.instances.clear();
        g.write_leds(eye);
    });
    g.sim.leds = Leds::default();
    time("write_leds, none coloured", &mut || {
        g.instances.clear();
        g.write_leds(eye);
    });
}
