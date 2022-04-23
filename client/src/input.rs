use std::error::Error;
use std::time::Instant;

use enum_map::EnumMap;
use three_d::{Camera, CameraAction, CameraControl, MetricSpace, Radians};
use xias::Xias;

#[derive(Debug, enum_map::Enum)]
pub enum Button {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveForward,
    MoveBackward,
    RotateClockwise,
    RotateCounterClockwise,
}

impl Button {
    pub fn from_key(key: three_d::Key) -> Option<Self> {
        Some(match key {
            three_d::Key::W => Self::MoveUp,
            three_d::Key::A => Self::MoveLeft,
            three_d::Key::S => Self::MoveDown,
            three_d::Key::D => Self::MoveRight,
            three_d::Key::Z => Self::MoveForward,
            three_d::Key::X => Self::MoveBackward,
            three_d::Key::Q => Self::RotateCounterClockwise,
            three_d::Key::E => Self::RotateClockwise,
            _ => return None,
        })
    }
}

pub struct Control {
    buttons:      EnumMap<Button, bool>,
    mouse:        CameraControl,
    linear_speed: f32,
    rotate_speed: Radians,
    update_time:  Instant,
}

impl Control {
    pub fn new(
        linear_speed: f32,
        rotate_speed: impl Into<Radians>,
        linear_sensitivity: f32,
        rotate_sensitivity: f32,
    ) -> Self {
        Self {
            buttons: EnumMap::default(),
            mouse: CameraControl {
                left_drag_horizontal:   CameraAction::None,
                left_drag_vertical:     CameraAction::None,
                middle_drag_horizontal: CameraAction::Left { speed: linear_sensitivity },
                middle_drag_vertical:   CameraAction::Up { speed: linear_sensitivity },
                right_drag_horizontal:  CameraAction::Yaw { speed: -rotate_sensitivity },
                right_drag_vertical:    CameraAction::Pitch { speed: -rotate_sensitivity },
                scroll_horizontal:      CameraAction::Roll { speed: rotate_sensitivity },
                scroll_vertical:        CameraAction::Forward { speed: linear_sensitivity },
            },
            linear_speed,
            rotate_speed: rotate_speed.into(),
            update_time: Instant::now(),
        }
    }

    pub fn handle_events(
        &mut self,
        camera: &mut Camera,
        events: &mut [three_d::Event],
    ) -> Result<bool, Box<dyn Error>> {
        let mut redraw = false;

        redraw |= self.mouse.handle_events(camera, events)?;

        for event in events {
            match event {
                three_d::Event::KeyPress { kind, modifiers: _, handled } => {
                    if *handled {
                        continue;
                    }

                    if let Some(button) = Button::from_key(*kind) {
                        self.buttons[button] = true;
                        *handled = true;
                    }
                }
                three_d::Event::KeyRelease { kind, modifiers: _, handled } => {
                    if *handled {
                        continue;
                    }

                    if let Some(button) = Button::from_key(*kind) {
                        self.buttons[button] = false;
                        *handled = true;
                    }
                }
                _ => {}
            }
        }

        let mut translate = three_d::Vec3::new(0.0, 0.0, 0.0);

        if self.buttons[Button::MoveUp] {
            translate += camera.right_direction().cross(camera.view_direction());
        }
        if self.buttons[Button::MoveDown] {
            translate -= camera.right_direction().cross(camera.view_direction());
        }
        if self.buttons[Button::MoveLeft] {
            translate -= camera.right_direction();
        }
        if self.buttons[Button::MoveRight] {
            translate += camera.right_direction();
        }
        if self.buttons[Button::MoveForward] {
            translate += camera.view_direction();
        }
        if self.buttons[Button::MoveBackward] {
            translate -= camera.view_direction();
        }

        let elapsed = self.update_time.elapsed().as_secs_f32();
        self.update_time = Instant::now();

        if translate != three_d::Vec3::new(0.0, 0.0, 0.0) {
            camera.translate(&(translate * self.linear_speed * elapsed))?;
            redraw = true;
        }

        if self.buttons[Button::RotateClockwise] {
            camera.roll(self.rotate_speed * elapsed)?;
            redraw = true;
        }
        if self.buttons[Button::RotateCounterClockwise] {
            camera.roll(-self.rotate_speed * elapsed)?;
            redraw = true;
        }

        Ok(redraw)
    }
}

pub fn handle_pick<'t, K>(
    gl: &three_d::Context,
    camera: &Camera,
    frame_input: &three_d::FrameInput,
    objects: impl Iterator<Item = (K, &'t dyn three_d::Object)>,
) -> Result<Option<K>, Box<dyn Error>> {
    let mut mouse_position = None;

    for event in &frame_input.events {
        if let three_d::Event::MouseMotion { position, .. } = event {
            mouse_position = Some((position.0, position.1));
            break;
        }
    }

    let mouse_position = match mouse_position {
        Some((x, y)) => (
            (frame_input.device_pixel_ratio * x).lossy_float(),
            (frame_input.device_pixel_ratio * y).lossy_float(),
        ),
        None => return Ok(None),
    };

    let mut min = None;

    for (key, object) in objects {
        if camera.in_frustum(&object.aabb()) {
            let pos = three_d::pick(gl, camera, mouse_position, &[object])?;
            if let Some(pos) = pos {
                match min {
                    Some((_, min_pos))
                        if camera.position().distance2(pos)
                            < camera.position().distance2(min_pos) =>
                    {
                        min = Some((key, pos));
                    }
                    None => {
                        min = Some((key, pos));
                    }
                    Some(_) => {}
                }
            }
        }
    }

    Ok(min.map(|(key, _)| key))
}
