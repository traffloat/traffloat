use bevy::ecs::system::{Res, SystemParam};
use bevy::math::Vec2;
use bevy::time;
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy_mod_config::ReadConfig;

#[derive(SystemParam)]
pub struct Param<'w, 's> {
    conf: ReadConfig<'w, 's, super::Conf>,
    time: Res<'w, Time<time::Real>>,
}

impl Param<'_, '_> {
    pub fn consume_input(&self, transform: &mut Transform, resp: &egui::Response) {
        let conf = self.conf.read();
        let dt = self.time.delta_secs();

        resp.ctx.input_mut(|input| {
            let mut base_translation = Vec2::ZERO;
            if input.key_down(egui::Key::W) {
                base_translation.y += 1.0;
            }
            if input.key_down(egui::Key::A) {
                base_translation.x -= 1.0;
            }
            if input.key_down(egui::Key::S) {
                base_translation.y -= 1.0;
            }
            if input.key_down(egui::Key::D) {
                base_translation.x += 1.0;
            }

            transform.translation += transform.rotation.mul_vec3(base_translation.extend(0.0))
                * conf.movement_speed
                * transform.scale
                * dt;

            let mut base_rotation = 0.0;
            if input.key_down(egui::Key::Q) {
                base_rotation -= conf.rotation_speed;
            }
            if input.key_down(egui::Key::E) {
                base_rotation += conf.rotation_speed;
            }

            if base_rotation != 0.0 {
                transform.rotation *= bevy::math::Quat::from_rotation_z(base_rotation * dt);
            }

            let mut base_zoom = 0.0;
            if input.key_down(egui::Key::Equals) {
                base_zoom -= 1.0;
            }
            if input.key_down(egui::Key::Minus) {
                base_zoom += 1.0;
            }
            transform.scale *= (1.0 + conf.zoom_rate).powf(base_zoom * dt);
        });
    }
}
