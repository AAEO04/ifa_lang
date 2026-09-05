use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt, LitStr, Token, braced};

// Syntax we want to parse:
// odu_registry_dispatch! {
//     domain 0 => self.dispatch_ogbe {
//         0x01 => "bere",
//         0x02 => "pari"
//     },
//     domain 1 => self.dispatch_oyeku(ctx) {
//         ...
//     }
// }

struct MethodDef {
    id: LitInt,
    _arrow: Token![=>],
    name: LitStr,
}

impl Parse for MethodDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MethodDef {
            id: input.parse()?,
            _arrow: input.parse()?,
            name: input.parse()?,
        })
    }
}

struct DomainDef {
    _domain: Ident, // "domain"
    id: LitInt,
    _arrow: Token![=>],
    target: syn::Expr, // e.g. self.dispatch_ogbe or dispatch_osa
    needs_ctx: bool,
    _comma: syn::Token![,],
    _brace: syn::token::Brace,
    methods: Punctuated<MethodDef, Token![,]>,
}

impl Parse for DomainDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _domain: Ident = input.parse()?;
        let id: LitInt = input.parse()?;

        let mut needs_ctx = false;
        if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            if ident == "ctx" {
                needs_ctx = true;
            }
        }

        let _arrow: Token![=>] = input.parse()?;
        let target: syn::Expr = input.parse()?;
        let _comma: syn::Token![,] = input.parse()?;

        let content;
        let _brace = braced!(content in input);
        let methods = content.parse_terminated(MethodDef::parse, Token![,])?;

        Ok(DomainDef {
            _domain,
            id,
            _arrow,
            target,
            needs_ctx,
            _comma,
            _brace,
            methods,
        })
    }
}

struct RegistryDef {
    domains: Punctuated<DomainDef, Token![,]>,
}

impl Parse for RegistryDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RegistryDef {
            domains: input.parse_terminated(DomainDef::parse, Token![,])?,
        })
    }
}

pub fn odu_registry_dispatch(input: TokenStream) -> TokenStream {
    let registry = syn::parse_macro_input!(input as RegistryDef);

    let mut call_arms = Vec::new();
    let mut call_fast_arms = Vec::new();

    for domain in registry.domains {
        let domain_id = domain.id;
        let target = domain.target;
        let needs_ctx = domain.needs_ctx;

        // Build call() arm
        let call_arm = if needs_ctx {
            quote! { #domain_id => #target(method_name, args, ctx) }
        } else {
            quote! { #domain_id => #target(method_name, args) }
        };
        call_arms.push(call_arm);

        // Build call_fast() match branches
        let mut fast_method_arms = Vec::new();
        for method in domain.methods {
            let method_id = method.id;
            let method_name = method.name;

            let fast_arm = if needs_ctx {
                quote! { #method_id => #target(#method_name, args, ctx) }
            } else {
                quote! { #method_id => #target(#method_name, args) }
            };
            fast_method_arms.push(fast_arm);
        }

        let fast_domain_arm = quote! {
            #domain_id => match low {
                #(#fast_method_arms,)*
                _ => Err(IfaError::Custom(format!("Unknown method id {} for domain {}", low, domain_id))),
            }
        };
        call_fast_arms.push(fast_domain_arm);
    }

    // We emit just the match bodies so the caller can place them inside their methods
    let path_cpu: syn::Path = syn::parse_str("crate::hardware::cpu::dispatch").unwrap();
    let path_gpu: syn::Path =
        syn::parse_str("crate::hardware::gpu::dispatch").unwrap();
    let path_storage: syn::Path = syn::parse_str("crate::hardware::storage::dispatch").unwrap();

    let expanded = quote! {
        pub(crate) fn generated_call(
            &self,
            domain_id: u8,
            method_name: &str,
            args: Vec<IfaValue>,
            ctx: &mut VmContext,
        ) -> IfaResult<IfaValue> {
            match domain_id {
                #(#call_arms,)*
                18 => #path_cpu(method_name, args, ctx),
                #[cfg(feature = "gpu")]
                19 => #path_gpu(method_name, args, ctx),
                #[cfg(not(feature = "gpu"))]
                19 => Err(IfaError::Custom("GPU feature is disabled".to_string())),
                20 => #path_storage(&self.storage, method_name, args),
                29 => crate::hardware::sys::dispatch(method_name, args),
                _ => Err(IfaError::Custom(format!("Unknown domain {}", domain_id))),
            }
        }

        pub(crate) fn generated_call_fast(
            &self,
            domain_id: u8,
            method_id: u16,
            args: Vec<IfaValue>,
            ctx: &mut VmContext,
        ) -> IfaResult<IfaValue> {
            self.check_effect(domain_id)?;
            let low = (method_id & 0xFF) as u8;
            match domain_id {
                #(#call_fast_arms,)*
                _ => self.call(domain_id, &format!("unresolved_{}", low), args, ctx),
            }
        }
    };

    TokenStream::from(expanded)
}
