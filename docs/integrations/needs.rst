Sphinx-needs
============

This package integrates with the `sphinx-needs` package to allow for the creation of requirements and test cases from Rust code.

To use this feature, add `sphinx_needs` to your `conf.py`:

.. code-block:: python

    extensions = [
        "sphinx_rust",
        "sphinx_needs",
        "sphinxcontrib.plantuml"
    ]
    rust_crates = [
        "../path/to/crate",
        ...
    ]
    ...

We can then create requirements and test cases from Rust code:

.. code-block:: restructuredtext

    .. needlist::
        :tags: rust

    .. needtable::
        :tags: rust
        :style: table

.. needlist::
  :tags: rust

.. needtable::
    :tags: rust
    :style: table
