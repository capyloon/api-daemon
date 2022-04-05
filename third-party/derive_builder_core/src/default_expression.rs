use std::convert::TryFrom;

use crate::BlockContents;
use quote::{ToTokens, TokenStreamExt};

/// A `DefaultExpression` can be either explicit or refer to the canonical trait.
#[derive(Debug, Clone)]
pub enum DefaultExpression {
    Explicit(BlockContents),
    Trait,
}

impl DefaultExpression {
    #[cfg(test)]
    pub fn explicit<I: Into<BlockContents>>(content: I) -> Self {
        DefaultExpression::Explicit(content.into())
    }
}

impl darling::FromMeta for DefaultExpression {
    fn from_word() -> darling::Result<Self> {
        Ok(DefaultExpression::Trait)
    }

    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(s) = value {
            let contents = BlockContents::try_from(s)?;
            if contents.is_empty() {
                Err(darling::Error::unknown_value("").with_span(s))
            } else {
                Ok(Self::Explicit(contents))
            }
        } else {
            Err(darling::Error::unexpected_lit_type(value))
        }
    }
}

impl ToTokens for DefaultExpression {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match *self {
            Self::Explicit(ref block) => block.to_tokens(tokens),
            Self::Trait => tokens.append_all(quote!(
                ::derive_builder::export::core::default::Default::default()
            )),
        }
    }
}
