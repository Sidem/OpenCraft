//! The weather the host draws and the calendar forecasts (`weather/`): queries only, at the camera and the core tick.

use wasm_bindgen::prelude::*;

use crate::math::IVec3;
use crate::weather::{self, Climate, FORECAST_FIELDS};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The weather at the camera now: cloud cover, gloom and precipitation (0 to 1), 1 if it falls as snow, the
    /// temperature in degrees Celsius, and the sky's kind (as in `weather_forecast`).
    pub fn weather_here(&self) -> Vec<f32> {
        let (p, tick) = (self.weather_spot(), self.sim.tick);
        let climate = Climate::at(self.sim.world.generator(), p.x, p.z);
        let sky = climate.sky(self.sim.world.generator().seed(), p.x, p.z, tick);
        let temp = climate.temperature(p.y, tick);
        let snow = sky.precip >= weather::RAIN_FROM && temp < 0;
        let unit = |v: u32| v as f32 / 1000.0;
        let kind = f32::from(sky.kind() as u8);
        vec![unit(sky.cover), unit(sky.gloom), unit(sky.precip), f32::from(u8::from(snow)), temp as f32 / 10.0, kind]
    }

    /// `days` days ahead at the camera, today first, `forecast_fields` numbers each: the worst sky (0 clear, 1 cloudy,
    /// 2 rain, 3 storm), 1 if it is snow, and the low and high temperatures in tenths of a degree.
    pub fn weather_forecast(&self, days: u32) -> Vec<i32> {
        weather::forecast(self.sim.world.generator(), self.weather_spot(), self.sim.tick, days.min(16))
    }

    /// Numbers per day in `weather_forecast`.
    pub fn forecast_fields(&self) -> u32 {
        FORECAST_FIELDS as u32
    }
}

impl Game {
    fn weather_spot(&self) -> IVec3 {
        let e = self.render_eye;
        IVec3::new(e.x.floor() as i32, e.y.floor() as i32, e.z.floor() as i32)
    }
}
