from dataclasses import dataclass, KW_ONLY
from typing import Self

from .. import Def, Id, Writer


@dataclass
class Type(Def):
    _: KW_ONLY

    viscosity: float
    vacuum_specific_volume: float
    critical_pressure: float
    saturation_gamma: float

    def aqueous(molar_mass: float) -> Self:
        return Type(
            viscosity=2.0,
            vacuum_specific_volume=18.0 / molar_mass,
            critical_pressure=1.2,
            saturation_gamma=100.0,
        )

    def gas_like(molar_mass: float) -> Self:
        return Type(
            viscosity=0.1,
            vacuum_specific_volume=22400.0 / molar_mass,
            critical_pressure=1000.0,
            saturation_gamma=100.0,
        )

    def save_id() -> str:
        return "traffloat.save.fluid.Type"

    def write(self, writer: Writer) -> Self:
        self.id = writer.write(
            Type,
            {
                "viscosity": self.viscosity,
                "vacuum_specific_volume": self.vacuum_specific_volume,
                "critical_pressure": self.critical_pressure,
                "saturation_gamma": self.saturation_gamma,
            },
        )

        return self.id
