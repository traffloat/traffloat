//! Building definitions

use typed_builder::TypedBuilder;

use super::reaction;

/// Identifies a building category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub usize);

/// A type of building.
#[derive(TypedBuilder, getset::CopyGetters, getset::Getters)]
pub struct Type {
    /// Name of the building type.
    #[getset(get = "pub")]
    name: String,
    /// Short summary of the building type.
    #[getset(get = "pub")]
    summary: String,
    /// Long description of the building type.
    #[getset(get = "pub")]
    description: String,
    /// Category of the building type.
    #[getset(get_copy = "pub")]
    category: CategoryId,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: String,
    /// Reactions associated with the building.
    #[getset(get = "pub")]
    reactions: Vec<(reaction::TypeId, ReactionPolicy)>,
    /// Extra features associated with the building.
    #[getset(get = "pub")]
    features: Vec<ExtraFeature>,
}

/// Whether the reaction can be configured by players
/// and the behaviour when inputs underflow or outputs overflow.
#[derive(TypedBuilder, getset::CopyGetters)]
#[builder(field_defaults(default))]
pub struct ReactionPolicy {
    #[get_copy = "pub"]
    configurable: bool,
    #[get_copy = "pub"]
    on_underflow: FlowPolicy,
    #[get_copy = "pub"]
    on_overflow: FlowPolicy,
}

#[derive(Debug, Clone, Copy)]
pub enum FlowPolicy {
    /// Reduce the rate of reaction such that the input/output capacity is just enough.
    ReduceRate,
}

impl Default for FlowPolicy {
    fn default() -> Self {
        Self::ReduceRate
    }
}

/// Extra features of a building
#[derive(Debug, Clone)]
pub enum ExtraFeature {
    /// The building is a core and must not be destroyed.
    Core,
    /// The building provides housing capacity, and inhabitants can be assigned to it.
    ProvidesHousing(u32),
}

/// Identifies a building category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategoryId(pub usize);

/// A category of building.
#[derive(TypedBuilder, getset::Getters)]
pub struct Category {
    /// Title of the building category.
    #[getset(get = "pub")]
    title: String,
    /// Description of the building category.
    #[getset(get = "pub")]
    description: String,
}
