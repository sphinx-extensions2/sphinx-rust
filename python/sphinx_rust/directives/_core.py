from __future__ import annotations

from typing import TYPE_CHECKING, Protocol

from docutils import nodes, utils
from sphinx import addnodes
from sphinx.errors import SphinxError
from sphinx.util.docutils import LoggingReporter, SphinxDirective
from sphinx.util.logging import getLogger

from sphinx_rust.config import RustConfig

if TYPE_CHECKING:
    from sphinx.environment import BuildEnvironment

    from sphinx_rust.domain import ObjType, RustDomain
    from sphinx_rust.sphinx_rust import TypeSegment


LOGGER = getLogger(__name__)


class RustAutoDirective(SphinxDirective):
    """Base directive to auto-document a Rust object."""

    required_arguments = 1
    optional_arguments = 0
    final_argument_whitespace = False
    has_content = False
    option_spec = {}

    @property
    def doc(self) -> nodes.document:
        return self.state.document  # type: ignore[no-any-return]

    @property
    def rust_config(self) -> RustConfig:
        return RustConfig.from_app(self.env.app)

    @property
    def rust_domain(self) -> RustDomain:
        # avoid circular import
        from sphinx_rust.domain import RustDomain  # noqa: PLC0415

        return self.env.domains[RustDomain.name]  # type: ignore[return-value]

    @property
    def cache_path(self) -> str:
        return str(self.env.rust_cache_path)  # type: ignore[attr-defined]

    def is_nested(self, warn: bool = True) -> bool:
        """Check if the directive is nested inside another directive.

        If we are going to generate section nodes, then this is not allowed,
        since it would break the documentation structure.
        """
        if warn and not self.state_machine.match_titles:
            # we are going to generate section nodes, and they will not work
            # if e.g. the directive is called from inside a directive
            LOGGER.warning(
                f"{self.name!r} directive can only be used at the root of the document",
                type="rust",
                subtype="root",
            )
        return not self.state_machine.match_titles

    def create_section(self, title: str) -> nodes.section:
        """Create a new section node."""
        section = nodes.section()
        self.set_source_info(section)
        section += nodes.title(text=title)
        self.doc.note_implicit_target(section, section)
        return section


def create_field_list(
    fields: list[tuple[list[nodes.Node], list[nodes.Node]]],
) -> nodes.field_list:
    """Create a field list from a list of field names and bodies."""
    field_list = nodes.field_list()
    for name, body in fields:
        field = nodes.field()
        field += nodes.field_name("", "", *name)
        field += nodes.field_body("", *body)
        field_list += field
    return field_list


def create_summary_table(
    rows: list[tuple[list[nodes.Node], list[nodes.Node]]],
) -> nodes.table:
    """Create a table with two columns from a list of rows."""
    table = nodes.table(align="left", classes=["colwidths-auto"])
    tgroup = nodes.tgroup(cols=2)
    table += tgroup
    tgroup += nodes.colspec(colwidth=1)
    tgroup += nodes.colspec(colwidth=1)
    tbody = nodes.tbody()
    tgroup += tbody
    for left, right in rows:
        row = nodes.row()
        tbody += row
        for cell in (left, right):
            entry = nodes.entry()
            row += entry
            entry += cell
    return table


class DocstringItem(Protocol):
    """An item with a docstring."""

    path: list[str]
    """Fully qualified name of the item."""
    path_str: str
    """Fully qualified name of the item."""
    docstring: str
    """The docstring of the item."""


def parse_docstring(
    env: BuildEnvironment,
    doc: nodes.document,
    item: DocstringItem,
    /,
    *,
    docstring: str | None = None,
) -> list[nodes.Node]:
    """parse into a dummy document and return created nodes.

    :param env: The build environment.
    :param doc: The parent document.
    :param item: The item to parse.
    :param docstring: If not ``None`` then use this as the docstring, rather than the items.
    """
    config = RustConfig.from_app(env.app)
    parser_type = config.rust_doc_formats.get(item.path[0], "restructuredtext")
    try:
        parser = env.app.registry.create_source_parser(env.app, parser_type)
    except SphinxError as e:
        LOGGER.warning(
            f"Error creating docstring parser for {item.path_str!r}: {e!s}",
            type="rust",
            subtype="parser",
        )
        return []

    docstring = item.docstring if docstring is None else docstring
    source_path = env.doc2path(  # TODO this actually should be the rust file path
        env.docname
    )
    # TODO how to handle line numbers?
    document = utils.new_document(source_path, doc.settings)
    document.reporter = LoggingReporter.from_reporter(doc.reporter)
    document.reporter.source = source_path
    # TODO cache parser creation

    parser.parse(docstring, document)
    # TODO merge document metadata with parent document, e.g. targets etc?
    # or docutils.Include actually runs the transforms on the included document, before returning its children
    return document.children


def create_object_xref(
    docname: str, full_name: str, objtype: ObjType, *, warn_dangling: bool = False
) -> addnodes.pending_xref:
    """Create a cross-reference node to a rust object.

    :param docname: The document name.
    :param path: The fully qualified path to the object, e.g. ``crate::module::Item``.
    """
    options = {
        "refdoc": docname,
        "refdomain": "rust",
        "reftype": objtype,
        "refexplicit": True,
        "refwarn": warn_dangling,
        "reftarget": full_name,
    }
    ref = addnodes.pending_xref(full_name, **options)
    name = full_name.split("::")[-1]
    ref += nodes.literal(name, name)

    return ref


def create_source_xref(
    docname: str,
    full_name: str,
    *,
    warn_dangling: bool = False,
    text: str | None = None,
    classes: list[str] | None = None,
) -> addnodes.pending_xref:
    """Create a cross-reference node to the source-code of a rust object.

    :param docname: The document name.
    :param path: The fully qualified path to the object, e.g. ``crate::module::Item``.
    """
    options = {
        "refdoc": docname,
        "refdomain": "std",
        "reftype": "ref",
        "refexplicit": True,
        "refwarn": warn_dangling,
        "reftarget": f"rust-code:{full_name}",
        "classes": classes or [],
    }
    ref = addnodes.pending_xref(full_name, **options)
    text = full_name.split("::")[-1] if text is None else text
    ref += nodes.literal(text, text)

    return ref


def type_segs_to_nodes(segs: list[TypeSegment]) -> list[nodes.Node]:
    """Convert a list of type segments to nodes."""
    nodes_: list[nodes.Node] = []
    for seg in segs:
        if seg.is_path:
            # TODO create cross-reference
            nodes_.append(nodes.strong("", seg.content))
        else:
            nodes_.append(nodes.Text(seg.content))
    return nodes_
