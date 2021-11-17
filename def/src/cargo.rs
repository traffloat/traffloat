//! Cargo definitions.

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::atlas::IconRef;
use crate::{lang, IdString};

/// Identifies a cargo type.
pub type Id = crate::Id<Def>;

impl_identifiable!(Def);

/// A type of cargo.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Def {
    /// ID of the cargo type.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:          Id,
    /// String ID of the cargo type.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:      IdString<Def>,
    /// Name of the cargo type.
    #[getset(get = "pub")]
    name:        lang::Item,
    /// Short summary of the cargo type.
    #[getset(get = "pub")]
    summary:     lang::Item,
    /// Long description of the cargo type.
    #[getset(get = "pub")]
    description: lang::Item,
    /// Category of the cargo type.
    #[getset(get_copy = "pub")]
    category:    category::Id,
    /// The texture of the cargo.
    #[getset(get = "pub")]
    texture:     IconRef,
}

/// Categories of cargo.
pub mod category {
    use getset::{CopyGetters, Getters};
    use serde::{Deserialize, Serialize};

    use crate::{lang, IdString};

    /// Identifies a cargo type.
    pub type Id = crate::Id<Def>;

    impl_identifiable!(Def);

    /// A category of cargo.
    #[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize)]
    #[cfg_attr(feature = "xy", derive(xylem::Xylem))]
    #[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
    pub struct Def {
        /// ID of the cargo category.
        #[getset(get_copy = "pub")]
        #[cfg_attr(feature = "xy", xylem(args(new = true)))]
        id:          Id,
        /// String ID of the cargo category.
        #[getset(get = "pub")]
        #[cfg_attr(feature = "xy", xylem(serde(default)))]
        id_str:      IdString<Def>,
        /// Title of the cargo category.
        #[getset(get = "pub")]
        title:       lang::Item,
        /// Description of the cargo category.
        #[getset(get = "pub")]
        description: lang::Item,
    }
}
