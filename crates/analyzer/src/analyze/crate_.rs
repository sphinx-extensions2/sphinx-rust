//! Analyze the crate
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::Module;

pub fn analyze_crate(path: &str) -> Result<Crate> {
    // check the path is a directory
    let path = std::path::Path::new(path);
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory"));
    }
    // check if Cargo.toml exists
    let cargo_toml = path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(anyhow::anyhow!("Cargo.toml does not exist in directory"));
    }

    // read the Cargo.toml and initialize the Crate struct
    let contents = std::fs::read_to_string(cargo_toml)?;
    let cargo_toml: CargoToml = toml::from_str(&contents)?;

    // check whether the crate is a library or binary
    let root_file = if cargo_toml.lib.is_some() {
        if cargo_toml.bin.is_some() {
            return Err(anyhow::anyhow!("Both lib and bin sections in Cargo.toml"));
        }
        "lib.rs"
    } else if cargo_toml.bin.is_some() {
        "main.rs"
    } else {
        return Err(anyhow::anyhow!("No lib or bin section in Cargo.toml"));
    };

    let mut crate_ = Crate {
        name: cargo_toml.package.name,
        version: cargo_toml.package.version,
        modules: vec![],
    };

    // read the src/lib directory
    let src = path.join("src").join(root_file);
    if !src.exists() {
        return Ok(crate_);
    }

    // read the top-level module
    let content = std::fs::read_to_string(src)?;
    let module = Module::parse(&crate_.name, &content)?;
    let mut modules = module
        .mod_declarations
        .iter()
        .map(|s| (path.join("src"), s.to_string(), crate_.name.clone()))
        .collect::<Vec<_>>();
    crate_.modules.push(module);

    // recursively find/read the public sub-modules
    let mut read_modules = vec![];
    while let Some((path, module_name, parent_path)) = modules.pop() {
        // TODO also check for directory with mod.rs
        let module_path = path.join(&module_name).with_extension("rs");
        if !module_path.exists() || read_modules.contains(&module_path) {
            continue;
        }
        read_modules.push(module_path.clone());
        let sub_path = path.clone();

        let content = std::fs::read_to_string(module_path)?;
        let path_name = format!("{}::{}", parent_path, module_name);
        let module = Module::parse(&path_name, &content)?;
        modules.extend(
            module
                .mod_declarations
                .iter()
                .map(|s| (sub_path.clone(), s.to_string(), path_name.clone()))
                .collect::<Vec<_>>(),
        );
        crate_.modules.push(module);
    }

    Ok(crate_)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crate {
    name: String,
    version: String,
    modules: Vec<Module>,
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
struct Lib {}

#[derive(Debug, Deserialize)]
struct Bin {}

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
        "#,
        )?;

        // Create a dummy sub-module file
        let dummy_module_path = temp_dir_path.join("src").join("my_submodule.rs");
        std::fs::create_dir_all(dummy_module_path.parent().unwrap())?;
        std::fs::write(
            &dummy_module_path,
            r#"
            //! The sub-module docstring
            /// The struct2 docstring
            pub struct DummyStruct2;
        "#,
        )?;

        // Analyze the dummy crate
        let crate_ = analyze_crate(temp_dir_path.to_str().unwrap())?;

        assert_yaml_snapshot!(crate_, @r###"
        ---
        name: my_crate
        version: 0.1.0
        modules:
          - name: my_crate
            docstring: The crate docstring
            mod_declarations:
              - my_module
            structs: []
            enums: []
          - name: "my_crate::my_module"
            docstring: The module docstring
            mod_declarations:
              - my_submodule
            structs:
              - name: "my_crate::my_module::DummyStruct1"
                docstring: The struct1 docstring
                fields: []
            enums: []
          - name: "my_crate::my_module::my_submodule"
            docstring: The sub-module docstring
            mod_declarations: []
            structs:
              - name: "my_crate::my_module::my_submodule::DummyStruct2"
                docstring: The struct2 docstring
                fields: []
            enums: []
        "###);

        Ok(())
    }
}
