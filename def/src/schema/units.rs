use serde::de::DeserializeOwned;
use traffloat_types::{time, units};
use xylem::{DefaultContext, NoArgs, Xylem};

use crate::Schema;

impl<T: DeserializeOwned + Xylem<Schema>> Xylem<Schema> for time::Rate<T> {
    type From = Self;
    type Args = NoArgs;

    fn convert_impl(
        from: Self::From,
        _: &mut DefaultContext,
        _: &Self::Args,
    ) -> anyhow::Result<Self> {
        Ok(from)
    }
}

impl<T: units::Unit + Xylem<Schema>> Xylem<Schema> for units::Portion<T> {
    type From = Self;
    type Args = NoArgs;

    fn convert_impl(
        from: Self::From,
        _: &mut DefaultContext,
        _: &Self::Args,
    ) -> anyhow::Result<Self> {
        Ok(from)
    }
}

impl_xylem_for_self! {
    units::LiquidVolume,
    units::LiquidViscosity,
    units::GasVolume,
    units::CargoSize,
    units::ElectricPower,
    units::ElectricEnergy,
    units::Brightness,
    units::Skill,
    units::RailForce,
    units::PipeForce,
    units::FanForce,
    units::VehicleSpeed,
    units::Hitpoint,
}
