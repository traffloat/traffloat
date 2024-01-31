use dynec::global;

mod attribute;

#[global]
pub struct DeltaTime {
    pub quantity: f64,
}
