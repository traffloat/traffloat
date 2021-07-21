//! Vanilla liquid type definitions.

use arcstr::literal;

use traffloat_types::def::{liquid, GameDefinition};

macro_rules! liquids {
    (
        $($ident:ident {
            name: $name:literal,
            summary: $summary:literal,
            description: $description:literal,
            viscosity: $viscosity:literal,
            texture: $texture:literal,
        })*
    ) => {
        /// IDs assigned to the vanilla game definition.
        pub struct Ids {
            $(
                pub $ident: liquid::TypeId,
            )*
        }

        /// Populates a [`GameDefinition`] with liquid definition.
        pub fn populate(def: &mut GameDefinition) -> Ids {
            $(
                let $ident = def.add_liquid(
                    liquid::Type::builder()
                        .name(literal!($name))
                        .summary(literal!($summary))
                        .description(literal!($description))
                        .viscosity($viscosity.into())
                        .texture(literal!($texture))
                        .build()
                );
            )*

            Ids {
                $($ident,)*
            }
        }
    }
}

liquids! {
    asteroidal_water {
        name: "Asteroidal water",
        summary: "Water found on asteroids",
        description: "Deposits of water can sometimes be found in asteroids. \
            Contaminated by asteroidal minerals, it must be filtered to be used in factories,
            or deionized so that it is drinkable by inhabitants.",
        viscosity: 1.,
        texture: "dummy",
    }

    filtered_water {
        name: "Filtered water",
        summary: "Water without insoluble impurities.",
        description: "Filtered water is removed of insoluble impurities, \
            so they can be used in other factories without clogging up the pipes.",
        viscosity: 1.,
        texture: "dummy",
    }

    deionized_water {
        name: "Deionized water",
        summary: "Drinking water",
        description: "Soluble impurities in water are removed from water during deionization. \
            This makes the water safe for inhabitant intake.",
        viscosity: 1.,
        texture: "dummy",
    }

    urine {
        name: "Urine",
        summary: "Waste produced by inhabitants",
        description: "Urines are organic waste produced by inhabitants in houses. \
            Arrange sewage pipes to remove them from houses and \
            recycle them by distillation into drinking water.",
        viscosity: 5.,
        texture: "dummy",
    }

    coolant {
        name: "Coolant",
        summary: "A liquid at very low temperature",
        description: "Coolants are produced by condensation of nitrogen. \
            They are required in factories with highly exothermic reactions.",
        viscosity: 1.,
        texture: "dummy",
    }
}
