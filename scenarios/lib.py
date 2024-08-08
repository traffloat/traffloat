import inspect
import json
from os import path

def scenario(name):
    return WriterCtx(name)

class SaveType:
    Building = "traffloat.save.Building"
    Facility = "traffloat.save.Facility"
    Corridor = "traffloat.save.Corridor"
    Duct = "traffloat.save.Duct"

class WriterCtx:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        self.writer = Writer()
        return self.writer
    def __exit__(self, exc_type, exc_val, exc_tb):
        if exc_type is not None or exc_val is not None or exc_tb is not None:
            return

        file = path.join(path.dirname(__file__), f"{self.name}.tfsave")

        with open(file, "w", encoding="utf-8") as f:
            json.dump({
                "types": list(
                    {
                        "type": ty,
                        "defs": self.writer.types[ty],
                    }
                    for _, ty in inspect.getmembers(SaveType)
                    if type(ty) == str and ty in self.writer.types
                ),
            }, f, separators = (",", ":"))

class Writer:
    def __init__(self):
        self.types = {}

    def write(self, ty, item):
        items = self.types.setdefault(ty, [])
        id = len(items)
        items.append(item)
        return id
