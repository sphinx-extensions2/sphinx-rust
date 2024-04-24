from __future__ import annotations

__version__: str

class Module:
    """Representation of a module."""

    name: str
    docstring: str
    structs: list[Struct]
    enums: list[Enum]

class Struct:
    """Representation of a struct."""

    name: str
    docstring: str

class Enum:
    """Representation of an enum."""

    name: str
    docstring: str

def module_from_id_path(identifier: str) -> Module:
    """Create a module from an identifier path, e.g. `crate::module`."""
