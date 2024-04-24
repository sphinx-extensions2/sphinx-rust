use pyo3::{exceptions::PyIOError, prelude::*};

use analyzer::analyze::module::Module;

#[pymodule]
/// sphinx_rust backend
// Note: The name of this function must match the `lib.name` setting in the `Cargo.toml`,
// else Python will not be able to import the module.
fn sphinx_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(analyze_module, m)?)?;
    // m.add_class::<analyze::Module>()?;
    // m.add_class::<analyze::Struct>()?;
    // m.add_class::<analyze::Field>()?;
    Ok(())
}

#[pyfunction]
/// Parse a module and return a high-level representation of it
pub(crate) fn analyze_module(name: &str, content: &str) -> PyResult<String> {
    let module = match Module::parse(name, content) {
        Ok(syntax) => syntax,
        Err(err) => {
            return Err(PyIOError::new_err(format!(
                "Could not parse content: {}",
                err
            )))
        }
    };
    Ok(module.to_json())
}
