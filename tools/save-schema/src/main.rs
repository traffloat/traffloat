//! Generate JSON schema for saves.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Context;
use bevy::app::App;
use bevy::state::state::States;
use bevy::utils::HashMap;
use clap::Parser as _;
use schemars::gen::SchemaGenerator;
use schemars::schema::{self, Schema, SchemaObject, SubschemaValidation};
use schemars::JsonSchema;
use traffloat_base::save;

#[derive(clap::Parser)]
#[command(name = "traffloat-save-schema", version = traffloat_version::VERSION, about)]
struct Options {
    #[clap(short, long, default_value = "assets/save-schema.json")]
    output: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
struct DummyState;

fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    let mut app = App::new();
    app.add_plugins((
        bevy::MinimalPlugins,
        traffloat_base::save::Plugin,
        traffloat_view::Plugin,
        traffloat_graph::Plugin,
        traffloat_fluid::Plugin(DummyState),
    ));

    let store = app.world().resource::<save::schema::Store>();

    {
        let mut guard = ACTIVE_SCHEMA_STORE.try_lock().expect("active schema store contention");
        assert!(guard.is_none(), "there is another active schema store");
        *guard = Some(store.0.clone());
    }

    let schema = schemars::schema_for!(JsonFile);

    {
        let mut guard = ACTIVE_SCHEMA_STORE.try_lock().expect("active schema store contention");
        guard.take().expect("active schema store got removed");
    }

    let writer =
        BufWriter::new(fs::File::create(options.output).context("open output file for writing")?);
    serde_json::to_writer(writer, &schema).context("writing json schema")?;

    Ok(())
}

#[allow(dead_code)] // dummy struct for schema generation
#[derive(JsonSchema)]
struct JsonFile {
    types: Vec<JsonFileTypeEntry>,
}

enum JsonFileTypeEntry {}

static ACTIVE_SCHEMA_STORE: Mutex<Option<HashMap<&'static str, save::schema::JsonSchemaDyn>>> =
    Mutex::new(None);

impl JsonSchema for JsonFileTypeEntry {
    fn schema_id() -> std::borrow::Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::JsonFileTypeEntry"))
    }

    fn schema_name() -> String { "JsonFileTypeEntry".to_owned() }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let guard = ACTIVE_SCHEMA_STORE.try_lock().expect("active schema store contention");
        let store = guard.as_ref().expect("no active schema store");

        Schema::Object(SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                one_of: Some(
                    store
                        .values()
                        .map(|variant| {
                            new_internally_tagged_enum("type", variant.ty, false)
                                .flatten((variant.json_schema)(gen))
                        })
                        .collect(),
                ),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

fn new_internally_tagged_enum(tag_name: &str, variant: &str, deny_unknown_fields: bool) -> Schema {
    let tag_schema = Schema::Object(SchemaObject {
        instance_type: Some(schema::InstanceType::String.into()),
        enum_values: Some(vec![variant.into()]),
        ..Default::default()
    });
    Schema::Object(SchemaObject {
        instance_type: Some(schema::InstanceType::Object.into()),
        object: Some(Box::new(schema::ObjectValidation {
            properties: {
                let mut props = BTreeMap::new();
                props.insert(tag_name.to_owned(), tag_schema);
                props
            },
            required: {
                let mut required = BTreeSet::new();
                required.insert(tag_name.to_owned());
                required
            },
            additional_properties: deny_unknown_fields.then(|| Box::new(false.into())),
            ..Default::default()
        })),
        ..SchemaObject::default()
    })
}
