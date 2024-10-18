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
    pub fn to_string(&self) -> String {
        let mut output = String::new();
        self.render(&mut output);
        output
    }
}

impl Default for DisplayText {
    fn default() -> Self { Self::Concat { children: Vec::new() } }
}
