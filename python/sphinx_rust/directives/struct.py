from __future__ import annotations

from docutils import nodes
from sphinx import addnodes
from sphinx.util.logging import getLogger
from sphinx.util.nodes import make_id

from sphinx_rust.sphinx_rust import load_struct

from ._core import (
    RustAutoDirective,
    create_field_list,
    parse_docstring,
    type_segs_to_nodes,
)

LOGGER = getLogger(__name__)


class RustStructAutoDirective(RustAutoDirective):
    """Directive to auto-document a Rust struct."""

    def run(self) -> list[nodes.Node]:
        if self.is_nested():
            return []

        qualifier = self.arguments[0]
        try:
            struct = load_struct(self.cache_path, qualifier)
        except OSError as e:
            LOGGER.warning(
                f"Error loading struct {qualifier!r}: {e!s}",
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

        desc = addnodes.desc()
        desc["objtype"] = f"{self.rust_domain.name}:struct"
        root += desc
        if struct.fields:
            sig_lines = [
                addnodes.desc_signature_line("", f"pub struct {struct.name} {{")
            ]
            # TODO properly print structs with tuple fields
            sig_lines.extend(
                addnodes.desc_signature_line(
                    "",
                    f"    pub {field.name}: ",
                    *type_segs_to_nodes(field.type_),
                    nodes.Text(","),
                )
                for field in struct.fields
            )
            sig_lines.append(addnodes.desc_signature_line("", "}"))
            signature = addnodes.desc_signature(struct.path_str, "", *sig_lines)
            signature["is_multiline"] = True
        else:
            signature = addnodes.desc_signature(
                struct.path_str,
                f"pub struct {struct.name}(/* private fields */);",
            )
        desc += signature
        # TODO add fields to signature
        # desc += addnodes.desc_content("", nodes.paragraph("", ))
        node_id = make_id(self.env, self.doc, "", struct.path_str)
        signature["ids"].append(node_id)
        self.doc.note_explicit_target(signature)
        self.rust_domain.note_object(struct.path_str, "struct", node_id, signature)

        if struct.docstring:
            root += parse_docstring(self.env, self.doc, struct)

        if struct.fields:
            section = self.create_section("Fields")
            root += section
            section += create_field_list(
                [
                    (
                        [nodes.Text(field.name or str(i))],
                        [
                            nodes.paragraph("", "", *type_segs_to_nodes(field.type_)),
                            *parse_docstring(self.env, self.doc, field),
                        ],
                    )
                    for i, field in enumerate(struct.fields)
                ]
            )

        return root.children
