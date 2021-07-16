//! Vanilla liquid type definitions.

use std::borrow::Cow;

/// Defines a liquid type.
pub struct Def {
    /// String identifying the type, used for cross-referencing in vanilla definition.
    pub(crate) id: Cow<'static, str>,
    /// Name of the liquid type.
    pub name: Cow<'static, str>,
    /// Short description string for the liquid type.
    pub summary: Cow<'static, str>,
    /// Long, multiline description string for the liquid type.
    pub description: Cow<'static, str>,
    /// Base texture string of the liquid.
    pub texture: Cow<'static, str>,
}

macro_rules! liquids {
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

        /// All liquid types.
        pub const ALL: &[Def] = &[$($ident),*];
    }
}

liquids! {
    ASTEROIDAL_WATER {
        name: "Asteroidal water",
        summary: "Water found on asteroids",
        description: "Deposits of water can sometimes be found in asteroids. \
            Contaminated by asteroidal minerals, it must be filtered to be used in factories,
            or deionized so that it is drinkable by inhabitants.",
        texture: "dummy",
    }

    FILTERED_WATER {
        name: "Filtered water",
        summary: "Water without insoluble impurities.",
        description: "Filtered water is removed of insoluble impurities, \
            so they can be used in other factories without clogging up the pipes.",
        texture: "dummy",
    }

    DEIONIZED_WATER {
        name: "Deionized water",
        summary: "Drinking water",
        description: "Soluble impurities in water are removed from water during deionization. \
            This makes the water safe for inhabitant intake.",
        texture: "dummy",
    }

    URINE {
        name: "Urine",
        summary: "Waste produced by inhabitants",
        description: "Urines are organic waste produced by inhabitants in houses. \
            Arrange sewage pipes to remove them from houses and \
            recycle them by distillation into drinking water.",
        texture: "dummy",
    }

    COOLANT {
        name: "Coolant",
        summary: "A liquid at very low temperature",
        description: "Coolants are produced by condensation of nitrogen. \
            They are required in factories with highly exothermic reactions.",
        texture: "dummy",
    }
}
