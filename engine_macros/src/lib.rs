extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, FnArg, Pat};

#[proc_macro_attribute]
pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let vis = &input.vis;
    //let body = &input.block;
    
    let wrapper_name = quote::format_ident!("{}_wrapper", name);

    let mut args_unpacking = quote! {};
    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = &pat_ident.ident;
                args_unpacking.extend(quote! {
                    let #arg_name = &mut *ctx.#arg_name;
                });
            }
        }
    }
    let arg_idents: Vec<syn::Ident> = input.sig.inputs.iter().filter_map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                Some(pat_ident.ident.clone())
            } else {
                None
            }
        } else {
            None
        }
    }).collect();

    let expanded = quote! {

        #vis #input

        fn #wrapper_name(ctx: &mut crate::systems::SysCtx) {
            #args_unpacking
            #name( #(#arg_idents),* );
        }

        inventory::submit! {
            crate::systems::GameSystem { 
                name: stringify!(#name),
                func: #wrapper_name 
            }
        }
    };

    TokenStream::from(expanded)
}