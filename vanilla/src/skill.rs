//! Vanilla skill type definitions.

use arcstr::literal;

use traffloat_types::def::{skill, GameDefinition};

macro_rules! skills {
    (
        $($ident:ident {
            name: $name:literal,
            description: $description:literal,
        })*
    ) => {
        /// IDs assigned to the vanilla game definition.
        pub struct Ids {
            $(
                pub $ident: skill::TypeId,
            )*
        }

        /// Populates a [`GameDefinition`] with gas definition.
        pub fn populate(def: &mut GameDefinition) -> Ids {
            $(
                let $ident = def.add_skill(
                    skill::Type::builder()
                        .name(literal!($name))
                        .description(literal!($description))
                        .build()
                );
            )*

            Ids {
                $($ident,)*
            }
        }
    }
}

skills! {
    age {
        name: "Age",
        description: "Age of the inhabitant. Some jobs require operators of a specific age range. \
            A high age increases the probability of health decrease.",
    }
    health {
        name: "Health",
        description: "Physical well-being of the inhabitant. The inhabitant dies when health drops to zero.",
    }
    happiness {
        name: "Happiness",
        description: "Inhabitants lose happiness when they work and gain happiness when they have entertainment.",
    }
    satisfaction {
        name: "Satisfaction",
        description: "Inhabitants lose satisfaction when they do not receive sufficient supplies in their house. \
            If satisfaction drops too low, inhabitants become outlaws and commit crimes.",
    }
    infamy {
        name: "Infamy",
        description: "Inhabitants earn infamy when they perform crimes. \
            Infamy values are used by security mechanisms to identify the crime threat of an inhabitant, \
            such as movement control and police patrol. \
            Inhabitants can lose infamy by imprisonment.",
    }
    morality {
        name: "Morality",
        description: "Inhabitants are educated with morality values to reduce the probability of committing crimes.",
    }
    driving {
        name: "Driving skill",
        description: "When an inhabitant with good driving skill operates a vehicle, it can move faster across rails.",
    }
    mechanic {
        name: "Mechanic skill",
        description: "Inhabitants are trained as mechanics to operate factories more effectively.",
    }
    construction {
        name: "Construction skill",
        description: "Inhabitants are trained as builders to construct new buildings faster.",
    }
    teaching {
        name: "Teaching skill",
        description: "Inhabitants are trained as teachers so that the next generation can learn better in schools.",
    }
    military {
        name: "Military skill",
        description: "Inhabitants are trained as soldiers or police. \
            Weapons they operate will be more effective, \
            and they have higher chance of successfully arresting an outlaw.",
    }
}
