use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    Expr, ExprLit, Lit, LitInt, LitStr, Result, Token,
};

/// Plugin configuration parsed from the macro input
struct PluginConfig {
    name: LitStr,
    priority: Option<LitInt>,
    description: Option<LitStr>,
    prefix: Option<LitStr>,
    by_prefix_only: Option<bool>,
    init_fn: Option<Ident>,
    default_list_fn: Option<Ident>,
    filter_fn: Option<Ident>,
}

impl Parse for PluginConfig {
    fn parse(input: ParseStream) -> Result<Self> {

        let mut name = None;
        let mut priority = None;
        let mut description = None;
        let mut prefix = None;
        let mut by_prefix_only = None;
        let mut init_fn = None;
        let mut default_list_fn = None;
        let mut filter_fn = None;

        // Parse comma-separated key: value pairs
        while !input.is_empty() {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            
            if input.is_empty() {
                break;
            }

            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name = Some(lit);
                }
                "priority" => {
                    let lit: LitInt = input.parse()?;
                    priority = Some(lit);
                }
                "description" => {
                    let lit: LitStr = input.parse()?;
                    description = Some(lit);
                }
                "prefix" => {
                    let lit: LitStr = input.parse()?;
                    prefix = Some(lit);
                }
                "by_prefix_only" => {
                    let expr: Expr = input.parse()?;
                    if let Expr::Lit(ExprLit { lit: Lit::Bool(lit_bool), .. }) = expr {
                        by_prefix_only = Some(lit_bool.value);
                    } else {
                        return Err(syn::Error::new_spanned(expr, "Expected boolean literal"));
                    }
                }
                "init" => {
                    let fn_name: Ident = input.parse()?;
                    init_fn = Some(fn_name);
                }
                "default_list" => {
                    let fn_name: Ident = input.parse()?;
                    default_list_fn = Some(fn_name);
                }
                "filter" => {
                    let fn_name: Ident = input.parse()?;
                    filter_fn = Some(fn_name);
                }
                _ => {
                    let key_str = key.to_string();
                    return Err(syn::Error::new_spanned(
                        key,
                        format!("Unknown plugin configuration key: {}", key_str),
                    ));
                }
            }
        }

        // Validate required fields
        let name = name.ok_or_else(|| {
            syn::Error::new(input.span(), "Plugin must have a 'name' field")
        })?;

        Ok(PluginConfig {
            name,
            priority,
            description,
            prefix,
            by_prefix_only,
            init_fn,
            default_list_fn,
            filter_fn,
        })
    }
}

impl PluginConfig {
    /// Generate the implementation of LauncherPlugin trait methods
    fn generate(&self) -> proc_macro2::TokenStream {
        let name_str = &self.name;

        // Generate priority method
        let priority = if let Some(ref priority_lit) = self.priority {
            quote! { #priority_lit }
        } else {
            quote! { 100 }
        };

        // Generate description method
        let description = if let Some(ref desc_lit) = self.description {
            quote! { Some(#desc_lit.to_string()) }
        } else {
            quote! { None }
        };

        // Generate prefix method
        let prefix = if let Some(ref prefix_lit) = self.prefix {
            quote! { Some(#prefix_lit.to_string()) }
        } else {
            quote! { None }
        };

        // Generate by_prefix_only method
        let by_prefix_only = if let Some(value) = self.by_prefix_only {
            quote! { #value }
        } else {
            quote! { false }
        };

        // Generate init method
        let init_method = if let Some(ref init_fn) = self.init_fn {
            quote! {
                fn init(&self) {
                    #init_fn(self);
                }
            }
        } else {
            quote! {
                fn init(&self) {
                    // Default empty init
                }
            }
        };

        // Generate default_list method
        let default_list_method = if let Some(ref default_list_fn) = self.default_list_fn {
            quote! {
                fn default_list(&self) -> Vec<Box<dyn waycast_core::LauncherListItem>> {
                    #default_list_fn(self)
                }
            }
        } else {
            quote! {
                fn default_list(&self) -> Vec<Box<dyn waycast_core::LauncherListItem>> {
                    Vec::new()
                }
            }
        };

        // Generate filter method
        let filter_method = if let Some(ref filter_fn) = self.filter_fn {
            quote! {
                fn filter(&self, query: &str) -> Vec<Box<dyn waycast_core::LauncherListItem>> {
                    #filter_fn(self, query)
                }
            }
        } else {
            quote! {
                fn filter(&self, query: &str) -> Vec<Box<dyn waycast_core::LauncherListItem>> {
                    Vec::new()
                }
            }
        };

        quote! {
            #init_method

            fn name(&self) -> String {
                #name_str.to_string()
            }

            fn priority(&self) -> i32 {
                #priority
            }

            fn description(&self) -> Option<String> {
                #description
            }

            fn prefix(&self) -> Option<String> {
                #prefix
            }

            fn by_prefix_only(&self) -> bool {
                #by_prefix_only
            }

            #default_list_method

            #filter_method

            // Stub implementations to satisfy rust-analyzer
            // These are only compiled when rust-analyzer is checking the code
            #[cfg(rust_analyzer)]
            fn init(&self) { unimplemented!("Generated by plugin! macro") }
            #[cfg(rust_analyzer)]
            fn name(&self) -> String { unimplemented!("Generated by plugin! macro") }
            #[cfg(rust_analyzer)]
            fn priority(&self) -> i32 { unimplemented!("Generated by plugin! macro") }
            #[cfg(rust_analyzer)]
            fn description(&self) -> Option<String> { unimplemented!("Generated by plugin! macro") }
            #[cfg(rust_analyzer)]
            fn prefix(&self) -> Option<String> { unimplemented!("Generated by plugin! macro") }
            #[cfg(rust_analyzer)]
            fn by_prefix_only(&self) -> bool { unimplemented!("Generated by plugin! macro") }
            #[cfg(rust_analyzer)]
            fn default_list(&self) -> Vec<Box<dyn waycast_core::LauncherListItem>> { unimplemented!("Generated by plugin! macro") }
            #[cfg(rust_analyzer)]
            fn filter(&self, _query: &str) -> Vec<Box<dyn waycast_core::LauncherListItem>> { unimplemented!("Generated by plugin! macro") }
        }
    }
}

/// The main plugin! proc macro
/// 
/// Usage inside impl LauncherPlugin block:
/// ```rust
/// impl LauncherPlugin for MyPlugin {
///     plugin! {
///         name: "calculator",
///         priority: 500,
///         prefix: "calc",
///         filter: calc_filter,
///     }
/// }
/// ```
#[proc_macro]
pub fn plugin(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as PluginConfig);
    config.generate().into()
}