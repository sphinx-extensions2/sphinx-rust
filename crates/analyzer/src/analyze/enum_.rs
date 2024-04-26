//! Analyze enums
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::ItemEnum;

use super::{docstring_from_attrs, struct_::Field};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a Enum
///
/// .. req:: Represent an enum
///     :id: RUST003
///     :tags: rust
///     :status: in-progress
pub struct Enum {
    /// The name of the enum
    pub name: String,
    /// The docstring of the enum
    pub docstring: String,
    pub variants: Vec<Variant>,
}

impl Enum {
    /// Extract the relevant information from the AST
    pub fn parse(parent: &str, ast: &ItemEnum) -> Self {
        let name = format!("{}::{}", parent, ast.ident);
        let docstring = docstring_from_attrs(&ast.attrs);
        let variants = ast.variants.iter().map(Variant::parse).collect::<Vec<_>>();
        Self {
            name,
            docstring,
            variants,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a Enum variant
pub struct Variant {
    /// The name of the variant
    pub name: String,
    /// The docstring of the variant
    pub docstring: String,
    pub discriminant: Option<String>, // TODO shouldn't just be a string
    pub fields: Vec<Field>,
}

impl Variant {
    /// Extract the relevant information from the AST
    pub fn parse(ast: &syn::Variant) -> Self {
        let name = ast.ident.to_string();
        let docstring = docstring_from_attrs(&ast.attrs);
        let discriminant = ast
            .discriminant
            .as_ref()
            .map(|(_, e)| quote! {#e}.to_string());
        let fields = ast.fields.iter().map(Field::parse).collect::<Vec<_>>();
        Self {
            name,
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
        let enum_ = Enum::parse("crate", &ast);
        assert_yaml_snapshot!(enum_, @r###"
        ---
        name: "crate::MyEnum"
        docstring: "Multi-line\ndocstring"
        variants:
          - name: MyVariant1
            docstring: variant without fields
            discriminant: ~
            fields: []
          - name: MyVariant2
            docstring: variant with discriminant
            discriminant: "1"
            fields: []
          - name: MyVariant3
            docstring: variant with unnamed fields
            discriminant: ~
            fields:
              - name: ~
                docstring: ""
                type_:
                  - Path: u8
          - name: MyVariant3
            docstring: variant with named fields
            discriminant: ~
            fields:
              - name: field
                docstring: field docstring
                type_:
                  - Path: u8
        "###);
    }
}
