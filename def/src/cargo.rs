//! Cargo definitions.

use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::atlas::Sprite;
use crate::lang;

/// A type of cargo.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize, Definition)]
pub struct Def {
    /// ID of the cargo type.
    #[getset(get_copy = "pub")]
    id:          Id,
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
    #[getset(get = "pub")]
    category:    category::Id,
    /// The texture of the cargo.
    #[getset(get = "pub")]
    texture:     Sprite,
}

/// Categories of cargo.
pub mod category {
    use codegen::Definition;
    use getset::{CopyGetters, Getters};
    use serde::{Deserialize, Serialize};

    use crate::lang;

    /// A category of cargo.
    #[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
    pub struct Def {
        /// ID of the cargo type.
        #[getset(get_copy = "pub")]
        id:          Id,
        /// Title of the cargo category.
        #[getset(get = "pub")]
        title:       lang::Item,
        /// Description of the cargo category.
        #[getset(get = "pub")]
        description: lang::Item,
    }
}