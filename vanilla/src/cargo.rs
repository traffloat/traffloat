//! Vanilla cargo type definitions.

use std::borrow::Cow;

use crate::{gas, liquid};

/// Defines a cargo type.
pub struct Def {
    /// String identifying the type, used for cross-referencing in vanilla definition.
    pub id: Cow<'static, str>,
    /// Name of the cargo type.
    pub name: Cow<'static, str>,
    /// Short description string for the cargo type.
    pub summary: Cow<'static, str>,
    /// Long, multiline description string for the cargo type.
    pub description: Cow<'static, str>,
    /// Category of cargo type.
    pub category: Category,
    /// Texture string of the cargo type.
    pub texture: Cow<'static, str>,
}

/// Category of the cargo type, only for display purpose.
#[derive(Clone, Copy, PartialEq, strum::EnumIter, strum::Display)]
#[strum(serialize_all = "title_case")]
pub enum Category {
    RawMineral,
    Organic,
    Ammunition,
    Container,
    IntermediateMaterial,
    Junk,
}

/// A short description for the category.
pub fn category_description(category: Category) -> &'static str {
    match category {
        Category::RawMineral =>  "Raw minerals are obtained by receiving and decomposing incoming asteroids. \
            Deflecting asteroids reduces the production of these minerals." ,
        Category::Organic =>  "Organic cargo is used for inhabitant birth and survival." ,
        Category::Ammunition => "Ammunition is ejected by defensive weapons to slow down or deflect asteroids.",
        Category::Container => "Other forms of resources can be packaged as container cargo. \
            This allows inhabitants to carry small amount of all resources to poorly developed regions.",
        Category::IntermediateMaterial => "Intermediate materials are derived from raw minerals \
            or other intermediate materials to feed into other factories. \
            They cannot be used in utility facilities.",
        Category::Junk => "Junk is the useless residue produced in other mechansims. \
            They must be ejected out of the colony using junk launchers, \
            otherwise they would fill up the space of the colony.",
    }
}

pub fn gas_bottle(gas: &gas::Def) -> Def {
    Def {
        id: gas.id.clone(),
        name: format!("Gas bottle ({})", gas.name).into(),
        summary: "Stores a small amount of gas".into(),
        description: "Produced in gas bottlers and centrifuges, gas bottles can be used to \
            transfer a small amount of gas to factories \
            as a replacement of diffusing gas slowly through corridors."
            .into(),
        category: Category::Container,
        texture: format!("{}-gas-bottle", gas.texture).into(),
    }
}

pub fn liquid_bottle(liquid: &liquid::Def) -> Def {
    Def {
        id: liquid.id.clone(),
        name: format!("Liquid bottle ({})", liquid.name).into(),
        summary: "Stores a small amount of liquid".into(),
        description: "Produced in liquid bottlers and centrifuges, liquid bottles can be used to \
            transfer a small amount of liquid to factories \
            as a replacement of constructing dedicated pipes through corridors."
            .into(),
        category: Category::Container,
        texture: format!("{}-liquid-bottle", liquid.texture).into(),
    }
}

macro_rules! cargos {
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
                id: Cow::Borrowed(stringify!($name)),
                name: Cow::Borrowed($name),
                summary: Cow::Borrowed($summary),
                description: Cow::Borrowed($description),
                category: Category::$category,
                texture: Cow::Borrowed($texture),
            };
        )*

        lazy_static::lazy_static! {
            pub static ref ALL: &'static [Def] = {
                let mut all = vec![$($ident),*];
                for gas in gas::ALL {
                    all.push(gas_bottle(gas));
                }
                for liquid in liquid::ALL {
                    all.push(liquid_bottle(liquid));
                }
                all.leak()
            };
        }
    }
}

cargos! {
    AMINO_ACID {
        name: "Amino acid",
        summary: "An organic mineral.",
        description: "Amino acids are found in small amounts in asteroids. \
            Use them to synthesize food and DNA.",
        category: RawMineral,
        texture: "dummy",
    }

    ROCK {
        name: "Rock",
        summary: "Chunks of rocks from asteroids.",
        description: "Rocks are the cheapest type of material obtained from asteroids. \
            They can be used as ammunition or disposed as junk.",
        category: RawMineral,
        texture: "dummy",
    }

    DNA {
        name: "DNA",
        summary: "Genetic material.",
        description: "DNA is used to produce inhabitants through asexual reproduction. \
            Although morally challenged, this is the most effective way \
            to start a new colony from scratch.",
        category: RawMineral,
        texture: "dummy",
    }

    PEPPLES {
        name: "Pepples",
        summary: "Stone pepples used as ammunition.",
        description: "Pepples are produced by decomposing rocks.\
            They are the basic type of ammunition for defense.",
        category: Ammunition,
        texture: "dummy",
    }

    SEDIMENT {
        name: "Sediment",
        summary: "Filtration and distillation residue.",
        description: "Sediments are waste produced during liquid processing. \
            They cannot be used for anything, and should be ejected with junk launchers \
            to avoid filling up storage space.",
        category: Junk,
        texture: "dummy",
    }

    BATTERY {
        name: "Battery",
        summary: "Stores a small amount of power",
        description: "Charged in charging stations and discharged in discharging stations, \
            batteries serve as an alternative method to transfer electricity between buildings. \
            They are useful for avoiding construction of power cables into low-consumption regions \
            and ensuring uninterrupted power supply in regions where cable often disconnects.
            ",
        category: Container,
        texture: "battery",
    }
}
