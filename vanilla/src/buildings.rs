//! Vanilla building definitions.

/// Shape of a building.
pub struct Shape {
    /// Name of the texture
    pub texture: &'static str,
}

/// Defines a building type.
pub struct Def {
    /// Name of the building type.
    pub name: &'static str,
    /// Short description string for the building type.
    pub summary: &'static str,
    /// Long, multiline description string for the building type.
    pub description: &'static str,
    /// Category of building.
    pub category: Category,
    /// Shape of the building.
    pub shape: Shape,
}

/// Category of the building, only for display purpose.
#[derive(Clone, Copy, PartialEq, strum::EnumIter, strum::Display)]
#[strum(serialize_all = "title_case")]
pub enum Category {
    Population,
    Transportation,
    Electricity,
    Liquid,
    Gas,
    Defense,
}

/// All building types.
pub const ALL: &[Def] = &[
    Def {
        name: "Core",
        summary: "The center of the whole colony",
        description: "The core is the ultimate building to protect. \
            It provides basic resources, including a small amount of uninterrupted power, \
            some oxygen generation and a few population housing. \
            Destruction of the core ends the game.",
        category: Category::Population,
        shape: Shape { texture: "core" },
    },
    Def {
        name: "House",
        summary: "Produces and supports the survival of inhabitants.",
        description: "",
        category: Category::Population,
        shape: Shape { texture: "house" },
    },
    Def {
        name: "Solar panel",
        summary: "Basic power production",
        description: "",
        category: Category::Electricity,
        shape: Shape {
            texture: "solar-panel",
        },
    },
];
