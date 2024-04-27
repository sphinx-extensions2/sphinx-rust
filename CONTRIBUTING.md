# Contributing

We welcome contributions to this project!

## Linting and formatting

The project uses [`pre-commit`](https://pre-commit.com/) to run linters and formatters.

```bash
pre-commit run --all-files
```

## Rust

The project uses [`cargo-insta`](https://insta.rs/) for snapshot testing.

```bash
cargo insta test -p analyzer
```

## Python

The project uses [`maturin`](https://www.maturin.rs) to build the Python package,
with the compiled Rust bindings.

You can use the [`tox`](https://tox.readthedocs.io/en/latest/) tool to run particular tasks,
with integrated python environment management and package building:

To start an IPython session with the development environment:

```bash
tox -e dev -- ipython
```

To run the analysis CLI tool:

```bash
tox -e dev -- python -m sphinx_rust.cli crates/py_binding --overwrite
```

To run the pytest tests:

```bash
tox -e test-py39
```

To build the documentation:

```bash
tox -e docs -- -E
```
