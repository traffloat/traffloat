//! Defines the imperative schema.

/// The style of definition file.
pub trait Style {
}

/// Compact style, used in production
pub struct Compact(());

impl Style for Compact {

}

/// Development style, used for Git-based development flow.
pub trait Dev(());

impl Style for Dev {

}
