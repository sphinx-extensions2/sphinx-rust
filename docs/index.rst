Sphinx Rust
===========

Auto-document Rust code with Sphinx!

.. warning::

    This project is in early development and is not yet ready for production use.

    It currently documents a subset of Rust code, and is not yet feature-complete, but this documents should give you a sense of what the final product will look like.

This package is intended to bring API documentation for rust crates to Sphinx:

- Auto-analysis of Rust crates, with minimal steps required to get started

- Auto-documentation of Rust crates: the documentation is generated to closely mirror the format of https://docs.rs

- Integrates with sphinx cross-referencing system and inter-sphinx indexing

- Supports writing docstrings for any valid Sphinx parser (reStructuredText, MyST markdown, etc.)

Installation
------------

.. code-block:: bash

    pip install sphinx-rust

Usage
-----

Add `sphinx_rust` to your `conf.py`, and specifiy the paths to the Rust crates you want to document:

.. code-block:: python

    extensions = [
        "sphinx_rust",
    ]
    rust_crates = [
        "../path/to/crate",
        ...
    ]
    ...

That's it!

Now you can use the `rust` cross-referencing roles to link to items in your Rust crates:

.. code-block:: restructuredtext

    - :rust:module:`analyzer::analyze`
    - :rust:struct:`analyze::struct_::Struct`
    - :rust:enum:`analyze::type_::TypeSegment`

- :rust:module:`analyzer::analyze`
- :rust:struct:`analyzer::analyze::struct_::Struct`
- :rust:enum:`analyzer::analyze::type_::TypeSegment`

--------------------

.. rubric:: Contents

.. toctree::

    Quick-start <self>

.. toctree::
    :caption: API Examples

    api/index
    needs

.. toctree::
    :caption: Development

    design
