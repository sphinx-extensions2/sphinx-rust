from typing import TYPE_CHECKING

from docutils import nodes
from sphinx.util.logging import getLogger

from sphinx_rust.sphinx_rust import load_enums, load_module, load_modules, load_structs

from ._core import (
    RustAutoDirective,
    create_summary_table,
    parse_docstring,
)

if TYPE_CHECKING:
    from sphinx_rust.sphinx_rust import Enum, Module, Struct

LOGGER = getLogger(__name__)


class RustModuleAutoDirective(RustAutoDirective):
    """Directive to auto-document a Rust module."""

    def run(self) -> list[nodes.Node]:
        if not self.state_machine.match_titles:
            # we are going to generate section nodes, and they will not work
            # if e.g. the directive is called from inside a directive
            LOGGER.warning(
                f"{self.name!r} directive can only be used at the root of the document",
                type="rust",
                subtype="root",
            )
            return []

        qualifier = self.arguments[0]
        try:
            module = load_module(self.cache_path, qualifier)
        except OSError as e:
            LOGGER.warning(
                f"Error loading crate {qualifier!r}: {e!s}",
                type="rust",
                subtype="cache",
            )
            return []

        if module is None:
            LOGGER.warning(
                f"module {qualifier!r} not found in the rust cache",
                type="rust",
                subtype="cache",
            )
            return []

        # TODO self.env.note_dependency

        root = nodes.Element()

        if module.docstring:
            root += parse_docstring(self.env, self.doc, module.docstring)

        items: list[Module | Struct | Enum]
        for name, items in [  # type: ignore[assignment]
            ("Modules", load_modules(self.cache_path, qualifier + "::")),
            ("Structs", load_structs(self.cache_path, qualifier + "::")),
            ("Enums", load_enums(self.cache_path, qualifier + "::")),
        ]:
            if items:
                section = self.create_section(name)
                root += section
                rows = [
                    (
                        # TODO should be references not just text
                        [nodes.Text(item.name)],
                        parse_docstring(
                            self.env,
                            self.doc,
                            # TODO the first line should only be a paragraph
                            item.docstring.splitlines()[0] if item.docstring else "\-",
                        ),
                    )
                    for item in sorted(items, key=lambda m: m.name)
                ]
                section += create_summary_table(rows)  # type: ignore[arg-type]

        return root.children
