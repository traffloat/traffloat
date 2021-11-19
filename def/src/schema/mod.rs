use arcstr::ArcStr;
use smallvec::SmallVec;
use xylem::{declare_schema, DefaultContext, SchemaExt, Xylem};

use crate::{lang, CustomizableName};

declare_schema! {
    /// The tfsave-builder schema
    pub Schema: SchemaExt
}

macro_rules! impl_xylem_for_self {
    ($($ty:ty),* $(,)?) => {
        $(
            #[cfg(feature = "xy")]
            impl xylem::Xylem<$crate::Schema> for $ty {
                type From = Self;
                type Args = xylem::NoArgs;

                fn convert_impl(from: Self, _: &mut xylem::DefaultContext, _: &Self::Args) -> anyhow::Result<Self> {
                    Ok(from)
                }
            }
        )*
    }
}

#[cfg(feature = "xy")]
mod stdext;

impl_xylem_for_self!(ArcStr);

#[cfg(feature = "xy")]
impl<A: smallvec::Array + 'static> Xylem<Schema> for SmallVec<A>
where
    <A as smallvec::Array>::Item: Xylem<Schema>,
{
    type From = Vec<<<A as smallvec::Array>::Item as Xylem<Schema>>::From>;
    type Args = <<A as smallvec::Array>::Item as Xylem<Schema>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut DefaultContext,
        args: &Self::Args,
    ) -> anyhow::Result<Self> {
        from.into_iter()
            .map(|item| {
                <<A as smallvec::Array>::Item as Xylem<Schema>>::convert(item, context, args)
            })
            .collect()
    }
}

/// Identifies a definition of type `T`.
///
/// See [`xylem::Id`].
pub type Id<T> = xylem::Id<Schema, T>;

/// The original string identifier of a definition.
///
/// Typically encoded in kebab-case,
/// the string ID is used only in persistent strings independent of the scenario,
/// such as in URLs for rule browsing.
pub type IdString<T> = xylem::IdString<Schema, T>;

#[cfg(feature = "xy")]
mod transform_matrix;

#[cfg(feature = "xy")]
mod units;

impl_xylem_for_self!(traffloat_types::geometry::Unit);

impl_xylem_for_self!(traffloat_types::space::Position);
impl_xylem_for_self!(traffloat_types::space::Vector);
impl_xylem_for_self!(nalgebra::Vector2<f64>);

#[cfg(feature = "xy")]
impl Xylem<Schema> for CustomizableName {
    type From = <lang::Item as Xylem<Schema>>::From;
    type Args = <lang::Item as Xylem<Schema>>::Args;

    fn convert_impl(
        from: Self::From,
        context: &mut DefaultContext,
        args: &Self::Args,
    ) -> Result<Self, <Schema as xylem::Schema>::Error> {
        let item = lang::Item::convert(from, context, args)?;
        Ok(CustomizableName::Original(item))
    }
}

#[cfg(feature = "xy")]
pub mod curdir;
