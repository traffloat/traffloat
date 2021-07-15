//! Vanilla data definition

/// Vanilla building definitions.
pub mod buildings {
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
        /// Shape of the building.
        pub shape: Shape,
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
            shape: Shape { texture: "core" },
        },
        Def {
            name: "House",
            summary: "Produces and supports the survival of inhabitants.",
            description: "",
            shape: Shape { texture: "house" },
        },
        Def {
            name: "Solar panel",
            summary: "Basic power production",
            description: "",
            shape: Shape {
                texture: "solar-panel",
            },
        },
    ];
}

/// Vanilla reaction definitions.
pub mod reactions {
    /// Defines a reaction type.
    pub struct Reaction {
        // TODO
    }
}
