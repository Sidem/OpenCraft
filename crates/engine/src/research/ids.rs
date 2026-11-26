//! Named indices into [`super::TECHS`] (ordered by ring, then the techs added since at the open end). Using constants keeps prerequisites readable and
//! lets tests and migration refer to techs without magic numbers.

// Red ring (deeds, 0..2)
pub const BELT_ROUTING: u8 = 0;
pub const BELT_LIFTS: u8 = 1;

// Green ring (2..17)
pub const UNDERPASSES: u8 = 2;
pub const MECHANICS: u8 = 3;
pub const FLUID_HANDLING: u8 = 4;
pub const MASONRY: u8 = 5;
pub const ASSEMBLY: u8 = 6;
pub const STEELMAKING: u8 = 7;
pub const HYDROPOWER: u8 = 8;
pub const ORE_SCANNING: u8 = 9;
pub const HAULER_GEAR: u8 = 10;
pub const WORK_GLOVES: u8 = 11;
pub const FORESTRY: u8 = 12;
pub const STONE_STYLES: u8 = 13;
pub const GLASS: u8 = 14;
pub const METALWORK: u8 = 15;
pub const PAINT: u8 = 16;

// Blue ring (17..33)
pub const MK3_KITS: u8 = 17;
pub const STEEL_TOOLS: u8 = 18;
pub const STEAM_POWER: u8 = 19;
pub const ORE_CRUSHING: u8 = 20;
pub const BULK_STORAGE: u8 = 21;
pub const ELECTRONICS: u8 = 22;
pub const SOLAR_POWER: u8 = 23;
pub const ADVANCED_SCANNING: u8 = 24;
pub const LOGIC: u8 = 25;
pub const FIELD_GEAR: u8 = 26;
pub const ORE_WASHING: u8 = 27;
pub const HOISTS: u8 = 28;
pub const RECYCLING: u8 = 29;
pub const SETTLEMENTS: u8 = 30;
pub const XRAY_GOGGLES: u8 = 31;
pub const LED_LAMPS: u8 = 32;

// Violet ring (33..60)
pub const MK4_KITS: u8 = 33;
pub const PROCESSORS: u8 = 34;
pub const ROBOTICS: u8 = 35;
pub const CONSTRUCTION_DRONES: u8 = 36;
pub const SWARM_LOGISTICS: u8 = 37;
pub const JETPACK: u8 = 38;
pub const PERSONAL_DRONE: u8 = 39;
pub const EARTHWORKS: u8 = 40;
pub const EXOSUIT: u8 = 41;
pub const BAUXITE_PROCESSING: u8 = 42;
pub const RAILWAYS: u8 = 43;
pub const HOVER_PACK: u8 = 44;
pub const CARGO_DRONES: u8 = 45;
pub const OIL_REFINING: u8 = 46;
pub const PLASTICS: u8 = 47;
pub const ACIDS_AND_LUBRICANTS: u8 = 48;
pub const DIESEL_POWER: u8 = 49;
pub const ELECTROLYSIS: u8 = 50;
pub const RESEARCH_CENTER: u8 = 51;
pub const NUCLEAR_POWER: u8 = 52;
pub const AUTO_ROUTING: u8 = 53;
pub const AI_SURVEY: u8 = 54;
pub const LONG_DISTANCE_POWER: u8 = 55;
pub const PIGMENTS: u8 = 56;
pub const NATURAL_GAS: u8 = 57;
pub const FLAMETHROWER: u8 = 58;
pub const SEISMIC_PROSPECTING: u8 = 59;

// Gold ring (60..72)
pub const MK5_KITS: u8 = 60;
pub const CHIP_FAB: u8 = 61;
pub const DATA_NETWORK: u8 = 62;
pub const DATACENTERS: u8 = 63;
pub const AI_LABS: u8 = 64;
pub const MINING_PRODUCTIVITY: u8 = 65;
pub const MACHINE_SPEED: u8 = 66;
pub const DRONE_SPEED: u8 = 67;
pub const OPTIMIZER: u8 = 68;
pub const PHOTONICS: u8 = 69;
pub const DRONE_SWARMS: u8 = 70;
pub const ROCKETRY: u8 = 71;

// Sky ring (72..76)
pub const ORBITAL_COMMS: u8 = 72;
pub const ORBITAL_POWER: u8 = 73;
pub const ORBITAL_SCIENCE: u8 = 74;
pub const DROP_PODS: u8 = 75;

// Added since, at the open end (`later.rs`); no migration names them, so only tests do for now
#[cfg_attr(not(test), allow(dead_code))]
pub const CLAY_AND_KILNS: u8 = 76;
#[cfg_attr(not(test), allow(dead_code))]
pub const PORCELAIN_AND_GLAZES: u8 = 77;
#[cfg_attr(not(test), allow(dead_code))]
pub const SPACE_ELEVATOR: u8 = 78;
