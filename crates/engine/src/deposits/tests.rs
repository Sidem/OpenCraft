use super::*;
use crate::block::IRON_ORE;

fn deposit(tier: Tier, r: f32) -> Deposit {
    Deposit {
        key: DepositKey { tier, cx: 0, cz: 0, ore: IRON_ORE, index: 0 },
        center: IVec3::new(0, 40, 0),
        radii: [r, r, r],
        seed: 7,
    }
}

#[test]
fn shape_stays_inside_bounds() {
    let d = deposit(Tier::Vein, 4.3);
    let (lo, hi) = d.bounds();
    let mut inside = 0;
    for y in lo.y - 3..=hi.y + 3 {
        for z in lo.z - 3..=hi.z + 3 {
            for x in lo.x - 3..=hi.x + 3 {
                let p = IVec3::new(x, y, z);
                if d.contains(p) {
                    inside += 1;
                    assert!(p.x >= lo.x && p.x <= hi.x && p.y >= lo.y && p.y <= hi.y && p.z >= lo.z && p.z <= hi.z);
                }
            }
        }
    }
    // Roughly a ball of radius 4.3 (volume ~333), give or take the ragged edge.
    assert!((200..500).contains(&inside), "{inside} blocks");
}

#[test]
fn ownership_prefers_bigger_tiers() {
    let lode = DepositKey { tier: Tier::Lode, cx: 5, cz: 5, ore: IRON_ORE, index: 0 };
    let outcrop = DepositKey { tier: Tier::Outcrop, cx: -5, cz: -5, ore: IRON_ORE, index: 0 };
    assert!(lode < outcrop);
}

/// First generated ore block of `tier` found scanning chunks near the origin.
fn find_ore(world: &mut World, tier: Tier) -> (IVec3, Deposit) {
    for cz in -3..=3 {
        for cx in -3..=3 {
            for cy in 0..WORLD_HEIGHT_CHUNKS {
                let c = IVec3::new(cx, cy, cz);
                let chunk = world.original_chunk(c);
                for i in 0..crate::chunk::CHUNK_VOLUME {
                    let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
                    if !block::is_ore(chunk.get(x, y, z)) {
                        continue;
                    }
                    let p = IVec3::new(cx * 32 + x as i32, cy * 32 + y as i32, cz * 32 + z as i32);
                    let d = world.generator_mut().deposit_at(p).expect("every ore block has an owner");
                    if d.tier() == tier {
                        return (p, d);
                    }
                }
            }
        }
    }
    panic!("no {tier:?} near the origin");
}

#[test]
fn members_are_exactly_the_blocks_a_deposit_owns() {
    let mut world = World::new(11, 2);
    for tier in [Tier::Outcrop, Tier::Vein] {
        let (p, d) = find_ore(&mut world, tier);
        let mut deps = Deposits::default();
        deps.ensure(&mut world, d);
        let st = deps.get(&d.key).unwrap();
        assert!(st.members.contains(&p));
        assert_eq!(st.remaining_blocks, st.initial_blocks, "nothing has been mined yet");
        for &m in &st.members {
            assert_eq!(world.generator_mut().deposit_at(m).unwrap().key, d.key);
        }
        println!("{} has {} blocks", d.name(), st.initial_blocks);
    }
}

#[test]
fn drawing_converts_the_nearest_block_even_while_unloaded() {
    let mut world = World::new(11, 2);
    let (p, d) = find_ore(&mut world, Tier::Outcrop);
    let mut deps = Deposits::default();
    deps.ensure(&mut world, d);
    let blocks = deps.get(&d.key).unwrap().remaining_blocks;
    let grade = Tier::Outcrop.grade() as f64;
    let cap = Tier::Outcrop.draw_cap();

    // A generous miner is still held to the outcrop's draw cap.
    let mut drawn = 0.0;
    for tick in 0..(grade as u64 + 5) {
        drawn += deps.draw(&mut world, &d.key, 50.0, p, tick, 1.0);
    }
    assert!((drawn - cap * (grade + 5.0)).abs() < 1e-6, "drawn {drawn}");
    let st = deps.get(&d.key).unwrap();
    assert_eq!(st.remaining_blocks, blocks - 1);
    assert_eq!(world.block_anywhere(p), Some(SPENT_ROCK), "the block at the drill goes first");
}

#[test]
fn taper_slows_the_last_fifth() {
    let mut st = DepositState {
        deposit: deposit(Tier::Outcrop, 2.0),
        members: Vec::new(),
        initial_blocks: 100,
        remaining_blocks: 100,
        partial: 0.0,
        budget: 0.0,
        budget_tick: 0,
    };
    assert_eq!(st.taper(), 1.0);
    st.remaining_blocks = 20;
    assert!((st.taper() - 1.0).abs() < 1e-9);
    st.remaining_blocks = 10;
    assert!((st.taper() - 0.5).abs() < 1e-9);
    st.remaining_blocks = 1;
    assert_eq!(st.taper(), TAPER_FLOOR);
}
