use core::hash;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem;

use bevy::app::{self, App};
use bevy::color::Color;
use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::event::{Event, EventWriter};
use bevy::ecs::query::Changed;
use bevy::ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy::ecs::system::Query;
use bevy::ui;
use bevy::ui::node_bundles::ButtonBundle;

pub struct Plugin<E>(PhantomData<fn() -> E>);

impl<E> Default for Plugin<E> {
    fn default() -> Self { Self(PhantomData) }
}

impl<E: Event + Clone> app::Plugin for Plugin<E> {
    fn build(&self, app: &mut App) {
        app.add_event::<E>();
        app.add_systems(
            app::Update,
            handle_buttons::<E>.before(HandleClickSystemSet::<E>::default),
        );
    }
}

#[derive(Component)]
pub struct OnClick<E>(E);

#[derive(Debug, Clone, SystemSet)]
pub struct HandleClickSystemSet<E>(PhantomData<E>);

impl<E> Default for HandleClickSystemSet<E> {
    fn default() -> Self { Self(PhantomData) }
}

impl<E> PartialEq for HandleClickSystemSet<E> {
    fn eq(&self, _: &Self) -> bool { true }
}

impl<E> Eq for HandleClickSystemSet<E> {}

impl<E> Hash for HandleClickSystemSet<E> {
    fn hash<H: hash::Hasher>(&self, _: &mut H) {}
}

#[derive(Component)]
pub struct LastInteraction(ui::Interaction);

fn handle_buttons<E: Event + Clone>(
    mut query: Query<
        (&ui::Interaction, &mut ui::BackgroundColor, &mut LastInteraction, &OnClick<E>),
        Changed<ui::Interaction>,
    >,
    mut event_writer: EventWriter<E>,
) {
    query.iter_mut().for_each(|(interaction, mut bg_color, mut last_interaction, on_click)| {
        let last_interaction = mem::replace(&mut last_interaction.0, *interaction);

        match *interaction {
            ui::Interaction::None => {
                bg_color.0 = BUTTON_COLOR_IDLE;
            }
            ui::Interaction::Hovered => {
                bg_color.0 = BUTTON_COLOR_HOVER;
                if let ui::Interaction::Pressed = last_interaction {
                    event_writer.send(on_click.0.clone());
                }
            }
            ui::Interaction::Pressed => {
                bg_color.0 = BUTTON_COLOR_PRESSED;
            }
        }
    });
}

#[derive(bundle::Bundle)]
pub struct Bundle<E: Event> {
    pub button:           ButtonBundle,
    pub on_click:         OnClick<E>,
    pub last_interaction: LastInteraction,
}

impl<E: Event> Bundle<E> {
    pub fn new(event: E) -> Self {
        Self {
            button:           ButtonBundle::default(),
            on_click:         OnClick(event),
            last_interaction: LastInteraction(ui::Interaction::None),
        }
    }
}

const BUTTON_COLOR_IDLE: Color = Color::hsl(0., 0., 0.2);
const BUTTON_COLOR_HOVER: Color = Color::hsl(0., 0., 0.4);
const BUTTON_COLOR_PRESSED: Color = Color::hsl(0., 0., 0.6);
