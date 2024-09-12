from typing import Dict, Callable

import save
import basic

scenarios: Dict[str, Callable[[save.Writer], None]] = {
    "basic": basic.write_scenario,
}
