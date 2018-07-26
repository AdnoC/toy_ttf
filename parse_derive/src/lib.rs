#![crate_type = "proc-macro"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::DeriveInput;
use synstructure::{BindingInfo, Structure};

fn parse_derive(mut s: Structure) -> TokenStream {
    let buf_var = quote! { buf };

    for var in s.variants() {
        let mut found_dyn_sized = false;
        for bind in var.bindings() {
            if is_array_buffer(bind) {
                found_dyn_sized = true;
            } else {
                if found_dyn_sized {
                    panic!("Dynamically sized fields must be last");
                }
            }
        }
    }

    let parse_main: Vec<_> = s.variants()[0]
        .bindings()
        .iter()
        .enumerate()
        .map(|(i, bi)| {
            let ty = &bi.ast().ty;
            let name = bi.ast().ident.clone().unwrap_or_else(|| ident_for_index(i));

            if let Some(len_src) = get_array_buffer_len(&bi) {
                quote! {
                    let res = {
                        let len = #len_src as usize;
                        let buf = &#buf_var[0..len];
                        let res = <#ty as Parse>::parse(buf);
                        (&#buf_var[len..], res.1)
                    };
                    let #buf_var = res.0;
                    let #name = res.1;
                }
            } else {
                quote! {
                    let res = <#ty as Parse>::parse(#buf_var);
                    let #buf_var = res.0;
                    let #name = res.1;
                }
            }
        })
        .collect();
    let parse_body = s.variants()[0].construct(|field, i| {
        let ty = &field.ty;
        let name = field.ident.clone().unwrap_or_else(|| ident_for_index(i));
        quote! {
            #name
        }
        // if let Some(len_src) = get_array_buffer_len(&bi) {
        //     quote! { unimplemented!() }
        // } else {
        //     quote! { unimplemented!() }
        // }
    });

    let size_body = s.fold(quote!(0), |acc, bi| {
        // if let Some(len_src) = get_array_buffer_len(&bi) {
        //     quote! {
        //         #acc + {
        //             let len = self.#len_src as usize;
        //             len * #bi.file_size()
        //         }
        //     }
        // } else {
        quote! {
            #acc + #bi.file_size()
        }
        // }
    });

    println!("Size body = {:?}", size_body.to_string());
    println!("");
    println!(
        "Parse main = {}",
        parse_main
            .iter()
            .fold(String::new(), |acc, val| acc + "\n" + &val.to_string())
    );
    println!("");
    println!("Parse body = {:?}", parse_body.to_string());
    println!(
        "\nty params = [{}]",
        s.referenced_ty_params()
            .iter()
            .fold(String::new(), |acc, val| acc + ", " + &val.to_string())
    );
    // extern crate parse_derive;

    let mut generics = s.ast().generics.clone();
    let (_, ty_gen, where_clause) = s.ast().generics.split_for_impl();

    let (parse_lt, impl_gen) = {
        use proc_macro2::Span;
        use syn::{GenericParam, Lifetime, LifetimeDef};
        let has_lt = generics.lifetimes().next().is_some();
        if has_lt {
            let lt = generics.lifetimes().next().unwrap();
            (lt.clone(), generics.split_for_impl().0)
        } else {
            let lt = LifetimeDef::new(Lifetime::new("'parse_impl", Span::call_site()));
            generics
                .params
                .insert(0, GenericParam::Lifetime(lt.clone()));
            (lt, generics.split_for_impl().0)
        }
    };

    let name = &s.ast().ident;
    let real_impl = quote! {
        use Parse;
        impl #impl_gen Parse<#parse_lt> for #name #ty_gen #where_clause {
            fn file_size(&self) -> usize {
                match *self {
                    #size_body
                }
            }

            fn parse(#buf_var: &#parse_lt [u8]) -> (&#parse_lt [u8], Self) {
                #(#parse_main)*

                let val = #parse_body;
                (#buf_var, val)
            }
        }
    };

    use proc_macro2::{Ident, Span};
    // Use hygene
    let const_name = Ident::new(
        &("_DERIVE_Parse_a_FOR_".to_string() + &name.to_string()),
        Span::call_site(),
    );
    quote! {
        #[allow(non_upper_case_globals)]
        const #const_name: () = {
            #real_impl
        };
    }
}

fn get_array_buffer_len(bi: &BindingInfo) -> Option<syn::Ident> {
    use proc_macro2::{Ident, Span};
    use syn::{Lit, Meta};
    bi.ast()
        .attrs
        .iter()
        .filter_map(|attr| attr.interpret_meta())
        .find(|meta| meta.name() == "arr_len_src")
        .map(|meta| match meta {
            Meta::NameValue(name_val) => name_val.lit,
            _ => panic!("arr_len_src needs a field name"),
        })
        .map(|lit| match lit {
            Lit::Str(name) => name,
            _ => panic!("arr_len_src needs a field name"),
        })
        .map(|name| name.value())
        .map(|name| Ident::new(&name, Span::call_site()))
}
fn is_array_buffer(bi: &BindingInfo) -> bool {
    bi.ast()
        .attrs
        .iter()
        .any(|attr| match attr.interpret_meta() {
            Some(meta) => meta.name() == "arr_len_src",
            None => false,
        })
}

fn ident_for_index(i: usize) -> syn::Ident {
    use proc_macro2::{Ident, Span};
    Ident::new(&format!("__field_{}", i), Span::call_site())
}

decl_derive!([Parse, attributes(arr_len_src)] => parse_derive);
