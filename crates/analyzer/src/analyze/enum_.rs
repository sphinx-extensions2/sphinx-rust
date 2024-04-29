//! Analyze enums
use quote::quote;
use syn::ItemEnum;

use crate::data_model::{Enum, Field, Variant};

use super::docstring_from_attrs;

impl Enum {
    /// Fully qualified name of the variant
    pub fn path_str(&self) -> String {
        self.path.join("::")
    }
    /// Extract the relevant information from the AST
    pub fn parse(parent: &[&str], ast: &ItemEnum) -> Self {
        let name = ast.ident.to_string();
        let path: Vec<&str> = parent.iter().copied().chain(Some(name.as_str())).collect();
        let docstring = docstring_from_attrs(&ast.attrs);
        let variants = ast
            .variants
            .iter()
            .map(|v| Variant::parse(&path, v))
            .collect::<Vec<_>>();
        Self {
            path: path.iter().map(|s| s.to_string()).collect(),
            docstring,
            variants,
        }
    }
}

impl Variant {
    /// Fully qualified name of the variant
    pub fn name(&self) -> String {
        self.path.join("::")
    }
    /// Extract the relevant information from the AST
    pub fn parse(parent: &[&str], ast: &syn::Variant) -> Self {
        let name = ast.ident.to_string();
        let path = parent
            .iter()
            .copied()
            .chain(Some(name.as_str()))
            .collect::<Vec<&str>>();
        let docstring = docstring_from_attrs(&ast.attrs);
        let discriminant = ast
            .discriminant
            .as_ref()
            .map(|(_, e)| quote! {#e}.to_string());
        let fields = ast
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| Field::parse(&path, i, f))
            .collect::<Vec<_>>();
        Self {
            path: path.iter().map(|s| s.to_string()).collect(),
            docstring,
            discriminant,
            fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use syn::parse_quote;

    #[test]
    fn test_parse_enum() {
        let ast: ItemEnum = parse_quote! {
            /// Multi-line
            /// docstring
            pub enum MyEnum {
                /// variant without fields
                MyVariant1,
                /// variant with discriminant
                MyVariant2 = 1,
                /// variant with unnamed fields
                MyVariant3(u8),
                /// variant with named fields
                MyVariant3 {
                    /// field docstring
                    field: u8,
                },
            }
        };
        let enum_ = Enum::parse(&["crate"], &ast);
        assert_yaml_snapshot!(enum_, @r###"
        ---
        path:
          - crate
          - MyEnum
        docstring: "Multi-line\ndocstring"
        variants:
          - path:
              - crate
              - MyEnum
              - MyVariant1
            docstring: variant without fields
            discriminant: ~
            fields: []
          - path:
              - crate
              - MyEnum
              - MyVariant2
            docstring: variant with discriminant
            discriminant: "1"
            fields: []
          - path:
              - crate
              - MyEnum
              - MyVariant3
            docstring: variant with unnamed fields
            discriminant: ~
            fields:
              - path:
                  - crate
                  - MyEnum
                  - MyVariant3
                  - "0"
                docstring: ""
                type_:
                  - Path: u8
          - path:
              - crate
              - MyEnum
              - MyVariant3
            docstring: variant with named fields
            discriminant: ~
            fields:
              - path:
                  - crate
                  - MyEnum
                  - MyVariant3
                  - field
                docstring: field docstring
                type_:
                  - Path: u8
        "###);
    }
}
