//! Analyze types
use quote::quote;

use crate::data_model::TypeSegment;

impl From<&str> for TypeSegment {
    fn from(s: &str) -> Self {
        TypeSegment::String(
            s.replace(" :: ", "::")
                .replace(" < ", "<")
                .replace(" >", ">"),
        ) // TODO this is a hack for now
    }
}

impl From<String> for TypeSegment {
    fn from(s: String) -> Self {
        TypeSegment::String(
            s.replace(" :: ", "::")
                .replace(" < ", "<")
                .replace(" >", ">"),
        ) // TODO this is a hack for now
    }
}

/// Converts a syn type to a list of text and Paths
pub(super) fn convert_type(ty: &syn::Type) -> Vec<TypeSegment> {
    let mut v = convert_type_inner(ty);
    // Merge adjacent strings
    v = v.iter().fold(Vec::new(), |mut acc, elem| {
        if let Some(TypeSegment::String(s)) = acc.last_mut() {
            if let TypeSegment::String(next) = elem {
                s.push_str(next);
                return acc;
            }
        }
        acc.push(elem.clone());
        acc
    });
    v
}

fn convert_type_inner(ty: &syn::Type) -> Vec<TypeSegment> {
    match ty {
        syn::Type::Array(array) => {
            let mut v = vec!["[".into()];
            v.extend(convert_type(&array.elem));
            v.push("; ".into());
            let len = &array.len;
            v.push(quote! { #len }.to_string().into());
            v.push("]".into());
            v
        }
        syn::Type::BareFn(func) => vec![quote! { #func }.to_string().into()], // TODO this needs to be expanded
        syn::Type::Group(group) => convert_type(&group.elem),
        syn::Type::ImplTrait(imp) => {
            let mut v = vec!["impl ".into()];
            for (i, elem) in imp.bounds.iter().enumerate() {
                if i > 0 {
                    v.push(" + ".into());
                }
                v.push(quote! { #elem }.to_string().into()); // TODO this needs to be expanded to capture traits
            }
            v
        }
        syn::Type::Infer(_) => vec!["_".into()],
        syn::Type::Macro(mac) => vec![quote! { #mac }.to_string().into()],
        syn::Type::Never(_) => vec!["!".into()],
        syn::Type::Paren(paren) => {
            let mut v = vec!["(".into()];
            v.extend(convert_type(&paren.elem));
            v.push(")".into());
            v
        }
        syn::Type::Path(path) => {
            // TODO this is wrong, it puts spaces between the path segments
            vec![TypeSegment::Path(
                quote! { #path }
                    .to_string()
                    .replace(" :: ", "::")
                    .replace(" < ", "<")
                    .replace(" >", ">"),
            )]
        }
        syn::Type::Ptr(ptr) => {
            let mut v = vec![];
            if ptr.const_token.is_some() {
                v.push("*const ".into());
            } else if ptr.mutability.is_some() {
                v.push("*mut ".into());
            } else {
                v.push("*".into());
            }
            v.extend(convert_type(&ptr.elem));
            v
        }
        syn::Type::Reference(ref_) => {
            let mut v = vec!["&".into()];
            if let Some(lifetime) = &ref_.lifetime {
                v.push(lifetime.ident.to_string().into());
            }
            if ref_.mutability.is_some() {
                v.push(" mut ".into());
            } else {
                v.push(" ".into());
            }
            v.extend(convert_type(&ref_.elem));
            v
        }
        syn::Type::Slice(slice) => {
            let mut v = vec!["[".into()];
            v.extend(convert_type(&slice.elem));
            v.push("]".into());
            v
        }
        syn::Type::TraitObject(trait_) => {
            let mut v = vec!["dyn ".into()];
            for (i, elem) in trait_.bounds.iter().enumerate() {
                if i > 0 {
                    v.push(" + ".into());
                }
                v.push(quote! { #elem }.to_string().into()); // TODO this needs to be expanded to capture traits
            }
            v
        }
        syn::Type::Tuple(tuple) => {
            let mut v = vec!["(".into()];
            for (i, elem) in tuple.elems.iter().enumerate() {
                if i > 0 {
                    v.push(", ".into());
                }
                v.extend(convert_type(elem));
            }
            v.push(")".into());
            v
        }
        syn::Type::Verbatim(verb) => vec![quote! { #verb }.to_string().into()],
        _ => vec![quote! { #ty }.to_string().into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn ty_to_type_array() {
        let ty = syn::parse_quote! { [u8; 10] };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: "["
        - Path: u8
        - String: "; 10]"
        "###);
    }

    #[test]
    fn ty_to_type_infer() {
        let ty = syn::parse_quote! { _ };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: _
        "###);
    }

    #[test]
    fn ty_to_type_impl_trait() {
        let ty = syn::parse_quote! { impl Bound1 + Bound2 + Bound3 };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: impl Bound1 + Bound2 + Bound3
        "###);
    }

    #[test]
    fn ty_to_type_never() {
        let ty = syn::parse_quote! { ! };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: "!"
        "###);
    }

    #[test]
    fn ty_to_type_paren() {
        let ty = syn::parse_quote! { (u8) };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: (
        - Path: u8
        - String: )
        "###);
    }

    #[test]
    fn ty_to_type_path() {
        let ty = syn::parse_quote! { std::collections::HashMap<u8, u16> };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - Path: "std::collections::HashMap<u8 , u16>"
        "###);
    }

    #[test]
    fn ty_to_type_ptr() {
        let ty = syn::parse_quote! { *const u8 };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: "*const "
        - Path: u8
        "###);
    }

    #[test]
    fn ty_to_type_ref() {
        let ty = syn::parse_quote! { &'a mut u8 };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: "&a mut "
        - Path: u8
        "###);
    }

    #[test]
    fn ty_to_type_slice() {
        let ty = syn::parse_quote! { [u8] };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: "["
        - Path: u8
        - String: "]"
        "###);
    }

    #[test]
    fn ty_to_type_trait() {
        let ty = syn::parse_quote! { dyn std::fmt::Debug + 'a };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: "dyn std::fmt::Debug + 'a"
        "###);
    }

    #[test]
    fn ty_to_type_tuple() {
        let ty = syn::parse_quote! { (u8, u16) };
        let result = convert_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        - String: (
        - Path: u8
        - String: ", "
        - Path: u16
        - String: )
        "###);
    }
}
