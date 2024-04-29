//! Analyze structs
use syn::{ItemStruct, Visibility};

use crate::data_model::{Field, Struct};

use super::{docstring_from_attrs, type_::convert_type};

impl Struct {
    /// Fully qualified name of the variant
    pub fn path_str(&self) -> String {
        self.path.join("::")
    }
    /// Extract the relevant information from the AST
    pub fn parse(parent: &[&str], ast: &ItemStruct) -> Self {
        let name = ast.ident.to_string();
        let path = parent
            .iter()
            .copied()
            .chain(Some(name.as_str()))
            .collect::<Vec<&str>>();
        let docstring = docstring_from_attrs(&ast.attrs);
        let mut struct_ = Self {
            path: path.iter().map(|s| s.to_string()).collect(),
            docstring,
            fields: vec![],
        };
        for (i, field) in ast.fields.iter().enumerate() {
            if let Visibility::Public(_) = field.vis {
                struct_.fields.push(Field::parse(&path, i, field));
            }
        }
        struct_
    }
}

impl Field {
    /// Extract the relevant information from the AST
    pub fn parse(parent: &[&str], position: usize, ast: &syn::Field) -> Self {
        let name = ast
            .ident
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or(position.to_string());
        let path = parent
            .iter()
            .copied()
            .chain(Some(name.as_str()))
            .collect::<Vec<&str>>();
        let docstring = docstring_from_attrs(&ast.attrs);
        let type_ = convert_type(&ast.ty);
        Self {
            path: path.iter().map(|s| s.to_string()).collect(),
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
        let struct_ = Struct::parse(&["crate"], &ast);
        assert_yaml_snapshot!(struct_, @r###"
        ---
        path:
          - crate
          - MyStruct
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
        let struct_ = Struct::parse(&["crate"], &ast);
        assert_yaml_snapshot!(struct_, @r###"
        ---
        path:
          - crate
          - MyStruct
        docstring: "Multi-line\ndocstring"
        fields:
          - path:
              - crate
              - MyStruct
              - my_field
            docstring: Docstring
            type_:
              - String: "["
              - Path: T
              - String: "; 1]"
        "###);
    }
}
