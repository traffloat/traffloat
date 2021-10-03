//! Styles for yew

use std::borrow::Cow;

use once_cell::sync::Lazy;
use yew::html::IntoPropValue;

/// A set of styles for an element.
pub struct Style {
    /// The actual rules of the style, without duplicate keys
    ///
    /// Do not modify directly; use [`push_rules`] instead.
    #[doc(hidden)]
    pub rules: Vec<(&'static str, Cow<'static, str>)>,
    string:    Lazy<String, Box<dyn FnOnce() -> String + Send>>,
}

impl Style {
    /// Creates a new Style.
    ///
    /// Prefer the [`style`] macro instead.
    pub fn new(rules: Vec<(&'static str, Cow<'static, str>)>) -> Self {
        Self { rules: rules.clone(), string: Lazy::new(Box::new(string_lazy(rules))) }
    }

    /// Add rules to this style.
    ///
    /// Note that this method clones the whole rules vector,
    /// and therefore has O(n) time complexity.
    pub fn push_rules(
        &mut self,
        rules: impl IntoIterator<Item = (&'static str, Cow<'static, str>)>,
    ) {
        self.rules.extend(rules);
        self.string = Lazy::new(Box::new(string_lazy(self.rules.clone())));
    }
}

fn string_lazy(rules: Vec<(&'static str, Cow<'static, str>)>) -> impl FnOnce() -> String + Send {
    move || {
        use std::fmt::Write;

        let mut string = String::new();
        for (key, value) in rules {
            write!(&mut string, "{}: {};", key, value).expect("String::write_fmt cannot fail");
        }
        string
    }
}

impl Clone for Style {
    fn clone(&self) -> Self {
        Self {
            rules:  self.rules.clone(),
            string: Lazy::new(Box::new(string_lazy(self.rules.clone()))),
        }
    }
}

macro_rules! style {
    (static $ident:ident = $(..$base:expr,)* $($name:literal: $value:expr),* $(,)?) => {
        ::lazy_static::lazy_static! {
            static ref $ident: $crate::style::Style = $crate::style::Style::new({
                use ::std::borrow::Cow;
                use ::std::collections::BTreeMap;

                let mut rules = BTreeMap::<&'static str, Cow<'static, str>>::new();
                $(
                    for (key, value) in &$base.rules {
                        rules.insert(*key, value.clone());
                    }
                 )*
                    $(
                        rules.insert($name, Cow::from($value));
                     )*
                    rules.into_iter().collect()
            });
        }
    };
    ($(..$base:expr,)* $($name:literal: $value:expr),* $(,)?) => {
        {
            style!{static STYLE = $(..$base,)* $($name: $value),*}
            &*STYLE
        }
    }
}

impl IntoPropValue<Option<Cow<'static, str>>> for &'static Style {
    fn into_prop_value(self) -> Option<Cow<'static, str>> {
        let string: &'static String = &*self.string;
        Some(Cow::Borrowed(string.as_str()))
    }
}

/// A wrapper for a non-static style.
pub struct NonStaticStyle(pub Style);

impl IntoPropValue<Option<Cow<'static, str>>> for NonStaticStyle {
    fn into_prop_value(self) -> Option<Cow<'static, str>> {
        let string: String = self.0.string.clone();
        Some(Cow::Owned(string))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::BTreeMap;

    use super::Style;

    #[test]
    pub fn test_style() {
        style! { static A =
            "foo": "bar",
            "qux": "corge",
        };
        assert_eq!(
            maplit::btreemap! {
                "foo" => Cow::Borrowed("bar"),
                "qux" => Cow::Borrowed("corge"),
            },
            A.rules.clone().into_iter().collect::<BTreeMap<_, _>>()
        );

        let b = style! {
            ..A,
            "grault": "waldo",
            "qux": "fred",
        };
        assert_eq!(
            maplit::btreemap! {
                "foo" => Cow::Borrowed("bar"),
                "grault" => Cow::Borrowed("waldo"),
                "qux" => Cow::Borrowed("fred"),
            },
            b.rules.clone().into_iter().collect::<BTreeMap<_, _>>()
        );
    }
}
