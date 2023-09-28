// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2019-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

//! Amplifying Rust language capabilities: helper functions for creating proc
//! macro libraries
//!
//! # Examples
//!
//! `#[name]` - single form
//! `#[name = "literal"]` - optional single value
//! `#[name = TypeName]` - path value
//! `#[name("literal", TypeName, arg = value)]` - list of arguments

#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    missing_docs,
    dead_code
)]
#![allow(clippy::large_enum_variant)]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

mod attr;
mod cls;
mod error;
mod parsers;
mod req;
mod val;
mod data;

pub use attr::{Attr, ExtractAttr, ParametrizedAttr, SingularAttr};
pub use cls::{LiteralClass, TypeClass, ValueClass};
pub use data::{
    DataInner, DataType, DeriveInner, Element, EnumKind, Field, FieldKind, Fields, Items,
    NamedField, Scope, Variant, Vis,
};
pub use error::Error;
pub use parsers::{MetaArg, MetaArgList, MetaArgNameValue};
pub use req::{ArgValueReq, AttrReq, ListReq, ValueReq};
pub use val::ArgValue;

/// Convenience macro for constructing [`struct@syn::Ident`] from literals
#[macro_export]
macro_rules! ident {
    ($ident:ident) => {
        ::syn::Ident::new(stringify!($ident), ::proc_macro2::Span::call_site())
    };
}

/// Convenience macro for constructing [`struct@syn::Path`] from literals
#[macro_export]
macro_rules! path {
    ($ident:ident) => {
        ::syn::Path::from(ident!($ident))
    };
}

#[cfg(test)]
mod test {
    use proc_macro2::Span;
    use syn::Ident;

    #[test]
    fn ident() {
        assert_eq!(ident!(u8), Ident::new("u8", Span::call_site()));
    }
}
