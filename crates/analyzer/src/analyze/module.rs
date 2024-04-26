//! Analyze modules
use anyhow::Result;
use serde::{Deserialize, Serialize};
use syn::parse_file;

use super::{docstring_from_attrs, enum_::Enum, struct_::Struct};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a module
pub struct Module {
    pub name: String,
    pub docstring: String,
    /// The public declarations in the module
    pub declarations: Vec<String>,
}

impl Module {
    /// Extract the relevant information from the AST
    pub fn parse(name: &str, content: &str) -> Result<(Self, Vec<Struct>, Vec<Enum>)> {
        let syntax = parse_file(content)?;

        let mut mod_ = Self {
            name: name.to_string(),
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
                        let struct_ = Struct::parse(name, struct_item);
                        structs.push(struct_);
                    }
                }
                syn::Item::Enum(enum_item) => {
                    if let syn::Visibility::Public(_) = enum_item.vis {
                        let enum_ = Enum::parse(name, enum_item);
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
    fn test_parse_enum() {
        let content = r###"
//! Multi-line
//! docstring

pub enum MyEnum {
    MyVariant1,
}
"###;
        let mod_ = Module::parse("test", content).unwrap();
        assert_yaml_snapshot!(mod_, @r###"
        ---
        - name: test
          docstring: "Multi-line\ndocstring"
          declarations: []
        - []
        - - name: "test::MyEnum"
            docstring: ""
            variants:
              - name: MyVariant1
                docstring: ""
                discriminant: ~
                fields: []
        "###);
    }
}
