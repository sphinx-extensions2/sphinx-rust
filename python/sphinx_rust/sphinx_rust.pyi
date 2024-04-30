from __future__ import annotations

__version__: str

def analyze_crate(crate_path: str, cache_path: str) -> AnalysisResult:
    """Analyse a crate and cache the results to disk.

    :param crate_path: The path to the crate to analyse.
    :param cache_path: The path to the cache directory (must exist).
    :raises IOError: If the analysis fails.
    """

def load_crate(cache_path: str, name: str, /) -> Crate | None:
    """Load a crate from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    :param name: The name of the crate to load.
    :raises IOError: If the load fails.
    """

def load_module(cache_path: str, full_name: str, /) -> Module | None:
    """Load a module from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    :param name: The fully qualified name of the module to load, e.g. ``a::b::c``.
    :raises IOError: If the load fails.
    """

def load_struct(cache_path: str, full_name: str, /) -> Struct | None:
    """Load a struct from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    :param name: The fully qualified name of the struct to load, e.g. ``a::b::c``.
    :raises IOError: If the load fails.
    """

def load_enum(cache_path: str, full_name: str, /) -> Enum | None:
    """Load an enum from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    :param full_name: The fully qualified name of the enum to load, e.g. ``a::b::c``.
    :raises IOError: If the load fails.
    """

def load_function(cache_path: str, full_name: str, /) -> Function | None:
    """Load a function from the cache, it it exists.

    :param cache_path: The path to the cache directory.
    param full_name: The fully qualified name of the function to load, e.g. ``a::b::c``.
    :raises IOError: If the load fails.
    """

def load_child_modules(cache_path: str, parent: list[str], /) -> list[Module]:
    """Load all modules from the cache that are children of the given parent

    :param cache_path: The path to the cache directory.
    :param parent: The fully qualified name of the ancestor.
    :raises IOError: If the load fails.
    """

def load_child_structs(cache_path: str, parent: list[str], /) -> list[Struct]:
    """Load all structs from the cache that are children of the given parent

    :param cache_path: The path to the cache directory.
    :param parent: The fully qualified name of the parent.
    :raises IOError: If the load fails.
    """

def load_child_enums(cache_path: str, parent: list[str], /) -> list[Enum]:
    """Load all enums from the cache that are children of the given parent

    :param cache_path: The path to the cache directory.
    :param parent: The fully qualified name of the parent.
    :raises IOError: If the load fails.
    """

def load_child_functions(cache_path: str, parent: list[str], /) -> list[Function]:
    """Load all functions from the cache that are children of the given parent

    :param cache_path: The path to the cache directory.
    :param parent: The fully qualified name of the parent.
    :raises IOError: If the load fails.
    """

def load_descendant_modules(
    cache_path: str, ancestor: list[str], include_self: bool, /
) -> list[Module]:
    """Load all modules from the cache that have a common ancestor.

    :param cache_path: The path to the cache directory.
    :param ancestor: The fully qualified name of the ancestor.
    :param include_self: Whether to include the ancestor in the results.
    :raises IOError: If the load fails.
    """

def load_descendant_structs(cache_path: str, ancestor: list[str], /) -> list[Struct]:
    """Load all structs from the cache that have a common ancestor.

    :param cache_path: The path to the cache directory.
    :param ancestor: The fully qualified name of the ancestor.
    :raises IOError: If the load fails.
    """

def load_descendant_enums(cache_path: str, ancestor: list[str], /) -> list[Enum]:
    """Load all enums from the cache that have a common ancestor.

    :param cache_path: The path to the cache directory.
    :param ancestor: The fully qualified name of the ancestor.
    :raises IOError: If the load fails.
    """

class AnalysisResult:
    """Representation of the result of an analysis."""

    crate_: str
    modules: list[str]
    structs: list[str]
    enums: list[str]
    functions: list[str]

class Crate:
    """Representation of a crate."""

    name: str
    """The name of the crate."""
    version: str
    """The version of the crate."""

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

class Function:
    """Representation of a function."""

    name: str
    """The name of the struct."""
    path: list[str]
    """The fully qualified path"""
    path_str: str
    """The fully qualified name as a string, e.g. ``a::b::c``"""
    docstring: str
