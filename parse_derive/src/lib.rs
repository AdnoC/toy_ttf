#![crate_type = "proc-macro"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use synstructure::{BindingInfo, Structure};

#[allow(needless_pass_by_value)]
fn parse_derive(s: Structure) -> TokenStream {
    let buf_var = quote! { _buf };
    let init_buf_var = quote! { _init_buf };

    let parse_main: Vec<_> = s.variants()[0]
        .bindings()
        .iter()
        .enumerate()
        .map(|(i, bi)| {
            let ty = &bi.ast().ty;
            let name = bi.ast().ident.clone().unwrap_or_else(|| ident_for_index(i));

            let limit_length = if is_len_src(&bi) {
                let limiter = quote! {
                    let #buf_var = {
                        let total_len = #name;
                        let eaten = #buf_var.as_ptr() as usize - #init_buf_var.as_ptr() as usize;
                        let remain = total_len - eaten;
                        &#buf_var[..remain]
                    };
                };
                Some(limiter)
            } else { None };
            let pb = if let Some(len_src) = get_array_buffer_len(&bi) {
                quote! {
                    let (#buf_var, #name) = {
                        let len = #len_src as usize;
                        let size = len * <#ty as Parse>::approx_file_size();
                        let (buf, remain) = #buf_var.split_at(size);
                        let res = <#ty as Parse>::parse(buf);
                        (remain, res.1)
                    };
                }
            } else {
                quote! {
                    let (#buf_var, #name) = <#ty as Parse>::parse(#buf_var);
                }
            };

            if let Some(modifier) = get_parse_mod(&bi) {
                quote! {
                    #pb
                    let #name = #modifier(#name);
                }
            } else { pb }

        })
        .collect();
    let parse_body = s.variants()[0].construct(|field, i| {
        let name = field.ident.clone().unwrap_or_else(|| ident_for_index(i));
        quote! {
            #name
        }
    });

    let size_body = s.variants()[0]
        .bindings()
        .iter()
        .fold(quote!(0), |acc, bi| {
        let ty = &bi.ast().ty;
        quote! {
            #acc + <#ty as Parse>::approx_file_size()
        }
    });

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

    // extern crate parse_derive;
    let name = &s.ast().ident;
    let real_impl = quote! {
        impl #impl_gen Parse<#parse_lt> for #name #ty_gen #where_clause {
            fn approx_file_size() -> usize {
                #size_body
            }

            fn parse(#buf_var: &#parse_lt [u8]) -> (&#parse_lt [u8], Self) {
                let #init_buf_var = #buf_var;
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
fn get_parse_mod(bi: &BindingInfo) -> Option<syn::Ident> {
    use proc_macro2::{Ident, Span};
    use syn::{Lit, Meta};
    bi.ast()
        .attrs
        .iter()
        .filter_map(|attr| attr.interpret_meta())
        .find(|meta| meta.name() == "parse_mod")
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
fn is_len_src(bi: &BindingInfo) -> bool {
    bi.ast()
        .attrs
        .iter()
        .any(|attr| match attr.interpret_meta() {
            Some(meta) => meta.name() == "len_src",
            None => false,
        })
}

fn ident_for_index(i: usize) -> syn::Ident {
    use proc_macro2::{Ident, Span};
    Ident::new(&format!("__field_{}", i), Span::call_site())
}

decl_derive!([Parse, attributes(arr_len_src, parse_mod, len_src)] => parse_derive);
