//! Sites: the cell order, marking by actions (ranges found from the world, limits, ids never reused),
//! the bytes, and the survey over loaded chunks. Hand-built ground far from spawn, as the quarry's
//! tests do.

use super::*;
use crate::action::Action;
use crate::block::{GRASS, IRON_ORE, STONE, WATER};
use crate::sim::{PlayerId, Sim};

const P: PlayerId = PlayerId(0);
const O: IVec3 = IVec3::new(-4000, 0, 4000);

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    O + IVec3::new(x, y, z)
}

fn col(x: i32, z: i32) -> Column {
    (O.x + x, O.z + z)
}

fn fill(sim: &mut Sim, lo: IVec3, hi: IVec3, b: BlockId) {
    for y in lo.y..=hi.y {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                sim.world.set_block_anywhere(IVec3::new(x, y, z), b);
            }
        }
    }
}

/// Stone to y 99 under grass at 100 and open sky, over x and z -4..=43, with in the 8 × 8 area 0..=7:
/// a hill at (1, 1) up to 104 capped by iron ore at 105; a tree at (5, 5) (logs 101..=103, leaves at
/// 104 over it and over (4, 5)); a pit at (6, 1) down to ground at 95; a pool at (2, 6) (water
/// 98..=100); a cave at (3, 3) (air 92..=93) under the grass.
fn ground(sim: &mut Sim) {
    fill(sim, at(-4, 90, -4), at(43, 99, 43), STONE);
    fill(sim, at(-4, 100, -4), at(43, 100, 43), GRASS);
    fill(sim, at(-4, 101, -4), at(43, 255, 43), AIR);
    fill(sim, at(1, 101, 1), at(1, 104, 1), STONE);
    sim.world.set_block_anywhere(at(1, 105, 1), IRON_ORE);
    fill(sim, at(5, 101, 5), at(5, 103, 5), LOG);
    fill(sim, at(4, 104, 5), at(5, 104, 5), LEAVES);
    fill(sim, at(6, 96, 1), at(6, 100, 1), AIR);
    fill(sim, at(2, 98, 6), at(2, 100, 6), WATER);
    fill(sim, at(3, 92, 3), at(3, 93, 3), AIR);
}

fn mark(sim: &mut Sim, a: Column, b: Column, level: i32, job: Job) {
    sim.apply(P, Action::MarkSite { a, b, level, job });
}

fn site(lo: Column, hi: Column, level: i32, job: Job, high: i32, low: i32) -> Site {
    Site { id: 0, lo, hi, level, job, high, low, done: 0 }
}

#[test]
fn cells_go_cut_layers_down_then_fill_layers_up_rows_back_and_forth() {
    let s = site((10, 20), (12, 21), 5, Job::Flatten, 7, 3);
    assert_eq!((s.cut_layers(), s.fill_layers(), s.cells()), (2, 2, 24));
    let cells: Vec<IVec3> = (0..s.cells()).map(|i| s.cell(i)).collect();
    let row = |y, z, xs: [i32; 3]| xs.map(|x| IVec3::new(x, y, z));
    let mut want = Vec::new();
    for y in [7, 6, 4, 5] {
        want.extend(row(y, 20, [10, 11, 12]));
        want.extend(row(y, 21, [12, 11, 10]));
    }
    assert_eq!(cells, want);
    assert!(!s.is_fill(11) && s.is_fill(12));

    let dig = Site { job: Job::Dig, ..s };
    assert_eq!((dig.cells(), dig.cell(11).y), (12, 6), "only the cut");
    let fill = Site { job: Job::Fill, ..s };
    assert_eq!((fill.cells(), fill.cell(0).y, fill.is_fill(0)), (12, 4, true), "only the fill");
    let nothing = Site { level: 9, job: Job::Dig, ..s };
    assert_eq!(nothing.cells(), 0, "nothing above the level");
}

#[test]
fn marking_finds_the_ranges_once_and_keeps_to_the_limits() {
    let mut sim = Sim::new(1337, 2);
    ground(&mut sim);
    mark(&mut sim, col(7, 7), col(0, 0), 100, Job::Flatten);
    let s = sim.factory.sites.list[0];
    assert_eq!((s.id, s.lo, s.hi), (0, col(0, 0), col(7, 7)), "corners in any order");
    assert_eq!((s.high, s.low), (105, 95), "the ore on the hill, the pit's floor (not the cave)");
    assert_eq!(s.cells(), (5 + 5) * 64);

    // Found once: later edits don't move them.
    sim.world.set_block_anywhere(at(0, 120, 0), STONE);
    assert_eq!(sim.factory.sites.list[0], s);

    let count = |sim: &Sim| sim.factory.sites.list.len();
    mark(&mut sim, col(7, 7), col(9, 9), 100, Job::Dig);
    assert_eq!(count(&sim), 1, "overlapping");
    mark(&mut sim, col(8, 0), col(8 + MAX_SITE, 0), 100, Job::Dig);
    assert_eq!(count(&sim), 1, "too long");
    mark(&mut sim, col(8, 0), col(8, 0), 0, Job::Fill);
    mark(&mut sim, col(8, 0), col(8, 0), WORLD_HEIGHT - 1, Job::Dig);
    assert_eq!(count(&sim), 1, "no room above or below");
    mark(&mut sim, (i32::MIN, 0), (i32::MAX, 0), 100, Job::Dig);
    assert_eq!(count(&sim), 1, "no overflow");

    mark(&mut sim, col(8, 0), col(8 + MAX_SITE - 1, 0), 100, Job::Dig);
    assert_eq!(sim.factory.sites.list[1].id, 1, "MAX_SITE fits");
    for i in 2..MAX_SITES as i32 {
        mark(&mut sim, col(i, 10), col(i, 10), 100, Job::Fill);
    }
    assert_eq!(count(&sim), MAX_SITES);
    mark(&mut sim, col(0, 20), col(0, 20), 100, Job::Fill);
    assert_eq!(count(&sim), MAX_SITES, "full");

    sim.apply(P, Action::RemoveSite { id: 0 });
    assert!(!sim.factory.sites.remove(0), "already gone");
    mark(&mut sim, col(0, 0), col(1, 1), 100, Job::Dig);
    let last = sim.factory.sites.list.last().unwrap();
    assert_eq!((last.id, count(&sim)), (MAX_SITES as u32, MAX_SITES), "ids are never reused");
}

#[test]
fn sites_are_saved_and_hashed() {
    let mut sim = Sim::new(1337, 2);
    ground(&mut sim);
    let before = sim.state_hash();
    mark(&mut sim, col(0, 0), col(7, 7), 100, Job::Flatten);
    mark(&mut sim, col(10, 0), col(12, 3), 98, Job::Fill);
    sim.apply(P, Action::RemoveSite { id: 0 });
    mark(&mut sim, col(0, 0), col(3, 3), 101, Job::Dig);
    assert_ne!(sim.state_hash(), before);

    let mut w = ByteWriter::default();
    sim.write_state(&mut w);
    let mut back = Sim::new(1337, 2);
    back.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    assert_eq!(back.factory.sites.list, sim.factory.sites.list);
    assert_eq!(back.state_hash(), sim.state_hash());
    mark(&mut back, col(20, 20), col(21, 21), 100, Job::Dig);
    assert_eq!(back.factory.sites.list[2].id, 3, "the counter was saved too");

    let bytes = |sites: &Sites| {
        let mut w = ByteWriter::default();
        sites.write_state(&mut w);
        w.bytes
    };
    let good = bytes(&sim.factory.sites);
    assert!(Sites::read_state(&mut ByteReader::new(&good)).is_some());
    let mut bad_job = good.clone();
    bad_job[4 + 4 + 4 + 20] = 3;
    assert!(Sites::read_state(&mut ByteReader::new(&bad_job)).is_none(), "unknown job");
    let mut bad_id = good.clone();
    bad_id[0] = 1;
    assert!(Sites::read_state(&mut ByteReader::new(&bad_id)).is_none(), "an id past the counter");
}

#[test]
fn the_survey_counts_what_a_job_would_move_among_loaded_chunks() {
    let mut sim = Sim::new(1337, 2);
    ground(&mut sim);
    sim.world.update_streaming(at(4, 100, 4).as_vec3(), &[]);
    while sim.world.work_step() {}
    let hash = sim.state_hash();
    let survey = |job| survey_site(&sim.world, col(0, 0), col(7, 7), 100, job).unwrap();

    let flat = survey(Job::Flatten);
    let want = SiteSurvey { cut: 10, fill: 8, ore: 1, trees: 5, water: 3, unseen: 0 };
    assert_eq!(flat, want, "the hill, its ore and the tree; the pit and the pool");
    assert_eq!(flat.spoil(), 10 - 1 - 5 - 8, "ground to bring in");
    assert_eq!(survey(Job::Dig), SiteSurvey { fill: 0, water: 0, ..want });
    assert_eq!(survey(Job::Fill), SiteSurvey { cut: 0, ore: 0, trees: 0, ..want });
    let low = survey_site(&sim.world, col(0, 0), col(7, 7), 96, Job::Dig).unwrap();
    assert_eq!(
        (low.cut, low.water),
        (10 + 64 * 4 - 4 - 3, 3),
        "down to 96: the ground too, round the pool and the pit"
    );

    let far = survey_site(&sim.world, (5000, 5000), (5003, 5001), 100, Job::Dig).unwrap();
    assert_eq!(far, SiteSurvey { unseen: 8, ..SiteSurvey::default() });
    assert!(survey_site(&sim.world, (0, 0), (MAX_SITE, 0), 100, Job::Dig).is_none(), "too big");
    assert_eq!(sim.state_hash(), hash, "a query");
}
