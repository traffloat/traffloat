use std::iter;

use bevy::app::{self, App};
use bevy::color::Color;
use bevy::diagnostic::{Diagnostic, DiagnosticPath, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::hierarchy::{self, BuildChildren};
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::render::view::Visibility;
use bevy::state::state::{self};
use bevy::text::{Text, TextSection, TextStyle};
use bevy::ui::node_bundles::{NodeBundle, TextBundle};
use bevy::ui::{self, Style, UiRect};
use traffloat_base::debug;
use typed_builder::TypedBuilder;

use crate::AppState;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FrameTimeDiagnosticsPlugin, RenderDiagnosticsPlugin));

        app.add_systems(state::OnEnter(AppState::GameView), setup);
        app.add_systems(app::Update, display_diagnostic_system);

        app.add_systems(app::Startup, |mut commands: Commands| {
            commands
                .spawn((
                    DisplayGroup::builder()
                        .vertical_priority(10)
                        .id("render")
                        .label("Render")
                        .build(),
                    debug::Bundle::new("FpsDiagnostic"),
                ))
                .with_children(|b| {
                    b.spawn(
                        Display::builder()
                            .horizontal_priority(0)
                            .label("FPS")
                            .target(FrameTimeDiagnosticsPlugin::FPS)
                            .build(),
                    );
                });
        });
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    justify_self: ui::JustifySelf::Start,
                    align_self: ui::AlignSelf::Start,
                    margin: UiRect::all(ui::Val::Px(5.)),
                    ..Default::default()
                },
                visibility: Visibility::Visible,
                ..Default::default()
            },
            ContainerNode,
            super::Owned,
            debug::Bundle::new("DiagnosticUi"),
        ))
        .with_children(|b| {
            b.spawn((
                TextBundle { text: Text::from_sections([]), ..Default::default() },
                LabelDisplay,
                debug::Bundle::new("DiagnosticText"),
            ));
        });
}

/// Marker component for the container node for all diagnostics.
#[derive(Component)]
pub struct ContainerNode;

/// Marker component for the label display node.
#[derive(Component)]
pub struct LabelDisplay;

/// Each entity indicates the request to display a diagnostic.
/// Each display must be a child of a [`DisplayGroup`].
#[derive(Component, TypedBuilder)]
#[builder(mutators(
    pub fn target(&mut self, path: impl Into<DiagnosticPath>) {
        let path = path.into();
        self.target_str.push(path.as_str().into());
        self.target.push(path);
    }
))]
pub struct Display {
    /// Paths of the diagnostics to display.
    #[builder(via_mutators = Vec::new())]
    pub target:              Vec<DiagnosticPath>,
    /// Priority of the diagnostic within the group.
    /// Smaller number indicates closer to start.
    pub horizontal_priority: i32,
    /// Display label of the diagnostic.
    #[builder(setter(into))]
    pub label:               String,
    #[builder(via_mutators = Vec::new())]
    target_str:              Vec<String>,
}

/// A group of diagnostics displayed as a single row.
#[derive(Component, TypedBuilder)]
pub struct DisplayGroup {
    /// Identifier for the group.
    #[builder(setter(into))]
    pub id:                String,
    /// Priority of vertical ordering between groups.
    /// Smaller number indicates higher position.
    pub vertical_priority: i32,
    /// Display label of the group.
    #[builder(setter(into))]
    pub label:             String,
}

fn display_diagnostic_system(
    mut label_query: Query<&mut Text, With<LabelDisplay>>,
    sources: Res<DiagnosticsStore>,
    display_group_query: Query<(&DisplayGroup, &hierarchy::Children)>,
    display_query: Query<&Display>,
) {
    let Ok(mut display_text) = label_query.get_single_mut() else { return };
    display_text.sections.clear();

    let mut groups: Vec<_> = display_group_query.iter().collect();
    groups.sort_by_key(|(group, _)| (group.vertical_priority, &group.id));

    for (group, children) in groups {
        display_text.sections.push(TextSection {
            value: group.label.clone(),
            style: TextStyle { color: Color::WHITE, font_size: 12., ..Default::default() },
        });

        let mut displays: Vec<_> =
            children.iter().filter_map(|&child| display_query.get(child).ok()).collect();
        displays.sort_by_key(|display| (display.horizontal_priority, &display.target_str));

        display_text.sections.extend(displays.iter().flat_map(|display| {
            iter::once(TextSection {
                value: format!(" {}", &display.label),
                style: TextStyle {
                    color: Color::srgb(0.5, 1.0, 0.5),
                    font_size: 12.,
                    ..Default::default()
                },
            })
            .chain(display.target.iter().enumerate().flat_map(|(index, path)| {
                [
                    TextSection {
                        value: if index == 0 { " " } else { ", " }.into(),
                        style: TextStyle {
                            color: Color::WHITE,
                            font_size: 12.,
                            ..Default::default()
                        },
                    },
                    TextSection {
                        value: sources
                            .get(path)
                            .and_then(Diagnostic::value)
                            .map_or_else(String::default, |value| format!("{value:.2}")),
                        style: TextStyle {
                            color: Color::srgb(1.0, 0.5, 1.0),
                            font_size: 12.,
                            ..Default::default()
                        },
                    },
                ]
            }))
        }));

        display_text.sections.push(TextSection {
            value: '\n'.into(),
            style: TextStyle { font_size: 12., ..Default::default() },
        });
    }
}
