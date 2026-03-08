extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn en_system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let vis = &input.vis;

    let register_name = quote::format_ident!("__en_register_{}", name);

    let expanded = quote! {
        #input

        #[doc(hidden)]
        #vis fn #register_name(schedule: &mut en_core::bevy_ecs::schedule::Schedule) {
            schedule.add_systems(#name);
        }

        en_core::inventory::submit! {
            en_core::SystemRegister {
                name: stringify!(#name),
                register: #register_name,
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn include_scripts(_input: TokenStream) -> TokenStream {
    let mut expanded = quote! {};
    
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    
    let scripts_path = std::path::Path::new(&manifest_dir).join("src").join("scripts");
    
    if let Ok(entries) = std::fs::read_dir(&scripts_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "rs") {
                let path_str = path.to_str().unwrap();
                expanded.extend(quote! {
                    include!(#path_str);
                });
            }
        }
    } else {
        return TokenStream::from(quote! {});
    }

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn en_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(item as ItemStruct);
    let name = &input_struct.ident;
    let name_str = name.to_string();

    let expanded = quote! {
        #[derive(bevy_ecs::prelude::Component, serde::Serialize, serde::Deserialize, Clone, Debug, smart_default::SmartDefault)]
        #input_struct

        inventory::submit! {
            ::en_core::ComponentTemplate {
                name: #name_str,
                generator: || serde_json::to_value(#name::default()).unwrap(),
                
                inserter: |entity_mut, value| {
                    if let Ok(component) = serde_json::from_value::<#name>(value) {
                        entity_mut.insert(component);
                    } else {
                        eprintln!("[EnEngine] Failed to deserialize component: {}", #name_str);
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn export_plugin(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn en_get_plugin_registry() -> *mut ::en_core::PluginRegistry {
            let mut components = Vec::new();
            for template in ::en_core::inventory::iter::<::en_core::ComponentTemplate> {
                components.push(template.clone());
            }

            let mut systems = Vec::new();
            for sys in ::en_core::inventory::iter::<::en_core::SystemRegister> {
                systems.push(sys.clone());
            }

            let registry = Box::new(::en_core::PluginRegistry {
                components,
                systems,
            });

            Box::into_raw(registry)
        }
    };

    TokenStream::from(expanded)
}