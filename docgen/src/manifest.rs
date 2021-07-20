use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Serialize, Serializer};

#[derive(Serialize)]
pub struct Mkdocs {
    pub site_name: &'static str,
    pub site_url: String,
    pub use_directory_urls: bool,
    pub site_author: &'static str,
    pub repo_url: &'static str,
    pub repo_name: &'static str,
    pub edit_uri: &'static str,
    pub copyright: &'static str,
    pub theme: Theme,
    pub markdown_extensions: &'static [&'static str],
    pub nav: Vec<Nav>,
}

#[derive(Serialize)]
pub struct Theme {
    pub name: &'static str,
    pub favicon: String,
    pub logo: String,
    pub features: &'static [&'static str],
    pub palette: serde_json::Value,
}

pub enum Nav {
    Index { title: String, items: Vec<Nav> },
    Path(PathBuf),
}

impl Serialize for Nav {
    fn serialize<S: Serializer>(&self, se: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Index { title, items } => {
                let mut map = BTreeMap::new();
                map.insert(title, items);
                map.serialize(se)
            }
            Self::Path(path) => path.serialize(se),
        }
    }
}
