//! Vanilla liquid type definitions.

#![deny(dead_code)]

/// Defines a liquid type.
pub struct Def {
    /// Name of the liquid type.
    pub name: &'static str,
    /// Short description string for the liquid type.
    pub summary: &'static str,
    /// Long, multiline description string for the liquid type.
    pub description: &'static str,
    /// Base texture string of the liquid.
    pub texture: &'static str,
}

pub(crate) const ASTEROIDAL_WATER: Def = Def {
    name: "Asteroidal water",
    summary: "Water found on asteroids",
    description: "Deposits of water can sometimes be found in asteroids. \
        Contaminated by asteroidal minerals, it must be filtered to be used in factories,
        or deionized so that it is drinkable by inhabitants.",
    texture: "dummy",
};

pub(crate) const FILTERED_WATER: Def = Def {
    name: "Filtered water",
    summary: "Water without insoluble impurities.",
    description: "Filtered water is removed of insoluble impurities, \
        so they can be used in other factories without clogging up the pipes.",
    texture: "dummy",
};

pub(crate) const DEIONIZED_WATER: Def = Def {
    name: "Deionized water",
    summary: "Drinking water",
    description: "Soluble impurities in water are removed from water during deionization. \
        This makes the water safe for inhabitant intake.",
    texture: "dummy",
};

pub(crate) const URINE: Def = Def {
    name: "Urine",
    summary: "Waste produced by inhabitants",
    description: "Urines are organic waste produced by inhabitants in houses. \
        Arrange sewage pipes to remove them from houses and \
        recycle them by distillation into drinking water.",
    texture: "dummy",
};

pub(crate) const COOLANT: Def = Def {
    name: "Coolant",
    summary: "A liquid at very low temperature",
    description: "Coolants are produced by condensation of nitrogen. \
        They are required in factories with highly exothermic reactions.",
    texture: "dummy",
};

/// All liquid types.
pub const ALL: &[Def] = &[
    ASTEROIDAL_WATER,
    FILTERED_WATER,
    DEIONIZED_WATER,
    URINE,
    COOLANT,
];
