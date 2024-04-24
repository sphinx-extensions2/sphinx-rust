use pyo3::{exceptions::PyValueError, prelude::*};

use analyzer::analyze;

#[pymodule]
/// sphinx_rust backend
// Note: The name of this function must match the `lib.name` setting in the `Cargo.toml`,
// else Python will not be able to import the module.
fn sphinx_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(analyze_module, m)?)?;
    m.add_function(wrap_pyfunction!(module_from_id_path, m)?)?;
    m.add_class::<Module>()?;
    m.add_class::<Struct>()?;
    Ok(())
}

#[pyfunction]
/// Parse a module and return a high-level representation of it
pub(crate) fn analyze_module(name: &str, content: &str) -> PyResult<String> {
    let module = match analyze::Module::parse(name, content) {
        Ok(syntax) => syntax,
        Err(err) => {
            return Err(PyValueError::new_err(format!(
                "Could not parse content: {}",
                err
            )))
        }
    };
    Ok(module.to_json())
}

#[pyfunction]
/// Given a path qualifier to a module, such as `crate::module`, return the module
pub(crate) fn module_from_id_path(_qualifier: &str) -> PyResult<Module> {
    let module = match analyze::Module::parse(
        "name",
        "//! Hallo\n/// My struct\npub struct Hallo;\n\npub enum There{\nYou\n}",
    ) {
        Ok(syntax) => syntax,
        Err(err) => {
            return Err(PyValueError::new_err(format!(
                "Could not parse content: {}",
                err
            )))
        }
    };
    Ok(module.into())
}

#[pyclass]
#[derive(Clone)]
struct Module {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub docstring: String,
    #[pyo3(get)]
    pub structs: Vec<Struct>,
    #[pyo3(get)]
    pub enums: Vec<Enum>,
}

impl From<analyze::Module> for Module {
    fn from(module: analyze::Module) -> Self {
        Module {
            name: module.name,
            docstring: module.docstring,
            structs: module.structs.into_iter().map(Struct::from).collect(),
            enums: module.enums.into_iter().map(Enum::from).collect(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
struct Struct {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub docstring: String,
}

impl From<analyze::Struct> for Struct {
    fn from(module: analyze::Struct) -> Self {
        Struct {
            name: module.name,
            docstring: module.docstring,
            // TODO add rest
        }
    }
}

#[pyclass]
#[derive(Clone)]
struct Enum {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub docstring: String,
}

impl From<analyze::Enum> for Enum {
    fn from(module: analyze::Enum) -> Self {
        Enum {
            name: module.name,
            docstring: module.docstring,
            // TODO add rest
        }
    }
}
