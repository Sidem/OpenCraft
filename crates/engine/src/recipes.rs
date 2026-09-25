//! Recipes as data tables: hand crafting for buildings ([`RECIPES`], listed in order by the build
//! menu), what machines make ([`MACHINE_RECIPES`]) and what burns as fuel ([`FUELS`]).
//! To add a recipe: add a row. Machine recipes are saved by index, so append those, never reorder.

use crate::block::*;
use crate::inventory::Inventory;
use crate::item::{ItemId, COPPER_INGOT, COPPER_WIRE, GREEN_PACK, IRON_INGOT, IRON_PLATE, IRON_ROD, RED_PACK, SCREW};

pub struct Recipe {
    pub output: ItemId,
    pub count: u32,
    pub inputs: &'static [(ItemId, u32)],
    /// One line for the build menu.
    pub blurb: &'static str,
}

impl Recipe {
    /// How many times `inv` can pay for this recipe (inputs are distinct items).
    pub fn affordable(&self, inv: &Inventory) -> u32 {
        self.inputs.iter().map(|&(item, n)| inv.count(item) / n).min().unwrap_or(0)
    }
}

pub const RECIPES: &[Recipe] = &[
    Recipe {
        output: b(MINER),
        count: 1,
        inputs: &[(b(IRON_ORE), 10), (b(COPPER_ORE), 6), (b(STONE), 12)],
        blurb: "Place it against an ore block. It drills the whole deposit, recovers 60% of what it draws, \
                and pushes ore into a belt, box or smelter beside it.",
    },
    Recipe {
        output: b(BELT),
        count: 4,
        inputs: &[(b(IRON_ORE), 1), (b(STONE), 2)],
        blurb: "Carries items the way you are facing when you place it. Belts feed into belts, machines, \
                and other belts from the side.",
    },
    Recipe {
        output: b(STORAGE),
        count: 1,
        inputs: &[(b(LOG), 6), (b(IRON_ORE), 2)],
        blurb: "Holds 24 stacks. Belts deliver into it; a belt leading away from it is fed from it. \
                Right-click to empty it into your inventory.",
    },
    Recipe {
        output: b(SMELTER),
        count: 1,
        inputs: &[(b(STONE), 16), (b(IRON_ORE), 4)],
        blurb: "Melts iron or copper ore into ingots while it has fuel: coal ore or logs. Belts bring \
                both in; a belt leading away takes the ingots. Right-click to open it.",
    },
    Recipe {
        output: b(CONSTRUCTOR),
        count: 1,
        inputs: &[(IRON_INGOT, 10), (COPPER_INGOT, 4), (b(STONE), 8)],
        blurb: "Shapes ingots into parts: plates, rods, screws and wire. Right-click to choose what it makes; \
                belts bring the ingots in and take the parts away.",
    },
    Recipe {
        output: b(SPLITTER),
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Takes items from belts leading into it and shares them between the belts leading away in \
                front, to the left and to the right.",
    },
    Recipe {
        output: b(FILTER),
        count: 1,
        inputs: &[(IRON_PLATE, 2), (COPPER_WIRE, 2), (b(BELT), 2)],
        blurb: "Sends the item you choose straight on and everything else to the left and right. \
                Right-click to choose the item.",
    },
    Recipe {
        output: b(RAMP_UP),
        count: 2,
        inputs: &[(IRON_PLATE, 1), (b(BELT), 2)],
        blurb: "A belt that climbs one level: it hands items on one block ahead and one up. Place it \
                facing uphill.",
    },
    Recipe {
        output: b(RAMP_DOWN),
        count: 2,
        inputs: &[(IRON_PLATE, 1), (b(BELT), 2)],
        blurb: "A belt that goes down one level: a belt one block up behind it feeds its high end. Place \
                it facing downhill.",
    },
    Recipe {
        output: b(LIFT),
        count: 2,
        inputs: &[(IRON_ROD, 2), (b(BELT), 2)],
        blurb: "Carries items straight up. Stack lifts facing the same way to climb higher; the top one \
                hands items on one block ahead and one up.",
    },
    Recipe {
        output: b(UNDERPASS_IN),
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Takes items under whatever is in front of it to an underpass exit facing the same way, \
                up to 5 blocks ahead. Lets belts cross.",
    },
    Recipe {
        output: b(UNDERPASS_OUT),
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Where items come back up from an underpass entry behind it, then carry on like a belt.",
    },
    Recipe {
        output: b(GENERATOR),
        count: 1,
        inputs: &[(IRON_INGOT, 12), (COPPER_INGOT, 8), (b(STONE), 10)],
        blurb: "Burns coal ore or logs into 60 kW of power, only while its grid needs it. Place a power pole \
                within 5 blocks; belts bring fuel in. Right-click to open it.",
    },
    Recipe {
        output: b(POLE),
        count: 2,
        inputs: &[(IRON_INGOT, 1), (COPPER_INGOT, 1), (b(LOG), 1)],
        blurb: "Links to every pole within 10 blocks and powers generators and machines within 5. \
                Constructors, splitters, filters and labs need power.",
    },
    Recipe {
        output: b(LAB),
        count: 1,
        inputs: &[(IRON_PLATE, 6), (COPPER_WIRE, 8), (b(BELT), 4)],
        blurb: "Uses science packs to research new machines (press R to choose what). Belts bring packs in; \
                needs power.",
    },
    Recipe {
        output: RED_PACK,
        count: 1,
        inputs: &[(IRON_PLATE, 1), (COPPER_WIRE, 2)],
        blurb: "A lab uses these to research the first techs.",
    },
    Recipe {
        output: GREEN_PACK,
        count: 1,
        inputs: &[(b(BELT), 2), (SCREW, 4)],
        blurb: "A lab uses these, with red packs, for later techs.",
    },
];

/// Something a machine makes: `inputs` are used up when a batch starts, `output` appears after
/// `seconds` of work.
pub struct MachineRecipe {
    /// The machine's block.
    pub machine: BlockId,
    pub inputs: &'static [(ItemId, u32)],
    pub output: (ItemId, u32),
    pub seconds: f64,
}

pub const MACHINE_RECIPES: &[MachineRecipe] = &[
    MachineRecipe { machine: SMELTER, inputs: &[(b(IRON_ORE), 1)], output: (IRON_INGOT, 1), seconds: 1.5 },
    MachineRecipe { machine: SMELTER, inputs: &[(b(COPPER_ORE), 1)], output: (COPPER_INGOT, 1), seconds: 1.5 },
    MachineRecipe { machine: CONSTRUCTOR, inputs: &[(IRON_INGOT, 2)], output: (IRON_PLATE, 1), seconds: 2.0 },
    MachineRecipe { machine: CONSTRUCTOR, inputs: &[(IRON_INGOT, 1)], output: (IRON_ROD, 1), seconds: 2.0 },
    MachineRecipe { machine: CONSTRUCTOR, inputs: &[(IRON_ROD, 1)], output: (SCREW, 4), seconds: 3.0 },
    MachineRecipe { machine: CONSTRUCTOR, inputs: &[(COPPER_INGOT, 1)], output: (COPPER_WIRE, 2), seconds: 2.0 },
];

/// The recipe `i` if `machine` makes it.
pub fn machine_recipe(machine: BlockId, i: u16) -> Option<&'static MachineRecipe> {
    MACHINE_RECIPES.get(i as usize).filter(|r| r.machine == machine)
}

/// Fuel and the seconds of machine work one item keeps a fire going.
pub const FUELS: &[(ItemId, f64)] = &[(b(COAL_ORE), 8.0), (b(LOG), 4.0)];

/// Seconds of work one `item` fuels, if it burns.
pub fn burn_time(item: ItemId) -> Option<f64> {
    FUELS.iter().find(|f| f.0 == item).map(|f| f.1)
}

/// Index of `machine`'s recipe that uses `item`, if any.
pub fn machine_recipe_using(machine: BlockId, item: ItemId) -> Option<u16> {
    let using = |r: &MachineRecipe| r.machine == machine && r.inputs.iter().any(|i| i.0 == item);
    MACHINE_RECIPES.iter().position(using).map(|i| i as u16)
}

/// The item that is block `id`, to keep the tables short.
const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}
