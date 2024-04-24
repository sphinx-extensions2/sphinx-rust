use serde::{Deserialize, Serialize};
use syn::parse_file;

use super::{docstring_from_attrs, enum_::Enum, struct_::Struct};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a module
pub struct Module {
    pub name: String,
    pub docstring: String,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
}

impl Module {
    /// Extract the relevant information from the AST
    pub fn parse(name: &str, content: &str) -> Result<Self, syn::Error> {
        let syntax = parse_file(content)?;

        let docstring = docstring_from_attrs(&syntax.attrs);

        let mut structs = vec![];
        let mut enums = vec![];

        for item in syntax.items {
            match &item {
                syn::Item::Struct(struct_item) => {
                    if let syn::Visibility::Public(_) = struct_item.vis {
                        let struct_ = Struct::parse(struct_item);
                        structs.push(struct_);
                    }
                }
                syn::Item::Enum(enum_item) => {
                    if let syn::Visibility::Public(_) = enum_item.vis {
                        let enum_ = Enum::parse(enum_item);
                        enums.push(enum_);
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            name: name.to_string(),
            docstring,
            structs,
            enums,
        })
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
        name: test
        docstring: "Multi-line\ndocstring"
        structs: []
        enums:
          - name: MyEnum
            docstring: ""
            variants:
              - name: MyVariant1
                docstring: ""
                discriminant: ~
                fields: []
        "###);
    }
}
