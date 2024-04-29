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

    @classmethod
    def from_app(cls, app: Sphinx) -> RustConfig:
        """Create a new RustConfig from the Sphinx application."""
        return cls(
            rust_crates=app.config.rust_crates,
            rust_doc_formats=app.config.rust_doc_formats,
            rust_viewcode=app.config.rust_viewcode,
        )

    @staticmethod
    def add_configs(app: Sphinx) -> None:
        """Add the configuration values for the Rust domain."""
        app.add_config_value("rust_crates", [], "env")
        app.add_config_value("rust_doc_formats", {}, "env")
        app.add_config_value("rust_viewcode", True, "env")
