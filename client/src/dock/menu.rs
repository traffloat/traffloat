use bevy::ecs::system::{ParamSet, SystemParam};
use traffloat_macro_util::fan_out;

use crate::dock::{save, settings};

pub trait Action: Default {
    fn shortcut(&self) -> egui::KeyboardShortcut;

    fn icon(&self) -> &'static str;

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
                $_:ident ($param:ty) $path:tt,
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
                    show_menu_button(ui, <$param>::default(), &mut param);
                )*
            }

            pub fn global_dock_shortcuts(&mut self, ctx: &egui::Context) {
                $(
                    ctx.input_mut(|input| {
                    define_actions_let_ps!(param, self.ps, $path);
                        consume_shortcut(input, <$param>::default(), &mut param, ctx);
                    });
                )*
            }
        }
    }
}

fn show_menu_button<A: Action>(
    ui: &mut egui::Ui,
    action: A,
    param: &mut <A::Params<'_, '_> as SystemParam>::Item<'_, '_>,
) {
    ui.add_enabled_ui(action.precondition(param), |ui| {
        let resp = ui.button(action.icon()).on_hover_text(format!(
            "{} ({})",
            action.text_label(),
            action.shortcut().format(&egui::ModifierNames::SYMBOLS, cfg!(target_os = "macos"))
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
    ctx: &egui::Context,
) {
    if action.precondition(param) && input.consume_shortcut(&action.shortcut()) {
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
        $message:ident ($param:ty)
    ) => {
        <$param as Action>::Params<$w, $s>
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
    Save(save::MenuActionSave),
    SaveAs(save::MenuActionSaveAs),
    Settings(settings::MenuAction),
}
