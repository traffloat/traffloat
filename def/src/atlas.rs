//! Texture atlas definitions

use arcstr::ArcStr;
use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

/// A texture atlas.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Definition)]
pub struct Def {
    /// Identifies the atlas.
    #[getset(get_copy = "pub")]
    id:       Id,
    /// The directory containing the variants.
    ///
    /// In tfsave-builder mode, `dir` contains the original files instead.
    #[getset(get = "pub")]
    dir:      ArcStr,
    /// Different variants of the atlas.
    #[getset(get = "pub")]
    variants: Vec<Variant>,
}

/// A variant (by resolution) of the atlas.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Definition)]
pub struct Variant {
    /// The name of this variant.
    #[getset(get = "pub")]
    name:      ArcStr,
    /// The dimension of each sprite.
    #[getset(get_copy = "pub")]
    dimension: u32,
}

/// References a named sprite set in an atlas.
///
/// While this type is called "sprite", it typically references multiple sprites for the same
/// object, e.g. different faces of a cube.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Definition)]
pub struct Sprite {
    /// The atlas holding the sprite.
    #[getset(get_copy = "pub")]
    src:  Id,
    /// Name of the sprite.
    #[getset(get = "pub")]
    name: ArcStr,
}
