from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path
import shutil
from typing import TYPE_CHECKING, Literal, TypedDict, Union

from sphinx import addnodes
from sphinx.domains import Domain
from sphinx.roles import XRefRole
from sphinx.util.logging import getLogger
from sphinx.util.nodes import make_refnode

from sphinx_rust.config import RustConfig
from sphinx_rust.directives.crate import RustCrateAutoDirective
from sphinx_rust.directives.enum import RustEnumAutoDirective
from sphinx_rust.directives.function import RustFunctionAutoDirective
from sphinx_rust.directives.module import RustModuleAutoDirective
from sphinx_rust.directives.struct import RustStructAutoDirective
from sphinx_rust.sphinx_rust import analyze_crate, load_descendant_modules

if TYPE_CHECKING:
    from docutils.nodes import Element
    from sphinx.addnodes import pending_xref
    from sphinx.application import Sphinx
    from sphinx.builders import Builder
    from sphinx.environment import BuildEnvironment

    from sphinx_rust.sphinx_rust import AnalysisResult


LOGGER = getLogger(__name__)


ObjType = Literal["crate", "module", "struct", "enum", "function"]


@dataclass
class ObjectEntry:
    """An entry in the domain's object inventory."""

    name: str
    docname: str
    node_id: str
    objtype: ObjType


class DomainData(TypedDict):
    """Data stored in the domain."""

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
        "function": RustFunctionAutoDirective,
    }

    roles = {
        "crate": XRefRole(),
        "module": XRefRole(),
        "struct": XRefRole(),
        "enum": XRefRole(),
        "function": XRefRole(),
    }

    initial_data: DomainData = {  # type: ignore[assignment]
        "objects": {},
    }

    @classmethod
    def app_setup(cls, app: Sphinx) -> None:
        RustConfig.add_configs(app)
        app.connect("builder-inited", cls.on_builder_inited)
        app.connect("build-finished", cls.on_build_finished)
        app.add_domain(cls)

    @staticmethod
    def on_builder_inited(app: Sphinx) -> None:
        """Analyze the Rust crates."""
        config = RustConfig.from_app(app)
        # create the cache directory
        app.env.rust_cache_path = cache = Path(str(app.doctreedir)) / "_rust_cache"  # type: ignore[attr-defined]
        cache.mkdir(exist_ok=True)
        srcdir = Path(
            str(app.srcdir)
        )  # for back-compatibility, assume it might not be a Path
        for crate in config.rust_crates:
            path = Path(str(app.srcdir)) / str(crate)
            # analyze the crate
            LOGGER.info(f"[rust] Analyzing crate: {path.resolve()!s}")
            try:
                result = analyze_crate(str(path), str(cache))
            except OSError as e:
                LOGGER.warning(
                    f"Error analyzing crate: {e!s}", type="rust", subtype="analyze"
                )
                return

            if config.rust_enable_auto_pages:
                create_pages(srcdir / config.rust_root_pages, result)

            if config.rust_viewcode:
                create_code_pages(result.crate_, srcdir / config.rust_root_pages, cache)

    @staticmethod
    def on_build_finished(app: Sphinx, exception: Union[Exception, None]) -> None:
        # if an exception occure, don't delete files
        if exception is not None:
            return

        config = RustConfig.from_app(app)

        if config.rust_keep_files:
            return

        srcdir = Path(str(app.srcdir))
        for crate in config.rust_crates:
            cache_path = app.env.rust_cache_path
            path = srcdir / str(crate)
            # FIXME do not parse again
            try:
                result = analyze_crate(str(path), str(cache_path))
            except OSError:
                continue

            shutil.rmtree(
                srcdir / config.rust_root_pages / result.crate_, ignore_errors=True
            )

    @property
    def objects(self) -> dict[str, ObjectEntry]:
        """fullname -> ObjectEntry mapping."""
        return self.data.setdefault("objects", {})  # type: ignore[no-any-return]

    def note_object(
        self,
        name: str,
        objtype: ObjType,
        node_id: str,
        signature: addnodes.desc_signature,
    ) -> None:
        """Note a new object."""
        # TODO can rust have duplicate names for different object types?
        if name in self.objects:
            LOGGER.warning(
                f"Duplicate target {name!r}",
                type="rust",
                subtype="target",
                location=signature,
            )
        else:
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
    root = srcdir.joinpath(result.crate_)
    if root.exists():
        # TODO only update changed files (so that sphinx knows what to rebuild)
        shutil.rmtree(root)
    root.mkdir(parents=True, exist_ok=True)
    root.joinpath(".gitignore").write_text("*\n")

    # create the sub-indexes
    indexes = []
    modules = [m for m in result.modules if m != result.crate_]
    if modules:
        create_object_pages(root, "module", modules)
        indexes.append("modules/index")
    if result.structs:
        create_object_pages(root, "struct", result.structs)
        indexes.append("structs/index")
    if result.enums:
        create_object_pages(root, "enum", result.enums)
        indexes.append("enums/index")
    if result.functions:
        create_object_pages(root, "function", result.functions)
        indexes.append("functions/index")

    # create the main index
    title = f"Crate ``{result.crate_}``"
    index_content = f"{title}\n{'=' * len(title)}\n\n.. rust:crate:: {result.crate_}"
    index_content += "\n\n.. toctree::\n    :maxdepth: 1\n    :hidden:\n\n"
    for index in indexes:
        index_content += f"    {index}\n"
    root.joinpath("index.rst").write_text(index_content)


def create_object_pages(folder: Path, otype: str, names: list[str]) -> None:
    """Create the pages for the objects of a certain type."""
    ofolder = folder.joinpath(otype + "s")
    ofolder.mkdir(exist_ok=True)
    index_content = f"{otype.capitalize()}s\n{'=' * (len(otype) + 1)}\n\n.. toctree::\n    :maxdepth: 1\n\n"
    for name in names:
        index_content += f"    {name}\n"
        title = f"{otype.capitalize()} ``{name}``"
        ofolder.joinpath(f"{name}.rst").write_text(
            f"{title}\n{'=' * len(title)}\n\n.. rust:{otype}:: {name}\n"
        )
    ofolder.joinpath("index.rst").write_text(index_content)


def create_code_pages(crate_name: str, srcdir: Path, cache: Path) -> None:
    if modules := [
        (m.path_str, m.file)
        for m in load_descendant_modules(str(cache), [crate_name], True)
        if m.file
    ]:
        code_folder = srcdir.joinpath(crate_name, "code")
        code_folder.mkdir(exist_ok=True, parents=True)
        for full_name, file_path in modules:
            # TODO catch exceptions here, if a relative path cannot be created
            rel_path = os.path.relpath(Path(file_path), code_folder)
            # note, this is available only in Python 3.12+
            # rel_path = Path(file_path).relative_to(code_folder, walk_up=True)
            # TODO only write the file if it doesn't exist or is different
            code_folder.joinpath(f"{full_name}.rst").write_text(
                "\n".join(
                    (
                        ":orphan:",
                        "",
                        f".. literalinclude:: {rel_path}",
                        f"   :name: rust-code:{full_name}",
                        "   :language: rust",
                        "   :linenos:",
                    )
                )
            )
