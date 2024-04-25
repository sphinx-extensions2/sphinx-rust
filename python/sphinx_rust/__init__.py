from __future__ import annotations

from typing import TYPE_CHECKING

from sphinx_rust.domain import RustDomain

from .sphinx_rust import __version__

if TYPE_CHECKING:
    from sphinx.application import Sphinx
    from sphinx.util.typing import ExtensionMetadata

__all__ = ("__version__", "setup")


def setup(app: Sphinx) -> ExtensionMetadata:
    RustDomain.app_setup(app)
    return {"version": __version__, "parallel_read_safe": True}
