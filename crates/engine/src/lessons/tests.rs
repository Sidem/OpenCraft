use super::*;
use crate::action::Action;
use crate::block::BELT;
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::{IVec3, Rng};
use crate::sim::{PlayerId, Sim};
use crate::Game;

#[test]
fn bits_are_distinct_and_a_bad_number_does_nothing() {
    let bits: Vec<u8> = (0..CAUSES as u8).collect();
    for cause in [Cause::Power, Cause::Input, Cause::Output, Cause::Fuel, Cause::Water, Cause::Deposit, Cause::Blocked]
    {
        assert!(bits.contains(&stall_bit(cause)));
        assert_ne!(mask(stall_bit(cause)) & STALL_MASK, 0);
    }
    assert_eq!(mask(32), 0);
    assert_eq!(mask(255), 0);
    let mut sim = Sim::new(7, 2);
    sim.apply(PlayerId(0), Action::LessonTaught { bit: 200 });
    assert_eq!(sim.taught, 0);
    sim.apply(PlayerId(0), Action::LessonTaught { bit: 2 });
    assert_eq!(sim.taught, 4);
}

#[test]
fn a_save_from_before_the_lessons_counts_every_stall_lesson_as_taught() {
    let mut w = ByteWriter::default();
    let sim = Sim::new(7, 2);
    sim.write_state(&mut w);
    let before = crate::offhand::before_offhand(&sim, &w.bytes);
    // Save 57 ended before the lessons taught (4 bytes) and the signs' count (4 more, save 63).
    let mut old = ByteReader::new(&before[..before.len() - 8]);
    old.version = 57;
    let mut back = Sim::new(7, 2);
    back.read_state(&mut old).expect("a save 57 loads");
    assert_eq!(back.taught, STALL_MASK);
    assert_eq!(old.u8(), None, "every byte read");

    let mut fresh = ByteReader::new(&w.bytes);
    let mut again = Sim::new(7, 2);
    again.read_state(&mut fresh).expect("reads back");
    assert_eq!(again.taught, 0, "a new world has taught nothing");
}

/// What a tip or lesson can never lose: the opening's progress, the main tips' and the lessons taught.
fn latches(g: &Game) -> (u8, u8, u32) {
    (g.sim.opening.map_or(2, |o| o.progress as u8), g.sim.hints_reached, g.sim.taught)
}

#[test]
fn nothing_taught_is_ever_forgotten_whatever_the_player_does_to_the_scene() {
    let mut g = Game::new_normal(2024, 2);
    let o = g.sim.opening.unwrap();
    let mut rng = Rng::new(99);
    let (mut progress, mut reached, mut taught) = latches(&g);
    for step in 0..400 {
        match rng.below(6) {
            0 | 1 => {
                // Break something in or around the scene: belts, boxes, the furnace.
                let at = o.gap + IVec3::new(rng.below(11) as i32 - 5, 0, rng.below(11) as i32 - 5);
                g.act(Action::BreakBlock { pos: at });
            }
            2 => g.give(BELT as u16, 1 + rng.below(3)),
            3 => g.give(crate::item::IRON_INGOT.0, 1),
            4 => g.act(Action::LessonTaught { bit: rng.below(8) as u8 }),
            _ => g.sim.players[0].as_mut().unwrap().inventory = Default::default(),
        }
        g.skip_time(1.1);
        if step % 100 == 99 {
            g = Game::load(&g.save(), 2).expect("a save loads");
        }
        let (p, r, t) = latches(&g);
        assert!(
            p >= progress && r >= reached && t & taught == taught,
            "step {step}: {:?} -> {:?}",
            (progress, reached, taught),
            (p, r, t)
        );
        (progress, reached, taught) = (p, r, t);
    }
}
