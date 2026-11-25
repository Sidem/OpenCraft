use super::*;

const SEEDS: [u32; 3] = [2024, 1337, 7];

/// Every clay bed or kaolin pocket seeded within `r` chunk columns of spawn.
fn seeded(g: &WorldGen, ore: u16, r: i32) -> Vec<Deposit> {
    let mut out = Vec::new();
    for cz in -r..=r {
        for cx in -r..=r {
            g.seed_clay(cx, cz, &mut out);
        }
    }
    out.retain(|d| d.ore() == ore);
    out
}

fn dist(p: IVec3) -> i32 {
    f64::from(p.x).hypot(f64::from(p.z)) as i32
}

/// The generated block at `p`.
fn block_at(g: &mut WorldGen, p: IVec3) -> u16 {
    let chunk = g.generate(IVec3::new(p.x >> 5, p.y >> 5, p.z >> 5));
    chunk.get((p.x & 31) as usize, (p.y & 31) as usize, (p.z & 31) as usize)
}

// Measured 2026-11-25 over seeds 1 to 30: the nearest bed lies 50 to 165 blocks out, 39 to 118 beds within 320.
#[test]
fn every_start_has_clay_by_the_water_within_a_walk() {
    for seed in SEEDS {
        let g = WorldGen::new(seed);
        let beds = seeded(&g, CLAY, 10);
        assert!(beds.len() >= 20, "seed {seed}: {} beds within 320 blocks", beds.len());
        let nearest = beds.iter().map(|d| dist(d.center)).min().unwrap();
        assert!(nearest <= 200, "seed {seed}: the nearest bed is {nearest} blocks out");
        for d in &beds {
            assert!(g.by_water(d.center.x, d.center.z), "seed {seed}: a bed at {:?} lies away from water", d.center);
            assert_eq!(d.tier(), Tier::Outcrop);
        }
    }
}

#[test]
fn a_clay_bed_breaks_the_surface_where_it_lies() {
    for seed in SEEDS {
        let mut g = WorldGen::new(seed);
        let beds = seeded(&g, CLAY, 6);
        let mut shown = 0;
        for d in &beds {
            let p = d.center;
            if block_at(&mut g, p) != CLAY {
                continue; // a richer deposit owns its middle
            }
            // The bed's middle lies one block under the ground: the top block or the one under the water.
            let top = IVec3::new(p.x, g.height_at(p.x, p.z), p.z);
            shown += usize::from(block_at(&mut g, top) == CLAY);
        }
        assert!(shown * 2 >= beds.len(), "seed {seed}: {shown} of {} beds show at the surface", beds.len());
    }
}

#[test]
fn kaolin_lies_in_highland_granite_away_from_the_start() {
    let mut found = 0;
    for seed in SEEDS {
        let mut g = WorldGen::new(seed);
        for d in seeded(&g, KAOLIN, 24) {
            let p = d.center;
            let h = g.height_at(p.x, p.z);
            assert_eq!(g.biome_at(p.x, p.z, h).base(), Biome::Highlands, "seed {seed}: kaolin at {p:?}");
            assert!(dist(p) >= KAOLIN_FROM, "seed {seed}: kaolin {} blocks out", dist(p));
            found += usize::from(block_at(&mut g, p) == KAOLIN);
        }
    }
    assert!(found >= SEEDS.len(), "kaolin is found in every world: {found}");
}
