from __future__ import annotations

from docutils import nodes
from sphinx import addnodes
from sphinx.util.logging import getLogger
from sphinx.util.nodes import make_id

from sphinx_rust.sphinx_rust import load_function

from ._core import (
    RustAutoDirective,
    parse_docstring,
)

LOGGER = getLogger(__name__)


class RustFunctionAutoDirective(RustAutoDirective):
    """Directive to auto-document a Rust function."""

    def run(self) -> list[nodes.Node]:
        if self.is_nested():
            return []

        qualifier = self.arguments[0]
        try:
            func = load_function(self.cache_path, qualifier)
        except OSError as e:
            LOGGER.warning(
                f"Error loading function {qualifier!r}: {e!s}",
                type="rust",
                subtype="cache",
            )
            return []

        if func is None:
            LOGGER.warning(
                f"function {qualifier!r} not found in the rust cache",
                type="rust",
                subtype="cache",
            )
            return []

        # TODO self.env.note_dependency

        root = nodes.Element()

        desc = addnodes.desc()
        desc["objtype"] = f"{self.rust_domain.name}:function"
        root += desc
        signature = addnodes.desc_signature(
            func.path_str,
            f"pub fn {func.name}(...);",
        )
        desc += signature
        # TODO add inputs / outputs to signature
        # desc += addnodes.desc_content("", nodes.paragraph("", ))
        node_id = make_id(self.env, self.doc, "", func.path_str)
        signature["ids"].append(node_id)
        self.doc.note_explicit_target(signature)
        self.rust_domain.note_object(func.path_str, "function", node_id, signature)

        if func.docstring:
            root += parse_docstring(self.env, self.doc, func)

        return root.children
