//! Saving game definition and state.

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::clock::Clock;
use crate::def::GameDefinition;
use crate::edge::save::Edge;
use crate::node::save::Node;
use crate::time::Instant;
use crate::SetupEcs;

/// The save schema version.
///
/// This value is only bumped when necessary to distinguish incompatible formats.
pub const SCHEMA_VERSION: u32 = 0;

/// The schema for a `.tsvt`/`.tsvb` file.
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    def: GameDefinition,
    state: GameState,
}

/// The state of the game.
#[derive(Serialize, Deserialize)]
pub struct GameState {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    clock: Instant,
}

/// Requests saving game state.
#[derive(TypedBuilder, getset::CopyGetters)]
pub struct Request {
    /// The format to save as.
    #[getset(get_copy = "pub")]
    format: Format,
}

/// The format to save as.
#[derive(Debug, Clone, Copy)]
pub enum Format {
    /// Text-based save format.
    ///
    /// Currently, this format uses [`serde_yaml`] as backend.
    #[cfg(feature = "serde_yaml")]
    Text,
    /// Binary-based save format.
    ///
    /// Currently, this format uses [`rmp-serde`] as backend.
    #[cfg(feature = "rmp-serde")]
    Binary,
}

/// Result for saving game state.
#[derive(getset::Getters, getset::CopyGetters)]
pub struct Response {
    /// The format of this response.
    #[getset(get_copy = "pub")]
    format: Format,
    /// The raw result data.
    #[getset(get = "pub")]
    data: Vec<u8>,
}

#[codegen::system]
fn save(
    #[subscriber] requests: impl Iterator<Item = Request>,
    #[publisher] results: impl FnMut(Response),
    #[resource] clock: &Clock,
    #[resource] def: &GameDefinition,
) {
    for request in requests {
        let file: SaveFile = SaveFile {
            def: def.clone(),
            state: GameState {
                nodes: Vec::new(), // TODO
                edges: Vec::new(), // TODO
                clock: clock.now(),
            },
        };

        let ret = match request.format {
            #[cfg(feature = "serde_yaml")]
            Format::Text => serde_yaml::to_vec(&file).map_err(|err| err.to_string()),
            #[cfg(feature = "rmp-serde")]
            Format::Binary => rmp_serde::to_vec_named(&file).map_err(|err| err.to_string()),
        };

        match ret {
            Ok(mut data) => {
                match request.format {
                    Format::Text => {
                        let string = format!("### SCHEMA_VERSION={:08X}\n", SCHEMA_VERSION);
                        data.splice(0..0, string.bytes());
                    }
                    Format::Binary => {
                        let mut vec = arrayvec::ArrayVec::<u8, 5>::new();
                        vec.push(0xFF);
                        vec.extend(SCHEMA_VERSION.to_le_bytes());
                        data.splice(0..0, vec);
                    }
                }
                results(Response {
                    format: request.format,
                    data,
                });
            }
            Err(err) => {
                log::error!("Error saving game: {}", err);
            }
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(save_setup)
}
