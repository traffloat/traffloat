use std::time::Duration;

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
use bevy::winit::{self, WinitSettings};

use crate::util::button;
use crate::AppState;

mod select_load;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(state::OnEnter(AppState::MainMenu), setup);
        app.add_systems(state::OnExit(AppState::MainMenu), teardown);
        app.add_plugins(button::Plugin::<ClickEvent>::default());
        app.add_systems(
            app::Update,
            handle_click.in_set(button::HandleClickSystemSet::<ClickEvent>::default()),
        );
        app.add_plugins(select_load::Plugin);
    }
}

#[derive(Component)]
struct Owned;

#[derive(Debug, Clone, Event)]
enum ClickEvent {
    Load,
}

fn setup(mut commands: Commands, mut winit_settings: ResMut<WinitSettings>) {
    *winit_settings = WinitSettings {
        focused_mode:   winit::UpdateMode::reactive(Duration::from_millis(100)),
        unfocused_mode: winit::UpdateMode::reactive_low_power(Duration::from_secs(1)),
    };

    commands.spawn((Camera2dBundle::default(), Owned));
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
            Owned,
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
                    builder.spawn(button::Bundle::new(ClickEvent::Load)).with_children(|builder| {
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
                    });
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

fn teardown(mut commands: Commands, query: Query<Entity, With<Owned>>) {
    query.into_iter().for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
}
