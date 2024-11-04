use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod translation;

/// A string visible to user without rich formatting.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[allow(clippy::module_name_repetitions)]
pub enum DisplayText {
    /// A translated template from a translation bundle.
    Template {
        /// Reference to translation bundle by its SHA1 hash.
        sha:   translation::GlossarySha,
        /// Index of template entry in the translation bundle.
        index: u16,
    },
    /// An arbitrary display string.
    Custom {
        /// The constant value.
        value: String,
    },
    // Resource(String), // TODO load resource string from locale
    /// Concatenation of multiple display nodes.
    Concat {
        /// List of child nodes, concatenated directly.
        children: Vec<Self>,
    },
}

impl DisplayText {
    /// Renders the display text to `output`.
    ///
    /// Signature may change in the future when we support i18n.
    pub fn render(
        &self,
        pack_provider: &impl translation::Provider,
        args: &[Argument],
        output: &mut String,
    ) {
        match self {
            &Self::Template { sha, index } => match pack_provider.get(sha) {
                Some(pack) => match pack.entries.get(usize::from(index)) {
                    Some(entry) => entry.render(args, output),
                    None => output.push_str("{BAD_REF}"),
                },
                None => output.push_str("{BAD_REF}"),
            },
            Self::Custom { value } => output.push_str(value),
            Self::Concat { children } => {
                for child in children {
                    child.render(pack_provider, args, output);
                }
            }
        }
    }

    /// Formats the output as a string.
    ///
    /// Signature may change in the future when we support i18n.
    #[must_use]
    pub fn render_to_string(
        &self,
        pack_provider: &impl translation::Provider,
        args: &[Argument],
    ) -> String {
        let mut output = String::new();
        self.render(pack_provider, args, &mut output);
        output
    }

    /// Concise debug formatting.
    #[must_use]
    pub fn short_debug(&self) -> impl fmt::Display + '_ {
        struct Wrapper<'a>(&'a DisplayText);

        impl<'a> fmt::Display for Wrapper<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.0 {
                    DisplayText::Template { sha, index } => {
                        write!(f, "{}#{index}", hex::encode(sha.0))
                    }
                    DisplayText::Custom { value } => write!(f, "{value}"),
                    DisplayText::Concat { children } => {
                        for child in children {
                            fmt::Display::fmt(&child.short_debug(), f)?;
                        }
                        Ok(())
                    }
                }
            }
        }

        Wrapper(self)
    }
}

impl Default for DisplayText {
    fn default() -> Self { Self::Concat { children: Vec::new() } }
}

/// Arguments passed to resolve a display text.
pub enum Argument<'a> {
    /// A string argument.
    String(&'a str),
    /// A numeric argument.
    Number(f64),
}
