//! Vanilla gas type definitions.

#![deny(dead_code)]

/// Defines a gas type.
pub struct Def {
    /// Name of the gas type.
    pub name: &'static str,
    /// Short description string for the gas type.
    pub summary: &'static str,
    /// Long, multiline description string for the gas type.
    pub description: &'static str,
    /// Base texture string of the gas type.
    pub texture: &'static str,
}

pub(crate) const OXYGEN: Def = Def {
    name: "Oxygen",
    summary: "Needed for breathing",
    description: "Oxygen is required for survival of inhabitants. \
        Inhabitants cannot work in buildings with low oxygen content, \
        except for construction work, where \
        sufficient oxygen must be available in adjacent buildings.",
    texture: "dummy",
};

pub(crate) const CARBON_DIOXIDE: Def = Def {
    name: "Carbon dioxide",
    summary: "Photosynthesis material",
    description: "Carbon dioxide is produced in houses and consumed in oxygen farms. \
        While high carbon dioxide level is not necessarily fatal, \
        they reduce the levels of other gases in the air.",
    texture: "dummy",
};

pub(crate) const NITROGEN: Def = Def {
    name: "Nitrogen",
    summary: "An abundant, safe gas for pressure regulation",
    description: "Nitrogen is found in abundant amounts. \
        Although chemically inactive, they are great for pressure regulation \
        and can be condensed to produce coolants.",
    texture: "dummy",
};

/// All gas types.
pub const ALL: &[Def] = &[
    OXYGEN,
    CARBON_DIOXIDE,
    NITROGEN,
];
