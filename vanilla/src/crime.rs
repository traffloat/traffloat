//! Crime definitions

use arcstr::literal;
use smallvec::smallvec;

use traffloat_types::def::{
    crime::{self, Action::*},
    GameDefinition,
};

macro_rules! crimes {
    (
        $skill:ident;

        $({
            name: $name:literal,
            description: $description:literal,
            action: $action:expr,
            trigger: $trigger_which:ident($trigger_min:literal, $trigger_max:literal),
            probability: $probability:literal,
            skill_changes: {
                $($skill_ty:expr => $skill_change:literal,)*
            },
        })*
    ) => {
        /// Populates the game definition with crime data.
        pub fn populate(def: &mut GameDefinition, $skill: &super::skill::Ids) {
            $(
                def.crime_mut().push(crime::Type::builder()
                    .name(literal!($name))
                    .description(literal!($description))
                    .action($action)
                    .trigger_skill($skill.$trigger_which)
                    .trigger_skill_range(($trigger_min.into()..$trigger_max.into()))
                    .probability($probability.into())
                    .skill_change(smallvec![
                        $(
                            ($skill_ty, $skill_change.into()),
                        )*
                    ])
                    .build());
            )*
        }
    };
}

crimes! {
    skill;

    {
        name: "Pickpocket",
        description: "Steal cargo carried by other inhabitants",
        action: InhabitantTheft(5f64.into()),
        trigger: satisfaction(-500f64, 0f64),
        probability: 0.01,
        skill_changes: {
            skill.infamy => 5.,
            skill.morality => -1.,
        },
    }
    {
        name: "Robbery",
        description: "Rob a building and remove the cargo stored inside",
        action: NodeTheft(5f64.into()),
        trigger: satisfaction(-1000f64, -50f64),
        probability: 0.0001,
        skill_changes: {
            skill.infamy => 20.,
            skill.morality => -1.5,
        },
    }
    {
        name: "Arson",
        description: "Set a building on fire",
        action: Arson,
        trigger: satisfaction(-1000f64, -500f64),
        probability: 0.00005,
        skill_changes: {
            skill.infamy => 100.,
            skill.morality => -2.5,
        },
    }
}
