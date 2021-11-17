use std::ops::Range;

use xylem::Xylem;

use crate::Schema;

impl<T: Xylem<Schema>> Xylem<Schema> for Range<T> {
    type From = Self;
    type Args = xylem::NoArgs;

    fn convert_impl(
        from: Self,
        _: &mut xylem::DefaultContext,
        _: &Self::Args,
    ) -> anyhow::Result<Self> {
        Ok(from)
    }
}

impl_xylem_for_self!(bool);
impl_xylem_for_self!(u32);
impl_xylem_for_self!(i32);
impl_xylem_for_self!(f64);
