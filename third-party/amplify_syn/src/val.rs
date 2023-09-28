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

use std::convert::TryFrom;
use std::fmt::{self, Debug, Formatter};

use proc_macro2::Span;
use quote::ToTokens;
use syn::parse::{Parse, Parser};
use syn::{
    Expr, Ident, Lit, LitBool, LitByteStr, LitChar, LitFloat, LitInt, LitStr, Path, PathSegment,
    Type, TypePath,
};

use crate::{Error, ValueClass};

/// Value for attribute or attribute argument, i.e. for `#[attr = value]` and
/// `#[attr(arg = value)]` this is the `value` part of the attribute. Can be
/// either a single literal or a single valid rust type name
#[derive(Clone)]
pub enum ArgValue {
    /// Attribute value represented by a literal
    Literal(Lit),

    /// Attribute value represented by a type name
    Type(Type),

    /// Attribute value represented by an expression
    Expr(Expr),

    /// No value is given
    None,
}

impl Debug for ArgValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ArgValue::Literal(lit) => write!(f, "ArgValue::Literal({})", lit.to_token_stream()),
            ArgValue::Type(ty) => write!(f, "ArgValue::Type({})", ty.to_token_stream()),
            ArgValue::Expr(expr) => write!(f, "ArgValue::Expr({})", expr.to_token_stream()),
            ArgValue::None => f.write_str("ArgValue::None"),
        }
    }
}

impl From<&str> for ArgValue {
    fn from(val: &str) -> Self { ArgValue::Literal(Lit::Str(LitStr::new(val, Span::call_site()))) }
}

impl From<String> for ArgValue {
    fn from(val: String) -> Self {
        ArgValue::Literal(Lit::Str(LitStr::new(&val, Span::call_site())))
    }
}

impl From<&[u8]> for ArgValue {
    fn from(val: &[u8]) -> Self {
        ArgValue::Literal(Lit::ByteStr(LitByteStr::new(val, Span::call_site())))
    }
}

impl From<Vec<u8>> for ArgValue {
    fn from(val: Vec<u8>) -> Self {
        ArgValue::Literal(Lit::ByteStr(LitByteStr::new(&val, Span::call_site())))
    }
}

impl From<char> for ArgValue {
    fn from(val: char) -> Self {
        ArgValue::Literal(Lit::Char(LitChar::new(val, Span::call_site())))
    }
}

impl From<usize> for ArgValue {
    fn from(val: usize) -> Self {
        ArgValue::Literal(Lit::Int(LitInt::new(&val.to_string(), Span::call_site())))
    }
}

impl From<isize> for ArgValue {
    fn from(val: isize) -> Self {
        ArgValue::Literal(Lit::Int(LitInt::new(&val.to_string(), Span::call_site())))
    }
}

impl From<f64> for ArgValue {
    fn from(val: f64) -> Self {
        ArgValue::Literal(Lit::Float(LitFloat::new(&val.to_string(), Span::call_site())))
    }
}

impl From<bool> for ArgValue {
    fn from(val: bool) -> Self {
        ArgValue::Literal(Lit::Bool(LitBool::new(val, Span::call_site())))
    }
}

impl From<Option<LitStr>> for ArgValue {
    fn from(val: Option<LitStr>) -> Self {
        match val {
            Some(val) => ArgValue::Literal(Lit::Str(val)),
            None => ArgValue::None,
        }
    }
}

impl From<Ident> for ArgValue {
    fn from(ident: Ident) -> Self {
        Path::from(PathSegment::parse.parse2(quote! { #ident }.into()).unwrap()).into()
    }
}

impl From<Path> for ArgValue {
    fn from(path: Path) -> Self { ArgValue::Type(Type::Path(TypePath { qself: None, path })) }
}

impl From<Option<LitByteStr>> for ArgValue {
    fn from(val: Option<LitByteStr>) -> Self {
        match val {
            Some(val) => ArgValue::Literal(Lit::ByteStr(val)),
            None => ArgValue::None,
        }
    }
}

impl From<Option<LitBool>> for ArgValue {
    fn from(val: Option<LitBool>) -> Self {
        match val {
            Some(val) => ArgValue::Literal(Lit::Bool(val)),
            None => ArgValue::None,
        }
    }
}

impl From<Option<LitChar>> for ArgValue {
    fn from(val: Option<LitChar>) -> Self {
        match val {
            Some(val) => ArgValue::Literal(Lit::Char(val)),
            None => ArgValue::None,
        }
    }
}

impl From<Option<LitInt>> for ArgValue {
    fn from(val: Option<LitInt>) -> Self {
        match val {
            Some(val) => ArgValue::Literal(Lit::Int(val)),
            None => ArgValue::None,
        }
    }
}

impl From<Option<LitFloat>> for ArgValue {
    fn from(val: Option<LitFloat>) -> Self {
        match val {
            Some(val) => ArgValue::Literal(Lit::Float(val)),
            None => ArgValue::None,
        }
    }
}

impl TryFrom<ArgValue> for String {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Str(s)) => Ok(s.value()),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for Vec<u8> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::ByteStr(s)) => Ok(s.value()),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for bool {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Bool(b)) => Ok(b.value),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for char {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Char(c)) => Ok(c.value()),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for LitStr {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Str(s)) => Ok(s),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for LitByteStr {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::ByteStr(s)) => Ok(s),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for LitBool {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Bool(s)) => Ok(s),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for LitChar {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Char(c)) => Ok(c),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for LitInt {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Int(i)) => Ok(i),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for LitFloat {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Float(f)) => Ok(f),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for Ident {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Type(Type::Path(ty)) => {
                if let Some(ident) = ty.path.get_ident() {
                    Ok(ident.clone())
                } else {
                    Err(Error::ArgValueMustBeType)
                }
            }
            _ => Err(Error::ArgValueMustBeType),
        }
    }
}

impl TryFrom<ArgValue> for Path {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Expr(expr) => Path::parse
                .parse2(expr.to_token_stream().into())
                .map_err(Error::from),
            ArgValue::Type(Type::Path(ty)) => Ok(ty.path),
            _ => Err(Error::ArgValueMustBeType),
        }
    }
}

impl TryFrom<ArgValue> for Expr {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(lit) => Expr::parse
                .parse2(lit.to_token_stream().into())
                .map_err(Error::from),
            ArgValue::Type(ty) => Expr::parse
                .parse2(ty.to_token_stream().into())
                .map_err(Error::from),
            ArgValue::Expr(expr) => Ok(expr),
            ArgValue::None => Err(Error::ArgValueMustBeExpr),
        }
    }
}

impl TryFrom<ArgValue> for Option<LitStr> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Str(s)) => Ok(Some(s)),
            ArgValue::None => Ok(None),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for Option<LitByteStr> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::ByteStr(s)) => Ok(Some(s)),
            ArgValue::None => Ok(None),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for Option<LitBool> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Bool(b)) => Ok(Some(b)),
            ArgValue::None => Ok(None),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for Option<LitChar> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Char(c)) => Ok(Some(c)),
            ArgValue::None => Ok(None),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for Option<LitInt> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Int(i)) => Ok(Some(i)),
            ArgValue::None => Ok(None),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for Option<LitFloat> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Literal(Lit::Float(f)) => Ok(Some(f)),
            ArgValue::None => Ok(None),
            _ => Err(Error::ArgValueMustBeLiteral),
        }
    }
}

impl TryFrom<ArgValue> for Option<Ident> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Type(Type::Path(ty)) => {
                if let Some(ident) = ty.path.get_ident() {
                    Ok(Some(ident.clone()))
                } else {
                    Err(Error::ArgValueMustBeType)
                }
            }
            ArgValue::None => Ok(None),
            _ => Err(Error::ArgValueMustBeType),
        }
    }
}

impl TryFrom<ArgValue> for Option<Path> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Type(Type::Path(ty)) => Ok(Some(ty.path)),
            ArgValue::Expr(expr) => Some(
                Path::parse
                    .parse2(expr.into_token_stream().into())
                    .map_err(Error::from),
            )
            .transpose(),
            ArgValue::None => Ok(None),
            _ => Err(Error::ArgValueMustBeType),
        }
    }
}

impl TryFrom<ArgValue> for Option<Expr> {
    type Error = Error;

    fn try_from(value: ArgValue) -> Result<Self, Self::Error> {
        match value {
            ArgValue::Expr(expr) => Ok(Some(expr)),
            ArgValue::Type(ty) => Some(
                Expr::parse
                    .parse2(ty.into_token_stream().into())
                    .map_err(Error::from),
            )
            .transpose(),
            ArgValue::Literal(lit) => Some(
                Expr::parse
                    .parse2(lit.into_token_stream().into())
                    .map_err(Error::from),
            )
            .transpose(),
            ArgValue::None => Ok(None),
        }
    }
}

impl ArgValue {
    /// Converts into literal value for [`ArgValue::Literal`] variant or fails
    /// with [`Error::ArgValueMustBeLiteral`] otherwise
    #[inline]
    pub fn into_literal_value(self) -> Result<Lit, Error> {
        match self {
            ArgValue::Literal(lit) => Ok(lit),
            ArgValue::Type(_) | ArgValue::Expr(_) | ArgValue::None => {
                Err(Error::ArgValueMustBeLiteral)
            }
        }
    }

    /// Converts into type value for [`ArgValue::Type`] variant or fails with
    /// [`Error::ArgValueMustBeType`] otherwise
    #[inline]
    pub fn into_type_value(self) -> Result<Type, Error> {
        match self {
            ArgValue::Literal(_) | ArgValue::Expr(_) | ArgValue::None => {
                Err(Error::ArgValueMustBeType)
            }
            ArgValue::Type(ty) => Ok(ty),
        }
    }

    /// Converts into type value for [`ArgValue::Expr`] variant or fails with
    /// [`Error::ArgValueMustBeExpr`] otherwise
    #[inline]
    pub fn into_type_expr(self) -> Result<Expr, Error> {
        match self {
            ArgValue::Literal(_) | ArgValue::Type(_) | ArgValue::None => {
                Err(Error::ArgValueMustBeExpr)
            }
            ArgValue::Expr(expr) => Ok(expr),
        }
    }

    /// Constructs literal value for [`ArgValue::Literal`] variant or fails with
    /// [`Error::ArgValueMustBeLiteral`] otherwise
    #[inline]
    pub fn to_literal_value(&self) -> Result<Lit, Error> {
        match self {
            ArgValue::Literal(lit) => Ok(lit.clone()),
            ArgValue::Type(_) | ArgValue::Expr(_) | ArgValue::None => {
                Err(Error::ArgValueMustBeLiteral)
            }
        }
    }

    /// Constructs type value for [`ArgValue::Type`] variant or fails with
    /// [`Error::ArgValueMustBeType`] otherwise
    #[inline]
    pub fn to_type_value(&self) -> Result<Type, Error> {
        match self {
            ArgValue::Literal(_) | ArgValue::Expr(_) | ArgValue::None => {
                Err(Error::ArgValueMustBeType)
            }
            ArgValue::Type(ty) => Ok(ty.clone()),
        }
    }

    /// Constructs type value for [`ArgValue::Expr`] variant or fails with
    /// [`Error::ArgValueMustBeExpr`] otherwise
    #[inline]
    pub fn to_type_expr(&self) -> Result<Expr, Error> {
        match self {
            ArgValue::Literal(_) | ArgValue::Type(_) | ArgValue::None => {
                Err(Error::ArgValueMustBeExpr)
            }
            ArgValue::Expr(expr) => Ok(expr.clone()),
        }
    }

    /// Returns a reference to a literal value for [`ArgValue::Literal`]
    /// variant, or `None` otherwise.
    #[inline]
    pub fn as_literal_value(&self) -> Option<&Lit> {
        match self {
            ArgValue::Literal(ref lit) => Some(lit),
            ArgValue::Type(_) | ArgValue::Expr(_) | ArgValue::None => None,
        }
    }

    /// Returns a reference to a literal value for  [`ArgValue::Type`]
    /// variant, or `None` otherwise.
    #[inline]
    pub fn as_type_value(&self) -> Option<&Type> {
        match self {
            ArgValue::Literal(_) | ArgValue::Expr(_) | ArgValue::None => None,
            ArgValue::Type(ref ty) => Some(ty),
        }
    }

    /// Returns a reference to a literal value for  [`ArgValue::Expr`]
    /// variant, or `None` otherwise.
    #[inline]
    pub fn as_type_expr(&self) -> Option<&Expr> {
        match self {
            ArgValue::Literal(_) | ArgValue::Type(_) | ArgValue::None => None,
            ArgValue::Expr(ref expr) => Some(expr),
        }
    }

    /// Tests whether the self is set to [`ArgValue::None`]
    #[inline]
    pub fn is_none(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        // Ancient rust versions do not known about `matches!` macro
        match self {
            ArgValue::None => true,
            _ => false,
        }
    }

    /// Tests whether the self is not set to [`ArgValue::None`]
    #[inline]
    pub fn is_some(&self) -> bool {
        // Ancient rust versions do not known about `matches!` macro
        #[allow(clippy::match_like_matches_macro)]
        match self {
            ArgValue::None => false,
            _ => true,
        }
    }

    /// Returns [`ValueClass`] for the current value, if any
    #[inline]
    pub fn value_class(&self) -> Option<ValueClass> {
        match self {
            ArgValue::Literal(lit) => Some(ValueClass::from(lit)),
            ArgValue::Type(ty) => Some(ValueClass::from(ty)),
            ArgValue::Expr(_) => Some(ValueClass::Expr),
            ArgValue::None => None,
        }
    }
}
