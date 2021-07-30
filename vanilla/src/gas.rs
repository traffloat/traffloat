//! Vanilla gas type definitions.

use arcstr::{format, literal};

use traffloat_types::def::{gas, GameDefinition};

macro_rules! gases {
    (
        $($ident:ident {
            name: $name:literal,
            summary: $summary:literal,
            description: $description:literal,
            texture: $texture:literal,
        })*
    ) => {
        /// IDs assigned to the vanilla game definition.
        pub struct Ids {
            $(
                pub $ident: gas::TypeId,
            )*
        }

        /// Populates a [`GameDefinition`] with gas definition.
        pub fn populate(def: &mut GameDefinition) -> Ids {
            $(
                let $ident = def.add_gas(
                    gas::Type::builder()
                        .name(literal!($name))
                        .summary(literal!($summary))
                        .description(literal!($description))
                        .texture_src(format!("{}", crate::VANILLA_TEXTURE))
                        .texture_name(literal!($texture))
                        .build()
                );
            )*

            Ids {
                $($ident,)*
            }
        }
    }
}

gases! {
    oxygen {
        name: "Oxygen",
        summary: "Needed for breathing",
        description: "Oxygen is required for survival of inhabitants. \
            Inhabitants cannot work in buildings with low oxygen content, \
            except for construction work, where \
            sufficient oxygen must be available in adjacent buildings.",
        texture: "oxygen",
    }

    carbon_dioxide {
        name: "Carbon dioxide",
        summary: "Photosynthesis material",
        description: "Carbon dioxide is produced in houses and consumed in oxygen farms. \
            While high carbon dioxide level is not necessarily fatal, \
            they reduce the levels of other gases in the air.",
        texture: "carbon-dioxide",
    }

    nitrogen {
        name: "Nitrogen",
        summary: "An abundant, safe gas for pressure regulation",
        description: "Nitrogen is found in abundant amounts. \
            Although chemically inactive, they are great for pressure regulation \
            and can be condensed to produce coolants.",
        texture: "nitrogen",
    }
}
