from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import shutil
from typing import TYPE_CHECKING, Literal, TypedDict

from sphinx import addnodes
from sphinx.domains import Domain
from sphinx.util.logging import getLogger
from sphinx.util.nodes import make_refnode

from sphinx_rust.directives.crate import RustCrateAutoDirective
from sphinx_rust.directives.enum import RustEnumAutoDirective
from sphinx_rust.directives.module import RustModuleAutoDirective
from sphinx_rust.directives.struct import RustStructAutoDirective
from sphinx_rust.roles import (
    RustCrateXRefRole,
    RustEnumXRefRole,
    RustModuleXRefRole,
    RustStructXRefRole,
)
from sphinx_rust.sphinx_rust import analyze_crate

if TYPE_CHECKING:
    from docutils.nodes import Element
    from sphinx.addnodes import pending_xref
    from sphinx.application import Sphinx
    from sphinx.builders import Builder
    from sphinx.environment import BuildEnvironment

    from sphinx_rust.sphinx_rust import AnalysisResult


LOGGER = getLogger(__name__)


ObjType = Literal["crate", "module", "struct", "enum"]


@dataclass
class ObjectEntry:
    """An entry in the domain's object inventory."""

    name: str
    docname: str
    node_id: str
    objtype: ObjType


class DomainData(TypedDict):
    objects: dict[str, ObjectEntry]


class RustDomain(Domain):
    """Rust domain."""

    name = "rust"
    label = "Rust"

    directives = {
        "crate": RustCrateAutoDirective,
        "module": RustModuleAutoDirective,
        "struct": RustStructAutoDirective,
        "enum": RustEnumAutoDirective,
    }

    roles = {
        "crate": RustCrateXRefRole(),
        "module": RustModuleXRefRole(),
        "struct": RustStructXRefRole(),
        "enum": RustEnumXRefRole(),
    }

    initial_data: DomainData = {  # type: ignore[assignment]
        "objects": {},
    }

    @classmethod
    def app_setup(cls, app: Sphinx) -> None:
        app.add_config_value("rust_crates", [], "env")
        app.connect("builder-inited", cls.on_builder_inited)
        app.add_domain(cls)

    @staticmethod
    def on_builder_inited(app: Sphinx) -> None:
        """Analyze the Rust crates."""
        # create the cache directory
        app.env.rust_cache_path = cache = Path(str(app.outdir)) / "rust_cache"  # type: ignore[attr-defined]
        cache.mkdir(exist_ok=True)
        srcdir = Path(
            str(app.srcdir)
        )  # for back-compatibility, assume it might not be a Path
        for crate in app.config.rust_crates:
            path = Path(str(app.srcdir)) / str(crate)
            # analyze the crate
            LOGGER.info(f"[rust] Analyzing crate: {path.resolve()!s}")
            try:
                result = analyze_crate(str(path), str(cache))
            except OSError as e:
                LOGGER.warning(
                    f"Error analyzing crate: {e!s}", type="rust", subtype="analyze"
                )
            create_pages(srcdir, result)

    @property
    def objects(self) -> dict[str, ObjectEntry]:
        """fullname -> ObjectEntry mapping."""
        return self.data.setdefault("objects", {})  # type: ignore[no-any-return]

    def note_object(
        self,
        name: str,
        objtype: ObjType,
        node_id: str,
        _signature: addnodes.desc_signature,
    ) -> None:
        # TODO check for duplicates
        self.objects[name] = ObjectEntry(name, self.env.docname, node_id, objtype)

    def clear_doc(self, docname: str) -> None:
        for fullname, obj in list(self.objects.items()):
            if obj.docname == docname:
                del self.objects[fullname]

    def merge_domaindata(self, docnames: list[str], otherdata: DomainData) -> None:  # type: ignore[override]
        for fullname, obj in otherdata["objects"].items():
            if obj.docname in docnames:
                self.objects[fullname] = obj

    def resolve_xref(  # noqa: PLR0913, PLR0917
        self,
        env: BuildEnvironment,
        fromdocname: str,
        builder: Builder,
        typ: str,
        target: str,
        node: pending_xref,
        contnode: Element,
    ) -> Element | None:
        if node.get("refdomain") != "rust":
            return None
        matches = [
            obj
            for name, obj in self.objects.items()
            if name == target and obj.objtype == typ
        ]
        if not matches:
            return None
        if len(matches) > 1:
            LOGGER.warning(
                f"Duplicate {typ} {target!r} in {fromdocname!r}",
                type="rust",
                subtype="resolve",
            )
        obj = matches[0]
        return make_refnode(builder, fromdocname, obj.docname, obj.node_id, contnode)

    def resolve_any_xref(  # noqa: PLR6301
        self,
        _env: BuildEnvironment,
        _fromdocname: str,
        _builder: Builder,
        _target: str,
        _node: pending_xref,
        _contnode: Element,
    ) -> list[tuple[str, Element]]:
        # TODO implement
        return []

    def get_objects(self) -> list[tuple[str, str, str, str, str, int]]:
        objects = []
        for name, obj in self.objects.items():
            objects.append((name, name, str(obj.objtype), obj.docname, obj.node_id, 1))
        return objects


def create_pages(srcdir: Path, result: AnalysisResult) -> None:
    """Create the pages for the analyzed crate."""
    # TODO don't write if not changed (but still remove outdated pages)
    # TODO write or append to .gitignore?
    root = srcdir.joinpath("api", "crates", result.crate_)
    if root.exists():
        shutil.rmtree(root)
    items = root.joinpath("items")
    items.mkdir(parents=True, exist_ok=True)
    pages = []
    for module in result.modules:
        module_title = "::".join(module.split("::")[1:])
        pages.append(module_title)
        title = f"Module ``{module_title}``"
        items.joinpath(f"{module_title}.rst").write_text(
            f"{title}\n{'=' * len(title)}\n\n.. rust:module:: {module}\n"
        )
    for struct in result.structs:
        struct_title = "::".join(struct.split("::")[1:])
        pages.append(struct_title)
        title = f"Struct ``{struct_title}``"
        items.joinpath(f"{struct_title}.rst").write_text(
            f"{title}\n{'=' * len(title)}\n\n.. rust:struct:: {struct}\n"
        )
    for enum in result.enums:
        enum_title = "::".join(enum.split("::")[1:])
        pages.append(enum_title)
        title = f"Enum ``{enum_title}``"
        items.joinpath(f"{enum_title}.rst").write_text(
            f"{title}\n{'=' * len(title)}\n\n.. rust:enum:: {enum}\n"
        )
    title = f"Crate ``{result.crate_}``"
    index_content = f"{title}\n{'=' * len(title)}\n\n.. rust:crate:: {result.crate_}"
    if pages:
        index_content += "\n\n.. toctree::\n    :maxdepth: 1\n    :hidden:\n\n"
        for page in pages:
            index_content += f"    items/{page}\n"
    root.joinpath("index.rst").write_text(index_content)
