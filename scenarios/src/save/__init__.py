import inspect
import json
from abc import abstractmethod
from dataclasses import dataclass
from os import path
from typing import Any, Generic, TypeVar

from .. import assets, glossary


class WriterCtx:
    def __init__(
        self, dir: str, name: str, asset_pool: assets.Pool, glossary_pool: glossary.Pool
    ):
        self.dir = dir
        self.name = name
        self.writer = Writer(asset_pool, glossary_pool)

    def __enter__(self):
        return self.writer

    def __exit__(self, exc_type, exc_val, exc_tb):
        if exc_type is not None or exc_val is not None or exc_tb is not None:
            return

        file = path.join(self.dir, f"{self.name}.tfsave")

        with open(file, "w", encoding="utf-8") as f:
            json.dump(
                {
                    "types": list(
                        {
                            "type": ty.save_id(),
                            "defs": defs,
                        }
                        for ty, defs in self.writer.types.items()
                    ),
                },
                f,
                separators=(",\n", ":"),
                cls=LateShaEncoder,
            )


class LateShaEncoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, glossary.ShaHandle):
            if o.sha is None:
                raise Exception("ShaHandle is not initialized yet")
            return o.sha

        return super().default(o)


class Def:
    @staticmethod
    @abstractmethod
    def save_id() -> str:
        """
        Type ID of the def entry.
        """


D = TypeVar("D", bound=Def)


@dataclass
class Id(Generic[D]):
    id: int

    def __hash__(self) -> int:
        return hash(self.id)


class Writer:
    def __init__(self, asset_pool: assets.Pool, glossary_pool: glossary.Pool):
        self.asset_pool = asset_pool
        self.glossary_pool = glossary_pool
        self.types: type[Def] = {}

    def write(self, ty: type[D], data: dict[str, Any]) -> Id[D]:
        items = self.types.setdefault(ty, [])
        id = len(items)
        items.append(data)
        return Id[D](id)
