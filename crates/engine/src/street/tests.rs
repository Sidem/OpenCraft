use super::*;
use crate::action::Action;
use crate::block::{post_dir, street_post, STONE, STREET};
use crate::factory::INSTANCE_FLOATS;
use crate::item::ItemId;
use crate::sim::PlayerId;
use crate::tests::run_until_ready;
use crate::Game;

const P: PlayerId = PlayerId(0);
const AT: IVec3 = IVec3::new(40, 200, 40);

fn place(sim: &mut Sim, pos: IVec3, facing: u8) -> bool {
    sim.put_block(P, pos, ItemId::block(STREET), facing, pos - UP)
}

/// The four blocks of the lamp based at `base`, bottom first.
fn column(sim: &mut Sim, base: IVec3) -> Vec<block::BlockId> {
    (0..HEIGHT).map(|k| sim.block(base + UP * k)).collect()
}

#[test]
fn a_lamp_is_three_posts_leaning_the_way_the_placer_faced_and_a_head() {
    for facing in 0..4u8 {
        let mut sim = Sim::new(7, 2);
        assert!(place(&mut sim, AT, facing));
        let post = street_post(facing);
        assert_eq!(column(&mut sim, AT), [post, post, post, STREET]);
        assert_eq!(post_dir(post), Some(facing));
    }
    let mut sim = Sim::new(7, 2);
    assert!(place(&mut sim, AT, 6));
    assert_eq!(sim.block(AT), street_post(2), "a facing is taken modulo four");
}

#[test]
fn it_needs_all_four_cells_free_and_changes_nothing_otherwise() {
    for k in 0..HEIGHT {
        let mut sim = Sim::new(7, 2);
        sim.world.set_block_anywhere(AT + UP * k, STONE);
        assert!(!place(&mut sim, AT, 0), "stone in cell {k}");
        let mut expect = vec![AIR; 4];
        expect[k as usize] = STONE;
        assert_eq!(column(&mut sim, AT), expect);
    }
}

#[test]
fn breaking_any_cell_clears_the_whole_lamp_and_counts_one_break() {
    for k in 0..HEIGHT {
        let mut sim = Sim::new(7, 2);
        place(&mut sim, AT, 1);
        sim.events.clear();
        sim.apply(P, Action::BreakBlock { pos: AT + UP * k });
        assert_eq!(column(&mut sim, AT), [AIR; 4], "broken from cell {k}");
        let broken = sim.events.iter().filter(|e| matches!(e, SimEvent::BlockBroken { .. })).count();
        assert_eq!(broken, 1, "one lamp is one break, from cell {k}");
    }
}

#[test]
fn lamps_stacked_or_side_by_side_break_apart() {
    let mut sim = Sim::new(7, 2);
    let (above, beside) = (AT + UP * HEIGHT, AT + IVec3::new(1, 0, 0));
    for base in [AT, above, beside] {
        assert!(place(&mut sim, base, 0));
    }
    sim.apply(P, Action::BreakBlock { pos: AT + UP * 2 });
    assert_eq!(column(&mut sim, AT), [AIR; 4]);
    assert_eq!(
        column(&mut sim, above),
        [street_post(0), street_post(0), street_post(0), STREET],
        "the one above stays"
    );
    assert_eq!(column(&mut sim, beside)[3], STREET, "and the one beside it");
    // Breaking the head of the lower one never takes the posts of the one above it.
    assert!(place(&mut sim, AT, 0));
    sim.apply(P, Action::BreakBlock { pos: AT + UP * 3 });
    assert_eq!(column(&mut sim, AT), [AIR; 4]);
    assert_eq!(column(&mut sim, above)[0], street_post(0));
}

#[test]
fn a_lamp_survives_a_save_and_a_load_as_plain_blocks() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let at = IVec3::new(g.body().pos.x as i32 + 3, 200, g.body().pos.z as i32 + 3);
    assert!(place(&mut g.sim, at, 2));
    let bytes = g.save();
    let mut back = Game::load(&bytes, 3).expect("loads");
    assert_eq!(column(&mut back.sim, at), [street_post(2), street_post(2), street_post(2), STREET]);
    back.sim.apply(P, Action::BreakBlock { pos: at + UP });
    assert_eq!(column(&mut back.sim, at), [AIR; 4], "and breaks whole after the load");
}

/// The model stays inside its column (the column is what the player aims at and what the light shines from), uses
/// the lantern's place, and is small: a few dozen boxes at most.
#[test]
fn the_model_fits_its_column_and_hangs_the_lantern_where_the_halo_is() {
    for dir in 0..4u8 {
        let mut out = Vec::new();
        street_model(&mut out, Vec3::new(0.0, 0.0, 0.0), dir);
        assert!(!out.is_empty() && out.len() / INSTANCE_FLOATS <= 48, "{} boxes", out.len() / INSTANCE_FLOATS);
        let mut lamp_glow = false;
        for b in out.chunks(INSTANCE_FLOATS) {
            let (c, size, yaw) = ([b[0], b[1], b[2]], [b[4], b[5], b[6]], b[3]);
            // Half the box's reach in x and z once turned by its yaw (a conservative bound), and its pitch ignored.
            let (s, co) = (yaw.sin().abs(), yaw.cos().abs());
            let half = [(size[0] * co + size[2] * s) / 2.0, size[1] / 2.0, (size[0] * s + size[2] * co) / 2.0];
            for (i, limit) in [0.5f32, 2.0, 0.5].into_iter().enumerate() {
                assert!(c[i].abs() + half[i] <= limit + 1e-3, "dir {dir}: box {b:?} leaves the column on axis {i}");
            }
            if b[8] == f32::from(crate::block::tex::LAMP) {
                let dir_vec = crate::factory::DIRS[dir as usize].as_vec3();
                let want = (LANTERN_OUT * dir_vec.x, LANTERN_Y, LANTERN_OUT * dir_vec.z);
                lamp_glow |= (f64::from(c[0]) - want.0).abs() < 0.2
                    && (f64::from(c[1]) - want.1).abs() < 0.3
                    && (f64::from(c[2]) - want.2).abs() < 0.2;
            }
        }
        assert!(lamp_glow, "dir {dir}: a glowing lamp-layer box hangs at the lantern's place");
    }
}

/// A lamp is drawn per street corner, so its model stays cheap: every dir draws the same few parts.
#[test]
fn the_model_is_cheap_and_the_same_for_every_dir() {
    let count = |dir| {
        let mut out = Vec::new();
        street_model(&mut out, Vec3::new(3.0, -1.0, 2.0), dir);
        out.len() / INSTANCE_FLOATS
    };
    assert!(count(0) <= 24, "{} boxes", count(0));
    assert!((1..4).all(|dir| count(dir) == count(0)));
}

#[test]
fn the_lantern_is_where_the_halo_hangs_and_the_arm_is_read_from_the_post() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let at = IVec3::new(g.body().pos.x as i32 + 3, 200, g.body().pos.z as i32 + 3);
    assert!(place(&mut g.sim, at, 3));
    let head = at + UP * 3;
    assert_eq!(arm_dir(&g.sim.world, head), 3);
    let l = lantern_at(head, 3);
    let (dx, dz) = (l.x - (f64::from(head.x) + 0.5), l.z - (f64::from(head.z) + 0.5));
    let way = crate::factory::DIRS[3].as_vec3();
    assert!((dx - way.x * LANTERN_OUT).abs() < 1e-9 && (dz - way.z * LANTERN_OUT).abs() < 1e-9);
    assert!(l.y > f64::from(head.y) + 0.4 && l.y < f64::from(head.y) + 1.0, "inside the head cell");
}
