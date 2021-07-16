//! Vanilla building definitions.

use std::borrow::Cow;

/// Shape of a building.
pub struct Shape {
    /// Name of the texture
    pub texture: Cow<'static, str>,
}

/// Defines a building type.
pub struct Def {
    /// Name of the building type.
    pub name: Cow<'static, str>,
    /// Short description string for the building type.
    pub summary: Cow<'static, str>,
    /// Long, multiline description string for the building type.
    pub description: Cow<'static, str>,
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

macro_rules! buildings {
    (
        $($ident:ident {
            name: $name:literal,
            summary: $summary:literal,
            description: $description:literal,
            category: $category:ident,
            texture: $texture:literal,
        })*
    ) => {
        $(
            pub(crate) const $ident: Def = Def {
                name: Cow::Borrowed($name),
                summary: Cow::Borrowed($summary),
                description: Cow::Borrowed($description),
                category: Category::$category,
                shape: Shape { texture: Cow::Borrowed($texture) },
            };
        )*

        /// All building types.
        pub const ALL: &[Def] = &[$($ident),*];
    }
}

buildings! {
    CORE {
        name: "Core",
        summary: "The center of the whole colony",
        description: "The core is the ultimate building to protect. \
            It provides basic resources, including a small amount of uninterrupted power, \
            some oxygen generation and a few population housing. \
            Destruction of the core ends the game.",
        category: Population,
        texture: "core",
    }
    HOUSE {
        name: "House",
        summary: "Produces and supports the survival of inhabitants.",
        description: "",
        category: Population,
        texture: "house",
    }
    SOLAR_PANEL {
        name: "Solar panel",
        summary: "Basic power production",
        description: "",
        category: Electricity,
        texture: "solar-panel",
    }
}
