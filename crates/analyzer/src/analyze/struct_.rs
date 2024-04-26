//! Analyze structs
use serde::{Deserialize, Serialize};
use syn::{ItemStruct, Visibility};

use super::{
    docstring_from_attrs,
    type_::{convert_type, TypeSegment},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a Struct
pub struct Struct {
    /// The name of the struct
    pub name: String,
    /// The docstring of the struct
    pub docstring: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a Struct field
pub struct Field {
    /// The name of the field
    pub name: Option<String>,
    /// The docstring of the field
    pub docstring: String,
    pub type_: Vec<TypeSegment>,
}

impl Struct {
    /// Extract the relevant information from the AST
    pub fn parse(parent: &str, ast: &ItemStruct) -> Self {
        let name = format!("{}::{}", parent, ast.ident);
        let docstring = docstring_from_attrs(&ast.attrs);
        let mut struct_ = Self {
            name,
            docstring,
            fields: vec![],
        };
        for field in ast.fields.iter() {
            if let Visibility::Public(_) = field.vis {
                struct_.fields.push(Field::parse(field));
            }
        }
        struct_
    }
}

impl Field {
    /// Extract the relevant information from the AST
    pub fn parse(ast: &syn::Field) -> Self {
        let name = ast.ident.as_ref().map(|name| name.to_string());
        let docstring = docstring_from_attrs(&ast.attrs);
        let type_ = convert_type(&ast.ty);
        Self {
            name,
            docstring,
            type_,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use syn::parse_quote;

    #[test]
    fn test_parse_struct_no_fields() {
        let ast: ItemStruct = parse_quote! {
            /// Multi-line
            /// docstring
            pub struct MyStruct;
        };
        let struct_ = Struct::parse("crate", &ast);
        assert_yaml_snapshot!(struct_, @r###"
        ---
        name: "crate::MyStruct"
        docstring: "Multi-line\ndocstring"
        fields: []
        "###);
    }

    #[test]
    fn test_parse_struct_fields() {
        let ast: ItemStruct = parse_quote! {
            /// Multi-line
            /// docstring
            pub struct MyStruct {
                /// Docstring
                pub my_field: [T; 1],
                /// a non-public field
                other: String,
            }
        };
        let struct_ = Struct::parse("crate", &ast);
        assert_yaml_snapshot!(struct_, @r###"
        ---
        name: "crate::MyStruct"
        docstring: "Multi-line\ndocstring"
        fields:
          - name: my_field
            docstring: Docstring
            type_:
              - String: "["
              - Path: T
              - String: "; 1]"
        "###);
    }
}
