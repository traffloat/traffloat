use std::fmt::Display;
use std::marker::PhantomData;

use bevy::ecs::system::{Commands, Res};
use egui_material_icons::icons;
use traffloat_scene::meta::{self, CargoType, FluidType};

use crate::dock::{self, TabPlacement, menu};
use crate::util::new_id;

pub struct MenuAction<T>(PhantomData<T>);

impl<T> Default for MenuAction<T> {
    fn default() -> Self { Self(PhantomData) }
}

impl<T: Type> menu::Action for MenuAction<T> {
    fn shortcut(&self) -> Option<egui::KeyboardShortcut> { None }

    fn icon(&self) -> egui_material_icons::MaterialIcon { T::icon() }

    fn text_label(&self) -> String { format!("Show {}", T::text_label()) }

    type Params<'w, 's> = Commands<'w, 's>;

    fn precondition(&self, _: &Commands) -> bool { true }

    fn execute(&self, commands: &mut Commands) {
        commands.queue(crate::dock::DockCommand(|dock| {
            dock.focus_or_create(
                || T::make_tab(),
                dock::ReplaceTab(|state| T::is_tab(&state.tab)).or_always(dock::NewWindow),
            );
        }));
    }
}

pub struct Tab<T> {
    selected: usize,
    _ph:      PhantomData<T>,
}

impl<T> Default for Tab<T> {
    fn default() -> Self { Self { selected: 0, _ph: PhantomData } }
}

impl<T: Type> dock::Tab for Tab<T> {
    type TitleSystemParam<'w, 's> = ();

    fn title(&self, (): ()) -> String { T::text_label().to_string() }

    type UiSystemParam<'w, 's> = Res<'w, meta::Types<T>>;

    fn ui(&mut self, types: Res<meta::Types<T>>, ui: &mut egui::Ui, dock: dock::Context) {
        egui::ComboBox::from_id_salt(new_id!(dock.id))
            .selected_text(types.0.get(self.selected).map_or("", |ty| ty.name()))
            .show_ui(ui, |ui| {
                for (index, ty) in types.0.iter().enumerate() {
                    ui.selectable_value(&mut self.selected, index, ty.name());
                }
            });

        if let Some(ty) = types.0.get(self.selected) {
            ty.info_ui(ui);
        }
    }

    type OnCloseSystemParam<'w, 's> = ();
    type BeforeRenderSystemParam<'w, 's> = ();
}

pub trait Type: Send + Sync + 'static {
    fn icon() -> egui_material_icons::MaterialIcon;
    fn text_label() -> impl Display;
    fn is_tab(state: &dock::TabEnum) -> bool;
    fn make_tab() -> dock::TabEnum;

    fn name(&self) -> &str;
    fn info_ui(&self, ui: &mut egui::Ui);
}

impl Type for FluidType {
    fn icon() -> egui_material_icons::MaterialIcon { icons::ICON_WATER_DROP }
    fn text_label() -> impl Display { "Fluid types" }
    fn is_tab(state: &dock::TabEnum) -> bool { state.is_meta_info_fluid() }
    fn make_tab() -> dock::TabEnum { dock::TabEnum::MetaInfoFluid(Tab::default()) }

    fn name(&self) -> &str { &self.name }
    fn info_ui(&self, ui: &mut egui::Ui) { ui.label(&self.description); }
}

impl Type for CargoType {
    fn icon() -> egui_material_icons::MaterialIcon { icons::ICON_STACK }
    fn text_label() -> impl Display { "Cargo types" }
    fn is_tab(state: &dock::TabEnum) -> bool { state.is_meta_info_cargo() }
    fn make_tab() -> dock::TabEnum { dock::TabEnum::MetaInfoCargo(Tab::default()) }

    fn name(&self) -> &str { &self.name }
    fn info_ui(&self, ui: &mut egui::Ui) {
        ui.label(&self.description);
        ui.label("Density:");
        ui.label(format!("{}", self.density));
    }
}
