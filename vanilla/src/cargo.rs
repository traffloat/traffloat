//! Vanilla cargo type definitions.

#![deny(dead_code)]

/// Defines a cargo type.
pub struct Def {
    /// Name of the cargo type.
    pub name: &'static str,
    /// Short description string for the cargo type.
    pub summary: &'static str,
    /// Long, multiline description string for the cargo type.
    pub description: &'static str,
    /// Category of cargo type.
    pub category: Category,
    /// Texture string of the cargo type.
    pub texture: &'static str,
}

/// Category of the cargo type, only for display purpose.
#[derive(Clone, Copy, PartialEq, strum::EnumIter, strum::Display)]
#[strum(serialize_all = "title_case")]
pub enum Category {
    RawMineral,
    Organic,
    Container,
    Intermediate,
}

pub(crate) const BATTERY: Def = Def {
    name: "Battery",
    summary: "Stores a small amount of power",
    description: "Charged in charging stations and discharged in discharging stations, \
        batteries serve as an alternative method to transfer electricity between buildings. \
        They are useful for avoiding construction of power cables into low-consumption regions \
        and ensuring uninterrupted power supply in regions where cable often disconnects.
        ",
    category: Category::Container,
    texture: "battery",
};

pub(crate) const GAS_BOTTLE: Def = Def {
    name: "Gas bottle",
    summary: "Stores a small amount of gas",
    description: "Produced in gas bottlers and centrifuges, gas bottlers can be used to \
        transfer a small amount of gases to factories \
        as a replacement of diffusing gases slowly through corridors.",
    category: Category::Container,
    texture: "gas-bottle",
};

/// All cargo types.
pub const ALL: &[Def] = &[BATTERY, GAS_BOTTLE];
