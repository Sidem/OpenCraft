//! Constructor: makes one input into parts with the recipe chosen in its panel (the `SetRecipe`
//! action; `MACHINE_RECIPES` rows for `CONSTRUCTOR`). Two buffers (slots from `MACHINES`): `input`
//! and `out`. Belts, miners and the panel deliver the recipe's input into it; anything else is
//! refused, and with no recipe it takes nothing. Like a box, it pushes one part a tick into the next
//! belt leading away.
//!
//! Invariants: a batch uses up its input when it starts and only starts when its output fits; work
//! is counted in whole ticks. Changing the recipe hands back the input buffer and an unfinished
//! batch's input. Unpowered for now; power (step 2.7) will scale its speed.

use crate::block::{tex, CONSTRUCTOR};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};
use crate::recipes::{machine_recipe, MachineRecipe};

use super::belt::Belt;
use super::buffer::Buffer;
use super::panel::{Panel, ROLE_INPUT, ROLE_OUTPUT};
use super::render::push_box;
use super::{ticks, Factory, Kind, Machine};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConstructorStatus {
    NoRecipe,
    Working,
    NoInput,
    OutputFull,
}

/// Every status, in declaration order: saves store `status as u8`.
const STATUSES: [ConstructorStatus; 4] = [
    ConstructorStatus::NoRecipe,
    ConstructorStatus::Working,
    ConstructorStatus::NoInput,
    ConstructorStatus::OutputFull,
];

pub struct Constructor {
    pub pos: IVec3,
    /// The chosen recipe (index into `MACHINE_RECIPES`).
    pub recipe: Option<u16>,
    pub input: Buffer,
    pub out: Buffer,
    /// Whether a batch is in progress, and the ticks of work it has had.
    pub busy: bool,
    pub progress: u32,
    /// Belt indices leading away from it.
    pub outs: Vec<u32>,
    pub next_out: usize,
    pub status: ConstructorStatus,
}

impl Constructor {
    pub fn new(pos: IVec3) -> Constructor {
        let slots = Kind::Constructor.def().slots;
        Constructor {
            pos,
            recipe: None,
            input: Buffer::new(slots),
            out: Buffer::new(slots),
            busy: false,
            progress: 0,
            outs: Vec::new(),
            next_out: 0,
            status: ConstructorStatus::NoRecipe,
        }
    }

    /// How many of `item` it would take now: only its recipe's input, up to the buffer's room.
    pub fn room_for(&self, item: ItemId) -> u32 {
        match self.recipe_def() {
            Some(r) if r.inputs.iter().any(|i| i.0 == item) => self.input.space_for(item),
            _ => 0,
        }
    }

    /// Takes one `item` into its input buffer; false if it isn't wanted or doesn't fit.
    pub fn accept(&mut self, item: ItemId) -> bool {
        self.room_for(item) > 0 && self.input.add(item, 1) == 0
    }

    /// Switches to `recipe` (which must be a constructor recipe, or `None`) and returns the inputs it
    /// held, including an unfinished batch's.
    pub fn set_recipe(&mut self, recipe: Option<u16>) -> Vec<Stack> {
        let mut back = self.input.contents();
        if let Some(r) = self.recipe_def().filter(|_| self.busy) {
            back.extend(r.inputs.iter().map(|&(item, count)| Stack { item, count }));
        }
        self.input = Buffer::new(self.input.slots.len());
        (self.recipe, self.busy, self.progress) = (recipe, false, 0);
        self.status = if recipe.is_some() { ConstructorStatus::NoInput } else { ConstructorStatus::NoRecipe };
        back
    }

    /// One tick: work on the batch (starting one if it can), then push a part out.
    pub fn step(&mut self, belts: &mut [Belt]) {
        self.work();
        self.out.feed(&self.outs, &mut self.next_out, belts);
    }

    fn work(&mut self) {
        let Some(r) = self.recipe_def() else {
            self.status = ConstructorStatus::NoRecipe;
            return;
        };
        if !self.busy {
            let (item, need) = r.inputs[0];
            let s = self.input.slots[0];
            if s.is_empty() || s.item != item || s.count < need {
                self.status = ConstructorStatus::NoInput;
                return;
            }
            if self.out.space_for(r.output.0) < r.output.1 {
                self.status = ConstructorStatus::OutputFull;
                return;
            }
            self.input.take(0, need);
            (self.busy, self.progress) = (true, 0);
        }
        self.status = ConstructorStatus::Working;
        self.progress += 1;
        if self.progress >= ticks(r.seconds) {
            self.out.add(r.output.0, r.output.1);
            (self.busy, self.progress) = (false, 0);
        }
    }

    fn recipe_def(&self) -> Option<&'static MachineRecipe> {
        machine_recipe(CONSTRUCTOR, self.recipe?)
    }

    /// The first readout line, also the panel's status.
    pub fn status_text(&self) -> String {
        match (self.status, self.recipe_def()) {
            (_, None) => "No recipe: choose what it makes".to_string(),
            (ConstructorStatus::Working, Some(r)) => format!(
                "Making {} · {} a minute",
                item::name(r.output.0),
                (r.output.1 as f64 * 60.0 / r.seconds).round() as u32
            ),
            (ConstructorStatus::OutputFull, _) => "Output full: put a belt leading away, or take the parts".to_string(),
            (_, Some(r)) => format!("Waiting for {} {}", r.inputs[0].1, item::name(r.inputs[0].0)),
        }
    }

    pub fn panel(&self) -> Panel {
        let total = self.recipe_def().map_or(1, |r| ticks(r.seconds));
        Panel {
            block: CONSTRUCTOR,
            recipe: self.recipe,
            choosable: true,
            progress: if self.busy { self.progress * 1000 / total } else { 0 },
            fire: 0,
            slots: vec![(ROLE_INPUT, self.input.slots[0]), (ROLE_OUTPUT, self.out.slots[0])],
            status: self.status_text(),
            filter: None,
        }
    }
}

impl Machine for Constructor {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: recipe, buffers, batch, round-robin position and status (`outs` comes from `relink`).
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.bool(self.recipe.is_some());
        w.u16(self.recipe.unwrap_or(0));
        self.input.write_state(w);
        self.out.write_state(w);
        w.bool(self.busy);
        w.u32(self.progress);
        w.u32(self.next_out as u32);
        w.u8(self.status as u8);
    }

    fn read_state(r: &mut ByteReader) -> Option<Constructor> {
        let mut c = Constructor::new(r.ivec3()?);
        let (chosen, recipe) = (r.bool()?, r.u16()?);
        c.recipe = chosen.then_some(recipe);
        let slots = c.input.slots.len();
        c.input = Buffer::read_state(r, slots)?;
        c.out = Buffer::read_state(r, slots)?;
        c.busy = r.bool()?;
        c.progress = r.u32()?;
        c.next_out = r.u32()? as usize;
        c.status = *STATUSES.get(r.u8()? as usize)?;
        let valid = c.recipe.is_none_or(|i| machine_recipe(CONSTRUCTOR, i).is_some());
        valid.then_some(c)
    }

    /// Its buffers, plus the input of an unfinished batch (given back rather than lost).
    fn contents(&self) -> Vec<Stack> {
        let mut all = self.input.contents();
        all.extend(self.out.contents());
        if let Some(r) = self.recipe_def().filter(|_| self.busy) {
            all.extend(r.inputs.iter().map(|&(item, count)| Stack { item, count }));
        }
        all
    }

    fn describe(&self, _: &Factory) -> String {
        let mut lines = vec![self.status_text()];
        let held =
            |label: &str, s: Stack| (!s.is_empty()).then(|| format!("{label} {} {}", s.count, item::name(s.item)));
        let parts: Vec<String> =
            [held("In:", self.input.slots[0]), held("Out:", self.out.slots[0])].into_iter().flatten().collect();
        if !parts.is_empty() {
            lines.push(parts.join(" · "));
        }
        lines.push("Right-click to open".to_string());
        lines.join("\n")
    }

    /// A teal press: body, a press head that pumps while working, and a status lamp on top.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        let body = [tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE, tex::FRAME];
        push_box(out, rel + Vec3::new(0.0, -0.15, 0.0), 0.0, [0.92, 0.7, 0.92], 0.0, body, false);
        let stroke = if self.status == ConstructorStatus::Working { (time * 6.0).sin().abs() * 0.08 } else { 0.0 };
        push_box(out, rel + Vec3::new(0.0, 0.28 - stroke, 0.0), 0.0, [0.5, 0.16, 0.5], 0.0, [tex::FRAME; 3], true);
        let lamp = match self.status {
            ConstructorStatus::Working => tex::LAMP_GREEN,
            ConstructorStatus::OutputFull => tex::LAMP_YELLOW,
            ConstructorStatus::NoRecipe => tex::LAMP_RED,
            ConstructorStatus::NoInput => tex::FRAME,
        };
        push_box(out, rel + Vec3::new(0.3, 0.42, 0.3), 0.0, [0.14, 0.1, 0.14], 0.0, [lamp; 3], false);
    }
}
