use std::collections::{btree_map, BTreeMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub struct Pool {
    pub map: BTreeMap<String, String>,
    pub dest: PathBuf,
    pub sub: String,
}

impl Pool {
    pub fn new(dest: PathBuf, sub: String) -> Result<Self> {
        let dest = dest.join(&sub);
        fs::create_dir_all(&dest).context("Could not create assets dir")?;
        Ok(Self {
            map: BTreeMap::new(),
            dest,
            sub,
        })
    }
}

impl Pool {
    pub fn map(&mut self, path: &Path) -> Result<String> {
        let count = self.map.len();
        Ok(match self.map.entry(path.display().to_string()) {
            btree_map::Entry::Vacant(entry) => {
                let dest = self
                    .dest
                    .join(count.to_string())
                    .with_extension(path.extension().unwrap_or(Default::default()));
                fs::copy(path, &dest).context("Could not copy to dest")?;
                let name = dest
                    .file_name()
                    .expect("Asset file has no name")
                    .to_string_lossy();
                entry.insert(format!("{}/{}", &self.sub, name)).clone()
            }
            btree_map::Entry::Occupied(entry) => entry.get().clone(),
        })
    }
}
