from __future__ import annotations

from docutils import nodes
from sphinx import addnodes
from sphinx.util.logging import getLogger
from sphinx.util.nodes import make_id

from sphinx_rust.sphinx_rust import load_enum

from ._core import (
    RustAutoDirective,
    create_field_list,
    parse_docstring,
)

LOGGER = getLogger(__name__)


class RustEnumAutoDirective(RustAutoDirective):
    """Directive to auto-document a Rust enum."""

    def run(self) -> list[nodes.Node]:
        if self.is_nested():
            return []

        qualifier = self.arguments[0]
        try:
            enum = load_enum(self.cache_path, qualifier)
        except OSError as e:
            LOGGER.warning(
                f"Error loading enum {qualifier!r}: {e!s}",
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

        desc = addnodes.desc()
        root += desc
        if enum.variants:
            desc["objtype"] = f"{self.rust_domain.name}:struct"
            sig_lines = [addnodes.desc_signature_line("", f"pub struct {enum.name} {{")]
            for var in enum.variants:
                # TODO types
                sig_lines.append(  # noqa: PERF401
                    addnodes.desc_signature_line(
                        "", f"    {var.name}(...),"
                    )  # TODO properly print variant
                )
            sig_lines.append(addnodes.desc_signature_line("", "}"))
            signature = addnodes.desc_signature(enum.path_str, "", *sig_lines)
            signature["is_multiline"] = True
        else:
            desc["objtype"] = f"{self.rust_domain.name}:enum"
            signature = addnodes.desc_signature(
                enum.path_str, f"pub enum {enum.name} {{}}"
            )
        desc += signature
        # TODO add variants to signature
        # desc += addnodes.desc_content("", nodes.paragraph("", ))
        node_id = make_id(self.env, self.doc, "", enum.path_str)
        signature["ids"].append(node_id)
        self.doc.note_explicit_target(signature)
        self.rust_domain.note_object(enum.path_str, "enum", node_id, signature)

        if enum.docstring:
            root += parse_docstring(self.env, self.doc, enum)

        if enum.variants:
            section = self.create_section("Variants")
            root += section
            # TODO document variant fields
            section += create_field_list(
                [
                    (
                        [nodes.Text(var.name)],
                        parse_docstring(self.env, self.doc, var),
                    )
                    for var in enum.variants
                ]
            )

        return root.children
