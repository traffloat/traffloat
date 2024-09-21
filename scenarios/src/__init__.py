import os
import shutil
from os import path

from . import all, assets, save


def main():
    proj_root = path.dirname(path.dirname(__file__))
    assets_dir = path.join(proj_root, "assets")
    if path.exists(assets_dir):
        shutil.rmtree(assets_dir)

    os.mkdir(assets_dir)

    pool = assets.Pool()

    for name, fn in all.scenarios.items():
        print(f"create scenario file {name}.tfsave")
        with save.WriterCtx(assets_dir, name, pool) as writer:
            fn(writer)

    for name, mesh in pool.all.items():
        print(f"create asset file {name}: {mesh.hash}")
        with open(path.join(assets_dir, f"{mesh.hash}.glb"), "wb") as f:
            f.write(mesh.buf)
