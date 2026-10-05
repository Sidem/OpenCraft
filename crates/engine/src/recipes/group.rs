//! The build menu's sections: every hand recipe belongs to one (`Recipe::group`).

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
