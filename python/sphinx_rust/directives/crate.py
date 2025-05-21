from __future__ import annotations

from typing import TYPE_CHECKING

from docutils import nodes
from sphinx import addnodes
from sphinx.util.logging import getLogger
from sphinx.util.nodes import make_id

from sphinx_rust.sphinx_rust import (
    load_child_enums,
    load_child_functions,
    load_child_modules,
    load_child_structs,
    load_crate,
    load_module,
)

from ._core import (
    RustAutoDirective,
    create_object_xref,
    create_source_xref,
    create_summary_table,
    parse_docstring,
)

if TYPE_CHECKING:
    from sphinx_rust.domain import ObjType
    from sphinx_rust.sphinx_rust import Enum, Function, Module, Struct

LOGGER = getLogger(__name__)


class RustCrateAutoDirective(RustAutoDirective):
    """Directive to auto-document a Rust crate."""

    def run(self) -> list[nodes.Node]:
        if self.is_nested():
            return []

        qualifier = self.arguments[0]
        try:
            crate = load_crate(self.cache_path, qualifier)
        except OSError as e:
            LOGGER.warning(
                f"Error loading crate {qualifier!r}: {e!s}",
                type="rust",
                subtype="cache",
            )
            return []

        if crate is None:
            LOGGER.warning(
                f"Crate {qualifier!r} not found in the rust cache",
                type="rust",
                subtype="cache",
            )
            return []

        crate_mod = load_module(self.cache_path, crate.name)

        # TODO self.env.note_dependency

        root = nodes.Element()

        # root += create_field_list(([nodes.Text("Version")], [nodes.Text(crate.version)]))
        root += nodes.paragraph("", f"Version: {crate.version}")

        desc = addnodes.desc()
        desc["objtype"] = f"{self.rust_domain.name}:crate"
        root += desc
        signature = addnodes.desc_signature(crate.name, f"pub mod {crate.name};")
        desc += signature
        node_id = make_id(self.env, self.doc, "", crate.name)
        signature["ids"].append(node_id)
        self.doc.note_explicit_target(signature)
        self.rust_domain.note_object(crate.name, "crate", node_id, signature)

        if self.rust_config.rust_viewcode and crate_mod and crate_mod.file:
            signature += create_source_xref(
                self.env.docname,
                crate_mod.path_str,
                text="[source]",
                classes=["viewcode-link"],
            )

        if crate_mod and crate_mod.docstring:
            root += parse_docstring(self.env, self.doc, crate_mod)

        items: list[Module | Struct | Enum | Function]
        objtype: ObjType
        for name, objtype, items in [  # type: ignore[assignment]
            ("Modules", "module", load_child_modules(self.cache_path, [crate.name])),
            ("Structs", "struct", load_child_structs(self.cache_path, [crate.name])),
            ("Enums", "enum", load_child_enums(self.cache_path, [crate.name])),
            (
                "Functions",
                "function",
                load_child_functions(self.cache_path, [crate.name]),
            ),
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
