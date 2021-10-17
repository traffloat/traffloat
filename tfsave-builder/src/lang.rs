//! Language file processing

use std::fs;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};
use codegen::ResolveContext;
use fluent::{FluentBundle, FluentResource};
use fluent_syntax::parser::ParserError;
use itertools::Itertools;
use traffloat_def::lang;
use unic_langid::LanguageIdentifier;

use crate::Timer;

/// Setup [`ResolveContext`] to validate language file references.
pub(crate) fn setup_context(
    context: &mut ResolveContext,
    input: &Path,
    output: &Path,
    lang_parse_timer: &Rc<Timer>,
) {
    let dir = input.parent().expect("Regular file has no parent").to_owned();

    context.add_listener::<lang::Def>(Rc::new({
        let lang_parse_timer = Rc::clone(lang_parse_timer);
        move |def: &lang::DefHumanFriendly, context| {
            for (lang, file) in def.languages() {
                let lang: LanguageIdentifier =
                    lang.parse().with_context(|| format!("Invalid language name {}", lang))?;

                fn parse_ftl(path: &Path, lang_parse_timer: &Timer) -> Result<FluentResource> {
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

                    Ok(resource)
                }

                fn format_err(file: &str, err: &ParserError) -> String {
                    let line = file[0..err.pos.start].chars().filter(|&c| c == '\n').count();
                    let col = match file[0..err.pos.start].rfind('\n') {
                        Some(pos) => err.pos.start - pos,
                        None => err.pos.start,
                    };
                    format!("{} on line {}:{}", err.kind, line + 1, col)
                }

                let path = dir.join(file.as_str());
                let res = parse_ftl(&path, &lang_parse_timer)
                    .with_context(|| format!("Reading translation file {}", path.display()))?;

                let mut bundle = FluentBundle::new(vec![lang.clone()]);
                bundle.add_resource(res).expect("Only one resource is added to the bundle");

                let id = context
                    .resolve_id::<lang::Def>(def.id())
                    .expect("Listeners are triggered after ID notification");

                {
                    let mut bundles = context.get_other::<lang::LoadedBundles>();
                    bundles.add(id, lang, bundle);
                }
            }

            Ok(())
        }
    }));
}
