Sphinx Rust
===========

Auto-document `Rust <https://www.rust-lang.org/>`__ code with :external:doc:`Sphinx <index>`!

.. warning::

    This project is in early development and is not yet ready for production use.

    It currently documents a subset of Rust code, and is not yet feature-complete, but this documents should give you a sense of what the final product will look like.

This package is intended to bring API documentation for Rust crates to Sphinx:

- Auto-analysis of Rust crates, with minimal steps required to get started

- Cleanly documented APIs;
  the documentation is generated to closely mirror the format of https://docs.rs

- Integrates with the Sphinx cross-referencing system and
  :external:doc:`intersphinx <usage/extensions/intersphinx>` indexing.

- Supports writing docstrings for any valid Sphinx parser
  (reStructuredText, `MyST markdown <https://myst-parser.readthedocs.io>`__, ...)

Installation
------------

.. image:: https://img.shields.io/pypi/v/sphinx-rust
    :target: https://pypi.org/project/sphinx-rust/

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

Now add a `toctree` in your `index.rst`, to point towards the generated documentation for each crate
(by default written to `api/crates` in the source directory of your Sphinx project):

.. code-block:: restructuredtext

    .. toctree::
        :caption: Rust API

        api/crates/crate/index

That's it!

Now you can use the `rust` cross-referencing roles to link to items in your Rust crates:

.. code-block:: restructuredtext

    - :rust:module:`analyzer::analyze`
    - :rust:struct:`analyze::struct_::Struct`
    - :rust:enum:`analyze::type_::TypeSegment`

- :rust:module:`analyzer::analyze`
- :rust:struct:`analyzer::analyze::struct_::Struct`
- :rust:enum:`analyzer::analyze::type_::TypeSegment`

Analysis CLI
------------

You can also use the `sphinx-rust` CLI to analyze Rust crates and generate a set of JSON files of the high-level API of the crate:

.. code-block:: bash

    python -m sphinx_rust.cli path/to/crate

Contents
--------

.. toctree::

    Quick-start <self>

.. toctree::
    :caption: Rust API

    api/crates/analyzer/index
    api/crates/sphinx_rust/index

.. toctree::
    :caption: Integrations

    integrations/myst
    integrations/needs

.. toctree::
    :caption: Development

    dev/design
    dev/contributing
