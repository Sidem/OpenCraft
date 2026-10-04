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
    Site { id: 0, lo, hi, level, job, high, low, done: 0, tunnel: None }
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
    bad_job[4 + 4 + 4 + 20] = 4;
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

fn tunnel(a: IVec3, b: IVec3, size: u8) -> Tunnel {
    Tunnel::new(a, b, size).expect("a tunnel")
}

#[test]
fn a_tunnel_runs_along_the_longer_axis_and_keeps_to_its_limits() {
    let a = IVec3::new(10, 50, 20);
    let t = tunnel(a, IVec3::new(14, 50, 22), 0);
    assert_eq!((t.to, t.steps(), t.cells()), (IVec3::new(14, 50, 20), 5, 10), "snapped onto x");
    assert_eq!(tunnel(a, IVec3::new(8, 50, 21), 1).to, IVec3::new(8, 50, 20), "backwards");
    assert_eq!(tunnel(a, IVec3::new(10, 50, 26), 2).cells(), 7 * 25, "along z, 5 × 5");
    let up = |dx: i32, dy: i32| Tunnel::new(a, a + IVec3::new(dx, dy, 0), 1);
    assert!(up(4, 2).is_some() && up(4, 3).is_none(), "1 block in 2 at most");
    assert!(up(5, -2).is_some() && up(2, 2).is_none() && up(0, 1).is_none(), "down too; no shafts");
    assert!(up(tunnel::MAX_TUNNEL as i32 - 1, 0).is_some() && up(tunnel::MAX_TUNNEL as i32, 0).is_none(), "length");
    assert!(Tunnel::new(a, a + IVec3::new(4, 0, 0), 3).is_none(), "no such section");
    assert!(Tunnel::new(IVec3::new(0, 253, 0), IVec3::new(4, 253, 0), 1).is_none(), "the roof is out of the world");
    assert!(Tunnel::new(IVec3::new(0, 0, 0), IVec3::new(4, 0, 0), 0).is_none(), "the floor is bedrock");
    assert!(Tunnel::new(IVec3::new(i32::MIN, 50, 0), IVec3::new(i32::MAX, 50, 0), 0).is_none(), "no overflow");
}

#[test]
fn tunnel_cells_stand_on_the_path_and_rise_with_it() {
    let a = IVec3::new(0, 50, 0);
    let t = tunnel(a, IVec3::new(4, 52, 0), 0);
    let cells: Vec<IVec3> = (0..t.cells()).map(|i| t.cell(i)).collect();
    let ys: Vec<i32> = cells.iter().map(|c| c.y).collect();
    assert_eq!(ys, [50, 51, 51, 52, 51, 52, 52, 53, 52, 53], "the path rises 1 in 2, a floor block and one above");
    assert_eq!(cells[0], a, "the first cell is the path's start");
    let big = tunnel(a, IVec3::new(0, 52, -6), 1);
    assert_eq!(
        (big.cell(0), big.cell(1), big.cell(3), big.cell(9)),
        (IVec3::new(-1, 50, 0), IVec3::new(0, 50, 0), IVec3::new(-1, 51, 0), IVec3::new(-1, 50, -1),)
    );
    for t in [t, big, tunnel(a, IVec3::new(-10, 45, 0), 2)] {
        let (lo, hi, bottom, top) = t.bounds();
        let all: Vec<IVec3> = (0..t.cells()).map(|i| t.cell(i)).collect();
        for x in lo.0 - 1..=hi.0 + 1 {
            for z in lo.1 - 1..=hi.1 + 1 {
                for y in bottom - 1..=top + 1 {
                    let p = IVec3::new(x, y, z);
                    assert_eq!(t.has_cell(p), all.contains(&p), "{p:?}");
                }
            }
        }
        assert_eq!((bottom, top), (all.iter().map(|c| c.y).min().unwrap(), all.iter().map(|c| c.y).max().unwrap()));
    }
}

#[test]
fn tunnels_clash_with_sites_only_where_they_share_heights() {
    let mut sim = Sim::new(1337, 2);
    ground(&mut sim);
    mark(&mut sim, col(0, 0), col(7, 7), 100, Job::Dig);
    let count = |sim: &Sim| sim.factory.sites.list.len();
    let tunnel_at = |sim: &mut Sim, from: IVec3, to: IVec3| {
        sim.apply(P, Action::MarkTunnel { from, to, size: 1 });
    };
    tunnel_at(&mut sim, at(-3, 101, 3), at(5, 101, 3));
    assert_eq!(count(&sim), 1, "through the dug air");
    tunnel_at(&mut sim, at(-3, 80, 3), at(5, 80, 3));
    assert_eq!(count(&sim), 2, "well below it");
    let s = sim.factory.sites.list[1];
    assert_eq!((s.job, s.id, s.cells(), s.tunnel.is_some()), (Job::Tunnel, 1, 9 * 9, true));
    assert!(s.covers(at(0, 80, 3)) && !s.covers(at(0, 85, 3)) && s.picks(at(0, 85 - 4, 3)));
    assert!(sim.factory.sites.list[0].covers(at(3, 101, 3)) && !sim.factory.sites.list[0].covers(at(3, 80, 3)));
    tunnel_at(&mut sim, at(1, 80, -2), at(1, 80, 5));
    assert_eq!(count(&sim), 2, "crossing the first tunnel");
    tunnel_at(&mut sim, at(1, 70, -2), at(1, 70, 5));
    assert_eq!(count(&sim), 3, "another below");
    mark(&mut sim, col(0, 0), col(1, 1), 70, Job::Tunnel);
    assert_eq!(count(&sim), 3, "a tunnel is not marked as an area");
}

#[test]
fn tunnels_are_saved_and_checked_on_loading() {
    let mut sim = Sim::new(1337, 2);
    ground(&mut sim);
    let before = sim.state_hash();
    sim.apply(P, Action::MarkTunnel { from: at(0, 80, 0), to: at(-8, 84, 0), size: 2 });
    mark(&mut sim, col(10, 10), col(12, 12), 100, Job::Dig);
    assert_ne!(sim.state_hash(), before);
    let bytes = |sites: &Sites| {
        let mut w = ByteWriter::default();
        sites.write_state(&mut w);
        w.bytes
    };
    let good = bytes(&sim.factory.sites);
    let back = Sites::read_state(&mut ByteReader::new(&good)).expect("reads back");
    assert_eq!(back.list, sim.factory.sites.list);
    assert_eq!(bytes(&back), good);
    let mut moved = good.clone();
    moved[4 + 4 + 4] ^= 1;
    assert!(Sites::read_state(&mut ByteReader::new(&moved)).is_none(), "columns that don't match the bore");
    assert!(Sites::read_state(&mut ByteReader::new(&good[..good.len() - 1])).is_none(), "cut short");
}

#[test]
fn the_tunnel_survey_counts_cells_and_those_beside_water() {
    let mut sim = Sim::new(1337, 2);
    ground(&mut sim);
    sim.world.update_streaming(at(4, 100, 4).as_vec3(), &[]);
    while sim.world.work_step() {}
    let hash = sim.state_hash();
    // 1 × 2 through stone at y 97..=98, passing the pool at (2, 6) (water 98..=100) one block away.
    let t = tunnel(at(0, 97, 5), at(5, 97, 5), 0);
    let s = survey_tunnel(&sim.world, &t);
    assert_eq!((s.cut, s.ore, s.trees, s.unseen), (12, 0, 0, 0));
    assert_eq!(s.water, 1, "the cell below the pool's corner at (2, 98, 5) touches it");
    let far = survey_tunnel(&sim.world, &tunnel(IVec3::new(5000, 97, 5000), IVec3::new(5004, 97, 5000), 0));
    assert_eq!((far.unseen, far.cut), (10, 0));
    assert_eq!(sim.state_hash(), hash, "a query");
}
