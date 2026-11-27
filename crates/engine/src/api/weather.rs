//! The weather the host draws and the calendar forecasts (`weather/`): queries only, at the camera and the core tick.
//! The forecast runs a copy of the air ahead, kept and extended (`ForecastMemo`) while the camera stays near.

use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use crate::math::IVec3;
use crate::weather::ice::{self, IceTable};
use crate::weather::{self, Ahead, Kind, Outlook, Reading, FORECAST_FIELDS};
use crate::worldgen::SEA_LEVEL;
use crate::{Game, TICK_RATE};

/// Days a forecast or a `weather_ahead` search looks ahead at most.
const MAX_FORECAST_DAYS: u32 = 16;
const MAX_AHEAD_DAYS: u64 = 60;
/// Blocks the camera may move before the remembered forecast is for somewhere else.
const MEMO_SPAN: i32 = 64;

/// The camera's outlook (`weather::Outlook`), kept while the camera stays within `MEMO_SPAN`: it stays true as time
/// passes, so the calendar's forecast costs a few steps of the air an in-game hour, not six days of them each time.
#[derive(Default)]
pub(crate) struct ForecastMemo(RefCell<Option<Outlook>>);

#[wasm_bindgen]
impl Game {
    /// The weather at the camera now, as a weather station reads it: cloud cover, gloom and precipitation (0 to 1), 1 if
    /// what falls is snow (below 0 °C), temperature (°C), the sky's kind (as in `weather_forecast`), relative humidity
    /// (%), dew point (°C), sea-level pressure (hPa), wind speed (m/s) and the direction it blows from (degrees, 0 north).
    pub fn weather_here(&self) -> Vec<f32> {
        let r = self.reading_here();
        let unit = |v: u32| v as f32 / 1000.0;
        let hundredths = |v: i32| v as f32 / 100.0;
        vec![
            unit(r.sky.cover),
            unit(r.sky.gloom),
            unit(r.sky.precip),
            f32::from(u8::from(r.snow)),
            hundredths(r.temp),
            f32::from(r.sky.kind() as u8),
            r.humidity as f32 / 10.0,
            hundredths(r.dew),
            hundredths(r.pressure),
            hundredths(r.wind()),
            r.wind_from() as f32,
        ]
    }

    /// `days` days ahead at the camera, today first, `forecast_fields` numbers each: the worst sky (0 clear, 1 cloudy,
    /// 2 rain, 3 storm), 1 if it is snow, and the low and high temperatures in tenths of a degree.
    pub fn weather_forecast(&self, days: u32) -> Vec<i32> {
        let (p, gen, air) = (self.weather_spot(), self.sim.world.generator(), &self.sim.atmosphere);
        let span = |v: IVec3| (v.x.div_euclid(MEMO_SPAN), v.y.div_euclid(MEMO_SPAN), v.z.div_euclid(MEMO_SPAN));
        let mut memo = self.forecast_memo.0.borrow_mut();
        let outlook = match memo.as_mut() {
            Some(o) if span(o.pos) == span(p) => o,
            _ => memo.insert(Outlook::new(air, p, self.sim.tick)),
        };
        let now = Reading::at(air, gen, p.x, p.y, p.z);
        outlook.days(gen, now, self.sim.tick, days.min(MAX_FORECAST_DAYS))
    }
    /// Numbers per day in `weather_forecast`.
    pub fn forecast_fields(&self) -> u32 {
        FORECAST_FIELDS as u32
    }

    /// The ice on every kind of water now (`weather::ice::IceTable`): rows lake, flowing, sea of `ice_cold_levels`
    /// levels each, 0 open water to 255, bearing a body from `ice_bears_level`. The water shader looks a surface up by
    /// the cold index and kind its vertices carry (`mesher/water.rs`).
    pub fn ice_table(&self) -> Vec<u8> {
        IceTable::at(self.sim.tick).bytes().to_vec()
    }
    pub fn ice_cold_levels(&self) -> u32 {
        ice::COLD_LEVELS as u32
    }
    pub fn ice_bears_level(&self) -> u32 {
        u32::from(ice::BEARS_LEVEL)
    }
    /// Still water at or below this height is sea (ice's third row).
    pub fn sea_level(&self) -> i32 {
        SEA_LEVEL
    }

    /// Seconds of game time until the camera's sky next turns to `want` (0 clear, 1 cloudy, 2 rain or storm, 3 storm,
    /// 4 snow), the air run ahead and read every in-game hour for up to `MAX_AHEAD_DAYS`; -1 when it does not. For the
    /// console's `weather wait`.
    pub fn weather_ahead(&self, want: u32) -> f64 {
        let (p, now) = (self.weather_spot(), self.sim.tick);
        let (mut ahead, gen) = (Ahead::new(&self.sim.atmosphere, now), self.sim.world.generator());
        for _ in 0..MAX_AHEAD_DAYS * 24 {
            let r = ahead.hour(gen, p);
            let (kind, wet) = (r.sky.kind(), r.sky.precip >= weather::RAIN_FROM);
            let found = match want {
                0 => kind == Kind::Clear,
                1 => kind == Kind::Cloudy,
                2 => kind >= Kind::Rain && !r.snow,
                3 => kind == Kind::Storm,
                _ => wet && r.snow,
            };
            if found {
                return (ahead.tick - now) as f64 / f64::from(TICK_RATE);
            }
        }
        -1.0
    }
}

impl Game {
    fn weather_spot(&self) -> IVec3 {
        let e = self.render_eye;
        IVec3::new(e.x.floor() as i32, e.y.floor() as i32, e.z.floor() as i32)
    }

    fn reading_here(&self) -> Reading {
        let p = self.weather_spot();
        Reading::at(&self.sim.atmosphere, self.sim.world.generator(), p.x, p.y, p.z)
    }
}
