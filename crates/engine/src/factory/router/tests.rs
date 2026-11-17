use super::super::tests::{round_trip, run, stocked_box};
use super::*;
use crate::block::{COAL_ORE, COPPER_ORE, IRON_ORE};
use crate::world::World;

const NORTH: u8 = 0;
const EAST: u8 = 1;
const SOUTH: u8 = 2;
const MERGER_AT: IVec3 = IVec3::new(0, 0, 0);
const OUT_BOX: IVec3 = IVec3::new(4, 0, 0);

/// A merger at the origin facing east, fed from a box of `n` of each ore down a belt behind it (iron), on its left
/// (coal, from the north) and on its right (copper, from the south), and putting out east into a box (or, with
/// `blocked`, onto a belt that leads nowhere).
fn merger_line(n: [u32; 3], blocked: bool) -> Factory {
    let mut f = Factory::default();
    let feeds = [
        (IRON_ORE, IVec3::new(-3, 0, 0), EAST),
        (COAL_ORE, IVec3::new(0, 0, -3), SOUTH),
        (COPPER_ORE, IVec3::new(0, 0, 3), NORTH),
    ];
    for ((ore, source, dir), n) in feeds.into_iter().zip(n) {
        stocked_box(&mut f, source, ore.into(), n);
        for k in 1..=2 {
            f.add_belt(source + super::super::DIRS[dir as usize] * k, dir);
        }
    }
    f.place(&mut World::new(1, 2), MERGER, MERGER_AT, EAST, MERGER_AT, 0);
    for x in 1..=3 {
        f.add_belt(IVec3::new(x, 0, 0), EAST);
    }
    if !blocked {
        f.add_storage(OUT_BOX);
    }
    f
}

fn arrived(f: &Factory, ore: BlockId) -> u32 {
    f.storage_count_at(OUT_BOX, ore.into())
}

/// Every item anywhere in the factory: boxes, belts and routers.
fn total(f: &Factory) -> u32 {
    let stacks = f.storages.iter().flat_map(|s| s.contents()).chain(f.belts.iter().flat_map(|b| b.contents()));
    stacks.chain(f.routers.iter().flat_map(|r| r.contents())).map(|s| s.count).sum()
}

#[test]
fn a_merger_shares_its_output_evenly_between_three_saturated_belts() {
    let mut f = merger_line([400; 3], false);
    run(&mut f, 120.0, |_| {});
    let got = [arrived(&f, IRON_ORE), arrived(&f, COAL_ORE), arrived(&f, COPPER_ORE)];
    let (lo, hi) = (*got.iter().min().unwrap(), *got.iter().max().unwrap());
    assert!(lo > 60, "the line ran: {got:?}");
    assert!(hi - lo <= 1, "each input gets a third, within one item: {got:?}");
    // The output belt runs full: a merger costs no throughput.
    let most = super::super::belt::most_a_minute(0) * 2;
    assert!(got.iter().sum::<u32>() + 10 >= most * 95 / 100, "{got:?} of at most {most}");
}

#[test]
fn an_idle_input_hands_its_share_to_the_others() {
    let mut three = merger_line([400; 3], false);
    let mut two = merger_line([400, 0, 400], false);
    run(&mut three, 60.0, |_| {});
    run(&mut two, 60.0, |_| {});
    let (iron, copper) = (arrived(&two, IRON_ORE), arrived(&two, COPPER_ORE));
    assert_eq!(arrived(&two, COAL_ORE), 0);
    assert!(iron.abs_diff(copper) <= 1, "{iron} and {copper} share evenly");
    let all = |f: &Factory| [IRON_ORE, COAL_ORE, COPPER_ORE].map(|o| arrived(f, o)).iter().sum::<u32>();
    assert!(all(&two).abs_diff(all(&three)) <= 2, "two inputs fill the output as three do");
}

#[test]
fn a_blocked_merger_loses_nothing_and_holds_one_item() {
    let mut f = merger_line([20; 3], true);
    run(&mut f, 60.0, |f| assert_eq!(total(f), 60, "nothing lost or made"));
    let r = &f.routers[0];
    assert_eq!(r.contents().len(), 1, "it holds one item while its output is full");
    assert!(f.belts.iter().all(|b| b.items.len() <= 3), "items wait in line on every belt");
}

#[test]
fn a_merger_round_trips_mid_run_and_runs_on_the_same() {
    let mut f = merger_line([200, 0, 200], false);
    run(&mut f, 9.5, |_| {});
    assert!(f.belts.iter().filter(|b| !b.items.is_empty()).count() > 3, "mid-run, items on the way");
    let mut g = round_trip(&f);
    run(&mut f, 20.0, |_| {});
    run(&mut g, 20.0, |_| {});
    let bytes = |f: &Factory| {
        let mut w = crate::bytes::ByteWriter::default();
        f.write_state(&mut w);
        w.bytes
    };
    assert!(bytes(&f) == bytes(&g), "the loaded copy runs on exactly as the original");
}

#[test]
fn a_merger_takes_only_from_behind_and_the_sides() {
    let mut f = merger_line([0; 3], false);
    // A belt heading into its front from the east: not an input.
    f.add_belt(IVec3::new(1, 0, 1), NORTH);
    f.relink();
    let r = &f.routers[0];
    assert!(r.ins.iter().all(|i| i.is_some()), "behind, left and right are inputs: {:?}", r.ins);
    assert_eq!(r.outs[1..], [None, None], "a merger has only its front output");
    assert_eq!(r.in_slot(EAST), Some(0));
    assert_eq!(r.in_slot(SOUTH), Some(1), "from the north: its left");
    assert_eq!(r.in_slot(NORTH), Some(2), "from the south: its right");
    assert_eq!(r.in_slot(3), None, "heading west: from in front");
    assert!(!r.can_accept(), "belts never push into a merger");
}

#[test]
fn router_kinds_keep_their_saved_byte() {
    // The kind byte was the filter flag: splitters and filters read back as they were written before the merger.
    for (kind, byte) in [(RouterKind::Splitter, 0u8), (RouterKind::Filter, 1), (RouterKind::Merger, 2)] {
        assert_eq!(kind as u8, byte);
        assert_eq!(RouterKind::of(kind.block()), kind);
        assert_eq!(RouterKind::from_u8(byte), Some(kind));
    }
    assert_eq!(RouterKind::from_u8(3), None);
}

/// Belts run straight into a merger's openings and out of its exit with no end bar there (`Belt::ends_in_opening`,
/// `fed_by_port`); the belt into the out box and the boxes' first belts keep theirs.
#[test]
fn belts_at_a_routers_openings_have_no_end_bars() {
    let mut f = merger_line([1; 3], false);
    f.relink();
    let at = |x, z| {
        &f.belts[match f.at.get(&IVec3::new(x, 0, z)) {
            Some(&Slot::Belt(j)) => j as usize,
            _ => panic!("no belt"),
        }]
    };
    for (x, z) in [(-1, 0), (0, -1), (0, 1)] {
        assert!(at(x, z).ends_in_opening(), "the belt at {x},{z} runs into the merger");
    }
    assert!(at(1, 0).fed_by_port && !at(2, 0).fed_by_port, "only the first belt leads out of the exit");
    assert!(!at(3, 0).ends_in_opening() && !at(-2, 0).fed_by_port, "boxes are not openings");
}
