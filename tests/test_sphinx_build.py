"""Simple test to build sphinx documentation."""

from __future__ import annotations

from pathlib import Path
from textwrap import dedent

from sphinx.testing.util import SphinxTestApp
from sphinx.util.console import strip_colors


def test_basic(make_app: type[SphinxTestApp], tmp_path: Path) -> None:
    """Basic sphinx build test."""
    tmp_path.joinpath("Cargo.toml").write_text(
        dedent("""\
    [package]
    name = "test"
    version = "0.1.0"
    [lib]
    """)
    )
    tmp_path.joinpath("src").mkdir()
    tmp_path.joinpath("src", "lib.rs").write_text("//! Test library.")
    tmp_path.joinpath("conf.py").write_text(
        dedent("""\
    extensions = ['sphinx_rust']
    rust_crates = ['.']
    """)
    )
    tmp_path.joinpath("index.rst").write_text(
        dedent("""\
    Test
    ====
    .. toctree:: api/crates/test/index
    """)
    )

    app = make_app("html", srcdir=tmp_path)
    app.build()
    assert strip_colors(app.warning.getvalue()) == ""  # noqa: PLC1901

    assert (
        Path(str(app.outdir)).joinpath("api", "crates", "test", "index.html").exists()
    )
