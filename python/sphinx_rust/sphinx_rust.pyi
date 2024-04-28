from __future__ import annotations

__version__: str

def analyze_crate(crate_path: str, cache_path: str) -> AnalysisResult:
    """Analyse a crate and cache the results to disk.

    :param crate_path: The path to the crate to analyse.
    :param cache_path: The path to the cache directory (must exist).
    :raises IOError: If the analysis fails.
    """

def load_crate(cache_path: str, name: str) -> Crate | None:
    """Load a crate from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    :param name: The name of the crate to load.
    :raises IOError: If the load fails.
    """

def load_module(cache_path: str, name: str) -> Module | None:
    """Load a module from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    :param name: The fully qualified name of the module to load.
    :raises IOError: If the load fails.
    """

def load_struct(cache_path: str, name: str) -> Struct | None:
    """Load a struct from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    :param name: The fully qualified name of the struct to load.
    :raises IOError: If the load fails.
    """

def load_enum(cache_path: str, name: str) -> Enum | None:
    """Load an enum from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    :param name: The fully qualified name of the enum to load.
    :raises IOError: If the load fails.
    """

def load_modules(cache_path: str, prefix: str) -> list[Module]:
    """Load all modules from the cache, whose fully qualified name begins with the prefix.

    :param cache_path: The path to the cache directory.
    :param prefix: The fully qualified name prefix of the modules to load.
    :raises IOError: If the load fails.
    """

def load_structs(cache_path: str, prefix: str) -> list[Struct]:
    """Load all structs from the cache, whose fully qualified name begins with the prefix.

    :param cache_path: The path to the cache directory.
    :param prefix: The fully qualified name prefix of the structs to load.
    :raises IOError: If the load fails.
    """

def load_enums(cache_path: str, prefix: str) -> list[Enum]:
    """Load all enums from the cache, whose fully qualified name begins with the prefix.

    :param cache_path: The path to the cache directory.
    :param prefix: The fully qualified name prefix of the enums to load.
    :raises IOError: If the load fails.
    """

class AnalysisResult:
    """Representation of the result of an analysis."""

    crate_: str
    modules: list[str]
    structs: list[str]
    enums: list[str]

class Crate:
    """Representation of a crate."""

    name: str
    """The name of the crate."""
    path: list[str]
    """The fully qualified path"""
    path_str: str
    """The fully qualified name as a string, e.g. ``a::b::c``"""
    version: str
    docstring: str

class Module:
    """Representation of a module."""

    file: str | None
    """The absolute path to the file containing the module."""
    name: str
    """The name of the module."""
    path: list[str]
    """The fully qualified path"""
    path_str: str
    """The fully qualified name as a string, e.g. ``a::b::c``"""
    docstring: str

class Struct:
    """Representation of a struct."""

    name: str
    """The name of the struct."""
    path: list[str]
    """The fully qualified path"""
    path_str: str
    """The fully qualified name as a string, e.g. ``a::b::c``"""
    docstring: str
    fields: list[Field]

class Enum:
    """Representation of an enum."""

    name: str
    """The name of the enum."""
    path: list[str]
    """The fully qualified path"""
    path_str: str
    """The fully qualified name as a string, e.g. ``a::b::c``"""
    docstring: str
    variants: list[Variant]

class Variant:
    """Representation of an enum variant."""

    name: str
    """The name of the variant."""
    path: list[str]
    """The fully qualified path"""
    path_str: str
    """The fully qualified name as a string, e.g. ``a::b::c``"""
    docstring: str
    fields: list[Field]

class Field:
    """Representation of a struct field."""

    name: str
    """The name of the field."""
    path: list[str]
    """The fully qualified path.

    Note, for fields of tuple structs, the final component is the index of the field
    """
    path_str: str
    """The fully qualified name as a string, e.g. ``a::b::c``"""
    docstring: str
    type_: list[TypeSegment]

class TypeSegment:
    """Representation of a segment of a type.

    Types are split into segments to allow for identification of referenceable elements
    """

    content: str
    is_path: bool
