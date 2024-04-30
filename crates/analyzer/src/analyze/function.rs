use crate::data_model::Function;

use super::docstring_from_attrs;

impl Function {
    /// Fully qualified name of the variant
    pub fn path_str(&self) -> String {
        self.path.join("::")
    }
    pub fn parse(parent: &[&str], ast: &syn::ItemFn) -> Self {
        let name = ast.sig.ident.to_string();
        let path: Vec<&str> = parent.iter().copied().chain(Some(name.as_str())).collect();
        let docstring = docstring_from_attrs(&ast.attrs);
        Self {
            path: path.iter().map(|s| s.to_string()).collect(),
            docstring,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_yaml_snapshot;

    #[test]
    fn test_function_parse() {
        let item: syn::ItemFn = syn::parse_quote! {
            /// This is a docstring
            pub fn my_function() {}
        };
        let func = Function::parse(&["my_module"], &item);
        assert_yaml_snapshot!(func, @r###"
        ---
        path:
          - my_module
          - my_function
        docstring: This is a docstring
        "###);
    }
}
