//! # Ifá-Macros
//!
//! Procedural macros for Ifá-Lang's cultural safety features.
//!
//! ## Macros
//!
//! - `#[ebo]` - Auto-implement Drop for RAII cleanup
//! - `#[iwa_pele]` - Compile-time balance checking
//! - `ajose!` - Reactive binding declarations

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::visit::Visit;
use syn::{
    DeriveInput, Expr, ItemFn, ItemStruct, Token, parse::Parse, parse::ParseStream,
    parse_macro_input,
};

mod odu_registry;

#[proc_macro]
pub fn odu_registry_dispatch(input: TokenStream) -> TokenStream {
    odu_registry::odu_registry_dispatch(input)
}

/// # Ẹbọ Derive Macro
///
/// Automatically implements `Drop` for RAII resource cleanup.
///
/// ## Usage
/// ```rust,ignore
/// #[derive(Ebo)]
/// #[ebo(cleanup = "close")]  // Optional: specify cleanup method
/// struct MyFile {
///     handle: std::fs::File,
/// }
/// // Drop is auto-implemented calling self.close()
/// ```
#[proc_macro_derive(Ebo, attributes(ebo))]
pub fn derive_ebo(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Parse ebo attribute for custom cleanup method
    let mut cleanup_method = None;
    for attr in &input.attrs {
        if !attr.path().is_ident("ebo") {
            continue;
        }
        let Ok(meta) = attr.meta.require_list() else {
            continue;
        };
        let tokens = meta.tokens.to_string();
        if !tokens.contains("cleanup") {
            continue;
        }
        // Extract method name from cleanup = "method_name"
        let Some(start) = tokens.find('"') else { continue };
        let Some(end) = tokens.rfind('"') else { continue };
        cleanup_method = Some(tokens[start + 1..end].to_string());
    }

    let drop_impl = if let Some(method) = cleanup_method {
        let method_ident = format_ident!("{}", method);
        quote! {
            impl Drop for #name {
                fn drop(&mut self) {
                    #[cfg(feature = "dep:log")]
                    log::debug!("[Ebo] Sacrificing {}", stringify!(#name));
                    self.#method_ident();
                }
            }
        }
    } else {
        quote! {
            impl Drop for #name {
                fn drop(&mut self) {
                    #[cfg(feature = "dep:log")]
                    log::debug!("[Ebo] Sacrificed: {}", stringify!(#name));
                }
            }
        }
    };

    TokenStream::from(drop_impl)
}

/// # Ìwà Pẹ̀lẹ́ Attribute Macro
///
/// Compile-time balance checking for paired operations.
///
/// ## Usage
/// ```rust,ignore
/// #[iwa_pele(open, close)]
/// fn network_task() {
///     let conn = Network::open();
///     // ... work
///     conn.close();  // REQUIRED or compile error
/// }
/// ```
#[proc_macro_attribute]
pub fn iwa_pele(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_vis = &input.vis;
    let fn_sig = &input.sig;
    let fn_block = &input.block;

    struct BlockVisitor<'a> {
        errors: &'a mut Vec<String>,
        pairs: &'a [(String, String)],
    }

    impl<'ast, 'a> Visit<'ast> for BlockVisitor<'a> {
        fn visit_block(&mut self, node: &'ast syn::Block) {
            // Count method calls strictly within this lexical block
            let mut counts = std::collections::HashMap::new();

            for stmt in &node.stmts {
                // We use a temporary simple visitor to count direct calls in this statement
                struct StmtCounter {
                    calls: std::collections::HashMap<String, usize>,
                }
                impl<'ast> Visit<'ast> for StmtCounter {
                    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
                        *self.calls.entry(call.method.to_string()).or_insert(0) += 1;
                        syn::visit::visit_expr_method_call(self, call);
                    }
                }
                let mut stmt_counter = StmtCounter {
                    calls: std::collections::HashMap::new(),
                };
                stmt_counter.visit_stmt(stmt);

                for (k, v) in stmt_counter.calls {
                    *counts.entry(k).or_insert(0) += v;
                }
            }

            // Lexical CFG check: resources allocated in this block must be freed in this block
            for (open, close) in self.pairs {
                let open_count = *counts.get(open).unwrap_or(&0);
                let close_count = *counts.get(close).unwrap_or(&0);
                let yanda_count = *counts.get("yanda").unwrap_or(&0);
                let or_gentle_count = *counts.get("or_gentle").unwrap_or(&0);
                let or_recover_count = *counts.get("or_recover").unwrap_or(&0);

                let total_close = close_count + yanda_count + or_gentle_count + or_recover_count;

                if open_count > total_close {
                    self.errors.push(format!(
                        "Ìwà Pẹ̀lẹ́ violation: Block leaks resource. {} '{}' calls but only {} balancing calls ({} '{}', {} yanda, {} or_gentle, {} or_recover) inside this lexical scope.",
                        open_count, open, total_close, close_count, close, yanda_count, or_gentle_count, or_recover_count
                    ));
                }
            }

            // Continue traversal for nested blocks
            syn::visit::visit_block(self, node);
        }
    }

    // Parse custom pairs from attribute, e.g., #[iwa_pele(open, close)]
    let attr_str = attr.to_string();
    let parts: Vec<&str> = attr_str.split(',').map(|s| s.trim()).collect();

    let mut pairs = Vec::new();
    if parts.len() >= 2 {
        for chunk in parts.chunks(2) {
            if chunk.len() == 2 {
                pairs.push((chunk[0].to_string(), chunk[1].to_string()));
            }
        }
    }

    if pairs.is_empty() {
        return TokenStream::from(quote! {
            compile_error!("iwa_pele requires paired arguments, e.g., #[iwa_pele(open, close)]");
        });
    }

    let mut errors = Vec::new();
    let mut visitor = BlockVisitor {
        errors: &mut errors,
        pairs: &pairs,
    };
    visitor.visit_block(fn_block);

    if !errors.is_empty() {
        let error_msg = errors.join("\n");
        return TokenStream::from(quote! {
            compile_error!(#error_msg);
        });
    }

    // Function passes static CFG balance check
    TokenStream::from(quote! {
        #fn_vis #fn_sig {
            #fn_block
        }
    })
}

/// # Ẹbọ Block Macro
///
/// Creates a scoped RAII block with guaranteed cleanup.
///
/// ## Usage
/// ```rust,ignore
/// ebo_block! {
///     let file = std::fs::File::open("data.txt")?;
///     // file auto-closed when block exits
/// }
/// ```
#[proc_macro]
pub fn ebo_block(input: TokenStream) -> TokenStream {
    let block: proc_macro2::TokenStream = input.into();

    TokenStream::from(quote! {
        {
            let _ebo_guard = ifa_vm::ebo::Ebo::new("ebo_block", || {});
            #block
        }
    })
}

// Parse helper for ajose! macro
struct AjoseBinding {
    source: Expr,
    target: Expr,
    should_freeze: bool,
}

impl Parse for AjoseBinding {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let source: Expr = input.parse()?;
        input.parse::<Token![=>]>()?;

        let should_freeze = if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let ident: syn::Ident = input.parse()?;
            if ident != "freeze" {
                return Err(syn::Error::new(ident.span(), "Expected 'freeze' modifier"));
            }
            true
        } else {
            false
        };

        let target: Expr = input.parse()?;
        Ok(AjoseBinding {
            source,
            target,
            should_freeze,
        })
    }
}

/// # Àjọṣe Reactive Binding Macro
///
/// Creates reactive bindings between values.
///
/// ## Usage
/// ```rust,ignore
/// ajose!(counter.value => label.text);          // Standard binding
/// ajose!(counter.value => #freeze shared.data); // Auto-freeze binding (Cross-thread)
/// ```
#[proc_macro]
pub fn ajose(input: TokenStream) -> TokenStream {
    let binding = parse_macro_input!(input as AjoseBinding);
    let source = &binding.source;
    let target = &binding.target;

    let freeze_logic = if binding.should_freeze {
        quote! { val.freeze().expect("ajose!: #freeze failed — value contains a Set or GC reference that cannot cross thread boundaries. Use a List or Map instead.") }
    } else {
        quote! { val.clone() }
    };

    TokenStream::from(quote! {
        ifa_vm::ajose::bind!(#source => #target, |val| #freeze_logic)
    })
}

/// # Observable Attribute Macro
///
/// Transforms a struct's fields into reactive `Signal<T>` fields.
///
/// ## Usage
/// ```rust,ignore
/// #[observable]
/// struct Counter {
///     value: i32,
/// }
/// // Generates: Counter { value: Signal<i32> }
/// ```
#[proc_macro_attribute]
pub fn observable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    if let syn::Fields::Named(fields) = &mut input.fields {
        for field in &mut fields.named {
            let orig_ty = &field.ty;
            // Wrap the field type in `ifa_vm::ajose::Signal<T>`
            let new_ty: syn::Type = syn::parse_quote!(ifa_vm::ajose::Signal<#orig_ty>);
            field.ty = new_ty;
        }
    }

    TokenStream::from(quote! {
        #input
    })
}
