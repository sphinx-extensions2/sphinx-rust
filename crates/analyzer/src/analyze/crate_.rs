//! Analyze the crate
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::data_model::{Crate, Enum, Function, Module, Struct};

pub fn analyze_crate(path: &str) -> Result<AnalysisResult> {
    // make the path absolute
    // TODO we use dunce to canonicalize the path because otherwise there is issues with python's os.path.relpath on windows, but maybe we should fix this on the Python side
    let path =
        dunce::canonicalize(path).context(format!("Error resolving crate path: {}", path))?;
    // check the path is a directory
    if !path.is_dir() {
        return Err(anyhow::anyhow!(format!(
            "Crate path is not a directory: {}",
            path.to_string_lossy()
        )));
    }
    // check if Cargo.toml exists
    let cargo_toml_path = path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!(format!(
            "Cargo.toml does not exist in: {}",
            path.to_string_lossy()
        )));
    }

    // read the Cargo.toml and initialize the Crate struct
    let contents = std::fs::read_to_string(&cargo_toml_path)?;
    let cargo_toml: CargoToml = toml::from_str(&contents).context(format!(
        "Error parsing: {}",
        cargo_toml_path.to_string_lossy()
    ))?;

    // check whether the crate is a library or binary
    let (crate_name, to_root) = if let Some(lib) = cargo_toml.lib {
        if cargo_toml.bin.is_some() {
            return Err(anyhow::anyhow!(format!(
                "Both lib and bin sections in: {}",
                path.to_string_lossy()
            )));
        }
        (
            lib.name.unwrap_or(cargo_toml.package.name),
            lib.path.unwrap_or("src/lib.rs".to_string()),
        )
    } else if let Some(bin) = cargo_toml.bin {
        (
            bin.name.unwrap_or(cargo_toml.package.name),
            bin.path.unwrap_or("src/main.rs".to_string()),
        )
    } else {
        return Err(anyhow::anyhow!(format!(
            "No lib or bin section in: {}",
            path.to_string_lossy()
        )));
    };

    let mut result = AnalysisResult::new(Crate {
        name: crate_name,
        version: cargo_toml.package.version.clone(),
    });

    // check existence of the root module
    let root_module = path.join(to_root);
    if !root_module.exists() {
        return Ok(result);
    }

    // read the top-level module
    let content = std::fs::read_to_string(&root_module)?;
    let (module, structs, enums, functions) =
        Module::parse(Some(&root_module), &[&result.crate_.name], &content).context(format!(
            "Error parsing module {}",
            root_module.to_string_lossy()
        ))?;
    let mut modules_to_read = module
        .declarations
        .iter()
        .map(|s| {
            (
                root_module.parent().unwrap().to_path_buf(),
                s.to_string(),
                vec![result.crate_.name.clone()],
            )
        })
        .collect::<Vec<_>>();

    result.modules.push(module);
    result.structs.extend(structs);
    result.enums.extend(enums);
    result.functions.extend(functions);

    // recursively find/read the public sub-modules
    let mut read_modules = vec![];
    while let Some((parent_dir, module_name, parent)) = modules_to_read.pop() {
        let (module_path, submodule_dir) = if parent_dir
            .join(&parent[1..].join("/"))
            .join(&module_name)
            .with_extension("rs")
            .exists()
        {
            (
                parent_dir
                    .join(&parent[1..].join("/"))
                    .join(&module_name)
                    .with_extension("rs"),
                parent_dir.join(&parent[1..].join("/")).join(&module_name),
            )
        } else if parent_dir
            .join(&parent[1..].join("/"))
            .join(&module_name)
            .join("mod.rs")
            .exists()
        {
            (
                parent_dir
                    .join(&parent[1..].join("/"))
                    .join(&module_name)
                    .join("mod.rs"),
                parent_dir.join(&parent[1..].join("/")).to_path_buf(),
            )
        } else {
            // TODO warn about missing module?
            continue;
        };

        if read_modules.contains(&module_path) {
            continue;
        }
        read_modules.push(module_path.clone());

        let content = std::fs::read_to_string(&module_path)?;
        let path: Vec<String> = [&parent[..], &[module_name]].concat();
        let (module, structs, enums, functions) = Module::parse(
            Some(&module_path),
            &path.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            &content,
        )
        .context(format!(
            "Error parsing module {}",
            module_path.to_string_lossy()
        ))?;
        modules_to_read.extend(
            module
                .declarations
                .iter()
                .map(|s| (submodule_dir.clone(), s.to_string(), path.clone()))
                .collect::<Vec<_>>(),
        );
        result.modules.push(module);
        result.structs.extend(structs);
        result.enums.extend(enums);
        result.functions.extend(functions);
    }

    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Result from a crate analysis
pub struct AnalysisResult {
    pub crate_: Crate,
    pub modules: Vec<Module>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub functions: Vec<Function>,
}

impl AnalysisResult {
    pub fn new(crate_: Crate) -> Self {
        Self {
            crate_,
            modules: vec![],
            structs: vec![],
            enums: vec![],
            functions: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
    bin: Option<Bin>,
    lib: Option<Lib>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct Lib {
    name: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Bin {
    name: Option<String>,
    path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn test_analyze_crate() -> Result<()> {
        // Create a temporary directory for the dummy crate
        let temp_dir = tempfile::tempdir()?;
        let temp_dir_path = temp_dir.path();

        // Create a dummy Cargo.toml file
        let cargo_toml_path = temp_dir_path.join("Cargo.toml");
        std::fs::write(
            cargo_toml_path,
            r#"
            [package]
            name = "my_crate"
            version = "0.1.0"

            [lib]
        "#,
        )?;

        // Create a dummy lib.rs file
        let lib_rs_path = temp_dir_path.join("src").join("lib.rs");
        std::fs::create_dir_all(lib_rs_path.parent().unwrap())?;
        std::fs::write(
            &lib_rs_path,
            r#"
            //! The crate docstring
            pub mod my_module;
        "#,
        )?;

        // Create a dummy module file
        let dummy_module_path = temp_dir_path.join("src").join("my_module.rs");
        std::fs::create_dir_all(dummy_module_path.parent().unwrap())?;
        std::fs::write(
            &dummy_module_path,
            r#"
            //! The module docstring
            pub mod my_submodule;
            /// The struct1 docstring
            pub struct DummyStruct1;
            /// The enum1 docstring
            pub enum DummyEnum1 {}
        "#,
        )?;

        // Create a dummy sub-module file
        let dummy_module_path = temp_dir_path
            .join("src")
            .join("my_module")
            .join("my_submodule.rs");
        std::fs::create_dir_all(dummy_module_path.parent().unwrap())?;
        std::fs::write(
            &dummy_module_path,
            r#"
            //! The sub-module docstring
            /// The struct2 docstring
            pub struct DummyStruct2;
            /// The enum2 docstring
            pub enum DummyEnum2 {}
        "#,
        )?;

        // Analyze the dummy crate
        let mut result = analyze_crate(temp_dir_path.to_str().unwrap())?;

        // Remove the file paths for snapshot testing, as they are non-deterministic
        for module in result.modules.iter_mut() {
            module.file = None;
        }

        assert_yaml_snapshot!(result, @r###"
        ---
        crate_:
          name: my_crate
          version: 0.1.0
        modules:
          - file: ~
            path:
              - my_crate
            docstring: The crate docstring
            declarations:
              - my_module
          - file: ~
            path:
              - my_crate
              - my_module
            docstring: The module docstring
            declarations:
              - my_submodule
          - file: ~
            path:
              - my_crate
              - my_module
              - my_submodule
            docstring: The sub-module docstring
            declarations: []
        structs:
          - path:
              - my_crate
              - my_module
              - DummyStruct1
            docstring: The struct1 docstring
            fields: []
          - path:
              - my_crate
              - my_module
              - my_submodule
              - DummyStruct2
            docstring: The struct2 docstring
            fields: []
        enums:
          - path:
              - my_crate
              - my_module
              - DummyEnum1
            docstring: The enum1 docstring
            variants: []
          - path:
              - my_crate
              - my_module
              - my_submodule
              - DummyEnum2
            docstring: The enum2 docstring
            variants: []
        functions: []
        "###);

        Ok(())
    }
}
