//! The backend for the sphinx_rust Python package.
//!
//! This module provides a Python interface to the ``analyzer`` crate.
//!
//! .. req:: Integrate rust with sphinx
//!     :id: RUST001
//!     :tags: rust
//!
//!     We need to integrate Sphinx with Rust so that we can use the `sphinx_rust` backend to generate documentation for Rust code.

use pyo3::{exceptions::PyIOError, prelude::*};

use analyzer::analyze;

#[pymodule]
/// sphinx_rust backend
// Note: The name of this function must match the `lib.name` setting in the `Cargo.toml`,
// else Python will not be able to import the module.
fn sphinx_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(analyze_crate, m)?)?;
    m.add_class::<Crate>()?;
    m.add_class::<Module>()?;
    m.add_class::<Struct>()?;
    m.add_class::<Field>()?;
    m.add_class::<TypeSegment>()?;
    m.add_class::<Enum>()?;
    m.add_class::<Variant>()?;
    m.add_class::<AnalysisResult>()?;
    m.add_function(wrap_pyfunction!(load_crate, m)?)?;
    m.add_function(wrap_pyfunction!(load_module, m)?)?;
    m.add_function(wrap_pyfunction!(load_struct, m)?)?;
    m.add_function(wrap_pyfunction!(load_enum, m)?)?;
    m.add_function(wrap_pyfunction!(load_modules, m)?)?;
    m.add_function(wrap_pyfunction!(load_structs, m)?)?;
    m.add_function(wrap_pyfunction!(load_enums, m)?)?;
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
        output.modules.push(mod_.name.clone());
        let mod_path = modules_path.join(format!("{}.json", mod_.name));
        serialize_to_file(&mod_path, &mod_)?;
    }
    let structs_path = cache_path.join("structs");
    if !structs_path.exists() {
        std::fs::create_dir(&structs_path)?;
    }
    for struct_ in &result.structs {
        output.structs.push(struct_.name.clone());
        let struct_path = structs_path.join(format!("{}.json", struct_.name));
        serialize_to_file(&struct_path, &struct_)?;
    }
    let enums_path = cache_path.join("enums");
    if !enums_path.exists() {
        std::fs::create_dir(&enums_path)?;
    }
    for enum_ in &result.enums {
        output.enums.push(enum_.name.clone());
        let enum_path = enums_path.join(format!("{}.json", enum_.name));
        serialize_to_file(&enum_path, &enum_)?;
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
        let existing_value = match std::fs::read_to_string(path) {
            Ok(value) => value,
            Err(err) => {
                return Err(PyIOError::new_err(format!(
                    "Could not read existing value: {}",
                    err
                )))
            }
        };
        if existing_value == value {
            return Ok(());
        }
    }
    match std::fs::write(path, value) {
        Err(err) => Err(PyIOError::new_err(format!(
            "Could not write value to file: {}",
            err
        ))),
        Ok(_) => Ok(()),
    }
}

#[pyfunction]
/// load a crate from the cache, if it exists
pub fn load_crate(cache_path: &str, name: &str) -> PyResult<Option<Crate>> {
    let path = std::path::Path::new(cache_path)
        .join("crates")
        .join(format!("{}.json", name));
    if !path.exists() {
        return Ok(None);
    }
    let contents = read_file(&path)?;
    let crate_: analyze::Crate = deserialize_object(name, &contents)?;
    Ok(Some(crate_.into()))
}

#[pyfunction]
/// load a module from the cache, if it exists
pub fn load_module(cache_path: &str, name: &str) -> PyResult<Option<Module>> {
    let path = std::path::Path::new(cache_path)
        .join("modules")
        .join(format!("{}.json", name));
    if !path.exists() {
        return Ok(None);
    }
    let contents = read_file(&path)?;
    let mod_: analyze::Module = deserialize_object(name, &contents)?;
    Ok(Some(mod_.into()))
}

#[pyfunction]
/// load a struct from the cache, if it exists
pub fn load_struct(cache_path: &str, name: &str) -> PyResult<Option<Struct>> {
    let path = std::path::Path::new(cache_path)
        .join("structs")
        .join(format!("{}.json", name));
    if !path.exists() {
        return Ok(None);
    }
    let contents = read_file(&path)?;
    let struct_: analyze::Struct = deserialize_object(name, &contents)?;
    Ok(Some(struct_.into()))
}

#[pyfunction]
/// load an enum from the cache, if it exists
pub fn load_enum(cache_path: &str, name: &str) -> PyResult<Option<Enum>> {
    let path = std::path::Path::new(cache_path)
        .join("enums")
        .join(format!("{}.json", name));
    if !path.exists() {
        return Ok(None);
    }
    let contents = read_file(&path)?;
    let enum_: analyze::Enum = deserialize_object(name, &contents)?;
    Ok(Some(enum_.into()))
}

#[pyfunction]
/// load all modules from the cache that begin with the given prefix
pub fn load_modules(cache_path: &str, prefix: &str) -> PyResult<Vec<Module>> {
    let path = std::path::Path::new(cache_path).join("modules");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut modules = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = match path.file_stem() {
                Some(name) => name,
                None => continue,
            };
            let name = match name.to_str() {
                Some(name) => name,
                None => continue,
            };
            if !name.starts_with(prefix) {
                continue;
            }
            let contents = read_file(&path)?;
            let mod_: analyze::Module = deserialize_object(name, &contents)?;
            modules.push(mod_.into());
        }
    }
    Ok(modules)
}

#[pyfunction]
/// load all structs from the cache that begin with the given prefix
pub fn load_structs(cache_path: &str, prefix: &str) -> PyResult<Vec<Struct>> {
    let path = std::path::Path::new(cache_path).join("structs");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut structs = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = match path.file_stem() {
                Some(name) => name,
                None => continue,
            };
            let name = match name.to_str() {
                Some(name) => name,
                None => continue,
            };
            if !name.starts_with(prefix) {
                continue;
            }
            let contents = read_file(&path)?;
            let struct_: analyze::Struct = deserialize_object(name, &contents)?;
            structs.push(struct_.into());
        }
    }
    Ok(structs)
}

#[pyfunction]
/// load all enums from the cache that begin with the given prefix
pub fn load_enums(cache_path: &str, prefix: &str) -> PyResult<Vec<Enum>> {
    let path = std::path::Path::new(cache_path).join("enums");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut enums = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = match path.file_stem() {
                Some(name) => name,
                None => continue,
            };
            let name = match name.to_str() {
                Some(name) => name,
                None => continue,
            };
            if !name.starts_with(prefix) {
                continue;
            }
            let contents = read_file(&path)?;
            let enum_: analyze::Enum = deserialize_object(name, &contents)?;
            enums.push(enum_.into());
        }
    }
    Ok(enums)
}

fn read_file(path: &std::path::Path) -> PyResult<String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(err) => Err(PyIOError::new_err(format!(
            "Could not read file: {}: {}",
            path.to_string_lossy(),
            err
        ))),
    }
}

/// Deserialize an object from a string.
fn deserialize_object<'a, T>(name: &str, content: &'a str) -> PyResult<T>
where
    T: serde::Deserialize<'a>,
{
    let obj: T = match serde_json::from_str(content) {
        Ok(crate_) => crate_,
        Err(err) => {
            return Err(PyIOError::new_err(format!(
                "Could not deserialize {}: {}",
                name, err
            )))
        }
    };
    Ok(obj)
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a crate
pub struct Crate {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub version: String,
    #[pyo3(get)]
    pub docstring: String,
}

#[pymethods]
impl Crate {
    pub fn __repr__(&self) -> String {
        format!("Crate(name={:?}, version={:?})", self.name, self.version)
    }
}

impl From<analyze::Crate> for Crate {
    fn from(crate_: analyze::Crate) -> Self {
        Crate {
            name: crate_.name,
            version: crate_.version,
            docstring: crate_.docstring,
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a module
pub struct Module {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub docstring: String,
}

#[pymethods]
impl Module {
    pub fn __repr__(&self) -> String {
        format!("Module(name={:?})", self.name)
    }
}

impl From<analyze::Module> for Module {
    fn from(module: analyze::Module) -> Self {
        Module {
            name: module.name,
            docstring: module.docstring,
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a segment of a type
/// types are split into segments to allow for identification of referenceable elements
pub struct TypeSegment {
    #[pyo3(get)]
    pub content: String,
    #[pyo3(get)]
    pub is_path: bool,
}

#[pymethods]
impl TypeSegment {
    pub fn __repr__(&self) -> String {
        if self.is_path {
            format!("ref({:?})", self.content)
        } else {
            format!("{:?}", self.content)
        }
    }
}

impl From<analyze::TypeSegment> for TypeSegment {
    fn from(field: analyze::TypeSegment) -> Self {
        match field {
            analyze::TypeSegment::Path(content) => TypeSegment {
                content,
                is_path: true,
            },
            analyze::TypeSegment::String(content) => TypeSegment {
                content,
                is_path: false,
            },
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a struct field
pub struct Field {
    #[pyo3(get)]
    pub name: Option<String>,
    #[pyo3(get)]
    pub docstring: String,
    #[pyo3(get)]
    pub type_: Vec<TypeSegment>,
}

#[pymethods]
impl Field {
    pub fn __repr__(&self) -> String {
        format!("Field(name={:?})", self.name)
    }
}

impl From<analyze::Field> for Field {
    fn from(field: analyze::Field) -> Self {
        Field {
            name: field.name,
            docstring: field.docstring,
            type_: field.type_.into_iter().map(TypeSegment::from).collect(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a struct
pub struct Struct {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub docstring: String,
    #[pyo3(get)]
    pub fields: Vec<Field>,
}

#[pymethods]
impl Struct {
    pub fn __repr__(&self) -> String {
        format!("Struct(name={:?})", self.name)
    }
}

impl From<analyze::Struct> for Struct {
    fn from(module: analyze::Struct) -> Self {
        Struct {
            name: module.name,
            docstring: module.docstring,
            fields: module.fields.into_iter().map(Field::from).collect(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of an enum variant
pub struct Variant {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub docstring: String,
    // TODO discriminant
    #[pyo3(get)]
    pub fields: Vec<Field>,
}

#[pymethods]
impl Variant {
    pub fn __repr__(&self) -> String {
        format!("Variant(name={:?})", self.name)
    }
}

impl From<analyze::Variant> for Variant {
    fn from(var: analyze::Variant) -> Self {
        Variant {
            name: var.name,
            docstring: var.docstring,
            fields: var.fields.into_iter().map(Field::from).collect(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of an enum
pub struct Enum {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub docstring: String,
    #[pyo3(get)]
    pub variants: Vec<Variant>,
}

#[pymethods]
impl Enum {
    pub fn __repr__(&self) -> String {
        format!("Enum(name={:?})", self.name)
    }
}

impl From<analyze::Enum> for Enum {
    fn from(module: analyze::Enum) -> Self {
        Enum {
            name: module.name,
            docstring: module.docstring,
            variants: module.variants.into_iter().map(Variant::from).collect(),
        }
    }
}
