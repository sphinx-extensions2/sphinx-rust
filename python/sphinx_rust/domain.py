from __future__ import annotations

from pathlib import Path
import shutil
from typing import TYPE_CHECKING, Any

from sphinx.domains import Domain
from sphinx.util.logging import getLogger

from sphinx_rust.directives.crate import RustCrateAutoDirective
from sphinx_rust.directives.enum import RustEnumAutoDirective
from sphinx_rust.directives.module import RustModuleAutoDirective
from sphinx_rust.directives.struct import RustStructAutoDirective
from sphinx_rust.sphinx_rust import analyze_crate

if TYPE_CHECKING:
    from docutils.nodes import Element
    from sphinx.addnodes import pending_xref
    from sphinx.application import Sphinx
    from sphinx.builders import Builder
    from sphinx.environment import BuildEnvironment


LOGGER = getLogger(__name__)


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
            # now write the pages
            # TODO don't write if not changed
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
            index_content = (
                f"{title}\n{'=' * len(title)}\n\n.. rust:crate:: {result.crate_}"
            )
            if pages:
                index_content += "\n\n.. toctree::\n    :maxdepth: 1\n    :hidden:\n\n"
                for page in pages:
                    index_content += f"    items/{page}\n"
            root.joinpath("index.rst").write_text(index_content)

    def merge_domaindata(
        self, _docnames: list[str], _otherdata: dict[str, Any]
    ) -> None:
        pass

    def resolve_any_xref(  # noqa: PLR6301
        self,
        _env: BuildEnvironment,
        _fromdocname: str,
        _builder: Builder,
        _target: str,
        _node: pending_xref,
        _contnode: Element,
    ) -> list[tuple[str, Element]]:
        return []
