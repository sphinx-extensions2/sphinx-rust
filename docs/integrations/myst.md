# MyST Markdown

[Markedly Structured Text (MyST)](https://myst-parser.readthedocs.io) is a Markdown flavor that has been designed to be a superset of CommonMark (Markdown) with additional features offered by reStructuredText and Sphinx.

Since `sphinx-rust` is designed to be agnostic to the input format, you can use MyST Markdown to write your docstrings.

To write with MyST Markdown, you need to add the `myst_parser` extension to your `conf.py`:

```python
extensions = [
    "myst_parser",
    "sphinx_rust",
]
```

`myst_parser` registers the MyST parser with Sphinx under the aliases `myst` and `markdown`,
alongside the default `restructuredtext` parser.

To specify a crate's docstrings are written in MyST Markdown, you can use the `rust_doc_formats` configuration to map crate's to a specific parser format:

```python
rust_doc_formats = {
    "analyzer": "restructuredtext",
    "sphinx_rust": "markdown",
}
```

Now simply write your docstrings in MyST Markdown and they will be rendered correctly by Sphinx, as in {rust:crate}`sphinx_rust`.
