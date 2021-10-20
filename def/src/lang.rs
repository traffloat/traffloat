//! Language bundle definitions

use std::collections::BTreeMap;
use std::path::PathBuf;

use arcstr::ArcStr;
use codegen::{Definition, ResolveContext};
use fluent::{FluentBundle, FluentResource};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

/// A bundle of language files.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Definition)]
pub struct Def {
    /// Identifies the language bundle.
    #[getset(get_copy = "pub")]
    id:        Id,
    /// Paths to language files.
    #[getset(get = "pub")]
    languages: BTreeMap<ArcStr, PathBuf>,
}

/// A translatable message template.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Definition)]
#[hf_post_convert(validate_lang)]
pub struct Item {
    /// The language bundle to use.
    #[getset(get_copy = "pub")]
    src: Id,
    /// The key for the string in the language bundle.
    #[getset(get = "pub")]
    key: ArcStr,
}

/// A cache of loaded language bundles.
#[cfg(feature = "convert-human-friendly")]
#[derive(Default)]
pub struct LoadedBundles(BTreeMap<Id, Vec<(LanguageIdentifier, FluentBundle<FluentResource>)>>);

#[cfg(feature = "convert-human-friendly")]
impl LoadedBundles {
    /// Add a localized language bundle.
    pub fn add(
        &mut self,
        id: usize,
        language: LanguageIdentifier,
        bundle: FluentBundle<FluentResource>,
    ) {
        let vec = self.0.entry(Id(id)).or_default();
        vec.push((language, bundle));
    }
}

#[cfg(feature = "convert-human-friendly")]
fn validate_lang(item: &mut Item, context: &mut ResolveContext) -> anyhow::Result<()> {
    use anyhow::Context;

    {
        let lb = context.get_other::<LoadedBundles>();
        let bundles = lb.0.get(&item.src).context("Undefined translation bundle reference")?;

        for (lang, bundle) in bundles {
            if bundle.get_message(item.key()).is_none() {
                anyhow::bail!("Undefined translation key {} in locale {}", item.key(), lang);
            }
        }
    }

    Ok(())
}
