#![allow(unused_mut, unused_variables, reason = "macro expansion")]

use bevy::app::{App, Plugin};
use bevy::ecs::system::{ParamSet, SystemParam};
use egui_material_icons::MaterialIcon;
use traffloat_macro_util::fan_out;
use traffloat_scene::meta;

use crate::dock::{meta_info, save, settings};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, _app: &mut App) {}
}

pub trait Action: Default {
    fn shortcut(&self) -> Option<egui::KeyboardShortcut>;

    fn icon(&self) -> MaterialIcon;

    fn text_label(&self) -> String;

    type Params<'w, 's>: SystemParam;

    fn precondition(&self, params: &<Self::Params<'_, '_> as SystemParam>::Item<'_, '_>) -> bool;

    fn execute(&self, params: &mut <Self::Params<'_, '_> as SystemParam>::Item<'_, '_>);
}

macro_rules! define_actions {
    (
        [$w:lifetime, $s:lifetime]
        $paramset_tuple:ty;
        {
            $(
                $_:ident $param:tt $path:tt,
            )*
        }
    ) => {
        #[derive(SystemParam)]
        pub struct Params<$w, $s> {
            ps: $paramset_tuple,
        }

        impl<$w, $s> Params<$w, $s> {
            pub fn show_menu_buttons(&mut self, ui: &mut egui::Ui) {
                $(
                    define_actions_let_ps!(param, self.ps, $path);
                    define_actions!(@show_menu_button ui, &mut param; $param);
                )*
            }

            pub fn global_dock_shortcuts(&mut self, ctx: &egui::Context) {
                $(
                    ctx.input_mut(|input| {
                        define_actions_let_ps!(param, self.ps, $path);
                        define_actions!(@consume_shortcut input, &mut param; $param);
                    });
                )*
            }
        }
    };
    (@show_menu_button $ui:expr, $param:expr; (Separator)) => {
        $ui.separator();
    };
    (@show_menu_button $ui:expr, $param:expr; (Action = $action:ty)) => {
        show_menu_button($ui, <$action>::default(), $param);
    };
    (@consume_shortcut $input:expr, $param:expr; (Separator)) => {};
    (@consume_shortcut $input:expr, $param:expr; (Action = $action:ty)) => {
        consume_shortcut($input, <$action>::default(), $param);
    };
}

fn show_menu_button<A: Action>(
    ui: &mut egui::Ui,
    action: A,
    param: &mut <A::Params<'_, '_> as SystemParam>::Item<'_, '_>,
) {
    ui.add_enabled_ui(action.precondition(param), |ui| {
        let resp = ui.button(action.icon()).on_hover_text(format!(
            "{}{}",
            action.text_label(),
            if let Some(shortcut) = action.shortcut() {
                format!(
                    " ({})",
                    shortcut.format(&egui::ModifierNames::SYMBOLS, cfg!(target_os = "macos"))
                )
            } else {
                String::new()
            }
        ));
        if resp.clicked() && action.precondition(param) {
            action.execute(param);
        }
    });
}

fn consume_shortcut<A: Action>(
    input: &mut egui::InputState,
    action: A,
    param: &mut <A::Params<'_, '_> as SystemParam>::Item<'_, '_>,
) {
    if let Some(shortcut) = action.shortcut()
        && action.precondition(param)
        && input.consume_shortcut(&shortcut)
    {
        action.execute(param);
    }
}

macro_rules! define_actions_let_ps {
    ($var:ident, $ps:expr, ($($path:ident)*)) => {
        let $var = &mut $ps;
        $(
            let mut $var = $var.$path();
        )*
    }
}

macro_rules! define_actions_item {
    (
        [$w:lifetime, $s:lifetime]
        $message:ident (Action = $param:ty)
    ) => {
        <$param as Action>::Params<$w, $s>
    };
    (
        [$w:lifetime, $s:lifetime]
        $message:ident (Separator)
    ) => {
        ()
    };
}

macro_rules! define_actions_tuple {
    (
        [$w:lifetime, $s:lifetime]
        $($params:ty,)*
    ) => {
        ParamSet<$w, $s, (
            $($params,)*
        )>
    }
}

fan_out! {
    ['w, 's]
    define_actions, define_actions_tuple, define_actions_item;
    8, 2;
    Save(Action = save::MenuActionSave),
    SaveAs(Action = save::MenuActionSaveAs),
    Settings(Action = settings::MenuAction),
    Separator1(Separator),
    MetaInfoFluid(Action = meta_info::MenuAction<meta::FluidType>),
    MetaInfoCargo(Action = meta_info::MenuAction<meta::CargoType>),
}
