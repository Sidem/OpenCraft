use super::*;
use crate::bytes::{ByteReader, ByteWriter};

fn by_name(name: &str) -> u8 {
    TECHS.iter().position(|t| t.name == name).unwrap() as u8
}

fn finish(r: &mut Research, tech: u8) {
    (0..TECHS[tech as usize].units).for_each(|_| r.add_unit(tech));
}

/// The current tech, then the queue.
fn order(r: &Research) -> Vec<u8> {
    r.current.into_iter().chain(r.queue().iter().copied()).collect()
}

#[test]
fn queueing_a_deep_tech_adds_its_missing_prerequisites_in_order() {
    let mut r = Research::default();
    let deep = by_name("Construction Drones");
    r.enqueue(deep);
    let chain = order(&r);
    assert!(chain.len() > 3, "the drone sits at the end of a long ladder");
    assert_eq!(chain.last(), Some(&deep));
    assert!(r.current.is_some(), "with nothing chosen, the first one starts at once");
    for (i, &t) in chain.iter().enumerate() {
        assert!(TECHS[t as usize].needs.iter().all(|n| chain[..i].contains(n) || r.state(*n) == TechState::Done));
    }
    // Labs walk the chain by themselves.
    while let Some(c) = r.current {
        finish(&mut r, c);
    }
    assert_eq!((r.state(deep), r.queue().len()), (TechState::Done, 0));
}

#[test]
fn queueing_ignores_what_is_done_chosen_or_unknown() {
    let mut r = Research::default();
    r.enqueue(0);
    r.enqueue(0);
    r.enqueue(200);
    assert_eq!(order(&r), vec![0]);
    finish(&mut r, 0);
    r.enqueue(0);
    assert_eq!(order(&r), Vec::<u8>::new(), "a done tech isn't queued");
}

#[test]
fn choosing_a_tech_now_puts_the_old_one_at_the_front() {
    let mut r = Research::default();
    finish(&mut r, 0);
    let (a, b, c) = (1, 2, by_name("Construction Drones"));
    r.set_current(Some(a));
    r.enqueue(b);
    r.enqueue(c);
    let before = order(&r);
    r.set_current(Some(b));
    assert_eq!((r.current, r.queue()[0]), (Some(b), a));
    assert_eq!(order(&r).len(), before.len(), "nothing was lost, and b is not queued twice");
    r.set_current(Some(c));
    assert_eq!(r.current, Some(b), "a locked tech can't be chosen");
}

#[test]
fn giving_up_the_current_tech_moves_on_and_drops_what_needed_it() {
    let mut r = Research::default();
    r.enqueue(by_name("Construction Drones"));
    let chain = order(&r);
    r.set_current(None);
    assert!(r.current.is_some() && !r.is_chosen(chain[0]), "the next one takes over");
    // The given-up tech is neither done nor ahead of anything, so nothing that needs it can stay.
    let left = order(&r);
    assert!(left.len() < chain.len());
    for (i, &t) in left.iter().enumerate() {
        assert!(TECHS[t as usize].needs.iter().all(|n| left[..i].contains(n) || r.state(*n) == TechState::Done));
    }
    let mut r = Research::default();
    r.set_current(None);
    assert_eq!(r.current, None, "nothing to give up is fine");
}

#[test]
fn unqueueing_takes_the_techs_that_need_it_too() {
    let mut r = Research::default();
    finish(&mut r, 0);
    r.set_current(Some(1));
    let deep = by_name("Construction Drones");
    r.enqueue(deep);
    let chain = order(&r);
    let mid = chain[chain.len() / 2];
    r.dequeue(mid);
    assert!(!r.is_chosen(mid) && !r.is_chosen(deep));
    assert_eq!(r.current, Some(chain[0]));
}

#[test]
fn nothing_is_queued_behind_an_endless_tech() {
    let mut r = Research::default();
    let bonus = (0..TECHS.len() as u8).find(|&t| is_bonus(t)).unwrap();
    (0..bonus).for_each(|t| finish(&mut r, t));
    r.enqueue(bonus);
    assert_eq!(r.current, Some(bonus));
    r.enqueue(bonus + 1);
    assert!(r.queue().is_empty(), "an endless tech never ends, so nothing could follow it");
    r.set_current(None);
    assert_eq!(r.current, None);
}

#[test]
fn the_queue_is_saved_and_the_labs_move_on_by_themselves() {
    let mut r = Research::default();
    r.enqueue(by_name("Construction Drones"));
    let mut w = ByteWriter::default();
    r.write_state(&mut w);
    let back = Research::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back, r);
    assert!(!back.queue().is_empty());
}

#[test]
fn the_queue_never_passes_its_length() {
    let mut r = Research::default();
    (0..TECHS.len() as u8).for_each(|t| r.enqueue(t));
    assert!(r.queue().len() <= MAX_QUEUE);
}
