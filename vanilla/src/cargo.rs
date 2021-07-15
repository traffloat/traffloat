//! Vanilla cargo type definitions.

#![deny(dead_code)]

/// Defines a cargo type.
pub struct Def {
    /// Name of the cargo type.
    pub name: &'static str,
    /// Short description string for the cargo type.
    pub summary: &'static str,
    /// Long, multiline description string for the cargo type.
    pub description: &'static str,
    /// Category of cargo type.
    pub category: Category,
    /// Texture string of the cargo type.
    pub texture: &'static str,
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

pub(crate) const AMINO_ACID: Def = Def {
    name: "Amino acid",
    summary: "An organic mineral.",
    description: "Amino acids are found in small amounts in asteroids. \
        Use them to synthesize food and DNA.",
    category: Category::RawMineral,
    texture: "dummy",
};

pub(crate) const ROCK: Def = Def {
    name: "Rock",
    summary: "Chunks of rocks from asteroids.",
    description: "Rocks are the cheapest type of material obtained from asteroids. \
        They can be used as ammunition or disposed as junk.",
    category: Category::RawMineral,
    texture: "dummy",
};

pub(crate) const DNA: Def = Def {
    name: "DNA",
    summary: "Genetic material.",
    description: "DNA is used to produce inhabitants through asexual reproduction. \
        Although morally challenged, this is the most effective way \
        to start a new colony from scratch.",
    category: Category::RawMineral,
    texture: "dummy",
};

pub(crate) const PEPPLES: Def = Def {
    name: "Pepples",
    summary: "Stone pepples used as ammunition.",
    description: "Pepples are produced by decomposing rocks.\
        They are the basic type of ammunition for defense.",
    category: Category::Ammunition,
    texture: "dummy",
};

pub(crate) const BATTERY: Def = Def {
    name: "Battery",
    summary: "Stores a small amount of power",
    description: "Charged in charging stations and discharged in discharging stations, \
        batteries serve as an alternative method to transfer electricity between buildings. \
        They are useful for avoiding construction of power cables into low-consumption regions \
        and ensuring uninterrupted power supply in regions where cable often disconnects.
        ",
    category: Category::Container,
    texture: "battery",
};

pub(crate) const GAS_BOTTLE: Def = Def {
    name: "Gas bottle",
    summary: "Stores a small amount of gas",
    description: "Produced in gas bottlers and centrifuges, gas bottles can be used to \
        transfer a small amount of gas to factories \
        as a replacement of diffusing gas slowly through corridors.",
    category: Category::Container,
    texture: "gas-bottle",
};

pub(crate) const LIQUID_BOTTLE: Def = Def {
    name: "Liquid bottle",
    summary: "Stores a small amount of liquid",
    description: "Produced in liquid bottlers and centrifuges, liquid bottles can be used to \
        transfer a small amount of liquid to factories \
        as a replacement of constructing dedicated pipes through corridors.",
    category: Category::Container,
    texture: "liquid-bottle",
};

/// All cargo types.
pub const ALL: &[Def] = &[
    AMINO_ACID,
    DNA,
    ROCK,
    PEPPLES,
    BATTERY,
    GAS_BOTTLE,
    LIQUID_BOTTLE,
];
