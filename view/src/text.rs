use core::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A string visible to user without rich formatting.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[allow(clippy::module_name_repetitions)]
pub enum DisplayText {
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
    pub fn render(&self, output: &mut String) {
        match self {
            Self::Custom { value } => output.push_str(value),
            Self::Concat { children } => {
                for child in children {
                    child.render(output);
                }
            }
        }
    }

    /// Formats the output as a string.
    ///
    /// Signature may change in the future when we support i18n.
    #[must_use]
    pub fn render_to_string(&self) -> String {
        let mut output = String::new();
        self.render(&mut output);
        output
    }

    /// Concise debug formatting.
    #[must_use]
    pub fn short_debug(&self) -> impl fmt::Display + '_ {
        struct Wrapper<'a>(&'a DisplayText);

        impl<'a> fmt::Display for Wrapper<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.0 {
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
