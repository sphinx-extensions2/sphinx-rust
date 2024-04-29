from __future__ import annotations

from typing import TYPE_CHECKING

from docutils import nodes
from sphinx import addnodes
from sphinx.util.logging import getLogger
from sphinx.util.nodes import make_id

from sphinx_rust.sphinx_rust import load_enums, load_module, load_modules, load_structs

from ._core import (
    RustAutoDirective,
    create_object_xref,
    create_source_xref,
    create_summary_table,
    parse_docstring,
)

if TYPE_CHECKING:
    from sphinx_rust.domain import ObjType
    from sphinx_rust.sphinx_rust import Enum, Module, Struct

LOGGER = getLogger(__name__)


class RustModuleAutoDirective(RustAutoDirective):
    """Directive to auto-document a Rust module."""

    def run(self) -> list[nodes.Node]:
        if self.is_nested():
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

        desc = addnodes.desc()
        root += desc
        signature = addnodes.desc_signature(module.path_str, f"pub mod {module.name};")
        desc += signature
        node_id = make_id(self.env, self.doc, "", module.path_str)
        signature["ids"].append(node_id)
        self.doc.note_explicit_target(signature)
        self.rust_domain.note_object(module.path_str, "module", node_id, signature)

        if self.rust_config.rust_viewcode and module and module.file:
            root += nodes.paragraph(
                "",
                "",
                create_source_xref(
                    self.env.docname, module.path_str, text="[View Source]"
                ),
            )

        if module.docstring:
            root += parse_docstring(self.env, self.doc, module)

        items: list[Module | Struct | Enum]
        objtype: ObjType
        for name, objtype, items in [  # type: ignore[assignment]
            ("Modules", "module", load_modules(self.cache_path, module.path, False)),
            ("Structs", "struct", load_structs(self.cache_path, module.path)),
            ("Enums", "enum", load_enums(self.cache_path, module.path)),
        ]:
            if items:
                section = self.create_section(name)
                root += section
                rows = [
                    (
                        [
                            nodes.paragraph(
                                "",
                                "",
                                create_object_xref(
                                    self.env.docname, item.path_str, objtype
                                ),
                            )
                        ],
                        parse_docstring(
                            self.env,
                            self.doc,
                            item,
                            # TODO the first line should only be a paragraph
                            docstring=item.docstring.splitlines()[0]
                            if item.docstring
                            else r"\-",
                        ),
                    )
                    for item in sorted(items, key=lambda m: m.path_str)
                ]
                section += create_summary_table(rows)  # type: ignore[arg-type]

        return root.children
