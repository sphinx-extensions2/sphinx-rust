//! This module contains the code for analyzing the input Rust code and extracting the necessary information from it.

pub mod crate_;
pub mod enum_;
pub mod function;
pub mod module;
pub mod struct_;
pub mod type_;

pub use self::crate_::analyze_crate;

/// Extracts the docstring from an object's attributes
///
/// An initial whitespace character is stripped from the start of each line.
///
/// :param attrs: The attributes of the object
///
// TODO also extract an optional docstring type from the attributes?
pub(super) fn docstring_from_attrs(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                match &attr.meta {
                    syn::Meta::NameValue(value) => {
                        if let syn::Expr::Lit(value) = &value.value {
                            if let syn::Lit::Str(value) = &value.lit {
                                let string = value.value();
                                match string.strip_prefix(' ') {
                                    Some(string) => Some(string.to_string()),
                                    None => Some(string),
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docstring_from_attrs() {
        let attrs: Vec<syn::Attribute> = vec![
            syn::parse_quote! { #[doc = "This is a docstring"] },
            syn::parse_quote! { #[doc = "Another docstring"] },
            syn::parse_quote! { #[other_attr] },
        ];
        let result = docstring_from_attrs(&attrs);
        assert_eq!(result, "This is a docstring\nAnother docstring");
    }
}
