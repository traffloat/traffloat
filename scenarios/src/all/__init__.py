from typing import Callable

from .. import save

from . import basic

scenarios: dict[str, Callable[[save.Writer], None]] = {
    "basic": basic.write_scenario,
}
