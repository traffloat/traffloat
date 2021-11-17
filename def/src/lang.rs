//! Language bundle definitions

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use arcstr::ArcStr;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use xylem::DefaultContext;

use crate::IdString;

/// Identifies a language bundle.
pub type Id = crate::Id<Def>;

impl_identifiable!(Def);

/// See [`Listener::hook`]
pub type ListenerHook = dyn Fn(&Def, &mut DefaultContext) -> anyhow::Result<()>;

/// A listener for new language bundles.
pub struct Listener {
    /// A function executed when a new language bundle is resolved.
    pub hook: Rc<ListenerHook>,
}

/// A bundle of language files.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(expose = DefXylem, derive(Deserialize), process))]
pub struct Def {
    /// Identifies the language bundle.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:        Id,
    /// String ID of the language bundle.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:    IdString<Def>,
    /// Paths to language files.
    #[getset(get = "pub")]
    languages: BTreeMap<ArcStr, PathBuf>,
}

/// A translatable message template.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), process))]
pub struct Item {
    /// The language bundle to use.
    #[getset(get_copy = "pub")]
    src: Id,
    /// The key for the string in the language bundle.
    #[getset(get = "pub")]
    key: ArcStr,
}

#[cfg(feature = "yew")]
impl yew::html::ImplicitClone for Item {}

/// Xylem-specific objects.
#[cfg(feature = "xy")]
pub mod xy {
    use std::any::TypeId;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use fluent::{FluentBundle, FluentResource};
    use unic_langid::LanguageIdentifier;
    use xylem::{Context, DefaultContext, Processable};

    use super::{Def, Id, Item, Listener};
    use crate::Schema;

    impl Processable<Schema> for Def {
        fn postprocess(&mut self, context: &mut DefaultContext) -> anyhow::Result<()> {
            let hook = context.get::<Listener>(TypeId::of::<()>()).expect("listener was not setup");
            let hook = Rc::clone(&hook.hook);
            hook(self, context)
        }
    }

    /// A cache of loaded language bundles.
    #[derive(Default)]
    pub struct LoadedBundles(BTreeMap<Id, Vec<(LanguageIdentifier, FluentBundle<FluentResource>)>>);

    impl LoadedBundles {
        /// Add a localized language bundle.
        pub fn add(
            &mut self,
            id: Id,
            language: LanguageIdentifier,
            bundle: FluentBundle<FluentResource>,
        ) {
            let vec = self.0.entry(id).or_default();
            vec.push((language, bundle));
        }
    }

    impl Processable<Schema> for Item {
        fn postprocess(&mut self, context: &mut DefaultContext) -> anyhow::Result<()> {
            use anyhow::Context;

            {
                let lb = context
                    .get::<LoadedBundles>(TypeId::of::<()>())
                    .context("No language bundles loaded yet")?;
                let bundles = lb.0.get(&self.src).with_context(|| {
                    format!("Dangling translation bundle reference {:?}", self.src())
                })?;

                for (lang, bundle) in bundles {
                    if bundle.get_message(self.key()).is_none() {
                        anyhow::bail!(
                            "Undefined translation key {} in locale {}",
                            self.key(),
                            lang
                        );
                    }
                }
            }

            Ok(())
        }
    }
}
