//! The weather the world map shows (`web/src/ui/map-weather.ts`): the weather stations and their reach, and the
//! weather sampled on a grid inside it (`weather/reading.rs`, read as a station reads it). Queries only.

use wasm_bindgen::prelude::*;

use crate::factory::Factory;
use crate::weather::Reading;
use crate::worldgen::SEA_LEVEL;
use crate::Game;

/// Numbers per sample in `weather_map`.
const MAP_FIELDS: usize = 10;
/// The closest samples may be, in blocks, and the most one call gives.
const MIN_STEP: i32 = 8;
const MAX_SAMPLES: usize = 8192;
/// How far past a station's circle points are still given, in grid steps (3/4: a square's half-diagonal).
const RIM_STEPS: (i64, i64) = (3, 4);

#[wasm_bindgen]
impl Game {
    /// Every weather station: x, z, the radius it measures (blocks), 1 if it measures now (powered, on) else 0.
    pub fn weather_stations(&self) -> Vec<i32> {
        let r = Factory::WEATHER_RADIUS;
        self.sim.factory.weather_stations().flat_map(|(p, on)| [p.x, p.z, r, i32::from(on)]).collect()
    }

    /// The weather on the `step`-block grid (aligned to multiples of `step`, at least `MIN_STEP`) inside x0..x1, z0..z1
    /// wherever a working station measures it (a point within `RIM_STEPS` of a step outside a circle too, so the map's
    /// squares fill the circle to its rim), `weather_map_fields` numbers per point: x, z, temperature (°C, at the
    /// ground), precipitation and cloud cover (0 to 1), 1 if snow, sea-level pressure (hPa), wind east and south (m/s),
    /// relative humidity (%). At most `MAX_SAMPLES` points (the grid coarsens to fit).
    pub fn weather_map(&self, x0: i32, z0: i32, x1: i32, z1: i32, step: i32) -> Vec<f32> {
        let stations: Vec<_> = self.sim.factory.weather_stations().filter(|s| s.1).map(|s| s.0).collect();
        if stations.is_empty() || x1 <= x0 || z1 <= z0 {
            return Vec::new();
        }
        let mut step = step.max(MIN_STEP);
        while ((x1 - x0) / step + 1) as usize * ((z1 - z0) / step + 1) as usize > MAX_SAMPLES {
            step *= 2;
        }
        let reach = i64::from(Factory::WEATHER_RADIUS) + i64::from(step) * RIM_STEPS.0 / RIM_STEPS.1;
        let (gen, air, r2) = (self.sim.world.generator(), &self.sim.atmosphere, reach.pow(2));
        let covered =
            |x: i32, z: i32| stations.iter().any(|p| i64::from(p.x - x).pow(2) + i64::from(p.z - z).pow(2) <= r2);
        let mut out = Vec::new();
        for z in (z0.div_euclid(step)..=z1.div_euclid(step)).map(|k| k * step) {
            for x in (x0.div_euclid(step)..=x1.div_euclid(step)).map(|k| k * step).filter(|&x| covered(x, z)) {
                let r = Reading::at(air, gen, x, gen.height_at(x, z).max(SEA_LEVEL) + 1, z);
                out.extend([
                    x as f32,
                    z as f32,
                    r.temp as f32 / 100.0,
                    r.sky.precip as f32 / 1000.0,
                    r.sky.cover as f32 / 1000.0,
                    f32::from(u8::from(r.snow)),
                    r.pressure as f32 / 100.0,
                    r.east as f32 / 100.0,
                    r.south as f32 / 100.0,
                    r.humidity as f32 / 10.0,
                ]);
            }
        }
        out
    }

    /// Numbers per point in `weather_map`.
    pub fn weather_map_fields(&self) -> u32 {
        MAP_FIELDS as u32
    }

    /// Changes whenever the air steps (the tick of its last step), so the map redraws its weather only then.
    pub fn weather_version(&self) -> f64 {
        self.sim.atmosphere.tick as f64
    }

    /// Whether a working weather station measures the camera's column (the minimap's readout shows more there).
    pub fn weather_measured_here(&self) -> bool {
        let e = self.render_eye;
        self.sim.factory.weather_covers(e.x.floor() as i32, e.z.floor() as i32)
    }
}
