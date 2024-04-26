from sphinx.roles import XRefRole


class RustCrateXRefRole(XRefRole):
    """Role to cross-reference a Rust crate."""


class RustModuleXRefRole(XRefRole):
    """Role to cross-reference a Rust module."""


class RustStructXRefRole(XRefRole):
    """Role to cross-reference a Rust struct."""


class RustEnumXRefRole(XRefRole):
    """Role to cross-reference a Rust enum."""
