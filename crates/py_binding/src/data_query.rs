//! Functions for querying the cache.

use pyo3::{exceptions::PyIOError, prelude::*};

use analyzer::data_model::{self as analyze_model};

use crate::data_model::{Crate, Enum, Function, Module, Struct};

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
    let crate_: analyze_model::Crate = deserialize_object(name, &contents)?;
    Ok(Some(crate_.into()))
}

#[pyfunction]
/// load a module from the cache, if it exists
pub fn load_module(cache_path: &str, full_name: &str) -> PyResult<Option<Module>> {
    let path = std::path::Path::new(cache_path)
        .join("modules")
        .join(format!("{}.json", full_name));
    if !path.exists() {
        return Ok(None);
    }
    let contents = read_file(&path)?;
    let mod_: analyze_model::Module = deserialize_object(full_name, &contents)?;
    Ok(Some(mod_.into()))
}

#[pyfunction]
/// load a struct from the cache, if it exists
pub fn load_struct(cache_path: &str, full_name: &str) -> PyResult<Option<Struct>> {
    let path = std::path::Path::new(cache_path)
        .join("structs")
        .join(format!("{}.json", full_name));
    if !path.exists() {
        return Ok(None);
    }
    let contents = read_file(&path)?;
    let struct_: analyze_model::Struct = deserialize_object(full_name, &contents)?;
    Ok(Some(struct_.into()))
}

#[pyfunction]
/// load an enum from the cache, if it exists
pub fn load_enum(cache_path: &str, full_name: &str) -> PyResult<Option<Enum>> {
    let path = std::path::Path::new(cache_path)
        .join("enums")
        .join(format!("{}.json", full_name));
    if !path.exists() {
        return Ok(None);
    }
    let contents = read_file(&path)?;
    let enum_: analyze_model::Enum = deserialize_object(full_name, &contents)?;
    Ok(Some(enum_.into()))
}

#[pyfunction]
/// load a function from the cache, if it exists
pub fn load_function(cache_path: &str, full_name: &str) -> PyResult<Option<Function>> {
    let path = std::path::Path::new(cache_path)
        .join("functions")
        .join(format!("{}.json", full_name));
    if !path.exists() {
        return Ok(None);
    }
    let contents = read_file(&path)?;
    let func: analyze_model::Function = deserialize_object(full_name, &contents)?;
    Ok(Some(func.into()))
}

/// Check if a path is a child of a given parent, and return the fully qualified name of the child.
fn is_child(path: &std::path::PathBuf, parent: &Vec<String>) -> Option<String> {
    let name = match path.file_stem() {
        Some(name) => name,
        None => return None,
    };
    let name = match name.to_str() {
        Some(name) => name,
        None => return None,
    };
    let name_path = name.split("::").collect::<Vec<_>>();
    if name_path.len() != parent.len() + 1 {
        return None;
    }
    for (a, b) in parent.iter().zip(name_path.iter().take(parent.len())) {
        if a != b {
            return None;
        }
    }
    Some(name.to_string())
}

#[pyfunction]
/// load all modules from the cache that are children of the given parent
pub fn load_child_modules(cache_path: &str, parent: Vec<String>) -> PyResult<Vec<Module>> {
    let path = std::path::Path::new(cache_path).join("modules");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut modules = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = is_child(&path, &parent) {
                let contents = read_file(&path)?;
                let mod_: analyze_model::Module = deserialize_object(&name, &contents)?;
                modules.push(mod_.into());
            }
        }
    }
    Ok(modules)
}

#[pyfunction]
/// load all structs from the cache that are children of the given parent
pub fn load_child_structs(cache_path: &str, parent: Vec<String>) -> PyResult<Vec<Struct>> {
    let path = std::path::Path::new(cache_path).join("structs");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut structs = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = is_child(&path, &parent) {
                let contents = read_file(&path)?;
                let struct_: analyze_model::Struct = deserialize_object(&name, &contents)?;
                structs.push(struct_.into());
            }
        }
    }
    Ok(structs)
}

#[pyfunction]
/// load all enums from the cache that are children of the given parent
pub fn load_child_enums(cache_path: &str, parent: Vec<String>) -> PyResult<Vec<Enum>> {
    let path = std::path::Path::new(cache_path).join("enums");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut enums = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = is_child(&path, &parent) {
                let contents = read_file(&path)?;
                let enum_: analyze_model::Enum = deserialize_object(&name, &contents)?;
                enums.push(enum_.into());
            }
        }
    }
    Ok(enums)
}

#[pyfunction]
/// load all function from the cache that are children of the given parent
pub fn load_child_functions(cache_path: &str, parent: Vec<String>) -> PyResult<Vec<Function>> {
    let path = std::path::Path::new(cache_path).join("functions");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut funcs = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = is_child(&path, &parent) {
                let contents = read_file(&path)?;
                let func: analyze_model::Function = deserialize_object(&name, &contents)?;
                funcs.push(func.into());
            }
        }
    }
    Ok(funcs)
}

/// Check if a path is an ancestor of a given parent, and return the fully qualified name of the child.
fn is_ancestor(
    path: &std::path::PathBuf,
    parent: &Vec<String>,
    include_self: bool,
) -> Option<String> {
    let name = path.file_stem()?.to_str()?;
    let name_path = name.split("::").collect::<Vec<_>>();
    if include_self && name_path == parent.iter().map(|s| s.as_str()).collect::<Vec<_>>() {
        return Some(name.to_string());
    }
    if name_path.len() <= parent.len() {
        return None;
    }
    for (a, b) in parent.iter().zip(name_path.iter().take(parent.len())) {
        if a != b {
            return None;
        }
    }
    Some(name.to_string())
}

#[pyfunction]
/// load all modules from the cache that have a common descendant
pub fn load_descendant_modules(
    cache_path: &str,
    ancestor: Vec<String>,
    include_self: bool,
) -> PyResult<Vec<Module>> {
    let path = std::path::Path::new(cache_path).join("modules");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut modules = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = is_ancestor(&path, &ancestor, include_self) {
                let contents = read_file(&path)?;
                let mod_: analyze_model::Module = deserialize_object(&name, &contents)?;
                modules.push(mod_.into());
            }
        }
    }
    Ok(modules)
}

#[pyfunction]
/// load all structs from the cache that have a common ancestor
pub fn load_descendant_structs(cache_path: &str, ancestor: Vec<String>) -> PyResult<Vec<Struct>> {
    let path = std::path::Path::new(cache_path).join("structs");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut structs = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = is_ancestor(&path, &ancestor, false) {
                let contents = read_file(&path)?;
                let struct_: analyze_model::Struct = deserialize_object(&name, &contents)?;
                structs.push(struct_.into());
            }
        }
    }
    Ok(structs)
}

#[pyfunction]
/// load all enums from the cache that that have a common ancestor
pub fn load_descendant_enums(cache_path: &str, ancestor: Vec<String>) -> PyResult<Vec<Enum>> {
    let path = std::path::Path::new(cache_path).join("enums");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut enums = vec![];
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = is_ancestor(&path, &ancestor, false) {
                let contents = read_file(&path)?;
                let enum_: analyze_model::Enum = deserialize_object(&name, &contents)?;
                enums.push(enum_.into());
            }
        }
    }
    Ok(enums)
}
