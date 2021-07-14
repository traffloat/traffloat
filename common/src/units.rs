//! Defines common units

ratio_def::units! {
    /// A common unit type
    Unit(std::fmt::Debug + Clone + Copy + Default + PartialEq + PartialOrd);

    #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)] f64:

    /// An amount of liquid
    LiquidVolume;

    /// The pressure of air in a room
    GasPressure;

    /// An absolute amount of gas
    GasVolume;

    /// The standard size for cargo
    CargoSize;

    /// Specific heat capacity
    HeatCapacity;

    /// Heat energy
    HeatEnergy;

    /// Dynamic electricity consumed immediately
    ElectricPower;

    /// Static electricity in stored form
    ElectricEnergy;

    /// Orthogonal area of a node receiving sunlight.
    Brightness;
}
