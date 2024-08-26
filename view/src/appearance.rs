//! Components of a [viewable] related to visual rendering.

use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

/// All appearance layers of the viewable,
/// used during serialization.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Layers {
    /// The exterior appearance of the viewable when its rendered area
    /// is only an insignificant portion of the viewport.
    pub distal:   Appearance,
    /// The exterior appearance of the viewable when its rendered area
    /// takes up a major portion of the viewport.
    pub proximal: Appearance,
    /// The appearance of the viewable when the viewport camera
    /// is within the bounds of the object.
    pub interior: Appearance,
}

impl Layers {
    /// Create an invisible appearance.
    #[must_use]
    pub fn null() -> Self {
        Self { distal: Appearance::Null, proximal: Appearance::Null, interior: Appearance::Null }
    }
}

/// Describes a way to display an object.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Appearance {
    /// Do not display anything.
    Null,
    /// Use PBR for display.
    Pbr {
        /// The object mesh.
        mesh:     GlbRef,
        /// The object material.
        material: GlbRef,
    },
    // /// Use billboard for display.
    // Billboard {
    // /// The sprite file for the billboard.
    // ///
    // /// The image is assumed to be a 1\*1 physical square centered at the object location.
    // sprite: ImageRef,
    // },
}

/// Reference to a image file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct ImageRef {
    /// Reference to GLB file by its SHA1 hash.
    #[serde(with = "hex_hash")]
    pub sha: [u8; 20],
}

/// Reference to a GLB node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct GlbRef {
    /// Reference to GLB file by its SHA1 hash.
    #[serde(with = "hex_hash")]
    pub sha:   [u8; 20],
    /// Index of the object inside the GLB file.
    pub index: u16,
}

impl GlbRef {
    /// A null model that loads an empty node.
    pub const NULL: Self = Self { sha: [0; 20], index: 0 };
}

mod hex_hash {
    use serde::{Deserialize, Serializer};

    pub(super) fn serialize<S>(sha: &[u8; 20], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut chars = [0u8; 40];
            hex::encode_to_slice(sha, &mut chars).expect("20 * 2 = 40");
            let str = std::str::from_utf8(&chars).expect("hex produced non-UTF8 bytes");
            serializer.serialize_str(str)
        } else {
            serializer.serialize_bytes(sha)
        }
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 20], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let hex = <&'de str>::deserialize(deserializer)?;
            if hex == "NULL" {
                return Ok(super::GlbRef::NULL.sha);
            }

            let mut bytes = [0u8; 20];
            hex::decode_to_slice(hex, &mut bytes)
                .map_err(<D::Error as serde::de::Error>::custom)?;

            Ok(bytes)
        } else {
            <[u8; 20]>::deserialize(deserializer)
        }
    }
}
