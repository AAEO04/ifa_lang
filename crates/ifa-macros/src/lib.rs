#![allow(clippy::collapsible_if)]

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
use syn::{DeriveInput, Expr, ItemFn, Token, parse::Parse, parse::ParseStream, parse_macro_input};

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
        if attr.path().is_ident("ebo") {
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = meta.tokens.to_string();
                if tokens.contains("cleanup") {
                    // Extract method name from cleanup = "method"
                    if let Some(start) = tokens.find('"') {
                        if let Some(end) = tokens.rfind('"') {
                            cleanup_method = Some(tokens[start + 1..end].to_string());
                        }
                    }
                }
            }
        }
    }

    let drop_impl = if let Some(method) = cleanup_method {
        let method_ident = format_ident!("{}", method);
        quote! {
            impl Drop for #name {
                fn drop(&mut self) {
                    println!("[Ebo] Sacrificing {}", stringify!(#name));
                    self.#method_ident();
                }
            }
        }
    } else {
        quote! {
            impl Drop for #name {
                fn drop(&mut self) {
                    println!("[Ebo] Sacrificed: {}", stringify!(#name));
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
/// #[iwa_pele]
/// fn network_task() {
///     let conn = Otura.so("example.com", 80);  // open
///     // ... work
///     conn.pa();  // close - REQUIRED or compile error
/// }
/// ```
///
/// ## Pairs Checked
/// - `so` / `pa` (open/close)
/// - `si` / `ti` (write open/close)  
/// - `mu` / `fi` (acquire/release)
/// - `bere` / `da` (start/stop)
#[proc_macro_attribute]
pub fn iwa_pele(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_sig = &input.sig;
    let fn_block = &input.block;

    struct BlockVisitor<'a> {
        errors: &'a mut Vec<String>,
        pairs: &'a [(&'static str, &'static str)],
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
                let open_count = *counts.get(*open).unwrap_or(&0);
                let close_count = *counts.get(*close).unwrap_or(&0);
                if open_count > close_count {
                    self.errors.push(format!(
                        "Ìwà Pẹ̀lẹ́ violation: Block leaks resource. {} '{}' calls but only {} '{}' calls inside this lexical scope.",
                        open_count, open, close_count, close
                    ));
                }
            }

            // Continue traversal for nested blocks
            syn::visit::visit_block(self, node);
        }
    }

    let pairs = [
        ("so", "pa"),   // socket open/close
        ("si", "ti"),   // file open/close
        ("mu", "fi"),   // acquire/release
        ("bere", "da"), // start/stop
    ];

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

    // Function passes static CFG balance check - emit ZERO-COST wrapper (no runtime prints)
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
            struct _EboGuard;
            impl Drop for _EboGuard {
                fn drop(&mut self) {
                    println!("--- Ebo Block Complete ---");
                }
            }
            println!("--- Ebo Block Started ---");
            let _guard = _EboGuard;
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
        quote! { .freeze().expect("Failed to freeze value") }
    } else {
        quote! { .clone() }
    };

    TokenStream::from(quote! {
        {
            println!("[Àjọṣe] Binding: {} => {}", stringify!(#source), stringify!(#target));
            // Create reactive subscription
            let _subscription = {
                let target_clone = #target.clone();
                move |new_value| {
                    // Apply freeze or clone based on syntax
                    *target_clone.borrow_mut() = new_value #freeze_logic;
                }
            };
            // Initial sync
            #target = #source #freeze_logic;
            _subscription
        }
    })
}

/// # Observable Wrapper Derive
///
/// Makes a struct's fields observable for reactive updates.
///
/// ## Usage
/// ```rust,ignore
/// #[derive(Observable)]
/// struct Counter {
///     value: i32,
/// }
/// // Generates Counter::watch_value() method
/// ```
#[proc_macro_derive(Observable)]
pub fn derive_observable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return TokenStream::from(quote! {
                compile_error!("Observable can only be derived for structs");
            });
        }
    };

    let mut watchers = Vec::new();

    for field in fields.iter() {
        if let Some(ident) = &field.ident {
            let watcher_name = format_ident!("watch_{}", ident);
            let field_ty = &field.ty;

            watchers.push(quote! {
                pub fn #watcher_name<F: Fn(&#field_ty) + 'static>(&self, callback: F) {
                    // Store callback for field changes
                    println!("[Observable] Watching: {}.{}", stringify!(#name), stringify!(#ident));
                    callback(&self.#ident);
                }
            });
        }
    }

    TokenStream::from(quote! {
        impl #name {
            #(#watchers)*
        }
    })
}
