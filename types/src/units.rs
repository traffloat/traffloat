//! Defines common units

units! {
    /// A common unit type
    Unit(std::fmt::Debug + Clone + Copy + Default + PartialEq + PartialOrd);

    #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)] f64:

    /// An amount of liquid.
    LiquidVolume("{} L");

    /// An absolute amount of gas.
    GasVolume("{} mol");

    /// The standard size for cargo.
    CargoSize("{}");

    /// Dynamic electricity consumed immediately.
    ElectricPower("{} W");

    /// Static electricity in stored form.
    ElectricEnergy("{} J");

    /// Orthogonal area of a node receiving sunlight.
    Brightness("{} m^2");

    /// Skill level of an inhabitant.
    Skill("{}");
}
