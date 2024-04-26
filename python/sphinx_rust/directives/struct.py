from typing import TYPE_CHECKING

from docutils import nodes
from sphinx.util.logging import getLogger

from sphinx_rust.sphinx_rust import load_struct

from ._core import (
    RustAutoDirective,
    create_field_list,
    parse_docstring,
)

if TYPE_CHECKING:
    pass

LOGGER = getLogger(__name__)


class RustStructAutoDirective(RustAutoDirective):
    """Directive to auto-document a Rust struct."""

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
            struct = load_struct(self.cache_path, qualifier)
        except OSError as e:
            LOGGER.warning(
                f"Error loading crate {qualifier!r}: {e!s}",
                type="rust",
                subtype="cache",
            )
            return []

        if struct is None:
            LOGGER.warning(
                f"struct {qualifier!r} not found in the rust cache",
                type="rust",
                subtype="cache",
            )
            return []

        # TODO self.env.note_dependency

        root = nodes.Element()

        # TODO signature block

        if struct.docstring:
            root += parse_docstring(self.env, self.doc, struct.docstring)

        if struct.fields:
            section = self.create_section("Fields")
            root += section
            section += create_field_list(
                [
                    (
                        [nodes.Text(field.name)],
                        [
                            nodes.paragraph(
                                "",
                                "",
                                *[
                                    nodes.strong("", s.content)
                                    if s.is_path
                                    else nodes.Text(s.content)
                                    for s in field.type_
                                ],
                            ),
                            *parse_docstring(self.env, self.doc, field.docstring),
                        ],
                    )
                    for field in struct.fields
                ]
            )

        return root.children
