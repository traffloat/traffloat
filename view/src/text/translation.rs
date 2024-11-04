//! Translation data.

use std::fmt;
use std::num::FpCategory;
use std::ops::Deref;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use super::Argument;
use crate::appearance::ResourceSha;

/// Identifier for a set of glosaries of different locales.
pub type GlossarySha = ResourceSha;

/// A localized glossary.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Glossary {
    /// The locale this translation bundle targets.
    pub locale:  String,
    /// The translation entries in this translation bundle.
    pub entries: Vec<Entry>,
}

/// A localized translation entry.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Entry {
    /// Heterogeneous elements of a translation string to be concatenated together.
    pub elements: Vec<Element>,
}

impl Entry {
    /// Appends the entry to an output string.
    pub fn render(&self, args: &[Argument], output: &mut String) {
        for element in &self.elements {
            element.render(args, output);
        }
    }
}

/// An element of a translation string.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum Element {
    /// Writes the wrapped string to the output.
    Literal {
        /// The content to wrap.
        content: String,
    },
    /// Writes the displayed form of a parameter to the output.
    Parameter {
        /// Index of the parameter.
        ///
        /// The actual value is expected to be an [`Argument::String`].
        index:     u16,
        /// Applies a transformation on the parameter value before displaying.
        /// Does nothing if it is not a number parameter.
        #[serde(default)]
        transform: NumberTransform,
        /// If a precision is set,
        /// indicates how many digits after the decimal point should be printed.
        #[serde(default)]
        precision: Option<u32>,
    },
    /// Evaluates to one of the branches by matching the value of a number.
    MatchNumber {
        /// Index of the parameter to match.
        ///
        /// Must be of numeric type.
        param_index: u16,
        /// The conditional arms to execute.
        ///
        /// The first matched arm in the list is executed.
        arms:        Vec<NumberMatchArm>,
        /// The default arm to execute if all arms are unmatched.
        default:     Vec<Element>,
    },
}

impl Element {
    /// Appends the element to an output string.
    pub fn render(&self, args: &[Argument], output: &mut String) {
        use std::fmt::Write;

        match *self {
            Self::Literal { ref content } => output.push_str(content),
            Self::Parameter { index, transform, precision } => match args.get(usize::from(index)) {
                Some(Argument::String(str)) => output.push_str(str),
                Some(Argument::Number(mut number)) => {
                    number = transform.apply(number);
                    let result = match precision {
                        None => write!(output, "{number}"),
                        Some(precision) => {
                            write!(output, "{number:precision$}", precision = precision as usize)
                        }
                    };
                    if let Err(fmt::Error) = result {
                        output.push_str("{FMT_ERR}");
                    }
                }
                None => output.push_str("{NO_ARG}"),
            },
            Self::MatchNumber { param_index, ref arms, ref default } => {
                let Some(&Argument::Number(number)) = args.get(usize::from(param_index)) else {
                    output.push_str("{NO_ARG}");
                    return;
                };

                let elements = arms
                    .iter()
                    .find(|&arm| arm.condition.evaluate(number))
                    .map_or(default, |arm| &arm.then);

                for element in elements {
                    element.render(args, output);
                }
            }
        }
    }
}

/// A match arm for [`Element::MatchNumber`].
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub struct NumberMatchArm {
    /// The condition evaluated with the matched value to determine if the match arm is executed.
    pub condition: NumberPredicate,
    /// The content to evaluate if the condition is matched.
    pub then:      Vec<Element>,
}

/// A predicate to determine for a number.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum NumberPredicate {
    /// Evaluates to true if and only if all items evaluate to true.
    ///
    /// Always evaluates to true if the list is empty.
    All {
        ///Items of conjunction.
        items: Vec<NumberPredicate>,
    },
    /// Evaluates to false if and only if all items evaluate to false.
    ///
    /// Always evaluates to false if the list is empty.
    Any {
        /// Items of disjunction.
        items: Vec<NumberPredicate>,
    },
    /// Inverts the base predicate.
    Not {
        /// Base predicate to be inverted.
        base: Box<NumberPredicate>,
    },
    /// Evaluates the base condition on the input transformed by `transform`.
    Transformed {
        /// The modulus to evaluate.
        transform: NumberTransform,
        /// The base condition to evaluate on the remainder.
        base:      Box<NumberPredicate>,
    },
    /// Evaluates to true if input `operator` `operand` is true.
    ///
    /// Float equality may be used if
    /// the input number is converted from a source integer of 32-bits or below.
    Range {
        /// The operator used to compare the input with `operand`.
        operator: NumberRangeOperator,
        /// The operand to compare with.
        operand:  f64,
    },
}

impl NumberPredicate {
    /// Evaluates the predicate for a number input.
    #[must_use]
    pub fn evaluate(&self, number: f64) -> bool {
        match *self {
            Self::All { ref items } => items.iter().all(|item| item.evaluate(number)),
            Self::Any { ref items } => items.iter().any(|item| item.evaluate(number)),
            Self::Not { ref base } => !base.evaluate(number),
            Self::Transformed { transform, ref base } => base.evaluate(transform.apply(number)),
            #[allow(clippy::float_cmp)] // intended for integer comparison
            Self::Range { operator, operand } => match operator {
                NumberRangeOperator::Equal => number == operand,
                NumberRangeOperator::NotEqual => number != operand,
                NumberRangeOperator::Lt => number < operand,
                NumberRangeOperator::Lte => number <= operand,
                NumberRangeOperator::Gt => number > operand,
                NumberRangeOperator::Gte => number >= operand,
            },
        }
    }
}

/// A transformation that takes a number and outputs a number.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum NumberTransform {
    /// Returns the input.
    #[default]
    Identity,
    /// Takes the reciprocal of the input.
    Reciprocal,
    /// Takes the absolute value of the input.
    Abs,
    /// Clips the number to a minimum value,
    /// i.e. `max(input, value)`.
    AtLeast(f64),
    /// Clips the number to a maximum value,
    /// i.e. `min(input, value)`.
    AtMost(f64),
    /// Adds a number to the input.
    Add(f64),
    /// Subtracts a number from the input.
    Sub(f64),
    /// Multiplies a number to the input.
    Mul(f64),
    /// Divides the input by a number.
    Div(f64),
    /// Divides the input by a number and takes the remainder.
    Rem(f64),
    /// Rounds the input.
    Round {
        /// Determines the precision of rounding.
        #[serde(default)]
        precision: RoundingPrecision,
        /// Specifies the direction of rounding.
        #[serde(default)]
        mode:      RoundingMode,
    },
}

impl NumberTransform {
    /// Applies the transformation on an input.
    #[must_use]
    pub fn apply(self, number: f64) -> f64 {
        match self {
            Self::Identity => number,
            Self::Reciprocal => number.recip(),
            Self::Abs => number.abs(),
            Self::AtLeast(value) => number.max(value),
            Self::AtMost(value) => number.min(value),
            Self::Add(value) => number + value,
            Self::Sub(value) => number - value,
            Self::Mul(value) => number * value,
            Self::Div(value) => number / value,
            Self::Rem(value) => number % value,
            Self::Round { precision, mode } => {
                match number.classify() {
                    FpCategory::Zero => return 0.0,
                    FpCategory::Nan | FpCategory::Infinite => return number,
                    _ => {}
                }
                if number == 0.0 {
                    return 0.0;
                }

                let unit = match precision {
                    RoundingPrecision::MultipleOf { unit } => unit,
                    RoundingPrecision::Decimal { places } => 10f64.powi(-places),
                    RoundingPrecision::SigFig { figures } => {
                        #[allow(clippy::cast_possible_truncation)]
                        // log10 of positive number must be within i32 range
                        let exponent = number.abs().log10().floor() as i32;
                        10f64.powi(exponent.saturating_sub_unsigned(figures.saturating_sub(1)))
                    }
                };

                let ratio = number / unit;
                let fract = ratio.fract();
                if fract == 0.0 {
                    return number;
                }

                let towards_zero = ratio.trunc() * unit;
                let towards_inf = (ratio.trunc() + 1.) * unit;

                let (towards_positive, towards_negative) = if number > 0.0 {
                    (towards_inf, towards_zero)
                } else {
                    (towards_zero, towards_inf)
                };

                match mode {
                    RoundingMode::HalfInf => {
                        if fract.abs() >= 0.5 {
                            towards_inf
                        } else {
                            towards_zero
                        }
                    }
                    RoundingMode::HalfCeil => {
                        if fract >= 0.5 || fract < 0.5 {
                            towards_inf
                        } else {
                            towards_zero
                        }
                    }
                    RoundingMode::Ceil => towards_positive,
                    RoundingMode::Floor => towards_negative,
                    RoundingMode::Zero => towards_zero,
                    RoundingMode::Inf => towards_inf,
                }
            }
        }
    }
}

/// Precision of rounding.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum RoundingPrecision {
    /// Rounds to a multiple of a non-zero value.
    MultipleOf {
        /// Output is a multiple of `unit`.
        unit: f64,
    },
    /// Rounds to a multiple of `pow(10, -value)`.
    Decimal {
        /// Number of decimal places.
        ///
        /// Negative value indicates the number of trailing digits to round to zero.
        places: i32,
    },
    /// Rounds to `value` significant figures in decimal form.
    SigFig {
        /// Number of significant figures.
        figures: u32,
    },
}

impl Default for RoundingPrecision {
    fn default() -> Self { Self::MultipleOf { unit: 1. } }
}

/// Type of rounding.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum RoundingMode {
    /// Rounds to the nearest unit, preferring the one towards infinity.
    #[default]
    HalfInf,
    /// Rounds to the nearest unit, preferring the one towards positive infinity.
    HalfCeil,
    /// Rounds to the nearest unit towards positive infinity.
    Ceil,
    /// Rounds to the nearest unit towards negative infinity.
    Floor,
    /// Rounds to the nearest unit towards zero.
    Zero,
    /// Rounds to the nearest unit towards infinity.
    Inf,
}

/// Operators for numeric range comparison.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum NumberRangeOperator {
    /// `==`
    Equal,
    /// `!=`
    NotEqual,
    /// `<`
    Lt,
    /// `<=`
    Lte,
    /// `>`
    Gt,
    /// `>=`
    Gte,
}

/// Provider for translation glossaries.
pub trait Provider {
    /// Gets a glossary by sha if synchronously available.
    fn get(&self, sha: GlossarySha) -> Option<impl Deref<Target = Glossary> + '_>;
}
