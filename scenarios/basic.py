#!/usr/bin/env python3

from lib import scenario, SaveType

with scenario("basic") as writer:
    writer.write(SaveType.Building, {
        "position": {"x": 0., "y": 0., "z": 5.},
    })
