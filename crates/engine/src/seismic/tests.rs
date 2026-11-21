use super::*;
use crate::action::Action;
use crate::block::{COAL_ORE, GENERATOR, IRON_ORE, POLE, STONE, WATER};
use crate::item::{GOLD_KIT, THUMPER_ITEM, VIOLET_KIT};
use crate::math::IVec3;
use crate::Game;

/// Places a Seismic Thumper at `pos` facing `dir`, wires it to a pole + coal generator, and ticks until powered.
fn setup_powered_thumper(seed: u32, dir: u8) -> (Game, IVec3) {
    let mut g = Game::new(seed, 3);
    // Find a flat open surface near (12, z=12) and level a 4×4 pad.
    let surf = g.sim.world.generator().height_at(12, 12).max(SEA_LEVEL + 4);
    let pos = IVec3::new(12, surf + 1, 12);
    for dx in -2..=4 {
        for dz in -2..=4 {
            g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + dx, surf, pos.z + dz), STONE);
            for dy in 1..=3 {
                g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + dx, surf + dy, pos.z + dz), AIR);
            }
        }
    }
    let pole = pos + IVec3::new(3, 0, 0);
    let gen = pos + IVec3::new(4, 0, 0);
    g.sim.world.set_block_anywhere_later(pole, POLE);
    g.sim.factory.place(&mut g.sim.world, POLE, pole, 0, pole, 0);
    g.sim.world.set_block_anywhere_later(gen, GENERATOR);
    g.sim.factory.place(&mut g.sim.world, GENERATOR, gen, 0, gen, 1); // Mk2 generator: 100 kW
    g.sim.factory.insert(gen, COAL_ORE.into(), 64);

    let inv = &mut g.sim.players[0].as_mut().unwrap().inventory;
    inv.add(THUMPER_ITEM, 1);
    let slot = (0..inv.slots.len()).find(|&i| inv.slots[i].item == THUMPER_ITEM).unwrap() as u8;
    g.act(Action::PlaceBlock { pos, slot, facing: dir, against: pos + IVec3::new(0, -1, 0) });
    g.act(Action::Connect { pole, to: pos });
    g.act(Action::Connect { pole, to: gen });
    g.run_ticks(15);
    (g, pos)
}

#[test]
fn seismic_query_never_moves_state_hash_and_returns_full_slice() {
    let (mut g, pos) = setup_powered_thumper(2024, 1);
    let hash_before = g.sim.state_hash();

    let hdr = g.seismic_header(pos).expect("thumper present");
    assert_eq!(hdr.info.anchor, pos);
    assert_eq!(hdr.info.dir, 1);
    assert_eq!(hdr.info.tier, 0);
    assert_eq!(hdr.info.status, 1, "thumper should be powered and working");
    assert_eq!(hdr.info.power_share, 1000);
    assert!(hdr.snr_percent >= 60);

    // Clicking any of the 8 footprint cells resolves to the same anchor.
    let part_cell = pos + IVec3::new(0, 1, 1);
    let hdr_part = g.seismic_header(part_cell).expect("part cell resolves to thumper");
    assert_eq!(hdr_part.info.anchor, pos);

    assert!(g.seismic_pulse_at(pos, 1, 0, false));
    let reading = g.seismic.reading.as_ref().unwrap();
    let h = (reading.top_y + 1) as usize;
    assert_eq!(reading.cells.len(), SLICE_WIDTH * h * CELL_BYTES);
    // Bedrock floor along the entire 64-block slice.
    for s in 0..SLICE_WIDTH {
        assert_eq!(reading.cells[s * CELL_BYTES + 2], CLASS_BEDROCK);
    }
    // Sky at top_y above thumper.
    let top_idx = (h - 1) * SLICE_WIDTH + SLICE_HALF as usize;
    assert_eq!(reading.cells[top_idx * CELL_BYTES + 2], CLASS_SKY);

    assert_eq!(g.sim.state_hash(), hash_before, "seismic query must never move core state hash");
}

#[test]
fn tiers_sharpen_noise_and_reveal_tunnels_water_and_ores() {
    let (mut g, pos) = setup_powered_thumper(1337, 1); // facing +X: slice runs along X at z = pos.z
    let surf = pos.y - 1;
    // Carve a uniform stone test zone, then embed a 4×4 iron ore vein, a 4×3 player tunnel and a 4×3 aquifer.
    for s in -20..=20 {
        for y in 12..=(surf - 2) {
            g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + s, y, pos.z), STONE);
        }
    }
    // Iron ore block cluster at s = +8..=+11, y = 26..=29.
    for dx in 8..=11 {
        for y in 26..=29 {
            g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + dx, y, pos.z), IRON_ORE);
        }
    }
    // Player-excavated tunnel at s = -10..=-7, y = 24..=26 (originally solid stone!).
    for dx in -10..=-7 {
        for y in 24..=26 {
            g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + dx, y, pos.z), AIR);
        }
    }
    // Underground aquifer at s = -2..=+1, y = 20..=22.
    for dx in -2..=1 {
        for y in 20..=22 {
            g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + dx, y, pos.z), WATER);
        }
    }

    let sample_at = |reading: &SeismicReading, s_off: i32, y: i32| {
        let s = (s_off + SLICE_HALF) as usize;
        let base = (y as usize * SLICE_WIDTH + s) * CELL_BYTES;
        (reading.cells[base], reading.cells[base + 2], reading.cells[base + 3])
    };

    let spread = |game: &mut Game| {
        let a = game.seismic_scan(pos, 100).unwrap();
        let b = game.seismic_scan(pos, 200).unwrap();
        let mut diff = 0u32;
        for s in -19..=-13 {
            for y in 16..=32 {
                let ia = sample_at(&a, s, y).0;
                let ib = sample_at(&b, s, y).0;
                diff += u32::from(ia.abs_diff(ib));
            }
        }
        diff
    };

    // --- Mk1: detects dense anomaly & water, masks exact ore id and masks tunnel as cave ---
    let mk1 = g.seismic_scan(pos, 42).unwrap();
    let (ore_imp1, ore_cls1, ore_det1) = sample_at(&mk1, 9, 27);
    let (rock_imp1, _, _) = sample_at(&mk1, -18, 27);
    let (_, tun_cls1, _) = sample_at(&mk1, -9, 25);
    let (_, wat_cls1, _) = sample_at(&mk1, 0, 21);
    assert!(ore_imp1 > rock_imp1, "iron ore ({ore_imp1}) should read denser than stone ({rock_imp1})");
    assert_eq!(ore_cls1, CLASS_ORE_UNKNOWN);
    assert_eq!(ore_det1, 0, "Mk1 must not leak exact ore id");
    assert_eq!(tun_cls1, CLASS_CAVE, "Mk1 masks player tunnel as generic cave");
    assert_eq!(wat_cls1, CLASS_WATER);
    let s1 = spread(&mut g);

    // Unlock upgrades and raise Mk1 -> Mk2 (4 violet kits) -> Mk3 (4 gold kits).
    g.sim.factory.research.complete_all();
    let inv = &mut g.sim.players[0].as_mut().unwrap().inventory;
    inv.add(VIOLET_KIT, 4);
    inv.add(GOLD_KIT, 4);
    g.act(Action::Upgrade { pos, tier: 3 }); // violet kit colour = 3
    g.run_ticks(10);
    assert_eq!(g.seismic_header(pos).unwrap().info.tier, 1);

    // --- Mk2: separates player tunnel and classifies metallic ore without exact ore id ---
    let mk2 = g.seismic_scan(pos, 42).unwrap();
    let (_, ore_cls2, ore_det2) = sample_at(&mk2, 9, 27);
    let (_, tun_cls2, _) = sample_at(&mk2, -9, 25);
    assert_eq!(ore_cls2, CLASS_ORE_METAL);
    assert_eq!(ore_det2, 0, "Mk2 classifies mineral family without exact ore id");
    assert_eq!(tun_cls2, CLASS_TUNNEL);
    let s2 = spread(&mut g);

    // --- Mk3: razor-sharp resolution + confident ore identification ---
    g.act(Action::Upgrade { pos, tier: 4 }); // gold kit colour = 4
    g.run_ticks(10);
    assert_eq!(g.seismic_header(pos).unwrap().info.tier, 2);

    let mk3 = g.seismic_scan(pos, 42).unwrap();
    let (_, ore_cls3, ore_det3) = sample_at(&mk3, 9, 27);
    let (_, tun_cls3, _) = sample_at(&mk3, -9, 25);
    assert_eq!(ore_cls3, CLASS_ORE_METAL);
    assert_eq!(ore_block_of(ore_det3), IRON_ORE);
    assert_eq!(tun_cls3, CLASS_TUNNEL);
    assert!(!mk3.anomalies.is_empty(), "anomalies should highlight embedded features");
    let s3 = spread(&mut g);

    // Noise variance across different pulse seeds shrinks monotonically Mk1 > Mk2 > Mk3.
    assert!(s1 > s2 && s2 > s3, "pulse-to-pulse noise must shrink with Mk: Mk1={s1}, Mk2={s2}, Mk3={s3}");
}

#[test]
fn thumper_save_round_trip_and_power_switch() {
    let (mut g, pos) = setup_powered_thumper(7, 2);
    g.act(Action::SetMachineEnabled { pos, on: false });
    g.run_ticks(5);
    assert_eq!(g.seismic_header(pos).unwrap().info.status, 2, "switched off");
    assert_eq!(g.seismic_header(pos).unwrap().snr_percent, 0);

    let bytes = g.save();
    let mut loaded = Game::load(&bytes, 3).expect("thumper save loads cleanly");
    loaded.run_ticks(5);
    let hdr = loaded.seismic_header(pos).expect("thumper survives save round-trip");
    assert_eq!(hdr.info.dir, 2);
    assert_eq!(hdr.info.status, 2);
}

#[test]
fn mri_slice_scrubber_rotation_and_3d_volume_find_off_axis_features_without_moving_state_hash() {
    let (mut g, pos) = setup_powered_thumper(1337, 1); // facing +X, so perp axis is +Z
    let surf = pos.y - 1;
    // Upgrade to Mk3 so exact ore IDs and player tunnels resolve cleanly.
    g.sim.factory.research.complete_all();
    let inv = &mut g.sim.players[0].as_mut().unwrap().inventory;
    inv.add(VIOLET_KIT, 4);
    inv.add(GOLD_KIT, 4);
    g.act(Action::Upgrade { pos, tier: 3 });
    g.act(Action::Upgrade { pos, tier: 4 });
    g.run_ticks(10);

    // Embed an off-axis iron ore block 6 blocks south (+Z) of the thumper (invisible at perp_offset = 0).
    for dx in 3..=6 {
        for dz in 5..=8 {
            for y in 24..=27 {
                g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + dx, y, pos.z + dz), IRON_ORE);
            }
        }
    }
    // And an off-axis excavated tunnel 8 blocks north (-Z) of the thumper.
    for dx in -6..=-3 {
        for dz in -9..=-7 {
            for y in 28..=30 {
                g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + dx, y, pos.z + dz), STONE);
            }
        }
    }
    for dx in -5..=-4 {
        for y in 28..=30 {
            g.sim.world.set_block_anywhere_later(IVec3::new(pos.x + dx, y, pos.z - 8), AIR);
        }
    }

    let hash_before = g.sim.state_hash();

    // 1. MRI slice scrubber shifted +6m along perp (+Z) hits the off-axis iron ore!
    let shifted = g.seismic_scan_slice(pos, 7, 6, false).unwrap();
    assert_eq!(shifted.perp_offset, 6);
    assert_eq!(shifted.dir, 1);
    let idx = (25usize * SLICE_WIDTH + (4 + SLICE_HALF) as usize) * CELL_BYTES;
    assert_eq!(ore_block_of(shifted.cells[idx + 3]), IRON_ORE);

    // 2. Flipping axis 90° rotates dir 1 (+X) -> 2 (+Z) and lets us slice N–S through the ore at perp = -4 (-X)!
    let flipped = g.seismic_scan_slice(pos, 7, -4, true).unwrap();
    assert_eq!(flipped.dir, 2);
    let fidx = (25usize * SLICE_WIDTH + (6 + SLICE_HALF) as usize) * CELL_BYTES;
    assert_eq!(ore_block_of(flipped.cells[fidx + 3]), IRON_ORE);

    // 3. 3D volume scan finds both the off-axis iron ore and the off-axis player tunnel without moving state_hash!
    let count = g.seismic_volume_at(pos, 7);
    assert!(count > 0, "3D volume must extract subsurface feature voxels");
    let vol = g.seismic.volume.as_ref().unwrap();
    assert_eq!(vol.surface.len(), VOL_SIZE * VOL_SIZE);
    let mut saw_iron = false;
    let mut saw_tunnel = false;
    for chunk in vol.voxels.chunks_exact(VOL_VOXEL_BYTES) {
        if ore_block_of(chunk[5]) == IRON_ORE {
            saw_iron = true;
        }
        if chunk[4] == CLASS_TUNNEL {
            saw_tunnel = true;
        }
    }
    assert!(saw_iron, "3D volume must detect embedded off-axis iron ore");
    assert!(saw_tunnel, "3D volume must detect embedded off-axis player tunnel");
    assert!(surf > 30);
    assert_eq!(g.sim.state_hash(), hash_before, "MRI slice and 3D volume queries must never move core state hash");
}
