//! Defines common units

units! {
    /// A common unit type
    Unit(std::fmt::Debug + Clone + Copy + Default + PartialEq + PartialOrd);

    #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)] f64:

    /// An amount of liquid.
    LiquidVolume("{} L");

    /// Viscosity of a liquid.
    LiquidViscosity("{} Pas");

    /// An absolute amount of gas.
    GasVolume("{} mol");

    /// The standard size for cargo.
    CargoSize("{} dm\u{b3}");

    /// Dynamic electricity consumed immediately.
    ElectricPower("{} W");

    /// Static electricity in stored form.
    ElectricEnergy("{} J");

    /// Orthogonal area of a node receiving sunlight.
    Brightness("{} m\u{b2}");

    /// Skill level of an inhabitant.
    Skill("{} SP");

    /// Driving force on a rail.
    RailForce("{} T");

    /// Pumping force on a liquid pipe.
    PipeForce("{} Pa");

    /// Pumping force for gas diffusion.
    FanForce("{} Pa");

    /// Speed of a vehicle on a rail.
    VehicleSpeed("{} m/s");

    /// Hitpoint of a building.
    Hitpoint("{} HP");
}
