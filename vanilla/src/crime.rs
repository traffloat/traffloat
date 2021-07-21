//! Crime definitions

use arcstr::literal;
use smallvec::smallvec;

use traffloat_types::def::{
    crime::{self, Action::*},
    GameDefinition,
};

macro_rules! crimes {
    ($({
        name: $name:literal,
        description: $description:literal,
        action: $action:expr,
        trigger: [$trigger_min:literal, $trigger_max:literal],
        probability: $probability:literal,
        happiness_change: $happiness_change:literal,
        morality_change: $skill_change:literal,
    })*) => {
        /// Populates the game definition with crime data.
        pub fn populate(def: &mut GameDefinition, skill: &super::skill::Ids) {
            $(
                def.crime_mut().push(crime::Type::builder()
                    .name(literal!($name))
                    .description(literal!($description))
                    .action($action)
                    .trigger_happiness_range(($trigger_min.into()..$trigger_max.into()))
                    .probability($probability.into())
                    .happiness_change($happiness_change.into())
                    .skill_change(smallvec![(skill.morality, $skill_change.into())])
                    .build());
            )*
        }
    };
}

crimes! {
    {
        name: "Pickpocket",
        description: "Steal cargo carried by other inhabitants",
        action: InhabitantTheft(5f64.into()),
        trigger: [-500f64, 0f64],
        probability: 0.01,
        happiness_change: -5.,
        morality_change: -1f64,
    }
    {
        name: "Robbery",
        description: "Rob a building and remove the cargo stored inside",
        action: NodeTheft(5f64.into()),
        trigger: [-1000f64, -50f64],
        probability: 0.0001,
        happiness_change: -20.,
        morality_change: -1.5,
    }
    {
        name: "Arson",
        description: "Set a building on fire",
        action: Arson,
        trigger: [-1000f64, -500f64],
        probability: 0.00005,
        happiness_change: -100.,
        morality_change: -2.5,
    }
}
