// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Elichai Turkel <elichai.turkel@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, Result};

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident_name = &input.ident;

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::std::error::Error for #ident_name #ty_generics #where_clause {
        }

        #[automatically_derived]
        impl #impl_generics From<#ident_name #ty_generics> for String #where_clause {
            fn from(err: #ident_name #ty_generics) -> Self {
                err.to_string()
            }
        }
    })
}
