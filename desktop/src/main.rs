use std::path::{Path, PathBuf};
use std::{fs, vec};

use anyhow::{Context, Result};
use traffloat_client::{edge, node, Config, Event, Server};
use traffloat_def::{AnyDef, TfsaveFile};

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mock = Static::load_tfsave("client/gen/scenarios/vanilla/scenario.tfsave")?;

    let config = Config::default();

    traffloat_client::run(mock, config).map_err(|err| anyhow::anyhow!("{}", err))
}

#[derive(derive_new::new)]
struct Static {
    vec:         vec::IntoIter<Event>,
    context_dir: PathBuf,
}

impl Static {
    fn load_tfsave(path: impl AsRef<Path>) -> Result<Self> {
        let buf = fs::read(path.as_ref())?;
        let file = TfsaveFile::parse(&buf)?;
        Self::from_tfsave(file, path.as_ref().parent().context("Path does not have parent")?)
    }

    fn from_tfsave(file: TfsaveFile, context_dir: &Path) -> Result<Self> {
        let mut events = Vec::new();

        for node in file.state().nodes() {
            let building = file
                .def()
                .iter()
                .find_map(|def| match def {
                    AnyDef::Building(building) if building.id() == node.building() => {
                        Some(building)
                    }
                    _ => None,
                })
                .context("Dangling building reference")?;

            events.push(Event::AddNode(node::View {
                id:       node.id(),
                position: node.position(),
                shapes:   building.shapes().clone(),
                color:    [1., 1., 1.],
            }));
        }

        for edge in file.state().edges() {
            events.push(Event::AddEdge(edge::View {
                id:     edge.endpoints(),
                radius: edge.radius(),
                color:  [0.2, 0.5, 0.9, 0.8],
            }))
        }

        Ok(Self::new(events.into_iter(), context_dir.to_path_buf()))
    }
}

impl Server for Static {
    fn receive(&mut self) -> Result<Option<Event>> { Ok(self.vec.next()) }

    fn load_asset(&self, path: &str) -> String {
        self.context_dir.join(path).to_str().expect("Non-UTF8 paths not supported").to_owned()
    }
}
