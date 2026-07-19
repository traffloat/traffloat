use std::hash::{Hash, Hasher};

use bevy::app::{App, Plugin};
use bevy::ecs::system::{Commands, SystemParam};
use bevy::ecs::world::World;
use egui_dock::tab_viewer::OnCloseResponse;
use rand::RngExt;
use rand::distr::Alphanumeric;
use traffloat_physics::generate;

use crate::dock::{self};
use crate::scene;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, _app: &mut App) {}
}

#[derive(Default)]
pub struct Tab {
    seed: String,
}

#[derive(SystemParam)]
pub struct UiParams<'w, 's> {
    commands: Commands<'w, 's>,
}

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = ();
    fn title(&self, (): Self::TitleSystemParam<'_, '_>) -> String { "New game".into() }

    type UiSystemParam<'w, 's> = UiParams<'w, 's>;
    fn ui(&mut self, mut params: Self::UiSystemParam<'_, '_>, ui: &mut egui::Ui, _: dock::Context) {
        if self.seed.is_empty() {
            let mut rng = rand::rng();
            self.seed = (0..16).map(|_| char::from(rng.sample(Alphanumeric))).collect();
        }
        ui.horizontal(|ui| {
            ui.label("Seed");
            ui.text_edit_singleline(&mut self.seed);
        });

        if ui.button("Start").clicked() {
            let config = generate::Config {
                seed: {
                    if let Ok(num) = self.seed.parse::<u64>() {
                        num
                    } else {
                        let mut hasher = fnv::FnvHasher::default();
                        self.seed.hash(&mut hasher);
                        hasher.finish()
                    }
                },
            };

            params.commands.queue(move |world: &mut World| {
                generate::generate(world, config);

                scene::singleplayer::setup(world);
                dock::init_camera_view(world);
            });
        }
    }

    type OnCloseSystemParam<'w, 's> = ();

    type BeforeRenderSystemParam<'w, 's> = ();
}
