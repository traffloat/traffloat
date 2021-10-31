//! Texture atlas definitions

use std::path::PathBuf;

use arcstr::ArcStr;
use codegen::{Definition, IdStr};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::geometry;

/// A texture atlas.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Definition)]
#[hf_post_convert(populate_atlas_context)]
pub struct Def {
    /// Identifies the atlas.
    #[getset(get_copy = "pub")]
    id:       Id,
    /// String ID of the atlas.
    #[getset(get = "pub")]
    id_str:   IdStr,
    /// The directory containing the variants.
    ///
    /// In tfsave-builder mode, `dir` contains the original files instead.
    #[getset(get = "pub")]
    dir:      PathBuf,
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

/// References a named icon sprite.
#[derive(Debug, Clone, Serialize, Deserialize, CopyGetters)]
pub struct IconRef {
    /// The ID of the sprite file in the assets folder.
    ///
    /// The path of the sprite file relative to the tfsave file is `./assets/{variant_name}/{sprite_id}.png`.
    #[getset(get_copy = "pub")]
    sprite_id: u32,
}

/// References a named model spritesheet.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Getters, CopyGetters)]
pub struct ModelRef {
    /// The ID of the model spritesheet in the assets folder.
    ///
    /// The path of the sprite file relative to the tfsave file is `./assets/{variant_name}/{spritesheet_id}.png`.
    /// The position of sprites in a model spritesheet file is defined by the shape.
    #[getset(get_copy = "pub")]
    spritesheet_id: u32,
    /// The shape of the model spritesheet.
    #[getset(get_copy = "pub")]
    shape:          geometry::Unit,
}

#[cfg(feature = "convert-human-friendly")]
mod hf {
    use std::convert::TryFrom;
    use std::rc::Rc;

    use anyhow::Context;
    use codegen::ResolveContext;

    use super::*;

    type AtlasCreationHook = dyn Fn(&mut Def, &mut codegen::ResolveContext) -> anyhow::Result<()>;

    /// An interface for tfsave-builder to inject its atlas creation hook
    /// to trigger texture building.
    #[derive(Default)]
    pub struct AtlasContext {
        /// A function executed when a new atlas is defined.
        ///
        /// This must be set by the context setting up the [`codegen::ResolveContext`].
        pub creation_hook: Option<Rc<AtlasCreationHook>>,
    }

    pub(super) fn populate_atlas_context(
        def: &mut Def,
        context: &mut ResolveContext,
    ) -> anyhow::Result<()> {
        let hook = {
            let atlas_context = context.get_other::<AtlasContext>();
            let hook = atlas_context.creation_hook.as_ref();
            let hook = hook.expect("Context did not initialize atlas creation hook");
            Rc::clone(hook)
        };
        hook(def, context)
    }

    /// An index used to resolve icon sprite IDs, stored in [`codegen::ResolveContext`].
    #[derive(Default)]
    pub struct IconIndex(std::collections::BTreeMap<(Id, ArcStr), u32>);

    impl IconIndex {
        /// Adds a sprite definition.
        pub fn add(&mut self, id: Id, name: ArcStr, texture_id: u32) {
            self.0.insert((id, name), texture_id);
        }
    }

    impl Definition for IconRef {
        type HumanFriendly = IconRefHumanFriendly;

        fn convert(
            hf: Self::HumanFriendly,
            context: &mut codegen::ResolveContext,
        ) -> anyhow::Result<Self> {
            let src_id = context
                .resolve_id::<Def>(hf.src.as_str())
                .with_context(|| format!("Undefined atlas reference: {}", &hf.src))?;
            let src_id = u32::try_from(src_id).context("Too many items")?;
            let entry = {
                let index = context.get_other::<IconIndex>();
                index.0.get(&(Id(src_id), hf.name.clone())).copied()
            };
            let sprite_id = match entry {
                Some(value) => value,
                None => anyhow::bail!("Undefined icon reference: {}/{}", &hf.src, &hf.name),
            };
            Ok(Self { sprite_id })
        }
    }

    /// Human-friendly schema for [`IconRef`].
    #[derive(Serialize, Deserialize)]
    pub struct IconRefHumanFriendly {
        src:  ArcStr,
        name: ArcStr,
    }

    /// An index used to resolve spritesheet IDs, stored in [`codegen::ResolveContext`].
    #[derive(Default)]
    pub struct ModelIndex(std::collections::BTreeMap<(Id, ArcStr), (u32, geometry::Unit)>);

    impl ModelIndex {
        /// Adds a spritesheet definition.
        pub fn add(&mut self, id: Id, name: ArcStr, texture_id: u32, shape: geometry::Unit) {
            self.0.insert((id, name), (texture_id, shape));
        }
    }

    /// Human-friendly schema for [`ModelRef`].
    #[derive(Serialize, Deserialize)]
    pub struct ModelRefHumanFriendly {
        src:  ArcStr,
        name: ArcStr,
    }

    impl Definition for ModelRef {
        type HumanFriendly = ModelRefHumanFriendly;

        fn convert(
            hf: Self::HumanFriendly,
            context: &mut codegen::ResolveContext,
        ) -> anyhow::Result<Self> {
            let src_id = context
                .resolve_id::<Def>(hf.src.as_str())
                .with_context(|| format!("Undefined atlas reference: {}", &hf.src))?;
            let entry = {
                let index = context.get_other::<ModelIndex>();
                let src_id = u32::try_from(src_id).context("Too many items")?;
                index.0.get(&(Id(src_id), hf.name.clone())).copied()
            };
            let (spritesheet_id, shape) = match entry {
                Some(value) => value,
                None => anyhow::bail!("Undefined model reference: {}/{}", &hf.src, &hf.name),
            };

            Ok(ModelRef { spritesheet_id, shape })
        }
    }
}

#[cfg(feature = "convert-human-friendly")]
use hf::populate_atlas_context;
#[cfg(feature = "convert-human-friendly")]
pub use hf::*;

/// Converts variant + sprite ID into asset path.
///
/// The return format is in the form `assets/variant/01234567.png`.
pub fn to_path(variant: &str, sprite_id: u32) -> String {
    format!("assets/{}/{:08x}.png", variant, sprite_id)
}
