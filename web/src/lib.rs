use std::{panic, vec};

use anyhow::{Context, Result};
use traffloat_client::{edge, node, Config, Event, Server};
use traffloat_def::{AnyDef, TfsaveFile};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(module = "/js/load.js")]
extern "C" {
    async fn load_file(path: &str) -> JsValue;
}

#[wasm_bindgen(start)]
pub async fn main() { start().await.expect("crash"); }

async fn start() -> Result<()> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    panic::set_hook(Box::new(|info| {
        web_sys::window()
            .expect("not running with DOM")
            .alert_with_message(&format!("{}", info))
            .expect("Cannot alert");
    }));

    let value = load_file("/gen/scenarios/vanilla/scenario.tfsave").await;
    let value: &js_sys::Uint8Array =
        value.dyn_ref().context("load_file did not return Uint8Array")?;

    let mock = Static::load_tfsave("/gen/scenarios/vanilla", &value.to_vec())?;

    let config = Config::default();

    traffloat_client::run(mock, config).map_err(|err| anyhow::anyhow!("{:?}", err))
}

#[derive(derive_new::new)]
struct Static {
    vec:         vec::IntoIter<Event>,
    context_dir: String,
}

impl Static {
    fn load_tfsave(context_dir: &str, buf: &[u8]) -> Result<Self> {
        let file = TfsaveFile::parse(&buf)?;

        Self::from_tfsave(file, context_dir)
    }

    fn from_tfsave(file: TfsaveFile, context_dir: &str) -> Result<Self> {
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

        Ok(Self::new(events.into_iter(), context_dir.to_string()))
    }
}

impl Server for Static {
    fn receive(&mut self) -> Result<Option<Event>> { Ok(self.vec.next()) }

    fn load_asset(&self, path: &str) -> String { format!("{}/{}", &self.context_dir, path) }
}
