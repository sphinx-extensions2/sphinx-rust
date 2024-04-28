//! Analyze modules
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use syn::parse_file;

use super::{docstring_from_attrs, enum_::Enum, struct_::Struct};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a module
///
/// .. req:: Represent a module
///     :id: RUST005
///     :tags: rust
///     :status: in-progress
pub struct Module {
    /// The path to the module file
    pub file: Option<String>,
    /// The fully qualified name of the module
    pub path: Vec<String>,
    pub docstring: String,
    /// The public declarations in the module
    pub declarations: Vec<String>,
}

impl Module {
    /// Fully qualified name of the variant
    pub fn path_str(&self) -> String {
        self.path.join("::")
    }
    /// Extract the relevant information from the AST
    pub fn parse(
        file: Option<&Path>,
        path: &[&str],
        content: &str,
    ) -> Result<(Self, Vec<Struct>, Vec<Enum>)> {
        let syntax = parse_file(content)?;
        let mut mod_ = Self {
            file: file.map(|f| f.to_string_lossy().to_string()), // TODO better way to serialize the path, also ?
            path: path.iter().map(|s| s.to_string()).collect(),
            docstring: docstring_from_attrs(&syntax.attrs),
            declarations: vec![],
        };

        let mut structs = vec![];
        let mut enums = vec![];

        for item in syntax.items {
            // TODO traits, functions, impls, et
            match &item {
                syn::Item::Mod(mod_item) => {
                    if let syn::Visibility::Public(_) = mod_item.vis {
                        // TODO handle modules that are not just declarations
                        mod_.declarations.push(mod_item.ident.to_string());
                    }
                }
                syn::Item::Struct(struct_item) => {
                    if let syn::Visibility::Public(_) = struct_item.vis {
                        let struct_ = Struct::parse(path, struct_item);
                        structs.push(struct_);
                    }
                }
                syn::Item::Enum(enum_item) => {
                    if let syn::Visibility::Public(_) = enum_item.vis {
                        let enum_ = Enum::parse(path, enum_item);
                        enums.push(enum_);
                    }
                }
                _ => {}
            }
        }

        Ok((mod_, structs, enums))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn test_parse_module() {
        let content = r###"
//! Multi-line
//! docstring

pub enum MyEnum {
    MyVariant1,
}
"###;
        let mod_ = Module::parse(None, &["test"], content).unwrap();
        assert_yaml_snapshot!(mod_, @r###"
        ---
        - file: ~
          path:
            - test
          docstring: "Multi-line\ndocstring"
          declarations: []
        - []
        - - path:
              - test
              - MyEnum
            docstring: ""
            variants:
              - path:
                  - test
                  - MyEnum
                  - MyVariant1
                docstring: ""
                discriminant: ~
                fields: []
        "###);
    }
}
