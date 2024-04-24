from docutils import nodes
from sphinx.util.docutils import SphinxDirective
from sphinx.util.logging import getLogger

from sphinx_rust.sphinx_rust import module_from_id_path

LOGGER = getLogger(__name__)


class RustModuleDirective(SphinxDirective):
    required_arguments = 1
    optional_arguments = 0
    final_argument_whitespace = False
    has_content = False
    option_spec = {}

    @property
    def doc(self) -> nodes.document:
        return self.state.document  # type: ignore[no-any-return]

    def create_section(self, title: str) -> nodes.section:
        section = nodes.section()
        self.set_source_info(section)
        section += nodes.title(text=title)
        self.doc.note_implicit_target(section, section)
        return section

    def parse_docstring(self, docstring: str) -> nodes.paragraph:  # noqa: PLR6301
        # TODO proper parsing
        return nodes.paragraph(text=docstring)

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
        # TODO handle errors
        module = module_from_id_path(qualifier)

        # TODO self.env.note_dependency

        root = self.create_section(f"Module {qualifier}")

        if module.docstring:
            root += self.parse_docstring(module.docstring)

        if module.structs:
            structs_section = self.create_section("Structs")
            root += structs_section
            for struct in module.structs:
                struct_section = self.create_section(f"Struct {struct.name}")
                structs_section += struct_section
                if struct.docstring:
                    struct_section += self.parse_docstring(struct.docstring)

        if module.enums:
            enums_section = self.create_section("Enums")
            root += enums_section
            for enum in module.enums:
                enum_section = self.create_section(f"enum {enum.name}")
                enums_section += enum_section
                if enum.docstring:
                    enums_section += self.parse_docstring(enum.docstring)

        return [root]
