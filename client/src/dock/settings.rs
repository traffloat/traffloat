use crate::dock;

pub struct Tab;

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = ();
    fn title(&self, param: Self::TitleSystemParam<'_, '_>) -> String { "Settings".into() }

    type UiSystemParam<'w, 's> = bevy_mod_config::manager::egui::Display<'w, 's>;
    fn ui(&mut self, mut param: Self::UiSystemParam<'_, '_>, ui: &mut egui::Ui, _: dock::Context) {
        param.show(ui);
    }

    type OnCloseSystemParam<'w, 's> = ();

    type BeforeRenderSystemParam<'w, 's> = ();
}
