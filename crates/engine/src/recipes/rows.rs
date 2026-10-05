//! Indices into `MACHINE_RECIPES` that research locks or that tests and specs name (rows are appended, never moved).

/// Rows research locks: the gear (Mechanics), bricks and quicklime (Masonry), the assembler's.
pub const GEAR_RECIPE: u16 = 8;
pub const BRICK_RECIPE: u16 = 9;
pub const QUICKLIME_RECIPE: u16 = 10;
/// The assembler's rows (Assembly): motor, concrete, red and green packs, green kits, belts.
pub const ASSEMBLY_RECIPES: [u16; 6] = [11, 12, 13, 14, 15, 30];
/// The blast furnace's row and the constructor's steel plate and beam (Steelmaking).
pub const STEEL_RECIPES: [u16; 3] = [16, 17, 18];
/// Blue science pack and blue kit, made by assemblers only.
pub const BLUE_RECIPES: [u16; 2] = [19, 20];
/// The crusher's two ore rows and slag row, and the smelter's rows for the crushed ore (Ore Crushing).
pub const CRUSH_RECIPES: [u16; 5] = [21, 22, 23, 24, 25];
/// The arc furnace's silicon and the assembler's circuit (Electronics).
pub const ELECTRONICS_RECIPES: [u16; 2] = [26, 27];
/// Violet science pack and violet kit, made by assemblers only.
pub const VIOLET_RECIPES: [u16; 2] = [28, 29];
/// The drone chain, made by assemblers only: processor (Processors), servo and actuator (Robotics), drone cell
/// (Drone Power), guidance module (Navigation), drone (Construction Drones).
pub const DRONE_RECIPES: [u16; 6] = [31, 32, 33, 34, 35, 36];
/// Aluminium (Bauxite Processing): the crusher's bauxite row, the electrolytic cell's ingot, the constructor's plate
/// and the assembler's battery.
pub const BAUXITE_RECIPES: [u16; 4] = [37, 38, 39, 40];
/// The assembler's cargo drone (Cargo Drones).
pub const CARGO_RECIPE: u16 = 41;
/// The constructor's empty canister (Oil Processing).
pub const CANISTER_MACHINE_RECIPE: u16 = 42;
/// The refinery's Distil and the cracker's Crack (Refining).
pub const DISTIL_RECIPE: u16 = 43;
pub const CRACK_RECIPE: u16 = 44;
/// The chemical plant's plastic (Plastics), and its acid and lubricant (Acid and Lubricants).
pub const PLASTIC_RECIPE: u16 = 45;
pub const ACID_RECIPE: u16 = 46;
pub const LUBRICANT_RECIPE: u16 = 47;
/// The electrolyser's hydrogen and oxygen (Electrolysis).
pub const ELECTROLYSE_RECIPE: u16 = 48;
/// Ore Washing: the washer's three rows, the smelter's two and the cell's row for washed ore, the crusher's tailings.
pub const WASH_RECIPES: [u16; 7] = [49, 50, 51, 52, 53, 54, 55];
/// The centrifuge's fuel cell (Nuclear Power).
pub const ENRICH_RECIPE: u16 = 56;
/// The assembler's gold science pack (Gold Science) and gold kit (Mk5 Machines).
pub const GOLD_RECIPES: [u16; 2] = [57, 58];
