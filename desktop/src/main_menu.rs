use bevy::app::{self, App};
use bevy::color::Color;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventReader};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, ResMut};
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::state::state::{self, NextState};
use bevy::text::{JustifyText, Text, TextStyle};
use bevy::ui::node_bundles::{NodeBundle, TextBundle};
use bevy::ui::{self, Style};

use crate::{util, AppState};

mod select_load;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(state::OnEnter(AppState::MainMenu), setup);
        app.add_systems(state::OnExit(AppState::MainMenu), cleanup);
        app.add_plugins(util::button::Plugin::<ClickEvent>::default());
        app.add_systems(
            app::Update,
            handle_click.in_set(util::button::HandleClickSystemSet::<ClickEvent>::default()),
        );
        app.add_plugins(select_load::Plugin);
    }
}

#[derive(Component)]
struct RootNode;

#[derive(Debug, Clone, Event)]
enum ClickEvent {
    Load,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: ui::Val::Percent(100.),
                    height: ui::Val::Percent(100.),
                    justify_content: ui::JustifyContent::Center,
                    align_content: ui::AlignContent::Center,
                    ..<_>::default()
                },
                background_color: ui::BackgroundColor(Color::hsl(0., 0., 0.05)),
                ..<_>::default()
            },
            RootNode,
        ))
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    style: Style {
                        justify_content: ui::JustifyContent::Center,
                        flex_direction: ui::FlexDirection::Column,
                        ..<_>::default()
                    },
                    ..<_>::default()
                })
                .with_children(|builder| {
                    builder.spawn(TextBundle {
                        text: Text::from_section(
                            "Traffloat",
                            TextStyle { font_size: 48., ..<_>::default() },
                        )
                        .with_justify(JustifyText::Center),
                        style: Style {
                            bottom: ui::Val::Px(24.),
                            justify_content: ui::JustifyContent::Center,
                            ..<_>::default()
                        },
                        ..<_>::default()
                    });
                    builder.spawn(TextBundle {
                        text: Text::from_section(
                            traffloat_version::VERSION,
                            TextStyle { font_size: 12., ..<_>::default() },
                        )
                        .with_justify(JustifyText::Center),
                        style: Style {
                            bottom: ui::Val::Px(24.),
                            justify_content: ui::JustifyContent::Center,
                            ..<_>::default()
                        },
                        ..<_>::default()
                    });
                    builder.spawn(util::button::Bundle::new(ClickEvent::Load)).with_children(
                        |builder| {
                            builder.spawn(TextBundle {
                                text: Text::from_section("Load", TextStyle::default())
                                    .with_justify(JustifyText::Center),
                                style: Style {
                                    width: ui::Val::Percent(100.),
                                    justify_content: ui::JustifyContent::Center,
                                    ..<_>::default()
                                },
                                ..<_>::default()
                            });
                        },
                    );
                });
        });
}

fn handle_click(
    mut events: EventReader<ClickEvent>,
    mut next_load_active_state: ResMut<NextState<select_load::ActiveState>>,
) {
    for event in events.read() {
        match event {
            ClickEvent::Load => {
                next_load_active_state.set(select_load::ActiveState::Active);
            }
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<RootNode>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}
