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

    // let parse_body = s.each(|bi| {
    //     if let Some(len_src) = get_array_buffer_len(&bi) {
    //         quote! { unimplemented!() }
    //     } else {
    //         quote! { unimplemented!() }
    //     }
    // });

    let size_body = s.fold(quote!(0), |acc, bi| {
        // let ty = &bi.ast().ty;
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

        // extern crate parse_derive;
    s.gen_impl(quote! {
        use Parse;
        gen impl Parse for @Self {
            fn file_size(&self) -> usize {
                match *self {
                    #size_body
                }
            }
        }
    })
            // fn parse(#buf_var: &[u8]) -> (Self, &[u8]) {
            //     (Default::default(), #buf_var)
            //     // unimplemented!()
            //
            //     // match *self {
            //     //     #parse_body
            //     // }
            // }
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
trait Parse: Sized {
    /// Size of the object when serialized in the file
    fn file_size(&self) -> usize;
    fn parse(buf: &[u8]) -> (Self, &[u8]);
}

#[allow(dead_code)]
trait ArrayBuffer<T: Parse> {
    fn new(start: &[u8], len: usize) -> Self;
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
