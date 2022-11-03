//! Derives the [`IterableEnum`] trait
//
//! This derive macro was inspired by the `strum` crate's `EnumIter` macro.
//! Original source: <https://github.com/Peternator7/strum>,
//! Copyright (c) 2019 Peter Glotfelty under the MIT License

mod iterable_enum;

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(IterableEnum)]
pub fn iterable_enum(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    crate::iterable_enum::iterable_enum_inner(&ast).into()
}
