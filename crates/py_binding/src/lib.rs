//! The backend for the sphinx_rust Python package.
//!
//! This module provides a Python interface to the ``analyzer`` crate.
//!
//! ```{req} Integrate rust with sphinx
//! :id: RUST001
//! :tags: rust
//!
//! We need to integrate Sphinx with Rust so that we can use the `sphinx_rust` backend to generate documentation for Rust code.
//! ```

use pyo3::{exceptions::PyIOError, prelude::*};

use analyzer::analyze;

pub mod data_model;
pub mod data_query;

#[pymodule]
/// sphinx_rust backend
// Note: The name of this function must match the `lib.name` setting in the `Cargo.toml`,
// else Python will not be able to import the module.
fn sphinx_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(analyze_crate, m)?)?;
    m.add_class::<data_model::Crate>()?;
    m.add_class::<data_model::Module>()?;
    m.add_class::<data_model::Struct>()?;
    m.add_class::<data_model::Field>()?;
    m.add_class::<data_model::TypeSegment>()?;
    m.add_class::<data_model::Enum>()?;
    m.add_class::<data_model::Variant>()?;
    m.add_class::<data_model::Function>()?;
    m.add_class::<AnalysisResult>()?;
    m.add_function(wrap_pyfunction!(data_query::load_crate, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_module, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_struct, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_enum, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_function, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_child_modules, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_child_structs, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_child_enums, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_child_functions, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_descendant_modules, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_descendant_structs, m)?)?;
    m.add_function(wrap_pyfunction!(data_query::load_descendant_enums, m)?)?;
    Ok(())
}

#[pyfunction]
/// analyse a crate and cache the results to disk
pub fn analyze_crate(crate_path: &str, cache_path: &str) -> PyResult<AnalysisResult> {
    // check that the cache path is a directory
    let cache_path = std::path::Path::new(cache_path);
    if !cache_path.is_dir() {
        return Err(PyIOError::new_err(format!(
            "cache_path is not an existing directory: {}",
            cache_path.to_string_lossy()
        )));
    }

    // perform the analysis
    let result = match analyze::analyze_crate(crate_path) {
        Ok(result) => result,
        Err(err) => {
            return Err(PyIOError::new_err(format!(
                "Could not analyze crate: {}",
                err.chain()
                    .map(|err| err.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            )))
        }
    };

    let mut output = AnalysisResult::default();

    // now cache the results
    // note we don't write to disk, if the file already exists and has the same contents
    // this is because Sphinx uses the file's mtime in determining whether to rebuild
    // TODO should also delete files that refer to objects that no longer exist
    let crates_path = cache_path.join("crates");
    if !crates_path.exists() {
        std::fs::create_dir(&crates_path)?;
    }
    output.crate_ = result.crate_.name.clone();
    let crate_path = crates_path.join(format!("{}.json", result.crate_.name));
    serialize_to_file(&crate_path, &result.crate_)?;

    let modules_path = cache_path.join("modules");
    if !modules_path.exists() {
        std::fs::create_dir(&modules_path)?;
    }
    for mod_ in &result.modules {
        output.modules.push(mod_.path_str().clone());
        let mod_path = modules_path.join(format!("{}.json", mod_.path_str()));
        serialize_to_file(&mod_path, &mod_)?;
    }
    let structs_path = cache_path.join("structs");
    if !structs_path.exists() {
        std::fs::create_dir(&structs_path)?;
    }
    for struct_ in &result.structs {
        output.structs.push(struct_.path_str().clone());
        let struct_path = structs_path.join(format!("{}.json", struct_.path_str()));
        serialize_to_file(&struct_path, &struct_)?;
    }
    let enums_path = cache_path.join("enums");
    if !enums_path.exists() {
        std::fs::create_dir(&enums_path)?;
    }
    for enum_ in &result.enums {
        output.enums.push(enum_.path_str().clone());
        let enum_path = enums_path.join(format!("{}.json", enum_.path_str()));
        serialize_to_file(&enum_path, &enum_)?;
    }
    let funcs_path = cache_path.join("functions");
    if !funcs_path.exists() {
        std::fs::create_dir(&funcs_path)?;
    }
    for func in &result.functions {
        output.functions.push(func.path_str().clone());
        let func_path = funcs_path.join(format!("{}.json", func.path_str()));
        serialize_to_file(&func_path, &func)?;
    }
    Ok(output)
}

#[pyclass]
#[derive(Debug, Clone, Default)]
/// pyo3 representation of the result of an analysis
pub struct AnalysisResult {
    #[pyo3(get)]
    pub crate_: String,
    #[pyo3(get)]
    pub modules: Vec<String>,
    #[pyo3(get)]
    pub structs: Vec<String>,
    #[pyo3(get)]
    pub enums: Vec<String>,
    #[pyo3(get)]
    pub functions: Vec<String>,
}

#[pymethods]
impl AnalysisResult {
    pub fn __repr__(&self) -> String {
        format!(
            "AnalysisResult(crate={:?},\n  modules={:?},\n  structs={:?},\n  enums={:?}\n)",
            self.crate_, self.modules, self.structs, self.enums
        )
    }
}

/// Serialize a value to a file.
/// The file is only written if the value is different from any existing value.
fn serialize_to_file<T>(path: &std::path::Path, value: &T) -> PyResult<()>
where
    T: serde::Serialize,
{
    let value = match serde_json::to_string(value) {
        Ok(value) => value,
        Err(err) => {
            return Err(PyIOError::new_err(format!(
                "Could not serialize value: {}",
                err
            )))
        }
    };
    if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(old_value) => {
                if value == old_value {
                    return Ok(());
                }
            }
            Err(_) => {}
        };
    }
    match std::fs::write(path, value) {
        Err(err) => Err(PyIOError::new_err(format!(
            "Could not write value to file: {}",
            err
        ))),
        Ok(_) => Ok(()),
    }
}
