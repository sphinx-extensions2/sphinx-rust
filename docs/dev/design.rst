Design
======

The design of the system is based on the following principles:

- Focus on auto-generated API:
  the user should have a minimal effort to add their Rust project to Sphinx.

- Minimal configuration:
  the user should not have to write a lot of configuration to get started,
  and the default configuration should be good enough for most projects.

- Good looking documentation:
  the generated documentation should be easy to read and navigate.
  To this end, the layout in https://docs.rs/ is used as a reference, as the "gold-standard" for Rust documentation.

- Support both reStructuredText and Markdown docstrings:
  the user should be able to write their API documentation in either reStructuredText or MyST Markdown.

- Integrate well with the Sphinx caching system:
  the system should be able to take advantage of the Sphinx caching system to avoid re-generating the API documentation when it is not necessary.

- Integrate well with the Sphinx warning system:
  the system should be able to generate warnings when reading / resolving the API documentation,
  with warnings pointing to the source code location of the issue.

- Have a clear separation of concerns:
  the system should have a clear separation between the analysis of the Rust project, and the generation of the Sphinx documentation.
  This separation is realised by the use of a high-level representation of the API, which is independent of the Sphinx documentation generation.

The core components of the system are:

- ``crates/analyzer`` a rust package that analyzes the source code of a Rust project (using ``syn``) and generates a "high-level representation" of the API, which is serializable/de-serializable to disk (using ``serde``).

- ``crates/py_binding`` a rust package that provides a Python binding to the analyzer, using ``pyo3``.

- ``python/sphinx_rust`` a Python package that provides a Sphinx extension to generate documentation from the high-level representation of the API, bundling ``crates/py_binding`` to integrate the analyzer within Sphinx.

The build steps within Sphinx are:

1. At the start of the Sphinx build (``builder-inited`` event),
    ``sphinx_rust`` calls the analyzer to generate the high-level representation of the API, and writes it to disk, within the Sphinx ``build`` directory.

2. Also at the start of the Sphinx build, after the analysis.
    ``sphinx_rust`` uses the high-level representation, to generate folders and files within the Sphinx ``source`` directory, that provide the outline of the API documentation.
    For example a folder per crate, and a file per Rust object (module, struct, enum, trait, function, etc.).

3. During the Sphinx read-phase, when a "directive" is encountered that requires the API documentation,
    ``sphinx_rust`` reads the high-level representation from disk, and generates the docutils AST for the Rust object that the directive specifies.
    It also stores relevant information in the Sphinx environment, to be used during the resolve-phase.

4. Also during the Sphinx read-phase, when a "cross-reference role" is encountered that requires the API documentation,
    ``sphinx_rust`` generates a ``pending_xref`` node that will be resolved during the resolve-phase.

5. During the Sphinx resolve-phase, when a "pending_xref" node is encountered,
    ``sphinx_rust`` uses the information stored in the Sphinx environment to resolve the cross-reference, or generate a warning if the cross-reference cannot be resolved (or the resolution has multiple matches).

Technical note on the analyzer
------------------------------

One annoying technical limitation of the current analyzer,
compared to what ``rustdoc`` does, is the "expansion" of macros.

``rustdoc`` integrates directly into the compiler, to generate its high-level representation of the API,
which is generated after the macros have been expanded: https://rustc-dev-guide.rust-lang.org/rustdoc.html.
See the source code here: https://github.com/rust-lang/rust/blob/25087e02b34775520856b6cc15e50f5ed0c35f50/src/librustdoc/lib.rs#L773.

This is difficult to hook into from a third-party crate, as it requires modifying the compiler itself.
You can view a debug of this intermediate representation like so:

```bash
rustup run nightly cargo rustc -- -Z unpretty=hir-tree
```

but without actually building the compiler, it's hard to get this information in a usable form (e.g. a serialized JSON).

Perhaps this can be achieved at a later date,
and at least the separation of concerns in the design allows for this to be swapped out.
