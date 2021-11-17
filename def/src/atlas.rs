//! Texture atlas definitions

use std::path::PathBuf;

use arcstr::ArcStr;
use derive_new::new;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::geometry;

use crate::{IdString, Schema};

/// Identifies a directory of source sprites.
pub type Id = crate::Id<Def>;

impl_identifiable!(Def);

/// A directory of source sprites.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), process))]
pub struct Def {
    /// Identifies the directory.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:       Id,
    /// String ID of the atlas.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:   IdString<Def>,
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
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Variant {
    /// The name of this variant.
    #[getset(get = "pub")]
    name:      ArcStr,
    /// The dimension of each sprite.
    #[getset(get_copy = "pub")]
    dimension: u32,
}

/// Identifies a spritesheet in the scenario scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpritesheetId(u32);

impl SpritesheetId {
    /// Creates a new sprite ID.
    pub fn new(id: u32) -> Self { Self(id) }

    /// Expresses the sprite ID as an integer.
    pub fn value(&self) -> u32 { self.0 }
}

/// References a named icon sprite.
#[derive(Debug, Clone, Serialize, Deserialize, CopyGetters)]
pub struct IconRef {
    /// The ID of the sprite file in the assets folder.
    ///
    /// The path of the sprite file relative to the tfsave file is `./assets/{variant_name}/{spritesheet_id}.png`.
    #[getset(get_copy = "pub")]
    spritesheet_id: SpritesheetId,
}

/// References a named model spritesheet.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Getters, CopyGetters)]
pub struct ModelRef {
    /// The ID of the model spritesheet in the assets folder.
    ///
    /// The path of the sprite file relative to the tfsave file is `./assets/{variant_name}/{spritesheet_id}.png`.
    /// The position of sprites in a model spritesheet file is defined by the shape.
    #[getset(get_copy = "pub")]
    spritesheet_id: SpritesheetId,
    /// The shape of the model spritesheet.
    #[getset(get_copy = "pub")]
    shape:          geometry::Unit,
}

/// Xylem-specific objects.
#[cfg(feature = "xy")]
pub mod xy {
    use std::any::TypeId;
    use std::borrow::Borrow;
    use std::cmp;
    use std::rc::Rc;

    use anyhow::Context as _;
    use xylem::{Context as _, DefaultContext, IdArgs, NoArgs, Processable, Xylem};

    use super::*;

    /// See [`AtlasContext::creation_hook`]
    pub type AtlasCreationHook = dyn Fn(&mut Def, &mut DefaultContext) -> anyhow::Result<()>;

    /// An interface for tfsave-builder to inject its atlas creation hook
    /// to trigger texture building.
    #[derive(new)]
    pub struct AtlasContext {
        /// A function executed when a new atlas is defined.
        ///
        /// This must be set when setting up the [`DefaultContext`].
        pub creation_hook: Rc<AtlasCreationHook>,
    }

    impl Processable<Schema> for Def {
        fn postprocess(&mut self, context: &mut DefaultContext) -> anyhow::Result<()> {
            let atlas_context = context
                .get::<AtlasContext>(TypeId::of::<()>())
                .expect("Context did not initialize atlas creation hook");
            let hook = Rc::clone(&atlas_context.creation_hook);
            hook(self, context)
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct IdName {
        id:   Id,
        name: String,
    }

    struct IdNameRef<'t> {
        id:   Id,
        name: &'t str,
    }

    trait AbstractIdName {
        fn id(&self) -> Id;
        fn name(&self) -> &str;
    }

    impl<'t> Borrow<dyn AbstractIdName + 't> for IdName {
        fn borrow(&self) -> &(dyn AbstractIdName + 't) { self }
    }

    impl<'t> Borrow<dyn AbstractIdName + 't> for IdNameRef<'t> {
        fn borrow(&self) -> &(dyn AbstractIdName + 't) { self }
    }

    impl AbstractIdName for IdName {
        fn id(&self) -> Id { self.id }

        fn name(&self) -> &str { self.name.as_str() }
    }

    impl<'t> AbstractIdName for IdNameRef<'t> {
        fn id(&self) -> Id { self.id }

        fn name(&self) -> &str { self.name }
    }

    impl PartialEq for dyn AbstractIdName + '_ {
        fn eq(&self, other: &Self) -> bool {
            self.id() == other.id() && self.name() == other.name()
        }
    }
    impl Eq for dyn AbstractIdName + '_ {}
    impl PartialOrd for dyn AbstractIdName + '_ {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> { Some(self.cmp(other)) }
    }
    impl Ord for dyn AbstractIdName + '_ {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            (self.id().cmp(&other.id())).then_with(|| self.name().cmp(other.name()))
        }
    }

    /// An index used to resolve icon sprite IDs, stored as a context resource in the root scope.
    #[derive(Default)]
    pub struct IconIndex(std::collections::BTreeMap<IdName, SpritesheetId>);

    impl IconIndex {
        /// Adds a sprite definition.
        pub fn add(&mut self, id: Id, name: String, spritesheet_id: SpritesheetId) {
            self.0.insert(IdName { id, name }, spritesheet_id);
        }

        /// Resolves a sprite name into the sprite ID.
        pub fn get(&self, id: Id, name: &str) -> Option<SpritesheetId> {
            self.0.get::<dyn AbstractIdName + '_>(&IdNameRef { id, name }).copied()
        }
    }

    /// Human-friendly schema for [`IconRef`].
    #[derive(Serialize, Deserialize)]
    pub struct IconRefXylem {
        src:  String,
        name: String,
    }

    impl Xylem<Schema> for IconRef {
        type From = IconRefXylem;
        type Args = NoArgs;

        fn convert_impl(
            from: Self::From,
            context: &mut DefaultContext,
            _: &NoArgs,
        ) -> anyhow::Result<Self> {
            let src_id = Id::convert(from.src.clone(), context, &IdArgs::default())
                .with_context(|| format!("Undefined atlas reference: {}", &from.src))?;
            let entry = {
                let index = context.get_mut::<IconIndex, _>(TypeId::of::<()>(), Default::default);
                index.get(src_id, from.name.as_str())
            };
            let spritesheet_id = match entry {
                Some(value) => value,
                None => anyhow::bail!("Undefined icon reference: {}/{}", &from.src, &from.name),
            };
            Ok(Self { spritesheet_id })
        }
    }

    /// An index used to resolve spritesheet IDs, stored as a context resource in the root scope..
    #[derive(Default)]
    pub struct ModelIndex(std::collections::BTreeMap<IdName, ModelRef>);

    impl ModelIndex {
        /// Adds a spritesheet definition.
        pub fn add(
            &mut self,
            id: Id,
            name: String,
            spritesheet_id: SpritesheetId,
            shape: geometry::Unit,
        ) {
            self.0.insert(IdName { id, name }, ModelRef { spritesheet_id, shape });
        }

        /// Resolves a model name into the model ID.
        fn get(&self, id: Id, name: &str) -> Option<ModelRef> {
            self.0.get::<dyn AbstractIdName + '_>(&IdNameRef { id, name }).copied()
        }
    }

    /// Human-friendly schema for [`ModelRef`].
    #[derive(Serialize, Deserialize)]
    pub struct ModelRefXylem {
        src:  String,
        name: ArcStr,
    }

    impl Xylem<Schema> for ModelRef {
        type From = ModelRefXylem;
        type Args = NoArgs;

        fn convert_impl(
            from: Self::From,
            context: &mut DefaultContext,
            _: &NoArgs,
        ) -> anyhow::Result<Self> {
            let src_id = Id::convert(from.src.clone(), context, &IdArgs::default())
                .with_context(|| format!("Undefined atlas reference: {}", &from.src))?;
            let entry = {
                let index = context.get_mut::<ModelIndex, _>(TypeId::of::<()>(), Default::default);
                index.get(src_id, from.name.as_str())
            };

            let model_ref = match entry {
                Some(value) => value,
                None => anyhow::bail!("Undefined model reference: {}/{}", &from.src, &from.name),
            };

            Ok(model_ref)
        }
    }
}

/// Converts variant + sprite ID into asset path.
///
/// The return format is in the form `assets/variant/01234567.png`.
pub fn to_path(variant: &str, spritesheet_id: SpritesheetId) -> String {
    format!("assets/{}/{:08x}.png", variant, spritesheet_id.value())
}
