use crate::proto::{self, BinRead, BinWrite, ProtoType};

ratio_def::units! {
    /// A common unit type
    Unit(std::fmt::Debug + Clone + Copy + Default + PartialEq + PartialOrd + ProtoType + BinWrite + BinRead);

    #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, codegen::Gen)] f32:

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
}
