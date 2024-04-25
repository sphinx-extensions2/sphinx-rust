from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

from sphinx.domains import Domain

from sphinx_rust.directives import RustCrateDirective
from sphinx_rust.sphinx_rust import analyze_crate

# from sphinx.directives import ObjectDescription
# from sphinx.domains import Domain, ObjType
# from sphinx.roles import XRefRole


if TYPE_CHECKING:
    from docutils.nodes import Element
    from sphinx.addnodes import pending_xref
    from sphinx.application import Sphinx
    from sphinx.builders import Builder
    from sphinx.environment import BuildEnvironment


class RustDomain(Domain):
    """Rust domain."""

    name = "rust"
    label = "Rust"

    directives = {
        "crate": RustCrateDirective,
    }

    @classmethod
    def app_setup(cls, app: Sphinx) -> None:
        app.add_config_value("rust_crates", [], "env")
        app.connect("builder-inited", cls.on_builder_inited)
        app.add_domain(cls)

    @staticmethod
    def on_builder_inited(app: Sphinx) -> None:
        # create the cache directory
        app.env.rust_cache_path = cache = Path(str(app.outdir)) / "rust_cache"  # type: ignore[attr-defined]
        cache.mkdir(exist_ok=True)
        # analyze the crates
        for crate in app.config.rust_crates:
            path = Path(str(app.srcdir)) / str(crate)
            # TODO log info, handle errors
            analyze_crate(str(path), str(cache))

    def merge_domaindata(
        self, _docnames: list[str], _otherdata: dict[str, Any]
    ) -> None:
        pass

    def resolve_any_xref(  # noqa: PLR6301
        self,
        _env: BuildEnvironment,
        _fromdocname: str,
        _builder: Builder,
        _target: str,
        _node: pending_xref,
        _contnode: Element,
    ) -> list[tuple[str, Element]]:
        return []


# class RustModuleDirective(ObjectDescription[str]):
#     """Directive to document a Rust module."""

#     def handle_signature(self, sig: str, signode: desc_signature) -> str:
#         return sig

#     def add_target_and_index(
#         self, name: str, sig: str, signode: desc_signature
#     ) -> None:
#         pass


# class RustStructDirective(ObjectDescription[str]):
#     """Directive to document a Rust struct."""

#     def handle_signature(self, sig: str, signode: desc_signature) -> str:
#         return sig

#     def add_target_and_index(
#         self, name: str, sig: str, signode: desc_signature
#     ) -> None:
#         pass


# class RustFieldDirective(ObjectDescription[str]):
#     """Directive to document a Rust struct field."""

#     def handle_signature(self, sig: str, signode: desc_signature) -> str:
#         return sig

#     def add_target_and_index(
#         self, name: str, sig: str, signode: desc_signature
#     ) -> None:
#         pass


# class RustModuleRole(XRefRole):
#     """Role to cross-reference a Rust module."""


# class RustStructRole(XRefRole):
#     """Role to cross-reference a Rust struct."""


# class RustFieldRole(XRefRole):
#     """Role to cross-reference a Rust struct field."""
