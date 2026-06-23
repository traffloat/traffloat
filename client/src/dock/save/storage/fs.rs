use std::collections::BTreeMap;
use std::path::PathBuf;
use std::{fs, mem};

use bevy::ecs::world::{CommandQueue, World};
use egui_notify::Toast;
use jiff::Timestamp;
use traffloat_physics::try_log;

use crate::dock::save::storage::Entry;
use crate::dock::{self};

struct FileSystemBackend {
    root:     PathBuf,
    index:    BTreeMap<String, Entry>,
    commands: CommandQueue,
}

impl FileSystemBackend {
    fn path_for_name(&self, name: &str) -> PathBuf {
        let mut path = self.root.join(name);
        path.add_extension("tfsave");
        path
    }

    fn push_toast(&mut self, toast: Toast) {
        self.commands.push(move |world: &mut World| {
            let mut toasts = world.resource_mut::<dock::Toasts>();
            toasts.0.add(toast);
        });
    }
}

impl Default for super::Storage {
    fn default() -> Self {
        let root =
            dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("traffloat").join("saves");
        if let Err(err) = fs::create_dir_all(&root) {
            tracing::error!("Failed to create saves directory \"{root:?}\": {err}");
        }

        let mut ret = super::Storage(Box::new(FileSystemBackend {
            root,
            index: BTreeMap::new(),
            commands: CommandQueue::default(),
        }));
        ret.0.reload();
        ret
    }
}

impl super::Backend for FileSystemBackend {
    fn flush(&mut self) -> CommandQueue { mem::take(&mut self.commands) }

    fn reload(&mut self) {
        // TODO move to background

        self.index.clear();

        let files =
            try_log!(fs::read_dir(&self.root), expect "Failed to read saves directory" or return);
        for file in files {
            let file = try_log!(file, expect "Failed to read save directory" or return);
            let name = file.file_name();
            let name = try_log!(name.to_str(), expect "file names should be UTF-8" or continue);
            let Some(name) = name.strip_suffix(".tfsave") else { continue };
            let metadata =
                try_log!(file.metadata(), expect "read metadata of file \"{name}\"" or continue);
            let modified = try_log!(metadata.modified(), expect "read modified time of file \"{name}\"" or continue);
            let modified = try_log!(Timestamp::try_from(modified), expect "modified time should be within bounds" or continue);
            let size = metadata.len();
            self.index.insert(name.to_string(), Entry { name: name.to_string(), modified, size });
        }
    }

    fn is_name_used(&mut self, name: &str) -> bool {
        // TODO handle case sensitivity
        self.index.contains_key(name)
    }

    fn submit_save(&mut self, name: &str, data: &[u8]) {
        // TODO move to background
        let path = self.path_for_name(name);
        if let Err(err) = fs::write(&path, data) {
            self.push_toast(Toast::error(format!(
                "Failed to write save file \"{}\": {err}",
                path.display(),
            )));
        }
        self.reload();
    }

    fn list_entries(&mut self) -> Vec<Entry> { self.index.values().cloned().collect() }

    fn load_file(&mut self, name: &str, callback: Box<dyn FnOnce(&mut World, &[u8]) + Send>) {
        // TODO move to background

        let path = self.path_for_name(name);
        match fs::read(&path) {
            Ok(data) => {
                self.commands.push(move |world: &mut World| callback(world, &data[..]));
            }
            Err(err) => {
                self.push_toast(Toast::error(format!(
                    "Failed to read save file \"{}\": {err}",
                    path.display(),
                )));
            }
        }
    }
}
