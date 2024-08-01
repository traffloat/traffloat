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
                "version": "0.14",
                "default-features": False,
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
        "package": {
            "authors": ["SOFe <sofe2038@gmail.com>"],
            "version": "0.0.3",
            "edition": "2021",
            "repository": "https://github.com/traffloat/traffloat",
            "homepage": "https://github.com/traffloat/traffloat",
            "license": "AGPL-3.0",
            "rust-version": "1.79",
        },
    },
    "profile": {
        "dev": {
            "opt-level": 1,
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
