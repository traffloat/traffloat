//! The hash routes used for traffloat.

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use traffloat::def;

/// The top level route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Route {
    /// The subject is a default scenario.
    Scenario {
        /// Key name of the scenario.
        name: String,
        /// Scenario subroute.
        sp: SpRoute,
    },
    /// The subject is an uploaded scenario.
    Custom {
        /// Scenario subroute.
        sp: SpRoute,
    },
    /// The subject is a multiplayer server.
    Server,
}

/// The route of a given singleplayer scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpRoute {
    Home,
    Rules(Rules),
    Game,
}

/// The rules of a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rules {
    Home,
    Building(def::building::TypeId),
    Cargo(def::cargo::TypeId),
}

impl Default for Route {
    fn default() -> Self {
        Self::Scenario { name: String::from("vanilla"), sp: SpRoute::Home }
    }
}

impl Route {
    /// Converts a route to a hash-path.
    pub fn to_path(&self) -> String {
        let (prefix, sp) = match self {
            Route::Scenario { name, sp } => (format!("scenario/{}", name), sp),
            Route::Custom { sp } => (String::from("custom"), sp),
            Route::Server => return String::from("server"),
        };
        match sp {
            SpRoute::Home => prefix,
            SpRoute::Rules(Rules::Home) => format!("{}/rules", prefix),
            SpRoute::Rules(Rules::Building(id)) => format!("{}/rules/building/{}", prefix, id.0),
            SpRoute::Rules(Rules::Cargo(id)) => format!("{}/rules/cargo/{}", prefix, id.0),
            SpRoute::Game => format!("{}/play", prefix),
        }
    }

    /// Parses a hash-path to a route.
    pub fn parse_path(mut path: &str) -> Self {
        path = path.trim_start_matches(&['#', '/'][..]);

        fn parse_sp(mut sp: &str) -> SpRoute {
            sp = sp.trim_start_matches('/');
            if sp == "play" {
                return SpRoute::Game;
            }
            if let Some(mut path) = sp.strip_prefix("rules") {
                path = path.trim_start_matches('/');
                if let Some(mut path) = path.strip_prefix("building") {
                    path = path.trim_start_matches('/');
                    SpRoute::Rules(Rules::Building(def::building::TypeId(path.into())))
                } else if let Some(mut path) = path.strip_prefix("cargo") {
                    path = path.trim_start_matches('/');
                    SpRoute::Rules(Rules::Cargo(def::cargo::TypeId(path.into())))
                } else {
                    SpRoute::Rules(Rules::Home)
                }
            } else {
                SpRoute::Home
            }
        }

        if let Some(mut path) = path.strip_prefix("scenario") {
            path = path.trim_start_matches('/');
            if let Some((name, sp)) = path.split_once('/') {
                Route::Scenario { name: name.to_string(), sp: parse_sp(sp) }
            } else {
                Route::Scenario { name: path.to_string(), sp: SpRoute::Home }
            }
        } else if let Some(sp) = path.strip_prefix("custom") {
            Route::Custom { sp: parse_sp(sp) }
        } else if let Some(_path) = path.strip_prefix("server") {
            Route::Server
        } else {
            Route::default()
        }
    }

    pub fn replace_state(&self) {
        let window = web_sys::window().expect("window is unset");
        let history = window.history().expect("window.history is unset");
        let json = serde_json::to_string(self).expect("Failed to encode route as JSON");
        history
            .replace_state_with_url(
                &JsValue::from_str(&json),
                "",
                Some(&format!("#/{}", self.to_path())),
            )
            .expect("replace_state_with_url threw an exception");
    }
}
