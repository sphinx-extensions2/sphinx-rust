from __future__ import annotations

from typing import TYPE_CHECKING

from docutils import nodes, utils
from sphinx import addnodes
from sphinx.util.docutils import LoggingReporter, SphinxDirective
from sphinx.util.logging import getLogger

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
        return self.state_machine.match_titles  # type: ignore[no-any-return]

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


def parse_docstring(
    env: BuildEnvironment,
    doc: nodes.document,
    docstring: str,
    /,
    *,
    filetype: str = "restructuredtext",
) -> list[nodes.Node]:
    """parse into a dummy document and return created nodes."""
    source_path = env.doc2path(  # TODO this actually should be the rust file path
        env.docname
    )
    # TODO how to handle line numbers?
    document = utils.new_document(source_path, doc.settings)
    document.reporter = LoggingReporter.from_reporter(doc.reporter)
    document.reporter.source = source_path
    # TODO cache parser creation
    parser = env.app.registry.create_source_parser(env.app, filetype)
    parser.parse(docstring, document)
    # TODO merge document metadata with parent document, e.g. targets etc?
    # or docutils.Include actually runs the transforms on the included document, before returning its children
    return document.children


def create_xref(
    docname: str, ident: str, objtype: ObjType, *, warn_dangling: bool = False
) -> addnodes.pending_xref:
    """Create a cross-reference node."""
    options = {
        "refdoc": docname,
        "refdomain": "rust",
        "reftype": objtype,
        "refexplicit": True,
        "refwarn": warn_dangling,
        "reftarget": ident,
    }
    ref = addnodes.pending_xref(ident, **options)
    name = ident.split("::")[-1]
    ref += nodes.literal(name, name)

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
