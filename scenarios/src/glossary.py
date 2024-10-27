from dataclasses import dataclass, field
from typing import Any, Self, Union, Optional
import hashlib
import json


@dataclass
class ShaHandle:
    sha: Optional[str] = None


@dataclass
class Id:
    sha_handle: ShaHandle
    index: int


@dataclass
class Element:
    json: Any

    def literal(content: str) -> Self:
        return Element(
            json={
                "type": "Literal",
                "content": content,
            }
        )

    def parameter(index: int) -> Self:
        return Element(
            json={
                "type": "Parameter",
                "index": index,
            }
        )


@dataclass
class LocalEntry:
    items: list[Element] = field(default=list)

    def build(elements: Union[list[Union[Element, str]], str]):
        elements = elements if isinstance(elements, list) else [elements]
        return LocalEntry(
            items=[
                element if isinstance(element, Element) else Element.literal(element)
                for element in elements
            ]
        )


@dataclass
class Entry:
    base: LocalEntry
    locales: dict[str, LocalEntry] = field(default=dict)


@dataclass
class File:
    locale: str
    data: bytes


@dataclass
class Glossary:
    name: str

    items: list[Entry] = field(default_factory=list)

    sha_handle: ShaHandle = field(default_factory=ShaHandle)
    output: Optional[list[File]] = None

    def add(
        self,
        base: Union[list[Union[Element, str]], str],
        **locales: dict[str, Union[list[Union[Element, str]], str]]
    ) -> Id:
        index = len(self.items)
        self.items.append(
            Entry(
                base=LocalEntry.build(base),
                locales={
                    locale: LocalEntry.build(elements)
                    for locale, elements in locales.items()
                },
            )
        )
        return Id(sha_handle=self.sha_handle, index=index)

    def finalize(self, locales: list[str]):
        output = []
        for locale in locales:
            json_data = json.dumps(
                {
                    "locale": locale,
                    "entries": [
                        {
                            "elements": [
                                element.json
                                for element in entry.locales.get(
                                    locale, entry.base
                                ).items
                            ]
                        }
                        for entry in self.items
                    ],
                },
                separators=(",\n", ":"),
            ).encode("utf-8")
            output.append(File(locale, json_data))

        output.sort(key=lambda file: file.locale)

        hash = (
            hashlib.sha1(b"".join(file.data for file in output), usedforsecurity=False)
            .digest()
            .hex()
        )
        self.output = output
        self.sha_handle.sha = hash


@dataclass
class Pool:
    all: list[Glossary] = field(default_factory=list)

    def push_finalized(self, glossary: Glossary):
        self.all.append(glossary)
