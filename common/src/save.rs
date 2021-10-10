//! Saving game definition and state.

use std::convert::TryInto;

use cfg_if::cfg_if;
use legion::world::{ComponentError, SubWorld};
use legion::{Entity, EntityStore, IntoQuery};
use safety::Safety;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
