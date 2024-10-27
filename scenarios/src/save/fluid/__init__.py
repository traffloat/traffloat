from dataclasses import dataclass, KW_ONLY
from typing import Any, Self

from .. import Def, Id, Writer
from ..types import DisplayText


@dataclass
class Type(Def):
    _: KW_ONLY

    display_label: DisplayText
    viscosity: float
    vacuum_specific_volume: float
    critical_pressure: float
    saturation_gamma: float

    @staticmethod
    def aqueous(display_label: DisplayText, molar_mass: float) -> Self:
        return Type(
            display_label=display_label,
            viscosity=2.0,
            vacuum_specific_volume=18.0 / molar_mass,
            critical_pressure=1.2,
            saturation_gamma=100.0,
        )

    @staticmethod
    def gas_like(display_label: str, molar_mass: float) -> Self:
        return Type(
            display_label=display_label,
            viscosity=0.1,
            vacuum_specific_volume=22400.0 / molar_mass,
            critical_pressure=1000.0,
            saturation_gamma=100.0,
        )

    @staticmethod
    def save_id() -> str:
        return "traffloat.save.fluid.Type"

    def write(self, writer: Writer) -> Id[Any]:  # `Id[Self]` does not work
        self.id = writer.write(
            Type,
            {
                "display_label": self.display_label.as_dict(),
                "viscosity": self.viscosity,
                "vacuum_specific_volume": self.vacuum_specific_volume,
                "critical_pressure": self.critical_pressure,
                "saturation_gamma": self.saturation_gamma,
            },
        )

        return self.id
