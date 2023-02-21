//! Lists in builders
//!
//! Use [`define_list_builder_helper`] and [`define_list_builder_accessors`] together when
//! a configuration (or other struct with a builder)
//! wants to contain a `Vec` of config sub-entries.
//!
//! ### How to use these macros
//!
//!  * For each kind of list, define a `ThingList` type alias for the validated form,
//!    and call [`define_list_builder_helper`] to define a `ThingListBuilder` helper
//!    type.  (Different lists with the same Rust type, but which ought to have a different
//!    default, are different "kinds" and should each have a separately named type alias.)
//!
//!    (Or, alternatively, with a hand-written builder type, make the builder field be
//!    `Option<Vec<ElementBuilder>>`.)
//!
// An alternative design would be declare the field on `Outer` as `Vec<Thing>`, and to provide
// a `VecBuilder`.  But:
//
//  (i) the `.build()` method would have to be from a trait (because it would be `VecBuilder<Item>`
//  which would have to contain some `ItemBuilder`, and for the benefit of `VecBuilder::build()`).
//  Although derive_builder` does not provide that trait now, this problem is not insuperable,
//  but it would mean us inventing a `Buildable` trait and a macro to generate it, or forking
//  derive_builder further.
//
//  (ii) `VecBuilder<Item>::build()` would have to have the same default list for every
//  type Item (an empty list).  So places where the default list is not empty would need special
//  handling.  The special handling would look quite like what we have here.
//
//!  * For each struct field containing a list, in a struct deriving `Builder`,
//!    decorate the field with `#[builder(sub_builder, setter(custom))]`
//!    to (i) get `derive_builder` call the appropriate build method,
//!    (ii) suppress the `derive_builder`-generated setter.
//!
// `ThingLisgtBuiler` exixsts for two reasons:
//
//  * derive_builder wants to call simply `build` on the builder struct field, and will
//    generate code for attaching the field name to any error which occurs.  We could
//    override the per-field build expression, but it would be quite a lot of typing and
//    would recapitulate the field name three times.
//
//  * The field accessors (which must be generated by a different macro_rules macros, at least
//    unless we soup up derive_builder some more) might need to do defaulting, too.  if
//    the builder field is its own type, that can be a method on that type.
//
//!  * For each struct containing lists, call [`define_list_builder_accessors`]
//!    to define the accessor methods.
//!
//! ### Example - list of structs with builders
//!
//! ```
//! use derive_builder::Builder;
//! use serde::{Deserialize, Serialize};
//! use tor_config::{define_list_builder_helper, define_list_builder_accessors, ConfigBuildError};
//!
//! #[derive(Builder, Debug, Eq, PartialEq)]
//! #[builder(build_fn(error = "ConfigBuildError"))]
//! #[builder(derive(Debug, Serialize, Deserialize))]
//! pub struct Thing { value: i32 }
//!
//! #[derive(Builder, Debug, Eq, PartialEq)]
//! #[builder(build_fn(error = "ConfigBuildError"))]
//! #[builder(derive(Debug, Serialize, Deserialize))]
//! pub struct Outer {
//!     /// List of things, being built as part of the configuration
//!     #[builder(sub_builder, setter(custom))]
//!     things: ThingList,
//! }
//!
//! define_list_builder_accessors! {
//!     struct OuterBuilder {
//!         pub things: [ThingBuilder],
//!     }
//! }
//!
//! /// Type alias for use by list builder macrology
//! type ThingList = Vec<Thing>;
//!
//! define_list_builder_helper! {
//!     pub(crate) struct ThingListBuilder {
//!         pub(crate) things: [ThingBuilder],
//!     }
//!     built: ThingList = things;
//!     default = vec![];
//! }
//!
//! let mut builder = OuterBuilder::default();
//! builder.things().push(ThingBuilder::default().value(42).clone());
//! assert_eq!{ builder.build().unwrap().things, &[Thing { value: 42 }] }
//!
//! builder.set_things(vec![ThingBuilder::default().value(38).clone()]);
//! assert_eq!{ builder.build().unwrap().things, &[Thing { value: 38 }] }
//! ```
//!
//! ### Example - list of trivial values
//!
//! ```
//! use derive_builder::Builder;
//! use serde::{Deserialize, Serialize};
//! use tor_config::{define_list_builder_helper, define_list_builder_accessors, ConfigBuildError};
//!
//! #[derive(Builder, Debug, Eq, PartialEq)]
//! #[builder(build_fn(error = "ConfigBuildError"))]
//! #[builder(derive(Debug, Serialize, Deserialize))]
//! pub struct Outer {
//!     /// List of values, being built as part of the configuration
//!     #[builder(sub_builder, setter(custom))]
//!     values: ValueList,
//! }
//!
//! define_list_builder_accessors! {
//!    struct OuterBuilder {
//!        pub values: [u32],
//!    }
//! }
//!
//! /// Type alias for use by list builder macrology
//! pub type ValueList = Vec<u32>;
//!
//! define_list_builder_helper! {
//!    pub(crate) struct ValueListBuilder {
//!        pub(crate) values: [u32],
//!    }
//!    built: ValueList = values;
//!    default = vec![27];
//!    item_build: |&value| Ok(value);
//! }
//!
//! let mut builder = OuterBuilder::default();
//! assert_eq!{ builder.build().unwrap().values, &[27] }
//!
//! builder.values().push(12);
//! assert_eq!{ builder.build().unwrap().values, &[27, 12] }
//! ```

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use educe::Educe;
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

pub use crate::define_list_builder_accessors;
pub use crate::define_list_builder_helper;

/// Define a list builder struct for use with [`define_list_builder_accessors`]
///
/// Generates an builder struct that can be used with derive_builder
/// and [`define_list_builder_accessors`] to configure a list of some kind.
///
/// **See the [`list_builder` module documentation](crate::list_builder) for an overview.**
///
/// ### Generated struct
///
/// This macro-generated builder struct contains `Option<Vec<ThingBuilder>>`, to allow it to
/// distinguish "never set" from "has been adjusted or set, possibly to the empty list".
///
/// This struct is not exposed as part of the API for setting the configuration.
/// Generally the visibility (`$vis`) should be private,
/// but sometimes `pub(crate)` or `pub` is necessary,
/// for example if the list is to be included in a struct in another module or crate.
/// Usually `$field_vis` should be the same as `$vis`.
///
/// `#[derive(Default, Clone, Debug, Serialize, Deserialize)]`
///  will be applied to the generated builder,
/// but you can specify other attributes too.
/// There is no need to supply any documentation; this is an internal struct and
/// the macro will supply a suitable (bland) doc comment.
/// (If you do supply documentation, the autogenerated docs will be appended,
/// so start with a summary line.)
/// Documentation for the semantics and default value should be applied
/// to the field(s) in the containing struct(s).
///
/// `#[serde(transparent)]` will be applied to the generated `ThingBuilder` struct,
/// so that it deserializes just like `Option<Vec<Thing>>`.
///
/// ### Input to the macro
///
/// For the input syntax, refer to the docs autogenerated from the macro's matcher.
///
/// The `built` clause specifies the type of the built value, and how to construct it.
/// In the expression part, `things` (the field name) will be the default-resolved `Vec<Thing>`;
/// it should be consumed by the expression.
/// If the built value is simply a `Vec`, you can just write `built: ThingList = things;`.
///
/// The `default` clause must provide an expression evaluating to a `Vec<ThingBuilder>`.
///
/// The `item_build` clause, if supplied, provides a closure with type
/// `FnMut(&ThingBuilder) -> Result<Thing, ConfigBuildError>`;
/// the default is to call `thing_builder.build()`.
///
/// The `#[ serde $serde_attrs:tt ]`, if supplied, replace the serde attribute
/// `#[serde(transparent)]`.
/// The transparent attribute is applied by default
/// to arrange that the serde view of the list is precisely `Option<Vec>`.
/// If serialisation is done another way, for example with `#[serde(into)]`,
/// that must be specified here.
///
/// `[$generics]` are generics for `$ListBuilder`.
/// Inline bounds (`T: Debug`) are not supported; use a `where` clause instead.
/// Due to limitations of `macro_rules`, the parameters must be within `[ ]` rather than `< >`,
/// and an extraneous pair of `[ ]` must appear around any `$where_clauses`.
//
// This difficulty with macro_rules is not well documented.
// The upstream Rust bug tracker has this issue
//   https://github.com/rust-lang/rust/issues/73174
//   Matching function signature is nearly impossible in declarative macros (mbe)
// which is not precisely this problem but is very nearby.
// There's also the vapourware "declarative macros 2.0"
//   https://github.com/rust-lang/rust/issues/39412
#[macro_export]
macro_rules! define_list_builder_helper {
    {
        $(#[ $docs_and_attrs:meta ])*
        $vis:vis
        struct $ListBuilder:ident $( [ $($generics:tt)* ] )?
        $( where [ $($where_clauses:tt)* ] )?
        {
            $field_vis:vis $things:ident : [$EntryBuilder:ty] $(,)?
        }
        built: $Built:ty = $built:expr;
        default = $default:expr;
        $( item_build: $item_build:expr; )?
        $(#[ serde $serde_attrs:tt ] )+
    } => {
        #[derive($crate::educe::Educe, Clone, Debug)]
        #[derive($crate::serde::Serialize, $crate::serde::Deserialize)]
        #[educe(Default)]
        $(#[ serde $serde_attrs ])+
        $(#[ $docs_and_attrs ])*
        /// Wrapper struct to help derive_builder find the right types and methods
        ///
        /// This struct is not part of the configuration API.
        /// Refer to the containing structures for information on how to build the config.
        $vis struct $ListBuilder $( < $($generics)* > )?
        $( where $($where_clauses)* )?
        {
            /// The list, as overridden
            $field_vis $things: Option<Vec<$EntryBuilder>>,
        }

        impl $( < $($generics)* > )? $ListBuilder $( < $($generics)* > )?
        $( where $($where_clauses)* )?
        {
            /// Resolve this list to a list of built items.
            ///
            /// If the value is still the [`Default`],
            /// a built-in default list will be built and returned;
            /// otherwise each applicable item will be built,
            /// and the results collected into a single built list.
            $vis fn build(&self) -> Result<$Built, $crate::ConfigBuildError> {
                let default_buffer;
                let $things = match &self.$things {
                    Some($things) => $things,
                    None => {
                        default_buffer = Self::default_list();
                        &default_buffer
                    }
                };

                let $things = $things
                    .iter()
                    .map(
                        $crate::macro_first_nonempty!{
                            [ $( $item_build )? ],
                            [ |item| item.build() ],
                        }
                    )
                    .collect::<Result<_, $crate::ConfigBuildError>>()?;
                Ok($built)
            }

            /// The default list
            fn default_list() -> Vec<$EntryBuilder> {
                 $default
            }

            /// Resolve the list to the default if necessary and then return `&mut Vec`
            $vis fn access(&mut self) -> &mut Vec<$EntryBuilder> {
                self.$things.get_or_insert_with(Self::default_list)
            }

            /// Resolve the list to the default if necessary and then return `&mut Vec`
            $vis fn access_opt(&self) -> &Option<Vec<$EntryBuilder>> {
                &self.$things
            }

            /// Resolve the list to the default if necessary and then return `&mut Vec`
            $vis fn access_opt_mut(&mut self) -> &mut Option<Vec<$EntryBuilder>> {
                &mut self.$things
            }
        }
    };

    // Expand the version without `#[ serde $serde_attrs ]` into a call
    // which provides `#[serde(transparent)]`.
    //
    // We can't use `macro_first_nonempty!` because macro calls cannot be invoked
    // to generate attributes, only items, expressions, etc.
    {
        $(#[ $docs_and_attrs:meta ])*
        $vis:vis
        struct $ListBuilder:ident $( [ $($generics:tt)* ] )?
        $( where [ $($where_clauses:tt)* ] )?
        {
            $field_vis:vis $things:ident : [$EntryBuilder:ty] $(,)?
        }
        built: $Built:ty = $built:expr;
        default = $default:expr;
        $( item_build: $item_build:expr; )?
    } => {
        define_list_builder_helper! {
            $(#[ $docs_and_attrs ])*
            $vis
            struct $ListBuilder $( [ $($generics)* ] )?
            $( where [ $($where_clauses)* ] )?
            {
                $field_vis $things : [$EntryBuilder],
            }
            built: $Built = $built;
            default = $default;
            $( item_build: $item_build; )?
            #[serde(transparent)]
        }
    };
}

/// Define accessor methods for a configuration item which is a list
///
/// **See the [`list_builder` module documentation](crate::list_builder) for an overview.**
///
/// Generates the following methods for each specified field:
///
/// ```skip
/// impl $OuterBuilder {
///     pub fn $things(&mut self) -> &mut Vec<$EntryBuilder> { .. }
///     pub fn set_$things(&mut self, list: Vec<$EntryBuilder>) { .. }
///     pub fn opt_$things(&self) -> &Option<Vec<$EntryBuilder>> { .. }
///     pub fn opt_$things_mut>](&mut self) -> &mut Option<Vec<$EntryBuilder>> { .. }
/// }
/// ```
///
/// Each `$EntryBuilder` should have been defined by [`define_list_builder_helper`];
/// the method bodies from this macro rely on facilities which will beprovided by that macro.
///
/// You can call `define_list_builder_accessors` once for a particular `$OuterBuilder`,
/// with any number of fields with possibly different entry (`$EntryBuilder`) types.
#[macro_export]
macro_rules! define_list_builder_accessors {
    {
        struct $OuterBuilder:ty {
            $(
                $vis:vis $things:ident: [$EntryBuilder:ty],
            )*
        }
    } => {
        impl $OuterBuilder { $( $crate::paste!{
            /// Access the being-built list (resolving default)
            ///
            /// If the field has not yet been set or accessed, the default list will be
            /// constructed and a mutable reference to the now-defaulted list of builders
            /// will be returned.
            $vis fn $things(&mut self) -> &mut Vec<$EntryBuilder> {
                #[allow(unused_imports)]
                use $crate::list_builder::DirectDefaultEmptyListBuilderAccessors as _;
                self.$things.access()
            }

            /// Set the whole list (overriding the default)
            $vis fn [<set_ $things>](&mut self, list: Vec<$EntryBuilder>) {
                #[allow(unused_imports)]
                use $crate::list_builder::DirectDefaultEmptyListBuilderAccessors as _;
                *self.$things.access_opt_mut() = Some(list)
            }

            /// Inspect the being-built list (with default unresolved)
            ///
            /// If the list has not yet been set, or accessed, `&None` is returned.
            $vis fn [<opt_ $things>](&self) -> &Option<Vec<$EntryBuilder>> {
                #[allow(unused_imports)]
                use $crate::list_builder::DirectDefaultEmptyListBuilderAccessors as _;
                self.$things.access_opt()
            }

            /// Mutably access the being-built list (with default unresolved)
            ///
            /// If the list has not yet been set, or accessed, `&mut None` is returned.
            $vis fn [<opt_ $things _mut>](&mut self) -> &mut Option<Vec<$EntryBuilder>> {
                #[allow(unused_imports)]
                use $crate::list_builder::DirectDefaultEmptyListBuilderAccessors as _;
                self.$things.access_opt_mut()
            }
        } )* }
    }
}

/// Extension trait, an alternative to `define_list_builder_helper`
///
/// Useful for a handwritten `Builder` which wants to contain a list,
/// which is an `Option<Vec<ItemBuilder>>`.
///
/// # Example
///
/// ```
/// use tor_config::define_list_builder_accessors;
///
/// #[derive(Default)]
/// struct WombatBuilder {
///     leg_lengths: Option<Vec<u32>>,
/// }
///
/// define_list_builder_accessors! {
///     struct WombatBuilder {
///         leg_lengths: [u32],
///     }
/// }
///
/// let mut wb = WombatBuilder::default();
/// wb.leg_lengths().push(42);
///
/// assert_eq!(wb.leg_lengths, Some(vec![42]));
/// ```
///
/// It is not necessary to `use` this trait anywhere in your code;
/// the macro `define_list_builder_accessors` arranges to have it in scope where it needs it.
pub trait DirectDefaultEmptyListBuilderAccessors {
    /// Entry type
    type T;
    /// Get access to the `Vec`, defaulting it
    fn access(&mut self) -> &mut Vec<Self::T>;
    /// Get access to the `Option<Vec>`
    fn access_opt(&self) -> &Option<Vec<Self::T>>;
    /// Get mutable access to the `Option<Vec>`
    fn access_opt_mut(&mut self) -> &mut Option<Vec<Self::T>>;
}
impl<T> DirectDefaultEmptyListBuilderAccessors for Option<Vec<T>> {
    type T = T;
    fn access(&mut self) -> &mut Vec<T> {
        self.get_or_insert_with(Vec::new)
    }
    fn access_opt(&self) -> &Option<Vec<T>> {
        self
    }
    fn access_opt_mut(&mut self) -> &mut Option<Vec<T>> {
        self
    }
}

define_list_builder_helper! {
    /// List of `T`, a straightforward type, being built as part of the configuration
    ///
    /// The default is the empty list.
    ///
    /// ### Example
    ///
    /// ```
    /// use derive_builder::Builder;
    /// use serde::{Deserialize, Serialize};
    /// use tor_config::ConfigBuildError;
    /// use tor_config::{define_list_builder_accessors, list_builder::VecBuilder};
    /// use std::net::SocketAddr;
    ///
    /// #[derive(Debug, Clone, Builder)]
    /// #[builder(build_fn(error = "ConfigBuildError"))]
    /// #[builder(derive(Debug, Serialize, Deserialize))]
    /// pub struct FallbackDir {
    ///     #[builder(sub_builder(fn_name = "build"), setter(custom))]
    ///     orports: Vec<SocketAddr>,
    /// }
    ///
    /// define_list_builder_accessors! {
    ///     struct FallbackDirBuilder {
    ///         pub orports: [SocketAddr],
    ///     }
    /// }
    ///
    /// let mut bld = FallbackDirBuilder::default();
    /// bld.orports().push("[2001:db8:0::42]:12".parse().unwrap());
    /// assert_eq!( bld.build().unwrap().orports[0].to_string(),
    ///             "[2001:db8::42]:12" );
    /// ```
    pub struct VecBuilder[T] where [T: Clone] {
        values: [T],
    }
    built: Vec<T> = values;
    default = vec![];
    item_build: |item| Ok(item.clone());
}

/// Configuration item specifiable as a list, or a single multi-line string
///
/// If a list is supplied, they are deserialized as builders.
/// If a single string is supplied, it is split into lines, and `#`-comments
/// and blank lines and whitespace are stripped, and then each line is parsed
/// as a builder.
/// (Eventually, the builders will be built.)
///
/// For use with `sub_builder` and [`define_list_builder_helper`],
/// with `#[serde(try_from)]` and `#[serde(into)]`.
///
/// # Example
///
/// ```
/// use derive_builder::Builder;
/// use serde::{Deserialize, Serialize};
/// use tor_config::{ConfigBuildError, MultilineListBuilder};
/// use tor_config::convert_helper_via_multi_line_list_builder;
/// use tor_config::{define_list_builder_accessors, define_list_builder_helper};
/// use tor_config::impl_standard_builder;
///
/// # fn generate_random<T: Default>() -> T { Default::default() }
///
/// #[derive(Debug, Clone, Builder, Eq, PartialEq)]
/// #[builder(build_fn(error = "ConfigBuildError"))]
/// #[builder(derive(Debug, Serialize, Deserialize))]
/// #[non_exhaustive]
/// pub struct LotteryConfig {
///     /// What numbers should win the lottery?  Setting this is lottery fraud.
///     #[builder(sub_builder, setter(custom))]
///     #[builder_field_attr(serde(default))]
///     winners: LotteryNumberList,
/// }
/// impl_standard_builder! { LotteryConfig }
///
/// /// List of lottery winners
/// //
/// // This type alias arranges that we can put `LotteryNumberList` in `LotteryConfig`
/// // and have derive_builder put a `LotteryNumberListBuilder` in `LotteryConfigBuilder`.
/// pub type LotteryNumberList = Vec<u16>;
///
/// define_list_builder_helper! {
///     struct LotteryNumberListBuilder {
///         numbers: [u16],
///     }
///     built: LotteryNumberList = numbers;
///     default = generate_random();
///     item_build: |number| Ok(*number);
///     #[serde(try_from="MultilineListBuilder<u16>")]
///     #[serde(into="MultilineListBuilder<u16>")]
/// }
///
/// convert_helper_via_multi_line_list_builder! {
///     struct LotteryNumberListBuilder {
///         numbers: [u16],
///     }
/// }
///
/// define_list_builder_accessors! {
///     struct LotteryConfigBuilder {
///         pub winners: [u16],
///     }
/// }
///
/// let lc: LotteryConfigBuilder = toml::from_str(r#"winners = [1,2,3]"#).unwrap();
/// let lc = lc.build().unwrap();
/// assert_eq!{ lc.winners, [1,2,3] }
///
/// let lc = r#"
/// winners = '''
///   ## Enny tells us this is the ticket they bought:
///
///   4
///   5
///   6
/// '''
/// "#;
/// let lc: LotteryConfigBuilder = toml::from_str(lc).unwrap();
/// let lc = lc.build().unwrap();
/// assert_eq!{ lc.winners, [4,5,6] }
/// ```
#[derive(Clone, Debug, Educe, Serialize)]
#[serde(untagged)]
#[educe(Default)]
#[non_exhaustive]
pub enum MultilineListBuilder<EB> {
    /// Config key not present
    #[educe(Default)]
    Unspecified,

    /// Config key was a string which is to be parsed line-by-line
    String(String),

    /// Config key was a list of the individual entry builders
    List(Vec<EB>),
}

/// Error from trying to parse a MultilineListBuilder as a list of particular items
///
/// Usually, this error is generated during deserialization.
#[derive(Error, Debug, Clone)]
#[error("multi-line string, line/item {item_number}: could not parse {line:?}: {error}")]
#[non_exhaustive]
pub struct MultilineListBuilderError<E: std::error::Error + Clone + Send + Sync> {
    /// The line number (in the multi-line text string) that could not be parsed
    ///
    /// Starting at 1.
    item_number: usize,

    /// The line that could not be parsed
    line: String,

    /// The parse error from `FromStr`
    ///
    /// This is not a `source` because we want to include it in the `Display`
    /// implementation so that serde errors are useful.
    error: E,
}

// We could derive this with `#[serde(untagged)]` but that produces quite terrible error
// messages, which do not reproduce the error messages from any of the variants.
//
// Instead, have a manual implementation, which can see whether the input is a list or a string.
impl<'de, EB: Deserialize<'de>> Deserialize<'de> for MultilineListBuilder<EB> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(MllbVisitor::default())
    }
}

/// Visitor for deserialize_any for [`MultilineListBuilder`]
#[derive(Educe)]
#[educe(Default)]
struct MllbVisitor<EB> {
    /// Variance: this visitor constructs `EB`s
    ret: PhantomData<fn() -> EB>,
}

impl<'de, EB: Deserialize<'de>> serde::de::Visitor<'de> for MllbVisitor<EB> {
    type Value = MultilineListBuilder<EB>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "list of items, or multi-line string")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut v = vec![];
        while let Some(e) = seq.next_element()? {
            v.push(e);
        }
        Ok(MultilineListBuilder::List(v))
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_string(v.to_owned())
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(MultilineListBuilder::String(v))
    }

    fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(MultilineListBuilder::Unspecified)
    }
}

impl<EB> From<Option<Vec<EB>>> for MultilineListBuilder<EB> {
    fn from(list: Option<Vec<EB>>) -> Self {
        use MultilineListBuilder as MlLB;
        match list {
            None => MlLB::Unspecified,
            Some(list) => MlLB::List(list),
        }
    }
}

impl<EB> TryInto<Option<Vec<EB>>> for MultilineListBuilder<EB>
where
    EB: FromStr,
    EB::Err: std::error::Error + Clone + Send + Sync,
{
    type Error = MultilineListBuilderError<EB::Err>;
    fn try_into(self) -> Result<Option<Vec<EB>>, Self::Error> {
        use MultilineListBuilder as MlLB;

        /// Helper for parsing each line of `iter` and collecting the results
        fn parse_collect<'s, I>(
            iter: impl Iterator<Item = (usize, &'s str)>,
        ) -> Result<Option<Vec<I>>, MultilineListBuilderError<I::Err>>
        where
            I: FromStr,
            I::Err: std::error::Error + Clone + Send + Sync,
        {
            Ok(Some(
                iter.map(|(i, l)| {
                    l.parse().map_err(|error| MultilineListBuilderError {
                        item_number: i + 1,
                        line: l.to_owned(),
                        error,
                    })
                })
                .try_collect()?,
            ))
        }

        Ok(match self {
            MlLB::Unspecified => None,
            MlLB::List(list) => Some(list),
            MlLB::String(s) => parse_collect(
                s.lines()
                    .enumerate()
                    .map(|(i, l)| (i, l.trim()))
                    .filter(|(_, l)| !(l.starts_with('#') || l.is_empty())),
            )?,
        })
    }
}

/// Implement `TryFrom<MultilineListBuilder>` and `Into<MultilineListBuilder>` for $Builder.
///
/// The input syntax is the `struct` part of that for `define_list_builder_helper`.
/// `$EntryBuilder` must implement `FromStr`.
//
// This is a macro because a helper trait to enable blanket impl would have to provide
// access to `$things`, defeating much of the point.
#[macro_export]
macro_rules! convert_helper_via_multi_line_list_builder { {
    struct $ListBuilder:ident { $things:ident: [$EntryBuilder:ty] $(,)? }
} => {
    impl std::convert::TryFrom<$crate::MultilineListBuilder<$EntryBuilder>> for $ListBuilder {
        type Error = $crate::MultilineListBuilderError<<$EntryBuilder as std::str::FromStr>::Err>;

        fn try_from(mllb: $crate::MultilineListBuilder<$EntryBuilder>)
                    -> std::result::Result<$ListBuilder, Self::Error> {
            Ok($ListBuilder { $things: mllb.try_into()? })
        }
    }

    impl From<$ListBuilder> for MultilineListBuilder<$EntryBuilder> {
        fn from(lb: $ListBuilder) -> MultilineListBuilder<$EntryBuilder> {
            lb.$things.into()
        }
    }
} }

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_duration_subtraction)]
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->
    use super::*;
    use derive_builder::Builder;

    #[derive(Eq, PartialEq, Builder)]
    #[builder(derive(Deserialize))]
    struct Outer {
        #[builder(sub_builder, setter(custom))]
        list: List,
    }

    define_list_builder_accessors! {
        struct OuterBuilder {
            list: [char],
        }
    }

    type List = Vec<char>;

    define_list_builder_helper! {
        struct ListBuilder {
            list: [char],
        }
        built: List = list;
        default = vec!['a'];
        item_build: |&c| Ok(c);
    }

    #[test]
    fn nonempty_default() {
        let mut b = OuterBuilder::default();
        assert!(b.opt_list().is_none());
        assert_eq! { b.build().expect("build failed").list, ['a'] };

        b.list().push('b');
        assert!(b.opt_list().is_some());
        assert_eq! { b.build().expect("build failed").list, ['a', 'b'] };

        for mut b in [b.clone(), OuterBuilder::default()] {
            b.set_list(vec!['x', 'y']);
            assert!(b.opt_list().is_some());
            assert_eq! { b.build().expect("build failed").list, ['x', 'y'] };
        }

        *b.opt_list_mut() = None;
        assert_eq! { b.build().expect("build failed").list, ['a'] };
    }

    #[test]
    fn vecbuilder() {
        // Minimal test, since rustdoc tests seem not to be finding the documentation inside
        // the declaration of VecBuilder.  (Or at least that's what the coverage says.)
        let mut b = VecBuilder::<u32>::default();
        b.access().push(1);
        b.access().push(2);
        b.access().push(3);
        assert_eq!(b.build().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn deser() {
        let o: OuterBuilder = toml::from_str("list = ['x','y']").unwrap();
        let o = o.build().unwrap();
        assert_eq!(o.list, ['x', 'y']);

        #[derive(Deserialize, Debug)]
        struct OuterWithMllb {
            #[serde(default)]
            list: MultilineListBuilder<u32>,
        }

        let parse_ok = |s: &str| {
            let o: OuterWithMllb = toml::from_str(s).unwrap();
            let l: Option<Vec<_>> = o.list.try_into().unwrap();
            l
        };

        let l = parse_ok("");
        assert!(l.is_none());

        let l = parse_ok("list = []");
        assert!(l.unwrap().is_empty());

        let l = parse_ok("list = [12,42]");
        assert_eq!(l.unwrap(), [12, 42]);

        let l = parse_ok(r#"list = """#);
        assert!(l.unwrap().is_empty());

        let l = parse_ok("list = \"\"\"\n12\n42\n\"\"\"\n");
        assert_eq!(l.unwrap(), [12, 42]);

        let e = toml::from_str::<OuterWithMllb>("list = [\"fail\"]")
            .unwrap_err()
            .to_string();
        assert!(dbg!(e).contains(r#"invalid type: string "fail", expected u32"#));

        let o = toml::from_str::<OuterWithMllb>("list = \"\"\"\nfail\n\"\"\"").unwrap();
        let l: Result<Option<Vec<_>>, _> = o.list.try_into();
        let e = l.unwrap_err().to_string();
        assert_eq!(e, "multi-line string, line/item 1: could not parse \"fail\": invalid digit found in string");
    }
}
