//! Port marks (presentation): what a multi-block machine shows where it takes and gives items, the same on the
//! placement ghost (`port_view.rs`) and, while a belt is in hand, on built machines. Each port face gets a stub one cell
//! out: a see-through belt (only where that cell is free) running the way items move, glowing arrows on it pointing that
//! way, and the item it carries riding it: an input the machine takes, its product, its byproduct. The main flow is
//! two arrows, a byproduct one; colour (the hatches' hues, `arrow_look`) only repeats what the arrows and the items
//! say. The arrows and items are solid and drawn after the ghost, so a port behind the machine shows through it. The
//! faces are the footprint's port rows, the data `takes_from` and `out_faces` give the simulation (`tests.rs` belts
//! every marked face of every machine at every turn and checks the items go where the marks say).
//!
//! With nothing in hand, built machines show the still version: a floor pad at each open port with chevrons pointing
//! the way items move and the item in a corner (`write_port_pads`); the hatches on the walls stay as sockets.
//!
//! To mark another kind of port: a `Flow` and its look in `arrow_look`.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::block::{tex, BlockId};
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::recipes::{MachineRecipe, MACHINE_RECIPES};

use super::belt::belt_preview;
use super::belt_shape::Shape;
use super::footprint::{self, Role};
use super::process::{self, Pick, Processor};
use super::render::{mark_beacon_tinted, push_box};
use super::{opposite, Factory, DIRS};

/// What a port does with items.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Flow {
    In = 0,
    Out = 1,
    Byproduct = 2,
}

/// Built machines within this many blocks of the camera show their marks while a belt is in hand.
pub const NEAR: f64 = 16.0;
/// Arrows glide this many cells a second; an item rides a stub in this many seconds.
const ARROW_SPEED: f64 = 0.8;
const RIDE_SECONDS: f64 = 1.4;
/// An arrow's arm: length, width, thickness; how high over the cell's floor the arrows and items ride.
const ARM: [f32; 3] = [0.09, 0.07, 0.36];
const RIDE_Y: f64 = 0.3;
/// A riding item's size (a full cube's width).
const ITEM_UNIT: f64 = 0.34;
/// Built machines within this many blocks show a pad at each open port.
pub const PAD_NEAR: f64 = 32.0;
/// A port pad: across, high, deep (from the machine's face to the far edge of its cell).
const PAD: [f64; 3] = [0.74, 0.04, 0.96];
/// The size of the item on a pad (a full cube's width).
const PAD_ITEM: f64 = 0.24;

/// The items a machine's ports show: one of the inputs it takes, its product, its byproduct.
#[derive(Default, Clone)]
pub struct Shown {
    pub inputs: Vec<ItemId>,
    pub product: Option<ItemId>,
    pub byproduct: Option<ItemId>,
}

/// Every face of the machine `block` anchored at `anchor`, placed facing `dir`, that takes or gives items: its cell, the
/// `DIRS` index it looks out of, and what it does. Empty for a machine of one cell (it takes and gives on every side).
pub fn port_faces(block: BlockId, anchor: IVec3, dir: u8) -> Vec<(IVec3, u8, Flow)> {
    let Some(fp) = process::spec(block).map(|s| &s.footprint).filter(|f| !f.is_single()) else { return Vec::new() };
    let roles = [(Role::In, Flow::In), (Role::Out, Flow::Out), (Role::Side, Flow::Byproduct)];
    roles
        .iter()
        .flat_map(|&(role, flow)| fp.faces(anchor, dir, role).into_iter().map(move |(c, s)| (c, s, flow)))
        .collect()
}

/// Draws the machine `block` as it would stand at `anchor` facing `dir` (camera-relative through `eye`): its model into
/// `ghost` (drawn see-through), its port marks into `solid` and `ghost`, with an example recipe's items. `free` says
/// whether a cell is open for a stub's belt.
#[allow(clippy::too_many_arguments)]
pub fn draw_ghost(
    block: BlockId,
    anchor: IVec3,
    dir: u8,
    eye: Vec3,
    time: f64,
    free: impl Fn(IVec3) -> bool,
    solid: &mut Vec<f32>,
    ghost: &mut Vec<f32>,
) {
    let Some(spec) = process::spec(block) else { return };
    let mut p = Processor::new(anchor, spec, 0);
    p.dir = footprint::layout_dir(block, dir);
    process::draw_model(&p, ghost, centre(anchor) - eye, time);
    let shown = example(spec);
    marks(block, anchor, p.dir, &shown, eye, time, &free, solid, ghost);
}

impl Factory {
    /// Port marks on every built machine with ports within [`NEAR`] of `eye`, with the items of its recipe.
    pub fn write_port_marks(
        &self,
        eye: Vec3,
        time: f64,
        free: impl Fn(IVec3) -> bool,
        solid: &mut Vec<f32>,
        ghost: &mut Vec<f32>,
    ) {
        for p in self.processors.iter().filter(|p| !p.footprint().is_single()) {
            if (centre(p.pos) - eye).length() > NEAR + 4.0 {
                continue;
            }
            marks(p.spec.block, p.pos, p.dir, &shown_by(p), eye, time, &free, solid, ghost);
        }
    }

    /// The floor pad at every open port (`free` in front of it) of the built machines within [`PAD_NEAR`] of `eye`:
    /// what a machine says about its ports at a glance, with nothing in hand. A low plate in the cell a belt would
    /// take, chevrons down its middle pointing the way items move (two for the main flow, one for a byproduct) and the
    /// port's item, small, in an outer corner. A port with something built in front of it shows none: the belt there
    /// says the rest.
    pub fn write_port_pads(&self, eye: Vec3, free: impl Fn(IVec3) -> bool, out: &mut Vec<f32>) {
        for p in self.processors.iter().filter(|p| !p.footprint().is_single()) {
            if (centre(p.pos) - eye).length() > PAD_NEAR + 4.0 {
                continue;
            }
            let (shown, mut next_input) = (shown_by(p), 0);
            for (cell, side, flow) in port_faces(p.spec.block, p.pos, p.dir) {
                let item = item_at(&shown, flow, &mut next_input);
                let stub = cell + DIRS[side as usize];
                if free(stub) {
                    pad(out, centre(stub) - eye, side, flow, item);
                }
            }
        }
    }
}

/// Draws a port pad in the cell centred at camera-relative `rel`, in front of a face looking out along `DIRS[side]`.
fn pad(out: &mut Vec<f32>, rel: Vec3, side: u8, flow: Flow, item: Option<ItemId>) {
    let away = DIRS[side as usize].as_vec3();
    let floor = rel - Vec3::new(0.0, 0.5, 0.0);
    // A box's own +z looks along (-sin yaw, 0, cos yaw): this yaw points it away from the machine.
    let yaw = ((side + 2) % 4) as f32 * FRAC_PI_2;
    push_box(out, floor + Vec3::new(0.0, PAD[1] / 2.0, 0.0), yaw, PAD.map(|a| a as f32), 0.0, [tex::FRAME; 3], false);
    // Measured out from the cell's centre: the machine's face at -0.5, the item near the far edge.
    let along = if flow == Flow::In { opposite(side) } else { side };
    let top = floor + Vec3::new(0.0, PAD[1] + f64::from(ARM[1]) / 2.0, 0.0);
    let tips: &[f64] = if flow == Flow::Byproduct { &[-0.12] } else { &[-0.28, -0.02] };
    for &u in tips {
        // An inward chevron's tip is its end nearer the machine, an outward one's the farther.
        let tip = if flow == Flow::In { u - 0.1 } else { u + 0.1 };
        chevron(out, top + away * tip, along, if flow == Flow::Byproduct { 0.9 } else { 1.0 }, arrow_look(flow));
    }
    // The item in an outer corner, small, so seen from in front it never hides the chevrons.
    if let Some(item) = item {
        let corner = away * 0.3 + Vec3::new(-away.z, 0.0, away.x) * 0.2;
        item_model(out, item, floor + corner + Vec3::new(0.0, PAD[1] + PAD_ITEM / 2.0, 0.0), PAD_ITEM);
    }
}

/// What a built machine's ports show: the items of the recipe it makes (chosen, else its last batch), else an example.
fn shown_by(p: &Processor) -> Shown {
    let made = if p.spec.pick == Pick::Chosen { p.recipe } else { p.shown.or(p.batch) };
    recipe_items(made.and_then(|i| p.spec.recipe(i)).or_else(|| first_recipe(p.spec)))
}

/// The item a port of `flow` shows: the inputs taken in turn over a machine's inlets (`next_input` counts them).
fn item_at(shown: &Shown, flow: Flow, next_input: &mut usize) -> Option<ItemId> {
    match flow {
        Flow::In => {
            *next_input += 1;
            shown.inputs.get((*next_input - 1) % shown.inputs.len().max(1)).copied()
        }
        Flow::Out => shown.product,
        Flow::Byproduct => shown.byproduct,
    }
}

/// The items an example recipe of `spec` (its first) shows.
fn example(spec: &'static process::ProcessSpec) -> Shown {
    recipe_items(first_recipe(spec))
}

fn first_recipe(spec: &'static process::ProcessSpec) -> Option<&'static MachineRecipe> {
    (spec.pick != Pick::Store).then(|| MACHINE_RECIPES.iter().find(|r| spec.categories.contains(&r.category))).flatten()
}

fn recipe_items(r: Option<&'static MachineRecipe>) -> Shown {
    let Some(r) = r else { return Shown::default() };
    Shown {
        inputs: r.inputs.iter().map(|i| i.0).collect(),
        product: r.outputs.first().map(|o| o.0),
        byproduct: r.outputs.get(1).map(|o| o.0),
    }
}

#[allow(clippy::too_many_arguments)]
fn marks(
    block: BlockId,
    anchor: IVec3,
    dir: u8,
    shown: &Shown,
    eye: Vec3,
    time: f64,
    free: &impl Fn(IVec3) -> bool,
    solid: &mut Vec<f32>,
    ghost: &mut Vec<f32>,
) {
    let mut next_input = 0;
    for (cell, side, flow) in port_faces(block, anchor, dir) {
        let stub = cell + DIRS[side as usize];
        // Items move into the machine at an inlet, out of it at an outlet.
        let along = if flow == Flow::In { opposite(side) } else { side };
        let item = item_at(shown, flow, &mut next_input);
        let rel = centre(stub) - eye;
        if free(stub) {
            belt_preview(ghost, stub, along, Shape::Flat, 0, rel, time);
        }
        arrows(solid, rel, along, flow, time);
        if let Some(item) = item {
            ride(solid, item, rel, along, time);
        }
    }
}

/// The arrows on a stub at camera-relative `rel` pointing along `DIRS[along]`: two gliding, or one small for a byproduct.
fn arrows(out: &mut Vec<f32>, rel: Vec3, along: u8, flow: Flow, time: f64) {
    let (count, scale) = if flow == Flow::Byproduct { (1, 0.9) } else { (2, 1.0) };
    let ahead = DIRS[along as usize].as_vec3();
    for k in 0..count {
        let shift = ((time * ARROW_SPEED + k as f64 / count as f64).fract() - 0.5) * 0.7;
        let tip = rel + ahead * (shift + 0.15) + Vec3::new(0.0, RIDE_Y - 0.5, 0.0);
        chevron(out, tip, along, scale, arrow_look(flow));
    }
}

/// One flat chevron with its tip at camera-relative `tip`, pointing along `DIRS[along]`, in colour `look`.
fn chevron(out: &mut Vec<f32>, tip: Vec3, along: u8, scale: f32, look: u32) {
    let yaw = ((along + 2) % 4) as f32 * FRAC_PI_2;
    for hand in [-1.0f32, 1.0] {
        // Each arm runs back from the tip at 45° to one side: its centre half its length back along it.
        let turn = yaw + hand * FRAC_PI_4;
        let half = f64::from(ARM[2] * scale) / 2.0;
        // A box's own +z looks along (-sin yaw, 0, cos yaw) (`process/model.rs` `local`).
        let back = Vec3::new(turn.sin() as f64, 0.0, -(turn.cos() as f64)) * half;
        push_box(out, tip + back, turn, ARM.map(|a| a * scale), 0.0, [tex::FRAME; 3], false);
        mark_beacon_tinted(out, look); // one flat colour that lights itself, day or night
    }
}

/// The colour of a port's arrows, the hues of its hatch (`tex::PORT_*`): blue in, orange out, purple byproduct.
fn arrow_look(flow: Flow) -> u32 {
    match flow {
        Flow::In => 0x2e9bff,
        Flow::Out => 0xff8c1a,
        Flow::Byproduct => 0xc070ff,
    }
}

/// `item` riding a stub at camera-relative `rel` along `DIRS[along]`, crossing it every `RIDE_SECONDS`.
fn ride(out: &mut Vec<f32>, item: ItemId, rel: Vec3, along: u8, time: f64) {
    let share = (time / RIDE_SECONDS).fract() - 0.5;
    let at = rel + DIRS[along as usize].as_vec3() * share + Vec3::new(0.0, RIDE_Y - 0.5 + ITEM_UNIT / 2.0, 0.0);
    item_model(out, item, at, ITEM_UNIT);
}

/// `item`'s model, `unit` big (a full cube's width), centred at camera-relative `at`.
fn item_model(out: &mut Vec<f32>, item: ItemId, at: Vec3, unit: f64) {
    if !crate::entities::push_item_model(out, item, at, 0.0, unit) {
        if let Some(def) = crate::item::def(item) {
            push_box(out, at, 0.0, def.size.map(|s| s * unit as f32), 0.0, def.tex, false);
        }
    }
}

/// The centre of cell `c`.
fn centre(c: IVec3) -> Vec3 {
    c.as_vec3() + Vec3::new(0.5, 0.5, 0.5)
}

#[cfg(test)]
mod tests;
