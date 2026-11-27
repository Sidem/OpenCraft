use super::*;

const YEAR: u64 = YEAR_DAYS * DAY_TICKS;

/// The tick at day `day` (from the world's first midnight) and hour `hour`.
fn at(day: u64, hour: u64) -> u64 {
    day * DAY_TICKS + hour * DAY_TICKS / 24 - START
}

fn every_hour_of_a_year() -> impl Iterator<Item = u64> {
    (1..=YEAR_DAYS * 24).map(|h| at(YEAR_DAYS, 0) + h * DAY_TICKS / 24)
}

#[test]
fn cold_index_bins_whole_degrees_and_marks_places_too_warm_to_freeze() {
    assert_eq!(cold_index(120), NEVER, "a year's mean of 12 °C never has a day below 0 °C");
    assert_eq!(cold_index(500), NEVER);
    assert_eq!(cold_index(119), NEVER - 1);
    assert_eq!(cold_index(-510), 0);
    assert_eq!(cold_index(-900), 0);
    assert_eq!(cold_index(-1), cold_index(-10), "tenths round down to the degree below");
}

#[test]
fn a_place_too_warm_to_freeze_never_ices_and_the_coldest_never_thaws() {
    for tick in every_hour_of_a_year().step_by(5) {
        let table = IceTable::at(tick);
        for kind in [Water::Lake, Water::Flowing, Water::Sea] {
            assert_eq!(table.level(kind, NEVER), 0, "tick {tick}");
            assert!(table.level(kind, 5) >= BEARS_LEVEL, "-46 °C a year bears all year, tick {tick}");
        }
    }
}

#[test]
fn lakes_freeze_before_flowing_water_and_the_sea_last() {
    for tick in every_hour_of_a_year().step_by(7) {
        let table = IceTable::at(tick);
        for cold in 0..COLD_LEVELS as u8 {
            let (lake, flow, sea) =
                (table.level(Water::Lake, cold), table.level(Water::Flowing, cold), table.level(Water::Sea, cold));
            assert!(lake >= flow && flow >= sea, "cold {cold} tick {tick}: {lake} {flow} {sea}");
        }
    }
}

#[test]
fn a_mild_place_has_a_frozen_lake_in_winter_only_and_an_open_sea() {
    // A year's mean of 1 °C: a lake high in the calm land round spawn.
    let cold = cold_index(10);
    let mid_winter = at(YEAR_DAYS + 33, 12);
    let mid_summer = at(YEAR_DAYS + 13, 12);
    assert!(IceTable::at(mid_winter).level(Water::Lake, cold) >= BEARS_LEVEL);
    assert_eq!(IceTable::at(mid_summer).level(Water::Lake, cold), 0);
    for tick in every_hour_of_a_year().step_by(3) {
        assert!(IceTable::at(tick).level(Water::Sea, cold) < BEARS_LEVEL, "tick {tick}");
    }
}

#[test]
fn ice_lags_the_cold_so_late_winter_is_thicker_than_early_winter() {
    // The coldest day is 31.5; days 29 and 34 are about as cold, but frost has piled up by the later one.
    let cold = cold_index(40);
    let early = IceTable::at(at(YEAR_DAYS + 29, 12)).level(Water::Flowing, cold);
    let late = IceTable::at(at(YEAR_DAYS + 34, 12)).level(Water::Flowing, cold);
    assert!(late > early, "early {early} late {late}");
}

#[test]
fn ice_grows_and_melts_smoothly_minute_by_minute() {
    let minute = DAY_TICKS / (24 * 60);
    let mut last = IceTable::at(at(YEAR_DAYS, 0));
    for m in 1..(YEAR_DAYS * 24 * 60 / 4) {
        let now = IceTable::at(at(YEAR_DAYS, 0) + m * 4 * minute);
        for (a, b) in last.bytes().iter().zip(now.bytes()) {
            assert!(a.abs_diff(*b) <= 3, "a jump of {} at minute {}", a.abs_diff(*b), m * 4);
        }
        last = now;
    }
}

#[test]
fn the_table_repeats_every_year() {
    for tick in every_hour_of_a_year().step_by(11) {
        assert_eq!(IceTable::at(tick), IceTable::at(tick + YEAR));
    }
}

#[test]
fn warmth_is_the_climate_on_the_lattice_and_between_its_corners_elsewhere() {
    let gen = WorldGen::new(1337);
    for (x, z) in [(3008, -2048), (-4096, 640), (0, 0)] {
        assert_eq!(warmth_at(&gen, x, z), Climate::at(&gen, x, z).warmth);
    }
    let corners = [(2016, 3008), (2048, 3008), (2016, 3040), (2048, 3040)].map(|(x, z)| Climate::at(&gen, x, z).warmth);
    let (lo, hi) = (*corners.iter().min().unwrap(), *corners.iter().max().unwrap());
    let w = warmth_at(&gen, 2030, 3021);
    assert!((lo..=hi).contains(&w), "{lo}..{hi}: {w}");
}

#[test]
fn the_table_fits_the_shaders_words() {
    // web/src/render/water-glsl.ts `ICE_WORDS` uvec4s of four levels each; the shader's flowing bit is 64.
    assert_eq!(IceTable::at(0).bytes().len(), 12 * 16);
}

#[test]
fn water_kinds_by_block_and_height() {
    use crate::block::{AIR, FLOW_1, FLOW_7, WATER};
    assert_eq!(Water::of(WATER, SEA_LEVEL + 1), Some(Water::Lake));
    assert_eq!(Water::of(WATER, SEA_LEVEL), Some(Water::Sea));
    assert_eq!(Water::of(FLOW_1, SEA_LEVEL - 5), Some(Water::Flowing));
    assert_eq!(Water::of(FLOW_7, SEA_LEVEL + 9), Some(Water::Flowing));
    assert_eq!(Water::of(AIR, SEA_LEVEL), None);
}
