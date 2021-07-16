//! Vanilla mechanism definitions.

use std::borrow::Cow;
use std::fmt;

use derive_new::new;
use smallvec::{smallvec, SmallVec};

use traffloat_types::{time, units};

/// A component attached to buildings which serve as factories.
#[derive(new, getset::Getters, getset::CopyGetters)]
pub struct Def {
    /// Category of the reaction.
    #[getset(get_copy = "pub")]
    category: Category,
    /// Name of the reaction.
    #[getset(get = "pub")]
    name: Cow<'static, str>,
    /// Description of the reaction.
    #[getset(get = "pub")]
    description: Cow<'static, str>,
    /// The catalysts required for the reaction to take place.
    #[getset(get = "pub")]
    catalysts: SmallVec<[Catalyst; 2]>,
    /// Inputs and outputs to the reaction.
    #[getset(get = "pub")]
    puts: SmallVec<[Put; 2]>,
}

/// Category of the reaction, only for display purpose.
#[derive(Clone, Copy, PartialEq, strum::EnumIter, strum::Display)]
#[strum(serialize_all = "title_case")]
pub enum Category {
    Electricity,
}

/// Temporary typedef for identifying a cargo type.
pub type CargoType = &'static str;
/// Temporary typedef for identifying a liquid type.
pub type LiquidType = &'static str;
/// Temporary typedef for identifying a gas type.
pub type GasType = &'static str;

/// A condition for a reaction.
#[derive(getset::Getters)]
pub struct Catalyst {
    /// The levels at which the reaction takes place at minimum or maximum rate.
    #[getset(get = "pub")]
    levels: CatalystLevel,
    /// The underflow, minimum, maximum, overflow multipliers.
    #[getset(get = "pub")]
    multipliers: [f64; 4],
}

/// A type of resource whose existence affects a reaction.
pub enum CatalystLevel {
    /// Existence of cargo
    Cargo {
        ty: CargoType,
        levels: [units::CargoSize; 2],
    },
    /// Existence of liquid
    Liquid {
        ty: LiquidType,
        levels: [units::LiquidVolume; 2],
    },
    /// Existence of gas
    Gas {
        ty: GasType,
        levels: [units::GasVolume; 2],
    },
    /// Existence of power
    Electricity { levels: [units::ElectricPower; 2] },
    /// Existence of light
    Light { levels: [units::Brightness; 2] },
    /// Existence of skilled operators
    Skill { levels: [units::Skill; 2] },
}

/// Minimum or maximum
#[derive(Clone, Copy)]
pub enum MinMax {
    Min,
    Max,
}

impl MinMax {
    fn levels<T>(&self, [min, max]: [T; 2]) -> T {
        match self {
            Self::Min => min,
            Self::Max => max,
        }
    }
}

/// Display impl for CatalystLevel
pub struct DisplayCatalystLevel<'t>(pub &'t CatalystLevel, pub MinMax);

impl<'t> fmt::Display for DisplayCatalystLevel<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mm = self.1;
        match self.0 {
            CatalystLevel::Cargo { ty, levels } => write!(f, "{} {}", mm.levels(*levels), ty),
            CatalystLevel::Liquid { ty, levels } => write!(f, "{} {}", mm.levels(*levels), ty),
            CatalystLevel::Gas { ty, levels } => write!(f, "{} {}", mm.levels(*levels), ty),
            CatalystLevel::Electricity { levels } => {
                write!(f, "{} electricity", mm.levels(*levels))
            }
            CatalystLevel::Light { levels } => write!(f, "{} light", mm.levels(*levels)),
            CatalystLevel::Skill { levels } => write!(f, "{} skill", mm.levels(*levels)),
        }
    }
}

impl CatalystLevel {
    pub fn ty(&self) -> Cow<'static, str> {
        match self {
            Self::Cargo { ty, .. } => Cow::Borrowed(ty),
            Self::Liquid { ty, .. } => Cow::Borrowed(ty),
            Self::Gas { ty, .. } => Cow::Borrowed(ty),
            Self::Electricity { .. } => Cow::Borrowed("Electricity"),
            Self::Light { .. } => Cow::Borrowed("Light"),
            Self::Skill { .. } => Cow::Borrowed("Skill"),
        }
    }

    pub fn levels(&self) -> [f64; 2] {
        match self {
            Self::Cargo {
                levels: [min, max], ..
            } => [min.value(), max.value()],
            Self::Liquid {
                levels: [min, max], ..
            } => [min.value(), max.value()],
            Self::Gas {
                levels: [min, max], ..
            } => [min.value(), max.value()],
            Self::Electricity {
                levels: [min, max], ..
            } => [min.value(), max.value()],
            Self::Light {
                levels: [min, max], ..
            } => [min.value(), max.value()],
            Self::Skill {
                levels: [min, max], ..
            } => [min.value(), max.value()],
        }
    }
}

/// An input or output to a reaction.
#[derive(getset::Getters)]
pub struct Put {
    /// The rate at which this conumsable is changed.
    #[getset(get = "pub")]
    rate: time::Rate<Consumable>,
}

/// A type of resource that can be consumed.
#[derive(Clone)]
pub enum Consumable {
    /// Consumed or generated cargo
    Cargo {
        ty: CargoType,
        size: units::CargoSize,
    },
    /// Consumed or generated liquid
    Liquid {
        ty: LiquidType,
        size: units::LiquidVolume,
    },
    /// Consumed or generated gas
    Gas { ty: GasType, size: units::GasVolume },
    /// Consumed or generated power
    Electricity { size: units::ElectricPower },
}

impl Consumable {
    pub fn size(&self) -> f64 {
        match self {
            Self::Cargo { size, .. } => size.value(),
            Self::Liquid { size, .. } => size.value(),
            Self::Gas { size, .. } => size.value(),
            Self::Electricity { size, .. } => size.value(),
        }
    }

    pub fn size_mut(&mut self) -> &mut f64 {
        use units::Unit;

        match self {
            Self::Cargo { size, .. } => size.value_mut(),
            Self::Liquid { size, .. } => size.value_mut(),
            Self::Gas { size, .. } => size.value_mut(),
            Self::Electricity { size, .. } => size.value_mut(),
        }
    }
}

impl fmt::Display for Consumable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Cargo { ty, size } => write!(f, "{} {}", size, ty),
            Self::Liquid { ty, size } => write!(f, "{} {}", size, ty),
            Self::Gas { ty, size } => write!(f, "{} {}", size, ty),
            Self::Electricity { size } => write!(f, "{} electricity", size),
        }
    }
}

macro_rules! reactions {
    ($($ident:ident {
        name: $name:literal,
        description: $description:literal,
        category: $category:ident,
        catalysts: [$(
            $catalyst_variant:ident {
                $(ty: $catalyst_type:expr,)?
                levels: $catalyst_min:literal .. $catalyst_max:literal,
                multipliers: [
                    $catalyst_underflow_mul:literal,
                    $catalyst_min_mul:literal,
                    $catalyst_max_mul:literal,
                    $catalyst_overflow_mul:literal
                ],
            },
        )*],
        puts: [$(
            $put_variant:ident {
                $(ty: $put_type:expr,)?
                rate: $put_rate:literal,
            },
        )*],
    })*) => {
        lazy_static::lazy_static! {
            $(
                pub(crate) static ref $ident: Def = Def {
                    name: Cow::Borrowed($name),
                    description: Cow::Borrowed($description),
                    category: Category::$category,
                    catalysts: smallvec![$(
                        Catalyst {
                            levels: CatalystLevel::$catalyst_variant {
                                $(ty: $catalyst_type)?
                                levels: [$catalyst_min.into(), $catalyst_max.into()],
                            },
                            multipliers: [
                                $catalyst_underflow_mul,
                                $catalyst_min_mul,
                                $catalyst_max_mul,
                                $catalyst_overflow_mul,
                            ],
                        },
                    )*],
                    puts: smallvec![$(
                        Put {
                            rate: time::Rate(Consumable::$put_variant {
                                $(ty: $put_type)?
                                size: $put_rate.into(),
                            }),
                        },
                    )*],
                };
            )*

            /// All reaction types.
            pub static ref ALL: Vec<&'static Def> = vec![$(&$ident),*];
        }
    };
}

reactions! {
    SOLAR_POWER {
        name: "Solar power",
        description: "Generates electricity from sunlight.",
        category: Electricity,
        catalysts: [
            Light {
                levels: 0. .. 10.,
                multipliers: [0., 0., 1., 1.],
            },
        ],
        puts: [
            Electricity {
                rate: 100.,
            },
        ],
    }
}
