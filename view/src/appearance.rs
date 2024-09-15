//! Components of a [viewable](super::viewable) related to visual rendering.

use std::borrow::Cow;

use bevy::ecs::component::Component;
use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject, StringValidation};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};

use crate::DisplayText;

/// All appearance layers of the viewable,
/// used during serialization.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Component)]
pub struct Appearance {
    /// Displays the building to users.
    pub label: DisplayText,

    /// The exterior appearance of the viewable when its rendered area
    /// is only an insignificant portion of the viewport.
    pub distal:   Layer,
    /// The exterior appearance of the viewable when its rendered area
    /// takes up a major portion of the viewport.
    pub proximal: Layer,
    /// The appearance of the viewable when the viewport camera
    /// is within the bounds of the object.
    pub interior: Layer,
}

impl Appearance {
    /// Create an invisible appearance.
    #[must_use]
    pub fn null() -> Self {
        Self {
            label:    DisplayText::default(),
            distal:   Layer::Null,
            proximal: Layer::Null,
            interior: Layer::Null,
        }
    }
}

/// Describes a way to display an object.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum Layer {
    /// Do not display anything.
    Null,
    /// Use PBR for display.
    Pbr {
        /// The object mesh.
        mesh:     GlbMeshRef,
        /// The object material.
        material: GlbMaterialRef,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize, JsonSchema)]
pub struct ImageRef {
    /// Reference to GLB file by its SHA1 hash.
    pub sha: [u8; 20],
}

/// Identifies a GLB file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GlbSha(pub [u8; 20]);

/// Reference to a primitive in a GLB mesh node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize, JsonSchema)]
pub struct GlbMeshRef {
    /// Reference to GLB file by its SHA1 hash.
    pub sha:       GlbSha,
    /// Index of the object inside the GLB file.
    pub mesh:      u16,
    /// Index of the primitive inside the GLB mesh.
    pub primitive: u16,
}

/// Reference to a GLB primitive node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize, JsonSchema)]
pub struct GlbMaterialRef {
    /// Reference to GLB file by its SHA1 hash.
    pub sha:   GlbSha,
    /// Index of the object inside the GLB file.
    pub index: u16,
}

impl Serialize for GlbSha {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut chars = [0u8; 40];
            hex::encode_to_slice(self.0, &mut chars).expect("20 * 2 = 40");
            let str = std::str::from_utf8(&chars).expect("hex produced non-UTF8 bytes");
            serializer.serialize_str(str)
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for GlbSha {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let hex = <&'de str>::deserialize(deserializer)?;

            let mut bytes = [0u8; 20];
            hex::decode_to_slice(hex, &mut bytes)
                .map_err(<D::Error as serde::de::Error>::custom)?;

            Ok(Self(bytes))
        } else {
            <[u8; 20]>::deserialize(deserializer).map(Self)
        }
    }
}

impl JsonSchema for GlbSha {
    fn schema_id() -> Cow<'static, str> { Cow::Borrowed(concat!(module_path!(), "::GlbSha")) }

    fn schema_name() -> String { "GlbSha".into() }

    fn is_referenceable() -> bool { false }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: None,
            string: Some(Box::new(StringValidation {
                pattern:    Some("[0-9a-f]{40}".into()),
                min_length: Some(40),
                max_length: Some(40),
            })),
            ..Default::default()
        })
    }
}
