use std::any::TypeId;
use std::collections::{hash_map, HashMap};

use anyhow::Context as _;
use serde::Deserialize;
use xias::Xias;
use xylem::{Context as _, Xylem};

use crate::Schema;

/// An expression that resolves to a number.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Expression {
    /// A literal number.
    Literal(f64),
    /// References a variable.
    Variable(String),
    /// A binary operation.
    BinaryOperation {
        /// The binary operation type.
        op:            BinaryOperator,
        /// The left operand.
        left:          Box<Expression>,
        /// The right operand.
        right:         Box<Expression>,
        /// The fallback value if the output is a non-finite value.
        #[serde(default)]
        on_non_finite: Option<Box<Expression>>,
    },
    /// Clamps the value of an expression to a range.
    ///
    /// Returns the `output` closest to `clamp` such that `at_least <= output <= at_most`.
    Clamp {
        /// The base expression.
        clamp:    Box<Expression>,
        /// The lower bound.
        #[serde(default)]
        at_least: Option<Box<Expression>>,
        /// The upper bound.
        #[serde(default)]
        at_most:  Option<Box<Expression>>,
    },
    /// Rounds a value to the nearest multiple of `unit`.
    Round {
        /// The base expression.
        round:     Box<Expression>,
        /// The rounding unit. The output is a multiple of this number.
        #[serde(default = "expression_one")]
        unit:      Box<Expression>,
        /// The rounding mode.
        #[serde(default)]
        mode:      RoundMode,
        /// If `from_zero` is true, all rounding modes are flipped when `round` is negative.
        #[serde(default)]
        from_zero: bool,
    },
    /// Takes the absolute value.
    Abs {
        /// The base expression.
        abs: Box<Expression>,
    },
}

/// Returns a literal 1, used for default values.
fn expression_one() -> Box<Expression> { Box::new(Expression::Literal(1.)) }

impl Expression {
    pub fn resolve(&self, context: &mut xylem::DefaultContext) -> anyhow::Result<f64> {
        let output = match self {
            &Self::Literal(value) => value,
            Self::Variable(var_name) => {
                let err = || format!("Variable {} not yet defined at this point", var_name);
                let var_map = context.get::<VarMap>(TypeId::of::<()>()).with_context(err)?;
                *var_map.map.get(var_name).with_context(err)?
            }
            Self::BinaryOperation { op, left, right, on_non_finite } => {
                let left = left.resolve(context).context("Error resolving left operand")?;
                let right = right.resolve(context).context("Error resolving right operand")?;
                let mut output = op.operate(left, right);

                if !output.is_finite() {
                    if let Some(fallback) = on_non_finite {
                        output =
                            fallback.resolve(context).context("Error resolving fallback value")?;
                    }
                }

                output
            }
            Self::Clamp { clamp, at_least, at_most } => {
                let mut value = clamp.resolve(context).context("Error resolving clamped value")?;

                if let Some(at_least) = at_least {
                    let bound =
                        at_least.resolve(context).context("Error resolving at_least bound")?;
                    value = value.max(bound);
                }

                if let Some(at_most) = at_most {
                    let bound =
                        at_most.resolve(context).context("Error resolving at_most bound")?;
                    value = value.min(bound);
                }

                value
            }
            &Self::Round { ref round, ref unit, mode, from_zero } => {
                let round = round.resolve(context).context("Error resolving rounded value")?;
                let unit = unit.resolve(context).context("Error resolving rounding unit")?;
                let ratio = round / unit;
                let mut modulus = ratio % 1.;
                if modulus < 0. {
                    if from_zero {
                        // -0.9 => 0.9
                        modulus *= -1.;
                    } else {
                        // -0.9 => 0.1
                        modulus += 1.;
                    }
                }
                match (mode.is_up(modulus), from_zero) {
                    (true, true) => {
                        // 1.5 => 2, -1.5 => -2
                        let sign = if ratio > 0. {
                            1.
                        } else if ratio < 0. {
                            -1.
                        } else {
                            0.
                        };
                        ratio.trunc() + sign
                    }
                    (true, false) => {
                        // 1.5 => 2, -1.5 => -1
                        ratio.ceil()
                    }
                    (false, true) => {
                        // 1.5 => 1, -1.5 => -1
                        ratio.trunc()
                    }
                    (false, false) => {
                        // 1.5 => 1, -1.5 => -2
                        ratio.floor()
                    }
                }
            }
            Self::Abs { abs } => {
                abs.resolve(context).context("Error resolving abs base value")?.abs()
            }
        };

        anyhow::ensure!(output.is_finite(), "Expression produces non-finite value {}", output);

        Ok(output)
    }
}

/// A binary operation type.
#[derive(Clone, Copy, Deserialize)]
pub enum BinaryOperator {
    /// `left + right`
    #[serde(alias = "+")]
    Add,
    /// `left - right`
    #[serde(alias = "-")]
    Sub,
    /// `left * right`
    #[serde(alias = "*")]
    Mul,
    /// `left / right`
    #[serde(alias = "/")]
    Div,
    /// `left % right`. `0 <= output < right` is always true, even if `left < 0`.
    #[serde(alias = "%")]
    Mod,
    /// `left` raised to the power of `right`, i.e. `left.powf(right)`
    #[serde(alias = "^")]
    #[serde(alias = "**")]
    Pow,
}

impl BinaryOperator {
    fn operate(&self, left: f64, right: f64) -> f64 {
        match self {
            Self::Add => left + right,
            Self::Sub => left - right,
            Self::Mul => left * right,
            Self::Div => left / right,
            Self::Mod => {
                let mut modulus = left % right;
                if modulus < 0. {
                    modulus += right;
                }
                modulus
            }
            Self::Pow => left.powf(right),
        }
    }
}

/// Rounding mode.
#[derive(Clone, Copy, Deserialize)]
pub enum RoundMode {
    /// `0 <= x < 0.5` down, `0.5 <= x <= 1` up
    ///
    /// Note that floating point error is possible.
    HalfUp,
    /// `0 <= x <= 0.5` down, `0.5 < x <= 1` up
    ///
    /// Note that floating point error is possible.
    HalfDown,
    /// `0 < x <= 1` up
    Ceil,
    /// `0 <= x < 1` down
    Floor,
}
impl Default for RoundMode {
    fn default() -> Self { Self::HalfUp }
}

impl RoundMode {
    fn is_up(&self, modulus: f64) -> bool {
        match self {
            Self::HalfUp => modulus >= 0.5,
            Self::HalfDown => modulus > 0.5,
            Self::Ceil => modulus > 0.,
            Self::Floor => modulus >= 1.,
        }
    }
}

impl Xylem<Schema> for f64 {
    type From = Expression;
    type Args = xylem::NoArgs;

    fn convert_impl(
        from: Expression,
        context: &mut xylem::DefaultContext,
        &xylem::NoArgs: &xylem::NoArgs,
    ) -> anyhow::Result<Self> {
        from.resolve(context)
    }
}

impl Xylem<Schema> for f32 {
    type From = Expression;
    type Args = xylem::NoArgs;

    fn convert_impl(
        from: Expression,
        context: &mut xylem::DefaultContext,
        &xylem::NoArgs: &xylem::NoArgs,
    ) -> anyhow::Result<Self> {
        let value = from.resolve(context)?;
        Ok(value.lossy_float())
    }
}

impl Xylem<Schema> for i32 {
    type From = Expression;
    type Args = xylem::NoArgs;

    fn convert_impl(
        from: Expression,
        context: &mut xylem::DefaultContext,
        &xylem::NoArgs: &xylem::NoArgs,
    ) -> anyhow::Result<Self> {
        let value = from.resolve(context)?.round();
        anyhow::ensure!(value.is_finite(), "expression resolves to non-finite value {}", value);
        anyhow::ensure!(Self::MIN as f64 <= value, "{:?} is too small to fit into i32", value);
        anyhow::ensure!(Self::MAX as f64 >= value, "{:?} is too large to fit into i32", value);
        Ok(value as Self)
    }
}

impl Xylem<Schema> for u32 {
    type From = Expression;
    type Args = xylem::NoArgs;

    fn convert_impl(
        from: Expression,
        context: &mut xylem::DefaultContext,
        &xylem::NoArgs: &xylem::NoArgs,
    ) -> anyhow::Result<Self> {
        let value = from.resolve(context)?.round();
        anyhow::ensure!(value.is_finite(), "expression resolves to non-finite value {}", value);
        anyhow::ensure!(Self::MIN as f64 <= value, "{:?} is too small to fit into u32", value);
        anyhow::ensure!(Self::MAX as f64 >= value, "{:?} is too large to fit into u32", value);
        Ok(value as Self)
    }
}

/// The map of variable values.
#[derive(Default)]
struct VarMap {
    map: HashMap<String, f64>,
}

/// Defines a variable.
pub struct Variable;

#[derive(Deserialize)]
pub struct VariableXylem {
    /// The name of the variable.
    name: String,
    /// The expression that provides the value of the variable.
    expr: Expression,
}

impl Xylem<Schema> for Variable {
    type From = VariableXylem;
    type Args = xylem::NoArgs;

    fn convert_impl(
        from: VariableXylem,
        context: &mut xylem::DefaultContext,
        &xylem::NoArgs: &Self::Args,
    ) -> anyhow::Result<Self> {
        let value = from
            .expr
            .resolve(context)
            .with_context(|| format!("Error resolving value of {}", &from.name))?;
        let var_map = context.get_mut::<VarMap, _>(TypeId::of::<()>(), VarMap::default);
        match var_map.map.entry(from.name) {
            hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
            }
            hash_map::Entry::Occupied(entry) => {
                anyhow::bail!("Variable {} was already defined", entry.key())
            }
        }
        Ok(Self)
    }
}
