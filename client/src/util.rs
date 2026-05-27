macro_rules! new_id_ {
    () => {{
        struct NewId;
        egui::Id::new(std::any::TypeId::of::<NewId>())
    }};
    ($($salt:expr),+ $(,)?) => {{
        struct NewId;
        egui::Id::new((std::any::TypeId::of::<NewId>(), $($salt,)+))
    }};
}

pub(crate) use new_id_ as new_id;

pub mod shapes;
