//! Declaration for the Keyword trait.

use crate::parse::rules;
use std::hash::Hash;

/// A Keyword identifies the possible types of a keyword for an Item.
///
/// These do not map one-to-one to Item strings: several Item strings
/// may be placed in a single Keyword -- for example, when their order
/// is significant with respect to one another, like "accept" and
/// "reject" in router descriptors.
///
/// Every keyword has an "index", which is a small number suitable for
/// indexing an array.  These are used in Section and SectionRules.
///
/// Turning a string into a keyword cannot fail: there is always an
/// "UNRECOGNIZED" keyword.
///
/// See macro::decl_keyword! for help defining a Keyword type for a
/// network document.
///
/// TODO: I'd rather have this be pub(crate), but I haven't figured out
/// how to make that work; there is a cascading change of other stuff that
/// would need to be more hidden.
pub trait Keyword: Hash + Eq + PartialEq + Copy + Clone {
    /// Find a Keyword corresponding to a string that appears in a
    /// network document.
    fn from_str(s: &str) -> Self;
    /// Try to find the keyword corresponding to a given index value,
    /// as used in Section and SectionRules.
    fn from_idx(i: usize) -> Option<Self>;
    /// Find a string corresponding to this keyword.  This may not be the
    /// actual string from the document; it is intended for reporting errors.
    fn to_str(self) -> &'static str;
    /// Return the index for this keyword.
    fn idx(self) -> usize;
    /// Return the number of indices for this keyword.
    fn n_vals() -> usize;
    /// Return the "UNRECOGNIZED" keyword.
    fn unrecognized() -> Self;
    /// Return the "ANN_UNRECOGNIZED" keyword.
    fn ann_unrecognized() -> Self;
    /// Return true iff this keyword denotes an annotation.
    fn is_annotation(self) -> bool;
    /// Convert from an index to a human-readable string.
    fn idx_to_str(i: usize) -> &'static str {
        Self::from_idx(i)
            .map(|x| x.to_str())
            .unwrap_or("<out of range>")
    }
    /// Return a new TokenFmtBuilder for creating rules about this keyword.
    fn rule(self) -> rules::TokenFmtBuilder<Self> {
        rules::TokenFmtBuilder::new(self)
    }
}
