// delegate_impl
//
// Copyright (C) 2019 SOFe
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, FnArg, ItemTrait, TraitItem};

#[derive(FromMeta)]
struct DelegateArgs {
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub path: Option<String>,
}

#[proc_macro_attribute]
pub fn delegate(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let args = match DelegateArgs::from_list(&args) {
        Ok(args) => args,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let item = parse_macro_input!(item as ItemTrait);
    let item_clone = item.clone();

    let item_name = item.ident;
    let item_path = match args.path {
        Some(path) => {
            let pieces = path
                .split("::")
                .filter(|piece| !piece.is_empty())
                .map(|piece| Ident::new(piece, Span::call_site()))
                .map(|piece| quote!(#piece::));
            quote!(#(#pieces)* #item_name)
        }
        None => quote!(#item_name),
    };

    let macro_name = if let Some(alt_name) = args.name {
        alt_name
    } else {
        use heck::SnakeCase;
        format!("delegate_{}", item_name.to_string().to_snake_case())
    };
    let macro_name = Ident::new(&macro_name, Span::call_site());

    let methods = item
        .items
        .into_iter()
        .filter_map(|item| match item {
            TraitItem::Method(method) => Some(method),
            _ => None,
        })
        .filter(|method| method.default.is_none())
        .filter(|method| {
            method.sig.decl.inputs.iter().any(|arg| match arg {
                FnArg::SelfRef(_) | FnArg::SelfValue(_) => true,
                _ => false,
            })
        })
        .map(|method| {
            let sig = method.sig;
            let name = &sig.ident;
            let params = sig.decl.inputs.iter().filter_map(|arg| match arg {
                FnArg::Captured(cap) => Some(&cap.pat),
                _ => None,
            });
            quote! {
                #sig { self.$inner.#name(#(#params),*) }
            }
        });

    let item_name_clone = item_name.clone();
    let methods_clone = methods.clone();

    let x = quote! {
        #item_clone

        #[macro_export]
        macro_rules! #macro_name {
            ($(<$($trait_types:ty),*> for)? $struct:ty : $inner:ident ; $($extra:tt)* ) => {
                impl $crate::#item_path for $struct {
                    #(#methods)*
                    $($extra)*
                }
            };
            (<$($types:ty),*> impl <$($trait_types:ty),*> for $struct:ty : $inner:expr ; $($extra:tt)*) => {
                impl #item_name_clone for $struct {
                    #(#methods_clone)*
                    $($extra)*
                }
            };
        }
    };
    println!("{}", &x.to_string());
    x.into()
}
