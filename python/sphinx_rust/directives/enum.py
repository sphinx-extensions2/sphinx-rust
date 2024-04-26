from typing import TYPE_CHECKING

from docutils import nodes
from sphinx.util.logging import getLogger

from sphinx_rust.sphinx_rust import load_enum

from ._core import (
    RustAutoDirective,
    create_field_list,
    parse_docstring,
)

if TYPE_CHECKING:
    pass

LOGGER = getLogger(__name__)


class RustEnumAutoDirective(RustAutoDirective):
    """Directive to auto-document a Rust enum."""

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
            enum = load_enum(self.cache_path, qualifier)
        except OSError as e:
            LOGGER.warning(
                f"Error loading crate {qualifier!r}: {e!s}",
                type="rust",
                subtype="cache",
            )
            return []

        if enum is None:
            LOGGER.warning(
                f"enum {qualifier!r} not found in the rust cache",
                type="rust",
                subtype="cache",
            )
            return []

        # TODO self.env.note_dependency

        root = nodes.Element()

        # TODO signature block

        if enum.docstring:
            root += parse_docstring(self.env, self.doc, enum.docstring)

        if enum.variants:
            section = self.create_section("Variants")
            root += section
            section += create_field_list(
                [
                    (
                        [nodes.Text(var.name)],
                        parse_docstring(self.env, self.doc, var.docstring),
                    )
                    for var in enum.variants
                ]
            )

        return root.children
