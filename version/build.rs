use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{Context, Result};
use quote::quote;

fn main() -> Result<()> {
    let path = PathBuf::from(env_var("OUT_DIR")?).join("out.rs");
    let mut out = fs::File::create(path).context("Open out.rs")?;

    let semver = env_var("CARGO_PKG_VERSION")?;

    let manifest_dir = PathBuf::from(env_var("CARGO_MANIFEST_DIR")?);
    let workspace = manifest_dir.parent().context("Could not detect cargo workspace")?;

    let (git_data, full_version) = match git_data(workspace) {
        Ok(data) => {
            let mut full_version = format!(
                "{}{}",
                semver,
                match (&data.sha, &data.branch) {
                    (Some(sha), Some(branch)) => format!("-{}@{}", branch, &sha[..7]),
                    _ => String::new(),
                }
            );
            if data.dirty {
                full_version.push_str("+dirty");
            }
            (data, full_version)
        }
        Err(err) => {
            eprintln!("Failed to load git: {err:?}");
            (GitData { sha: None, branch: None, dirty: true }, format!("{semver}-gitless"))
        }
    };

    let git_rev = option_tokens(git_data.sha.as_deref());
    let git_branch = option_tokens(git_data.branch.as_deref());
    let git_dirty = git_data.dirty;

    writeln!(
        &mut out,
        "{}",
        quote! {
            /// Semver version string of the crates.
            pub const SEMVER: &str = #semver;

            /// The git revision SHA.
            pub const GIT_REVISION: Option<&str> = #git_rev;

            /// The git branch name, or [`None`] if detached.
            pub const GIT_BRANCH: Option<&str> = #git_branch;

            /// Whether the git workspace is dirty.
            pub const GIT_IS_DIRTY: bool = #git_dirty;

            /// The human-readable full version string.
            pub const VERSION: &str = #full_version;
        }
    )
    .context("Write out.rs")?;

    println!("cargo:rerun-if-changed={}", workspace.join(".git/HEAD").display());

    Ok(())
}

fn env_var(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("Fetching env var ${name}"))
}

fn git_data(path: &Path) -> Result<GitData> {
    let repo = git2::Repository::open(path).context("Opening workspace git repository")?;
    let head = repo.head()?;
    let head_direct = head.resolve()?;
    let sha = head_direct.target().map(hex::encode);

    let branch = head.shorthand().map(str::to_owned);

    let dirty = repo
        .diff_index_to_workdir(None, None)
        .context("Computing git diff")?
        .deltas()
        .next()
        .is_some();

    Ok(GitData { sha, branch, dirty })
}

struct GitData {
    sha:    Option<String>,
    branch: Option<String>,
    dirty:  bool,
}

fn option_tokens(os: Option<&str>) -> proc_macro2::TokenStream {
    match os {
        Some(str) => quote!(Some(#str)),
        None => quote!(None),
    }
}
