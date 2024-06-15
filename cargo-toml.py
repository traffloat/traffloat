#!/usr/bin/env python3

import os
from os.path import join
import tomli_w
import tomli

members = {}
for (dir, subdirs, files) in os.walk("."):
    if "Cargo.toml" in files:
        if dir.startswith("./"):
            dir = dir[2:]
        with open(join(dir, "Cargo.toml"), "rb") as fh:
            manifest = tomli.load(fh)
        if "package" in manifest:
            members[dir] = manifest

contents = {
    "workspace": {
        "members": list(members),
        "resolver": "2",
        "dependencies": {
            "bevy": {
                # "git": "https://github.com/bevyengine/bevy",
                "version": "0.13",
                "features": ["dynamic_linking"],
            },
        }
        | {
            manifest["package"]["name"]: {"path": path}
            for path, manifest in members.items()
        },
        "lints": {
            "rust": {
                "missing_docs": "warn",
            },
            "clippy": {
                "pedantic": {"level": "warn", "priority": -1},
                "needless_pass_by_value": "allow",
            },
        },
    },
    "profile": {
        "dev": {
            "opt-level": 3,
            "package": {
                manifest["package"]["name"]: {"opt-level": 0}
                for path, manifest in members.items()
            },
        },
        "release": {
            "lto": True,
            "opt-level": 3,
        },
    },
}

with open("Cargo.toml", "wb") as file:
    tomli_w.dump(contents, file)
