//! Language file processing

use std::any::TypeId;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context as _, Result};
use fluent::{FluentBundle, FluentResource};
use fluent_syntax::parser::ParserError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use traffloat_def::curdir::CurrentDir;
use traffloat_def::lang::{self, ListenerHook};
use unic_langid::LanguageIdentifier;
use xylem::{Context as _, DefaultContext};

use crate::Timer;

/// A data structure containing the raw FTL data for copying.
#[derive(Default, Serialize, Deserialize)]
struct RawList(BTreeMap<String, Vec<String>>);

/// Setup [`DefaultContext`] to validate language file references.
pub(crate) fn setup_context(context: &mut DefaultContext, lang_parse_timer: &Rc<Timer>) {
    let hook: Rc<ListenerHook> = Rc::new({
        let lang_parse_timer = Rc::clone(lang_parse_timer);
        move |def, context| {
            for (lang, file) in def.languages() {
                let lang: LanguageIdentifier =
                    lang.parse().with_context(|| format!("Invalid language name {}", lang))?;

                fn parse_ftl(
                    path: &Path,
                    lang_parse_timer: &Timer,
                ) -> Result<(FluentResource, String)> {
                    let path =
                        fs::canonicalize(path).context("Canonicalizing translation file path")?;
                    let file = fs::read_to_string(path).context("Reading translation file")?;

                    let resource = {
                        lang_parse_timer.start();
                        match FluentResource::try_new(file.clone()) {
                            Ok(res) => res,
                            Err((_, err)) => anyhow::bail!(
                                "Error parsing FTL: {}",
                                err.into_iter().map(|err| format_err(&file, &err)).join(", ")
                            ),
                        }
                    };

                    Ok((resource, file))
                }

                fn format_err(file: &str, err: &ParserError) -> String {
                    let line = file[0..err.pos.start].chars().filter(|&c| c == '\n').count();
                    let col = match file[0..err.pos.start].rfind('\n') {
                        Some(pos) => err.pos.start - pos,
                        None => err.pos.start,
                    };
                    format!("{} on line {}:{}", err.kind, line + 1, col)
                }

                let path = context
                    .get::<CurrentDir>(TypeId::of::<()>())
                    .expect("CurrentDir was not initialized")
                    .path()
                    .join(file);
                let (res, raw) = parse_ftl(&path, &lang_parse_timer)
                    .with_context(|| format!("Reading translation file {}", path.display()))?;

                let mut bundle = FluentBundle::new(vec![lang.clone()]);
                bundle.add_resource(res).expect("Only one resource is added to the bundle");

                {
                    let bundles = context.get_mut::<lang::xy::LoadedBundles, _>(
                        TypeId::of::<()>(),
                        Default::default,
                    );
                    bundles.add(def.id(), lang.clone(), bundle);
                }

                {
                    let list = context.get_mut::<RawList, _>(TypeId::of::<()>(), Default::default);
                    let vec = list.0.entry(lang.to_string()).or_default();
                    assert!(vec.len() == def.id().index());
                    vec.push(raw);
                }
            }

            Ok(())
        }
    });
    context.get_mut::<lang::Listener, _>(TypeId::of::<()>(), move || lang::Listener { hook });
}

/// Save translation output.
pub(crate) fn save(output: &Path, context: &mut DefaultContext) -> Result<()> {
    let list = context.get::<RawList>(TypeId::of::<()>()).expect("No language files included");
    for (lang, vec) in &list.0 {
        let path = output.join(lang.as_str()).with_extension("tflang");
        let mut bytes = rmp_serde::to_vec(&vec).context("Encoding translations")?;
        bytes.splice(0..0, b"\xFF\0TFLANG".iter().copied());
        fs::write(path, bytes).context("Writing tflang file")?;
    }
    Ok(())
}
