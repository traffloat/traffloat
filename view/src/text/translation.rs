//! Translation data.

use std::fmt;
use std::ops::Deref;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Argument;
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
    /// Echos a parameter to the output.
    Parameter {
        /// Index of the parameter.
        ///
        /// The actual value is expected to be an [`Argument::String`].
        index: u16,
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
            Self::Parameter { index } => match args.get(usize::from(index)) {
                Some(Argument::String(str)) => output.push_str(str),
                Some(Argument::Number(number)) => {
                    if let Err(fmt::Error) = write!(output, "{number}") {
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
    /// Evaluates the base condition on the input modulo `modulus`.
    Modulo {
        /// The modulus to evaluate.
        modulus: f64,
        /// The base condition to evaluate on the remainder.
        base:    Box<NumberPredicate>,
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
            Self::Modulo { modulus, ref base } => base.evaluate(number % modulus),
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
