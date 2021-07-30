//! Vanilla cargo type definitions.

use arcstr::{format, literal};

use traffloat_types::def::{cargo, GameDefinition};

macro_rules! cargos {
    (
        container = $container:ident;
        $($category_ident:ident $category:literal ($category_description:literal) {
            $($ident:ident {
                name: $name:literal,
                summary: $summary:literal,
                description: $description:literal,
                texture: $texture:literal,
            })*
        })*
    ) => {
        /// IDs assigned to the vanilla game definition.
        pub struct Ids {
            $(
                $(
                    pub $ident: cargo::TypeId,
                )*
            )*
            pub first_liquid: cargo::TypeId,
            pub first_gas: cargo::TypeId,
        }

        /// Populates a [`GameDefinition`] with cargo definition.
        pub fn populate(def: &mut GameDefinition) -> Ids {
            $(
                let $category_ident = def.add_cargo_category(
                    cargo::Category::builder()
                        .title(literal!($category))
                        .description(literal!($category_description))
                        .build()
                );
                $(
                    let $ident = def.add_cargo(
                        cargo::Type::builder()
                            .name(literal!($name))
                            .summary(literal!($summary))
                            .description(literal!($description))
                            .texture_src(format!("{}", crate::VANILLA_TEXTURE))
                            .texture_name(literal!($texture))
                            .category($category_ident)
                            .build()
                    );
                )*
            )*

            let first_liquid = cargo::TypeId(def.cargo().len());
            let liquids = def.liquid().to_vec();
            for liquid in liquids {
                def.add_cargo(
                    cargo::Type::builder()
                        .name(format!("Liquid bottle ({})", liquid.name()))
                        .summary(format!("Stores a small amount of {}", liquid.name()))
                        .description(literal!("Produced in liquid bottlers and centrifuges, liquid bottles can be used to \
                            transfer a small amount of liquid to factories \
                            as a replacement of constructing dedicated pipes through corridors."))
                        .texture_src(format!("{}", crate::VANILLA_TEXTURE))
                        .texture_name(format!("{}-liquid-bottle", liquid.texture_name()))
                        .category($container)
                        .build()
                );
            }

            let first_gas = cargo::TypeId(def.cargo().len());
            let gases = def.gas().to_vec();
            for gas in gases {
                def.add_cargo(
                    cargo::Type::builder()
                        .name(format!("Gas bottle ({})", gas.name()))
                        .summary(format!("Stores a small amount of {}", gas.name()))
                        .description(literal!("Produced in gas bottlers and centrifuges, gas bottles can be used to \
                            transfer a small amount of gas to factories \
                            as a replacement of diffusing gas slowly through corridors."))
                        .texture_src(format!("{}", crate::VANILLA_TEXTURE))
                        .texture_name(format!("{}-gas-bottle", gas.texture_name()))
                        .category($container)
                        .build()
                );
            }

            Ids {
                $(
                    $($ident,)*
                )*
                first_liquid,
                first_gas,
            }
        }
    }
}

cargos! {
    container = container;

    raw_mineral "Raw mineral" ("Raw minerals are obtained by receiving and decomposing incoming asteroids. \
            Deflecting asteroids reduces the production of these minerals.") {
        amino_acid {
            name: "Amino acid",
            summary: "An organic mineral.",
            description: "Amino acids are found in small amounts in asteroids. \
                Use them to synthesize food and DNA.",
            texture: "amino-acid",
        }
        rock {
            name: "Rock",
            summary: "Chunks of rocks from asteroids.",
            description: "Rocks are the cheapest type of material obtained from asteroids. \
                They can be used as ammunition or disposed as junk.",
            texture: "rock",
        }
    }

    organic "Organic" ("Organic cargo is used for inhabitant birth and survival." ) {
        dna {
            name: "DNA",
            summary: "Genetic material.",
            description: "DNA is used to produce inhabitants through asexual reproduction. \
                Although morally challenged, this is the only way \
                to start a new colony from scratch.",
            texture: "dna",
        }
    }

    ammunition "Ammunition" ("Ammunition is ejected by defensive weapons to slow down or deflect asteroids.") {
        pepples {
            name: "Pepples",
            summary: "Stone pepples used as ammunition.",
            description: "Pepples are produced by decomposing rocks.\
                They are the basic type of ammunition for defense.",
            texture: "pepples",
        }
    }

    junk "Junk" ("Other forms of resources can be packaged as container cargo. \
            This allows inhabitants to carry small amount of all resources to poorly developed regions.") {
        sediment {
            name: "Sediment",
            summary: "Filtration and distillation residue.",
            description: "Sediments are waste produced during liquid processing. \
                They cannot be used for anything, and should be ejected with junk launchers \
                to avoid filling up storage space.",
            texture: "sediment",
        }
    }

    intermediate_material "Intermediate material" ("Intermediate materials are derived from raw minerals \
            or other intermediate materials to feed into other factories. \
            They cannot be used in utility facilities.") {
        aluminium {
            name: "Aluminium",
            summary: "A metal used for construction",
            description: "",
            texture: "aluminium",
        }
    }

    container "Container" ("Junk is the useless residue produced in other mechansims. \
            They must be ejected out of the colony using junk launchers, \
            otherwise they would fill up the space of the colony.") {
        battery {
            name: "Battery",
            summary: "Stores a small amount of power",
            description: "Charged in charging stations and discharged in discharging stations, \
                batteries serve as an alternative method to transfer electricity between buildings. \
                They are useful for avoiding construction of power cables into low-consumption regions \
                and ensuring uninterrupted power supply in regions where cable often disconnects.
                ",
            texture: "battery",
        }
    }
}
