//! Coal generator: burns fuel (`recipes::FUELS`) to supply its grid with `GENERATOR_POWER`. Belts,
//! miners and the panel bring fuel into its one buffer; `Power::balance` lights and burns it, and only
//! while the grid needs power. Right-click opens its panel.
//!
//! Invariants: the fire is counted in whole ticks; `running` is last tick's result (for the view,
//! not saved).

use crate::block::{tex, GENERATOR};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};
use crate::recipes::burn_time;
use crate::TICK_RATE;

use super::buffer::Buffer;
use super::panel::{Panel, ROLE_FUEL};
use super::power::GENERATOR_POWER;
use super::render::push_box;
use super::{Factory, Kind, Machine};

pub struct Generator {
    pub pos: IVec3,
    pub fuel: Buffer,
    /// Ticks left in the fire.
    pub burn: u32,
    /// Whether it supplied power last tick (derived).
    pub running: bool,
}

impl Generator {
    pub fn new(pos: IVec3) -> Generator {
        Generator { pos, fuel: Buffer::new(Kind::Generator.def().slots), burn: 0, running: false }
    }

    /// How many of `item` it would take now: fuel only, up to the buffer's room.
    pub fn room_for(&self, item: ItemId) -> u32 {
        if burn_time(item).is_some() {
            self.fuel.space_for(item)
        } else {
            0
        }
    }

    pub fn accept(&mut self, item: ItemId) -> bool {
        self.room_for(item) > 0 && self.fuel.add(item, 1) == 0
    }

    /// What it is doing (needs the factory for its grid).
    pub fn status_text(&self, f: &Factory) -> String {
        let pole = f.generators.iter().position(|g| g.pos == self.pos).and_then(|i| f.power.gen_pole.get(i).copied());
        match pole.flatten() {
            _ if f.dirty => String::new(),
            None => f.power.grid_line(None),
            Some(_) if self.running => format!("Supplying {GENERATOR_POWER} kW"),
            Some(_) if self.fuel.total() == 0 && self.burn == 0 => "Out of fuel: bring coal ore or logs".to_string(),
            Some(_) => "Idle: nothing on its grid needs power".to_string(),
        }
    }

    /// The panel: its fuel and fire (the status line needs the factory, so the caller adds it).
    pub fn panel(&self, status: String) -> Panel {
        Panel {
            block: GENERATOR,
            recipe: None,
            choosable: false,
            progress: 0,
            fire: self.burn.div_ceil(TICK_RATE),
            slots: vec![(ROLE_FUEL, self.fuel.slots[0])],
            status,
            filter: None,
        }
    }
}

impl Machine for Generator {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: fuel and fire.
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        self.fuel.write_state(w);
        w.u32(self.burn);
    }

    fn read_state(r: &mut ByteReader) -> Option<Generator> {
        let mut g = Generator::new(r.ivec3()?);
        g.fuel = Buffer::read_state(r, g.fuel.slots.len())?;
        g.burn = r.u32()?;
        Some(g)
    }

    fn contents(&self) -> Vec<Stack> {
        self.fuel.contents()
    }

    fn describe(&self, f: &Factory) -> String {
        let mut lines = vec![self.status_text(f)];
        let s = self.fuel.slots[0];
        let mut parts = Vec::new();
        if !s.is_empty() {
            parts.push(format!("Fuel: {} {}", s.count, item::name(s.item)));
        }
        if self.burn > 0 {
            parts.push(format!("fire for {} s", self.burn.div_ceil(TICK_RATE)));
        }
        if !parts.is_empty() {
            lines.push(parts.join(" · "));
        }
        lines.push("Right-click to open".to_string());
        lines.join("\n")
    }

    /// A steel housing with a spinning-looking fan cap and a status lamp.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let body = [tex::GENERATOR_TOP, tex::GENERATOR_SIDE, tex::FRAME];
        push_box(out, rel + Vec3::new(0.0, -0.1, 0.0), 0.0, [0.92, 0.8, 0.92], 0.0, body, false);
        let spin = if self.running { (time * 8.0) as f32 } else { 0.0 };
        push_box(out, rel + Vec3::new(0.0, 0.33, 0.0), spin, [0.5, 0.06, 0.12], 0.0, [tex::FRAME; 3], true);
        let lamp = match (self.running, self.fuel.total() > 0 || self.burn > 0) {
            (true, _) => tex::LAMP_GREEN,
            (false, true) => tex::FRAME,
            (false, false) => tex::LAMP_RED,
        };
        push_box(out, rel + Vec3::new(0.32, 0.34, 0.32), 0.0, [0.14, 0.1, 0.14], 0.0, [lamp; 3], false);
    }
}
