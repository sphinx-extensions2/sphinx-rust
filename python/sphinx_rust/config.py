from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from sphinx.application import Sphinx


@dataclass
class RustConfig:
    """Configuration for the Rust extension."""

    rust_crates: list[str]
    rust_doc_formats: dict[str, str]
    rust_viewcode: bool
    rust_root_pages: str
    """pages auto generation path, default api/crates, relative to source"""
    rust_enable_auto_pages: bool
    """if create auto pages, default True"""
    rust_keep_files: bool
    """if keep files at the end, if False delete all auto generated files"""

    @classmethod
    def from_app(cls, app: Sphinx) -> RustConfig:
        """Create a new RustConfig from the Sphinx application."""
        return cls(
            rust_crates=app.config.rust_crates,
            rust_doc_formats=app.config.rust_doc_formats,
            rust_viewcode=app.config.rust_viewcode,
            rust_root_pages=app.config.rust_root_pages,
            rust_enable_auto_pages=app.config.rust_enable_auto_pages,
            rust_keep_files=app.config.rust_keep_files,
        )

    @staticmethod
    def add_configs(app: Sphinx) -> None:
        """Add the configuration values for the Rust domain."""
        app.add_config_value("rust_crates", [], "env")
        app.add_config_value("rust_doc_formats", {}, "env")
        app.add_config_value("rust_viewcode", True, "env")
        app.add_config_value("rust_root_pages", "api/crates", "env")
        app.add_config_value("rust_enable_auto_pages", True, "env")
        app.add_config_value("rust_keep_files", True, "env")
