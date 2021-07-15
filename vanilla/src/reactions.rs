//! Vanilla mechanism definitions.

/// Defines a reaction type.
pub struct Def {
    /// Name of the reaction.
    pub name: &'static str,
    /// Category of the reaction.
    pub category: Category,
    /// Catalysts for the reaction.
    pub catalysts: &'static [Catalyst],
    /// Inputs and outputs for the reaction.
    pub puts: &'static [Put],
}

/// A catalyst for a reaction.
pub enum Catalyst {}

/// An input or output for a reaction.
pub enum Put {}

/// Category of the reactions, only for display purpose.
#[derive(Clone, Copy, PartialEq, strum::EnumIter, strum::Display)]
#[strum(serialize_all = "title_case")]
pub enum Category {
    Population,
}

/// All reaction types.
pub const ALL: &[Def] = &[];
