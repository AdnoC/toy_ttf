#![crate_type = "proc-macro"]

extern crate proc_macro;
extern crate proc_macro2;
// #[macro_use]
// extern crate synstructure;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::DeriveInput;
use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_derive(Parse)]
pub fn parse_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let gen = parse_impl(ast);
    gen.into_token_stream().into()
}

fn parse_impl(ast: syn::DeriveInput) -> impl ToTokens {
    quote! { struct Hi; }
}
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
