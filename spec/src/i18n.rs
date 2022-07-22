//! Translations management.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::Schema;

/// A translatable string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18n<Arg: Argument = ()> {
    locales: BTreeMap<ArcStr, ArcStr>,
    _ph:     PhantomData<fn(Arg)>,
}

impl<Arg: Argument> Xylem<Schema> for I18n<Arg> {
    type From = BTreeMap<String, String>;
    type Args = xylem::NoArgs;

    fn convert_impl(
        _from: Self::From,
        _context: &mut xylem::DefaultContext,
        &xylem::NoArgs: &xylem::NoArgs,
    ) -> anyhow::Result<Self> {
        todo!()
    }
}

/// The type used to resolve an [`I18n`] template.
pub trait Argument: 'static {}

impl Argument for () {}
