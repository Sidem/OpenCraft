//! Recipes as data tables: hand crafting for buildings ([`RECIPES`], listed in order by the build
//! menu), what machines make ([`MACHINE_RECIPES`]) and what burns as fuel ([`FUELS`]).
//! To add a recipe: add a row (its `group` is its build-menu section). Machine recipes are saved by
//! index, so append those, never reorder.

use crate::block::*;
use crate::inventory::Inventory;
use crate::item::{
    ItemId, COPPER_INGOT, COPPER_WIRE, CORE_DRILL, GREEN_PACK, IRON_AXE, IRON_INGOT, IRON_PICKAXE, IRON_PLATE,
    IRON_ROD, IRON_SHOVEL, RED_PACK, SCANNER, SCREW, STICK, STONE_AXE, STONE_PICKAXE, STONE_SHOVEL,
};
use crate::tools::{IRON_TIER, STONE_TIER};

/// The build menu's sections, in the order it shows them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Materials,
    Production,
    Logistics,
    Power,
    Science,
    Tools,
    Building,
}

pub const GROUPS: [Group; 7] = [
    Group::Materials,
    Group::Production,
    Group::Logistics,
    Group::Power,
    Group::Science,
    Group::Tools,
    Group::Building,
];

impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Group::Materials => "Materials",
            Group::Production => "Production",
            Group::Logistics => "Logistics",
            Group::Power => "Power",
            Group::Science => "Science",
            Group::Tools => "Tools",
            Group::Building => "Building",
        }
    }
}

pub struct Recipe {
    pub output: ItemId,
    /// Its section in the build menu.
    pub group: Group,
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
        group: Group::Production,
        count: 1,
        inputs: &[(b(IRON_ORE), 10), (b(COPPER_ORE), 6), (b(STONE), 12)],
        blurb: "Place it against an ore block. It drills the whole deposit, recovers 60% of what it draws, \
                and pushes ore into a belt, box, smelter or generator beside it. Needs power (5 kW).",
    },
    Recipe {
        output: b(BELT),
        group: Group::Logistics,
        count: 4,
        inputs: &[(b(IRON_ORE), 1), (b(STONE), 2)],
        blurb: "Carries items. Hold right-click and drag to lay a line; it climbs and drops one-block steps \
                by itself. Belts feed belts, machines, and belts from the side. Press R on a belt to turn it.",
    },
    Recipe {
        output: b(STORAGE),
        group: Group::Logistics,
        count: 1,
        inputs: &[(b(PLANKS), 8), (b(IRON_ORE), 2)],
        blurb: "Holds 24 stacks. Belts deliver into it; a belt leading away from it is fed from it. \
                Right-click to empty it into your inventory.",
    },
    Recipe {
        output: b(SMELTER),
        group: Group::Production,
        count: 1,
        inputs: &[(b(STONE), 16), (b(IRON_ORE), 4)],
        blurb: "Melts iron or copper ore into ingots while it has fuel: coal ore or logs. Belts bring \
                both in; a belt leading away takes the ingots. Right-click to open it.",
    },
    Recipe {
        output: b(CONSTRUCTOR),
        group: Group::Production,
        count: 1,
        inputs: &[(IRON_INGOT, 10), (COPPER_INGOT, 4), (b(STONE), 8)],
        blurb: "Shapes ingots into parts: plates, rods, screws and wire. Right-click to choose what it makes; \
                belts bring the ingots in and take the parts away.",
    },
    Recipe {
        output: b(SPLITTER),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Takes items from belts leading into it and shares them between the belts leading away in \
                front, to the left and to the right.",
    },
    Recipe {
        output: b(FILTER),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 2), (COPPER_WIRE, 2), (b(BELT), 2)],
        blurb: "Sends the item you choose straight on and everything else to the left and right. \
                Right-click to choose the item.",
    },
    Recipe {
        output: b(LIFT),
        group: Group::Logistics,
        count: 2,
        inputs: &[(IRON_ROD, 2), (b(BELT), 2)],
        blurb: "Carries items straight up. Stack lifts facing the same way to climb higher; the top one \
                hands items on one block ahead and one up. You can climb a lift stack like a ladder.",
    },
    Recipe {
        output: b(UNDERPASS_IN),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Takes items under whatever is in front of it to an underpass exit facing the same way, \
                up to 5 blocks ahead. Lets belts cross.",
    },
    Recipe {
        output: b(UNDERPASS_OUT),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Where items come back up from an underpass entry behind it, then carry on like a belt.",
    },
    Recipe {
        output: b(PUMP),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 6), (IRON_ROD, 4), (COPPER_WIRE, 6)],
        blurb: "Lifts 2 blocks of still water a second out of the water it touches, highest first. Pipe it to \
                an outlet; needs 5 kW. It can't lower the sea.",
    },
    Recipe {
        output: b(PIPE),
        group: Group::Logistics,
        count: 4,
        inputs: &[(IRON_PLATE, 2)],
        blurb: "Joins pumps to outlets. Pipes connect on every side.",
    },
    Recipe {
        output: b(OUTLET),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
        blurb: "Pours the water its pipes bring out in front of it (it faces the way you do), filling from the \
                bottom up.",
    },
    Recipe {
        output: b(GENERATOR),
        group: Group::Power,
        count: 1,
        inputs: &[(b(IRON_ORE), 6), (b(COPPER_ORE), 4), (b(STONE), 12)],
        blurb: "Burns coal ore (270 kJ) or logs (135 kJ) into up to 60 kW, and only as much as its grid uses. \
                Place a power pole within 5 blocks; a belt or a miner beside it brings fuel. Right-click to open it.",
    },
    Recipe {
        output: b(POLE),
        group: Group::Power,
        count: 2,
        inputs: &[(b(IRON_ORE), 1), (b(COPPER_ORE), 1), (STICK, 2)],
        blurb: "Links to every pole within 10 blocks and powers generators and machines within 5. \
                Miners, constructors, splitters, filters, labs, pumps and quarries need power.",
    },
    Recipe {
        output: b(LAB),
        group: Group::Science,
        count: 1,
        inputs: &[(IRON_PLATE, 6), (COPPER_WIRE, 8), (b(BELT), 4)],
        blurb: "Uses science packs to research new machines (press T to choose what). Belts bring packs in; \
                needs power.",
    },
    Recipe {
        output: RED_PACK,
        group: Group::Science,
        count: 1,
        inputs: &[(IRON_PLATE, 1), (COPPER_WIRE, 2)],
        blurb: "A lab uses these to research the first techs.",
    },
    Recipe {
        output: GREEN_PACK,
        group: Group::Science,
        count: 1,
        inputs: &[(b(BELT), 2), (SCREW, 4)],
        blurb: "A lab uses these, with red packs, for later techs.",
    },
    Recipe {
        output: b(MINER_MK2),
        group: Group::Production,
        count: 1,
        inputs: &[(b(MINER), 1), (IRON_PLATE, 8), (SCREW, 16), (COPPER_WIRE, 12)],
        blurb: "Drills twice as fast as a Mk1 and recovers 75% of what it draws, so the same deposit gives \
                more ore. Needs power.",
    },
    Recipe {
        output: b(QUARRY),
        group: Group::Production,
        count: 1,
        inputs: &[(IRON_PLATE, 12), (IRON_ROD, 8), (SCREW, 16), (COPPER_WIRE, 8)],
        blurb: "Digs the ground in front of it for real, about 2 blocks a second, into a belt or box beside it. \
                Leaves ore standing for miners. Hold it to see its box, R turns it; right-click it to choose the \
                size and depth. Needs 10 kW.",
    },
    Recipe {
        output: b(FAST_BELT),
        group: Group::Logistics,
        count: 2,
        inputs: &[(b(BELT), 2), (IRON_PLATE, 1), (SCREW, 4)],
        blurb: "Carries items twice as fast as a belt. Mixes freely with ordinary belts.",
    },
    // Tools: the count is the uses (tools.rs), so a craft makes one fresh tool.
    Recipe {
        output: STONE_PICKAXE,
        group: Group::Tools,
        count: STONE_TIER.uses,
        inputs: &[(b(STONE), 3), (STICK, 2)],
        blurb: "Hold it to break stone and ore twice as fast. Wears out after 150 blocks.",
    },
    Recipe {
        output: STONE_AXE,
        group: Group::Tools,
        count: STONE_TIER.uses,
        inputs: &[(b(STONE), 3), (STICK, 2)],
        blurb: "Hold it to chop wood twice as fast. Wears out after 150 blocks.",
    },
    Recipe {
        output: STONE_SHOVEL,
        group: Group::Tools,
        count: STONE_TIER.uses,
        inputs: &[(b(STONE), 1), (STICK, 2)],
        blurb: "Hold it to dig dirt, grass and sand twice as fast. Wears out after 150 blocks.",
    },
    Recipe {
        output: IRON_PICKAXE,
        group: Group::Tools,
        count: IRON_TIER.uses,
        inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
        blurb: "Breaks stone and ore four times as fast and keeps 4 ore per block instead of 3. Lasts 600 blocks.",
    },
    Recipe {
        output: IRON_AXE,
        group: Group::Tools,
        count: IRON_TIER.uses,
        inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
        blurb: "Chops wood four times as fast. Lasts 600 blocks.",
    },
    Recipe {
        output: IRON_SHOVEL,
        group: Group::Tools,
        count: IRON_TIER.uses,
        inputs: &[(IRON_PLATE, 1), (IRON_ROD, 2)],
        blurb: "Digs dirt, grass and sand four times as fast. Lasts 600 blocks.",
    },
    Recipe {
        output: SCANNER,
        group: Group::Tools,
        count: 1,
        inputs: &[(IRON_PLATE, 4), (COPPER_WIRE, 6), (SCREW, 4)],
        blurb: "Hold it and right-click to list the ore deposits within 48 blocks: what, how deep, which way \
                and how big. Never wears out.",
    },
    Recipe {
        output: CORE_DRILL,
        group: Group::Tools,
        count: 1,
        inputs: &[(IRON_PLATE, 6), (COPPER_WIRE, 2), (SCREW, 8)],
        blurb: "Hold right-click on the ground for 3 seconds to learn exactly how much ore lies beneath it, \
                and at what depth. Never wears out.",
    },
    Recipe {
        output: b(LAMP),
        group: Group::Building,
        count: 2,
        inputs: &[(b(GLASS), 1), (IRON_PLATE, 1), (COPPER_WIRE, 2)],
        blurb: "A block that lights up everything within about 20 blocks, day and night: caves, tunnels \
                and your factory after dark.",
    },
    Recipe {
        output: b(TORCH),
        group: Group::Building,
        count: 4,
        inputs: &[(STICK, 1), (b(COAL_ORE), 1)],
        blurb: "A small light for the first nights and caves: bright up close, about 5 blocks around. Stands \
                on top of any solid block.",
    },
    Recipe {
        output: b(PLANKS),
        group: Group::Materials,
        count: 4,
        inputs: &[(b(LOG), 1)],
        blurb: "Sawn from a log. Build with them, or make sticks, boxes and more from them. They burn, briefly.",
    },
    Recipe {
        output: STICK,
        group: Group::Materials,
        count: 4,
        inputs: &[(b(PLANKS), 2)],
        blurb: "Handles for tools, torches and ladders, and posts for power poles.",
    },
    Recipe {
        output: b(LADDER),
        group: Group::Building,
        count: 3,
        inputs: &[(STICK, 4)],
        blurb: "A frame you climb: stand in it and hold jump to go up, crouch to go down; let go and you stay \
                put. Stack them up a cliff or down a shaft; at the top, walk off onto the ledge.",
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
    // Appended: machines save the index of their recipe.
    MachineRecipe { machine: SMELTER, inputs: &[(b(QUARTZ_ORE), 1)], output: (b(GLASS), 2), seconds: 2.0 },
    MachineRecipe { machine: SMELTER, inputs: &[(b(SAND), 1)], output: (b(GLASS), 1), seconds: 2.0 },
];

/// The recipe `i` if `machine` makes it.
pub fn machine_recipe(machine: BlockId, i: u16) -> Option<&'static MachineRecipe> {
    MACHINE_RECIPES.get(i as usize).filter(|r| r.machine == machine)
}

/// Fuel: the seconds of smelting one item keeps a fire going, and the energy it gives a generator in
/// kJ (a coal runs a Mk1 miner long enough to mine about 32 coal).
pub const FUELS: &[(ItemId, f64, u32)] = &[(b(COAL_ORE), 8.0, 270), (b(LOG), 4.0, 135), (b(PLANKS), 1.0, 34)];

/// Seconds of work one `item` fuels, if it burns.
pub fn burn_time(item: ItemId) -> Option<f64> {
    FUELS.iter().find(|f| f.0 == item).map(|f| f.1)
}

/// The energy one `item` gives a generator, in kJ, if it burns.
pub fn fuel_energy(item: ItemId) -> Option<u32> {
    FUELS.iter().find(|f| f.0 == item).map(|f| f.2)
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
