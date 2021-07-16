//! Vanilla gas type definitions.

use std::borrow::Cow;

/// Defines a gas type.
pub struct Def {
    /// String identifying the type, used for cross-referencing in vanilla definition.
    pub(crate) id: Cow<'static, str>,
    /// Name of the gas type.
    pub name: Cow<'static, str>,
    /// Short description string for the gas type.
    pub summary: Cow<'static, str>,
    /// Long, multiline description string for the gas type.
    pub description: Cow<'static, str>,
    /// Base texture string of the gas type.
    pub texture: Cow<'static, str>,
}

macro_rules! gases {
    (
        $($ident:ident {
            name: $name:literal,
            summary: $summary:literal,
            description: $description:literal,
            texture: $texture:literal,
        })*
    ) => {
        $(
            pub(crate) const $ident: Def = Def {
                id: Cow::Borrowed(stringify!($name)),
                name: Cow::Borrowed($name),
                summary: Cow::Borrowed($summary),
                description: Cow::Borrowed($description),
                texture: Cow::Borrowed($texture),
            };
        )*

        /// All gas types.
        pub const ALL: &[Def] = &[$($ident),*];
    }
}

gases! {
    OXYGEN {
        name: "Oxygen",
        summary: "Needed for breathing",
        description: "Oxygen is required for survival of inhabitants. \
            Inhabitants cannot work in buildings with low oxygen content, \
            except for construction work, where \
            sufficient oxygen must be available in adjacent buildings.",
        texture: "dummy",
    }

    CARBON_DIOXIDE {
        name: "Carbon dioxide",
        summary: "Photosynthesis material",
        description: "Carbon dioxide is produced in houses and consumed in oxygen farms. \
            While high carbon dioxide level is not necessarily fatal, \
            they reduce the levels of other gases in the air.",
        texture: "dummy",
    }

    NITROGEN {
        name: "Nitrogen",
        summary: "An abundant, safe gas for pressure regulation",
        description: "Nitrogen is found in abundant amounts. \
            Although chemically inactive, they are great for pressure regulation \
            and can be condensed to produce coolants.",
        texture: "dummy",
    }
}
