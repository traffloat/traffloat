use bevy::app::{self, App};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::hierarchy::BuildChildren;
use bevy::render::view::Visibility;
use bevy::state::state::{self};
use bevy::text::{Text, TextStyle};
use bevy::ui::node_bundles::{NodeBundle, TextBundle};
use bevy::ui::{self, Style, UiRect};
use traffloat_view::appearance::Appearance;

use super::DelegateViewable;
use crate::AppState;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(state::OnEnter(AppState::GameView), setup);
        app.add_systems(app::Update, update_text_system);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: ui::Val::Px(150.),
                    height: ui::Val::Px(250.),
                    justify_self: ui::JustifySelf::End,
                    align_self: ui::AlignSelf::End,
                    border: UiRect::all(ui::Val::Px(5.)),
                    padding: UiRect::all(ui::Val::Px(5.)),
                    ..Default::default()
                },
                background_color: ui::BackgroundColor(Color::linear_rgb(0.05, 0.05, 0.15)),
                border_color: ui::BorderColor(Color::linear_rgb(0.8, 0.6, 0.2)),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            ContainerNode,
        ))
        .with_children(|b| {
            b.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle { font_size: 24., ..Default::default() },
                    ),
                    ..Default::default()
                },
                LabelDisplay,
            ));
        });
}

/// Marker component for the container node for the info panel.
#[derive(Component)]
struct ContainerNode;

/// Marker component for the label display node.
#[derive(Component)]
struct LabelDisplay;

fn update_text_system(
    focus: Res<super::Focus>,
    mut container_query: Query<&mut Visibility, With<ContainerNode>>,
    mut display_query: Query<&mut Text, With<LabelDisplay>>,
    object_query: Query<&Appearance, With<DelegateViewable>>,
) {
    if let Some(focus_entity) = focus.entity {
        if let Ok(mut vis) = container_query.get_single_mut() {
            *vis = Visibility::Visible;
        }

        if let Ok(mut display) = display_query.get_single_mut() {
            let section = display.sections.get_mut(0).unwrap();
            section.value.clear();

            let appearance =
                object_query.get(focus_entity).expect("object focus entity is invalid");
            appearance.label.render(&mut section.value);
        }
    } else if let Ok(mut vis) = container_query.get_single_mut() {
        *vis = Visibility::Hidden;
    }
}
