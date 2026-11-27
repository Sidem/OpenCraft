//! The weather on the world map (`api/weather_map.rs`): a powered weather station measures a circle round it, and the
//! map's samples fall only inside it, with a station's sane numbers.

use super::power_up;
use crate::block::WEATHER_STATION;
use crate::factory::Factory;
use crate::math::IVec3;
use crate::Game;

#[test]
fn the_map_shows_the_weather_inside_a_working_stations_circle_only() {
    let mut g = Game::new(2024, 3);
    g.update(1.0 / 60.0);
    let mast = IVec3::new(g.render_eye.x as i32 + 40, 200, g.render_eye.z as i32);
    assert!(g.weather_map(-3000, -3000, 3000, 3000, 64).is_empty(), "no station, nothing measured");
    g.sim.factory.place(&mut g.sim.world, WEATHER_STATION, mast, 0, mast, 0);
    g.run_ticks(3);
    assert_eq!(g.weather_stations(), [mast.x, mast.z, Factory::WEATHER_RADIUS, 0], "unpowered");
    assert!(g.weather_map(-3000, -3000, 3000, 3000, 64).is_empty() && !g.weather_measured_here());
    power_up(&mut g, mast + IVec3::new(2, 0, 0));
    g.run_ticks(3);
    assert_eq!(g.weather_stations()[3], 1, "powered");
    assert!(g.weather_measured_here(), "the camera is 40 blocks away");
    let (map, n) = (g.weather_map(mast.x - 1000, mast.z - 1000, mast.x + 1000, mast.z + 1000, 64), 10);
    assert_eq!(map.len() % n, 0);
    let r = Factory::WEATHER_RADIUS as f32;
    let points = map.len() / n;
    assert!(points > 300, "{points} points in a 640-block circle at 64-block steps");
    for p in map.chunks(n) {
        let (dx, dz) = (p[0] - mast.x as f32, p[1] - mast.z as f32);
        assert!(dx.hypot(dz) <= r + 48.5, "{p:?} is past the circle's rim (3/4 of a 64-block step)");
        assert!(
            (-40.0..50.0).contains(&p[2]) && (950.0..1060.0).contains(&p[6]) && (0.0..=100.0).contains(&p[9]),
            "{p:?}"
        );
    }
    let coarse = g.weather_map(-200_000, -200_000, 200_000, 200_000, 8).len() / n;
    assert!(coarse > 0 && coarse < points, "a huge view coarsens the grid to its cap: {coarse}");
}
