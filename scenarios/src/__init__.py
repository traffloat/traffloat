import os
import shutil
from os import path

from . import all, assets, glossary, save


def main():
    proj_root = path.dirname(path.dirname(__file__))
    assets_dir = path.join(proj_root, "assets")
    if path.exists(assets_dir):
        shutil.rmtree(assets_dir)

    os.mkdir(assets_dir)

    asset_pool = assets.Pool()
    glossary_pool = glossary.Pool()

    for name, fn in all.scenarios.items():
        print(f"create scenario file {name}.tfsave")
        with save.WriterCtx(assets_dir, name, asset_pool, glossary_pool) as writer:
            fn(writer)

    for name, mesh in asset_pool.all.items():
        print(f"create asset file {name}: {mesh.hash}")
        with open(path.join(assets_dir, f"{mesh.hash}.glb"), "wb") as f:
            f.write(mesh.buf)

    for g in glossary_pool.all:
        print(f"create glossary file {g.name}: {g.sha_handle.sha}")
        os.mkdir(path.join(assets_dir, g.sha_handle.sha))
        for file in g.output:
            with open(
                path.join(assets_dir, g.sha_handle.sha, f"{file.locale}.tfglos"),
                "wb",
            ) as f:
                f.write(file.data)
