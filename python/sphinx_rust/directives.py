from docutils import nodes
from sphinx.util.docutils import SphinxDirective
from sphinx.util.logging import getLogger

from sphinx_rust.sphinx_rust import load_crate, load_enums, load_modules, load_structs

LOGGER = getLogger(__name__)


class RustCrateDirective(SphinxDirective):
    """Directive to auto-document a Rust crate."""

    required_arguments = 1
    optional_arguments = 0
    final_argument_whitespace = False
    has_content = False
    option_spec = {}

    @property
    def doc(self) -> nodes.document:
        return self.state.document  # type: ignore[no-any-return]

    @property
    def cache_path(self) -> str:
        return str(self.env.rust_cache_path)  # type: ignore[attr-defined]

    def create_section(self, title: str) -> nodes.section:
        section = nodes.section()
        self.set_source_info(section)
        section += nodes.title(text=title)
        self.doc.note_implicit_target(section, section)
        return section

    def parse_docstring(self, docstring: str) -> nodes.paragraph:  # noqa: PLR6301
        # TODO proper parsing
        return nodes.paragraph(text=docstring)

    def run(self) -> list[nodes.Node]:  # noqa: PLR0912, PLR0914, PLR0915
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
        crate = load_crate(self.cache_path, qualifier)

        if crate is None:
            LOGGER.warning(
                f"Crate {qualifier!r} not found in the rust cache",
                type="rust",
                subtype="cache",
            )
            return []

        # TODO self.env.note_dependency

        root = self.create_section(f"Crate {qualifier}")
        # TODO add version
        if crate.docstring:
            root += self.parse_docstring(crate.docstring)

        modules = load_modules(
            self.cache_path, qualifier + "::"
        )  # TODO limit to only direct children?
        if modules:
            modules_section = self.create_section("Modules")
            root += modules_section
            for module in sorted(modules, key=lambda m: m.name):
                module_section = self.create_section(module.name)
                modules_section += module_section
                if module.docstring:
                    module_section += self.parse_docstring(module.docstring)

        structs = load_structs(
            self.cache_path, qualifier + "::"
        )  # TODO limit to only direct children?
        if structs:
            structs_section = self.create_section("Structs")
            root += structs_section
            for struct in sorted(structs, key=lambda m: m.name):
                struct_section = self.create_section(struct.name)
                structs_section += struct_section
                if struct.docstring:
                    struct_section += self.parse_docstring(struct.docstring)
                if struct.fields:
                    fields_section = self.create_section("Fields")
                    struct_section += fields_section
                    field_list = nodes.field_list()
                    fields_section += field_list
                    for field in sorted(struct.fields, key=lambda m: m.name):
                        field_node = nodes.field("", nodes.field_name(text=field.name))
                        field_list += field_node
                        if field.docstring:
                            field_list += nodes.field_body(
                                "", self.parse_docstring(field.docstring)
                            )

        enums = load_enums(
            self.cache_path, qualifier + "::"
        )  # TODO limit to only direct children?
        if enums:
            enums_section = self.create_section("Enums")
            root += enums_section
            for enum in sorted(enums, key=lambda m: m.name):
                enum_section = self.create_section(enum.name)
                enums_section += enum_section
                if enum.docstring:
                    enums_section += self.parse_docstring(enum.docstring)
                if enum.variants:
                    vars_section = self.create_section("Variants")
                    enums_section += vars_section
                    var_list = nodes.field_list()
                    vars_section += var_list
                    for var in sorted(enum.variants, key=lambda m: m.name):
                        var_node = nodes.field("", nodes.field_name(text=var.name))
                        var_list += var_node
                        if var.docstring:
                            var_list += nodes.field_body(
                                "", self.parse_docstring(var.docstring)
                            )

        return [root]
