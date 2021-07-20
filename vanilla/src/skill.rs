//! Vanilla skill type definitions.

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
                        .name(String::from($name))
                        .description(String::from($description))
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
