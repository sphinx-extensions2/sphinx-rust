from sphinx_rust import __version__

project = "sphinx-rust"
version = __version__

extensions = [
    "sphinx_rust",
    "myst_parser",
    "sphinx_needs",
    "sphinxcontrib.plantuml",
    "sphinx.ext.intersphinx",
]
html_theme = "furo"
html_static_path = ["_static"]
html_css_files = ["custom.css"]

rust_crates = [
    "../crates/analyzer",
    "../crates/py_binding",
]
rust_doc_formats = {
    "analyzer": "restructuredtext",
    "sphinx_rust": "markdown",
}

intersphinx_mapping = {
    "sphinx": ("https://www.sphinx-doc.org", None),
}
