use quote::quote;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argument {
    /// A lifetime argument.
    Lifetime(String),
    /// A type argument.
    Type(Type),
    /// A const expression.
    Const(String),
    /// A binding (equality constraint) on an associated type: the `Item =
    /// u8` in `Iterator<Item = u8>`.
    AssocType(String), // TODO should be expanded
    /// An equality constraint on an associated constant: the `PANIC =
    /// false` in `Trait<PANIC = false>`.
    AssocConst(String), // TODO should be expanded
    /// An associated type bound: `Iterator<Item: Display>`.
    Constraint(String), // TODO should be expanded
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathArguments {
    None,
    /// The `<'a, T>` in `std::slice::iter<'a, T>`.
    AngleBracketed(Vec<Argument>),
    /// The `(A, B) -> C` in `Fn(A, B) -> C`.
    Parenthesized(String), // TODO should be expanded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegment {
    pub ident: String,
    arguments: PathArguments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitBound {
    /// A trait used as a bound on a type parameter.
    /// (Path, if has &)
    // TODO modifier, lifetimes
    Trait((Vec<PathSegment>, bool)),
    Lifetime(String),
    Verbatim(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    /// A fixed size array type: ``[T; n]``.
    Array((Box<Type>, String)),
    /// Indication that a type should be inferred by the compiler: _.
    Infer,
    /// The never type: !.
    Never,
    /// A type contained within invisible delimiters.
    Group(Box<Type>),
    /// An ``impl Bound1 + Bound2 + Bound3`` type where Bound is a trait or a lifetime.
    ImplTrait(Vec<TraitBound>),
    /// A parenthesized type equivalent to the inner type.
    Paren(Box<Type>),
    /// A path like ``std::slice::Iter``,
    /// optionally qualified with a self-type as in <Vec<T> as ``SomeTrait>::Associated``.
    Path(Vec<PathSegment>),
    /// A raw pointer type: ``*const T`` or ``*mut T``.
    /// ``(type, const, mut)``.
    Ptr((Box<Type>, bool, bool)),
    /// A reference type: ``&'a T`` or ``&'a mut T``.
    /// ``(type, lifetime, mutability)``.
    Reference((Box<Type>, String, bool)),
    /// Slice type: ``[T]``.
    Slice(Box<Type>),
    /// A trait object type ``dyn Bound1 + Bound2 + Bound3`` where Bound is a trait or a lifetime.
    Trait(Vec<TraitBound>),
    /// A tuple type: ``(T, U, ..)``.
    Tuple(Vec<Type>),
    /// Tokens in type position not interpreted by Syn.
    Unknown(String),
}

fn syn_path_to_path(path: &syn::Path) -> Vec<PathSegment> {
    path.segments
        .iter()
        .map(|segment| PathSegment {
            ident: segment.ident.to_string(),
            arguments: match &segment.arguments {
                syn::PathArguments::None => PathArguments::None,
                syn::PathArguments::AngleBracketed(ab) => PathArguments::AngleBracketed(
                    ab.args
                        .iter()
                        .map(|arg| match arg {
                            syn::GenericArgument::Lifetime(lifetime) => {
                                Argument::Lifetime(lifetime.ident.to_string())
                            }
                            syn::GenericArgument::Type(ty) => Argument::Type(ty_to_type(ty)),
                            syn::GenericArgument::Const(expr) => {
                                Argument::Const(quote! { #expr }.to_string())
                            }
                            syn::GenericArgument::AssocType(binding) => {
                                Argument::AssocType(quote! { #binding }.to_string())
                            }
                            syn::GenericArgument::AssocConst(binding) => {
                                Argument::AssocType(quote! { #binding }.to_string())
                            }
                            syn::GenericArgument::Constraint(constraint) => {
                                Argument::Constraint(quote! { #constraint }.to_string())
                            }
                            _ => Argument::Unknown(quote! { #arg }.to_string()),
                        })
                        .collect(),
                ),
                syn::PathArguments::Parenthesized(paren) => {
                    PathArguments::Parenthesized(quote! { #paren }.to_string())
                }
            },
        })
        .collect()
}

pub(super) fn ty_to_type(ty: &syn::Type) -> Type {
    match ty {
        syn::Type::Array(array) => {
            let len = &array.len;
            Type::Array((
                Box::new(ty_to_type(&array.elem)),
                quote! { #len }.to_string(),
            ))
        }
        syn::Type::BareFn(_) => unimplemented!(), // TODO bare function type
        syn::Type::Group(group) => Type::Group(Box::new(ty_to_type(&group.elem))),
        syn::Type::ImplTrait(imp) => Type::ImplTrait(
            imp.bounds
                .iter()
                .map(|bound| match &bound {
                    syn::TypeParamBound::Trait(trait_) => TraitBound::Trait((
                        syn_path_to_path(&trait_.path),
                        trait_.paren_token.is_some(),
                    )),
                    syn::TypeParamBound::Lifetime(lifetime) => {
                        TraitBound::Lifetime(lifetime.ident.to_string())
                    }
                    _ => TraitBound::Verbatim(quote! { #bound }.to_string()),
                })
                .collect(),
        ),
        syn::Type::Infer(_) => Type::Infer,
        syn::Type::Macro(_) => unimplemented!(), // TODO macro type
        syn::Type::Never(_) => Type::Never,
        syn::Type::Paren(paren) => Type::Paren(Box::new(ty_to_type(&paren.elem))),
        syn::Type::Path(path) => Type::Path(syn_path_to_path(&path.path)),
        syn::Type::Ptr(ptr) => Type::Ptr((
            Box::new(ty_to_type(&ptr.elem)),
            ptr.const_token.is_some(),
            ptr.mutability.is_some(),
        )),
        syn::Type::Reference(ref_) => Type::Reference((
            Box::new(ty_to_type(&ref_.elem)),
            ref_.lifetime
                .as_ref()
                .map_or_else(|| "".to_string(), |lifetime| lifetime.ident.to_string()),
            ref_.mutability.is_some(),
        )),
        syn::Type::Slice(slice) => Type::Slice(Box::new(ty_to_type(&slice.elem))),
        syn::Type::TraitObject(trait_) => Type::Trait(
            trait_
                .bounds
                .iter()
                .map(|bound| match &bound {
                    syn::TypeParamBound::Trait(trait_) => TraitBound::Trait((
                        syn_path_to_path(&trait_.path),
                        trait_.paren_token.is_some(),
                    )),
                    syn::TypeParamBound::Lifetime(lifetime) => {
                        TraitBound::Lifetime(lifetime.ident.to_string())
                    }
                    _ => TraitBound::Verbatim(quote! { #bound }.to_string()),
                })
                .collect(),
        ),
        syn::Type::Tuple(tuple) => Type::Tuple(tuple.elems.iter().map(ty_to_type).collect()),
        syn::Type::Verbatim(verb) => Type::Unknown(quote! { #verb }.to_string()),
        _ => Type::Unknown(quote! { #ty }.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn ty_to_type_array() {
        let ty = syn::parse_quote! { [u8; 10] };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Array:
          - Path:
              - ident: u8
                arguments: None
          - "10"
        "###);
    }

    #[test]
    fn ty_to_type_infer() {
        let ty = syn::parse_quote! { _ };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Infer
        "###);
    }

    #[test]
    fn ty_to_type_impl_trait() {
        let ty = syn::parse_quote! { impl Bound1 + Bound2 + Bound3 };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        ImplTrait:
          - Trait:
              - - ident: Bound1
                  arguments: None
              - false
          - Trait:
              - - ident: Bound2
                  arguments: None
              - false
          - Trait:
              - - ident: Bound3
                  arguments: None
              - false
        "###);
    }

    #[test]
    fn ty_to_type_never() {
        let ty = syn::parse_quote! { ! };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Never
        "###);
    }

    #[test]
    fn ty_to_type_paren() {
        let ty = syn::parse_quote! { (u8) };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Paren:
          Path:
            - ident: u8
              arguments: None
        "###);
    }

    #[test]
    fn ty_to_type_path() {
        let ty = syn::parse_quote! { std::collections::HashMap<u8, u16> };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Path:
          - ident: std
            arguments: None
          - ident: collections
            arguments: None
          - ident: HashMap
            arguments:
              AngleBracketed:
                - Type:
                    Path:
                      - ident: u8
                        arguments: None
                - Type:
                    Path:
                      - ident: u16
                        arguments: None
        "###);
    }

    #[test]
    fn ty_to_type_ptr() {
        let ty = syn::parse_quote! { *const u8 };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Ptr:
          - Path:
              - ident: u8
                arguments: None
          - true
          - false
        "###);
    }

    #[test]
    fn ty_to_type_ref() {
        let ty = syn::parse_quote! { &'a mut u8 };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Reference:
          - Path:
              - ident: u8
                arguments: None
          - a
          - true
        "###);
    }

    #[test]
    fn ty_to_type_slice() {
        let ty = syn::parse_quote! { [u8] };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Slice:
          Path:
            - ident: u8
              arguments: None
        "###);
    }

    #[test]
    fn ty_to_type_trait() {
        let ty = syn::parse_quote! { dyn std::fmt::Debug + 'a };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Trait:
          - Trait:
              - - ident: std
                  arguments: None
                - ident: fmt
                  arguments: None
                - ident: Debug
                  arguments: None
              - false
          - Lifetime: a
        "###);
    }

    #[test]
    fn ty_to_type_tuple() {
        let ty = syn::parse_quote! { (u8, u16) };
        let result = ty_to_type(&ty);
        assert_yaml_snapshot!(result, @r###"
        ---
        Tuple:
          - Path:
              - ident: u8
                arguments: None
          - Path:
              - ident: u16
                arguments: None
        "###);
    }
}
