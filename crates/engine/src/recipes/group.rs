//! The build menu's sections: every hand recipe belongs to one (`Recipe::group`). The order inside a section is menu_order.rs.

/// The build menu's sections, in the order it shows them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Materials,
    Production,
    Logistics,
    Power,
    Science,
    Computing,
    Space,
    Tools,
    Building,
}

pub const GROUPS: [Group; 9] = [
    Group::Materials,
    Group::Production,
    Group::Logistics,
    Group::Power,
    Group::Science,
    Group::Computing,
    Group::Space,
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
            Group::Computing => "Computing",
            Group::Space => "Space",
            Group::Tools => "Tools",
            Group::Building => "Building",
        }
    }
}
