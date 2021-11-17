//! Current-directory context resources.

use std::any::TypeId;
use std::path::PathBuf;

use derive_new::new;
use getset::{Getters, Setters};
use xylem::{Context, DefaultContext, Xylem};

use crate::Schema;

/// Stores the context directory.
#[derive(new, Getters, Setters)]
pub struct CurrentDir {
    /// The path to the context directory.
    #[getset(get = "pub", set = "pub")]
    path: PathBuf,
}

impl Xylem<Schema> for PathBuf {
    type From = Self;
    type Args = PathArgs;

    fn convert_impl(
        from: Self::From,
        context: &mut DefaultContext,
        args: &Self::Args,
    ) -> anyhow::Result<Self> {
        let mut path = context
            .get::<CurrentDir>(TypeId::of::<()>())
            .expect("CurrentDir resource not set up")
            .path()
            .clone();

        if let Some(prefix) = &args.prefix {
            path.push(prefix);
        }

        path.push(from);

        Ok(path)
    }
}

/// Xylem arguments for [`PathBuf`].
#[derive(Default)]
pub struct PathArgs {
    /// The part between context directory and the value path.
    pub prefix: Option<&'static str>,
}
