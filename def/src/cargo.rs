//! Cargo definitions.

use gusket::Gusket;
use serde::{Deserialize, Serialize};

use crate::atlas::IconRef;
use crate::{lang, IdString};

/// Identifies a cargo type.
pub type Id = crate::Id<Def>;

impl_identifiable!(Def);

/// A type of cargo.
#[derive(Debug, Clone, Gusket, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
#[gusket(all, immut)]
pub struct Def {
    /// ID of the cargo type.
    #[gusket(copy)]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:          Id,
    /// String ID of the cargo type.
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:      IdString<Def>,
    /// Name of the cargo type.
    name:        lang::Item,
    /// Short summary of the cargo type.
    summary:     lang::Item,
    /// Long description of the cargo type.
    description: lang::Item,
    /// Category of the cargo type.
    #[gusket(copy)]
    category:    category::Id,
    /// The texture of the cargo.
    #[gusket(copy)]
    texture:     IconRef,
}

/// Categories of cargo.
pub mod category {
    use gusket::Gusket;
    use serde::{Deserialize, Serialize};

    use crate::{lang, IdString};

    /// Identifies a cargo type.
    pub type Id = crate::Id<Def>;

    impl_identifiable!(Def);

    /// A category of cargo.
    #[derive(Debug, Clone, Gusket, Serialize, Deserialize)]
    #[cfg_attr(feature = "xy", derive(xylem::Xylem))]
    #[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
    pub struct Def {
        /// ID of the cargo category.
        #[gusket(copy)]
        #[cfg_attr(feature = "xy", xylem(args(new = true)))]
        id:          Id,
        /// String ID of the cargo category.
        #[cfg_attr(feature = "xy", xylem(serde(default)))]
        id_str:      IdString<Def>,
        /// Title of the cargo category.
        title:       lang::Item,
        /// Description of the cargo category.
        description: lang::Item,
    }
}
