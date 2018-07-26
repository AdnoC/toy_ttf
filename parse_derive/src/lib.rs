#![crate_type = "proc-macro"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::DeriveInput;
use proc_macro2::TokenStream;
use quote::ToTokens;
use synstructure::{BindingInfo, Structure};

    // use proc_macro::TokenStream;
// #[proc_macro_derive(Parse, attributes(len_src, file_only))]
// pub fn parse_derive(input: TokenStream) -> proc_macro::TokenStream {
//     let gen = quote! { struct Hi; };
//     gen.into_token_stream().into()
//     // let ast: DeriveInput = syn::parse(input).unwrap();
//     // let gen = parse_impl(ast);
//     // gen.into()
// }

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
        .map(|bi| {
            let ty = &bi.ast().ty;
            let name = &bi.ast().ident;

            if let Some(len_src) = get_array_buffer_len(&bi) {
                quote! {
                    use ArrayBuffer;
                    let res = <#ty as ArrayBuffer>::new(#buf_var, #len_src);
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
        let name = &field.ident;
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
        if let Some(len_src) = get_array_buffer_len(&bi) {
            quote! {
                #acc + {
                    let len = self.#len_src as usize;
                    len * #bi.file_size()
                }
            }
        } else {
            quote! {
                #acc + #bi.file_size()
            }
        }
    });

    println!("Size body = {:?}", size_body.to_string());
    println!("");
    println!("Parse main = {}", parse_main.iter().fold(String::new(), |acc, val| acc + "\n" + &val.to_string()));
    println!("");
    println!("Parse body = {:?}", parse_body.to_string());

        // extern crate parse_derive;
    s.gen_impl(quote! {
        use Parse;
        gen impl Parse for @Self {
            fn file_size(&self) -> usize {
                match *self {
                    #size_body
                }
            }

            fn parse(#buf_var: &[u8]) -> (&[u8], Self) {
                #(#parse_main)*

                let val = #parse_body;
                (#buf_var, val)
            }
        }
    })
}

fn get_array_buffer_len(bi: &BindingInfo) -> Option<syn::Ident> {
    use syn::{Lit, Meta};
    use proc_macro2::{Ident, Span};
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
            None => false
        })
}

#[allow(dead_code)]
trait Parse {
    /// Size of the object when serialized in the file
    fn file_size(&self) -> usize;
    fn parse(buf: &[u8]) -> (&[u8], Self);
}

#[allow(dead_code)]
trait ArrayBuffer<T: Parse> {
    fn new(start: &[u8], len: usize) -> (&[u8], Self);
}
#[allow(dead_code)]
struct ArrBuf<'a, T: Parse + 'a>(&'a [u8], ::std::marker::PhantomData<&'a [T]>);

fn gen_final_struct(mut s: Structure) -> TokenStream {
    // Would also need to remove the custom attributes from the generated type def
    s.filter(|bi| {
        !bi.ast()
            .attrs
            .iter()
            .any(|attr| match attr.interpret_meta() {
                Some(meta) => meta.name() == "file_only",
                None => false,
            })
    });

    use proc_macro2::{Ident, Span};

    let mut ast = s.ast().clone();
    ast.ident = Ident::new("QWER", Span::call_site());
    ast.into_token_stream()
}

decl_derive!([Parse, attributes(arr_len_src)] => parse_derive);




// fn parse_derive(mut s: synstructure::Structure) -> TokenStream {
//     // No way to check whether a field implements a trait, so have an attribute
//     // to ignore fields that don't implement Trace.
//     // https://github.com/dtolnay/syn/issues/77
//     s.filter(|bind_info| {
//         !bind_info
//             .ast()
//             .attrs
//             .iter()
//             .any(|attr| match attr.interpret_meta() {
//                 Some(meta) => meta.name() == "ignore_trace",
//                 None => false,
//             })
//     });
//
//     let body = s.each(|bind_info| {
//         quote! {
//             _tracer.add_target(#bind_info);
//         }
//     });
//
//     s.gen_impl(quote! {
//         extern crate ters_gc;
//         gen impl ters_gc::trace::Trace for @Self {
//             fn trace(&self, _tracer: &mut ters_gc::trace::Tracer) {
//                 match *self {
//                     #body
//                 }
//             }
//         }
//     }).into()
// }
//
