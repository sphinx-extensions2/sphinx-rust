from sphinx_rust import __version__

project = "sphinx-rust"
version = __version__

extensions = ["sphinx_rust"]
html_theme = "furo"

rust_crates = [
    "../crates/analyzer",
    "../crates/py_binding",
]
