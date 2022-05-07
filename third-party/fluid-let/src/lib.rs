// Copyright (c) 2019, ilammy
// Licensed under MIT license (see LICENSE)

//! Dynamically scoped variables.
//!
//! _Dynamic_ or _fluid_ variables are a handy way to define global configuration values.
//! They come from the Lisp family of languages where they are relatively popular in this role.
//!
//! # Declaring dynamic variables
//!
//! [`fluid_let!`] macro is used to declare dynamic variables. Dynamic variables
//! are _global_, therefore they must be declared as `static`:
//!
//! [`fluid_let!`]: macro.fluid_let.html
//!
//! ```
//! use std::fs::File;
//!
//! use fluid_let::fluid_let;
//!
//! fluid_let!(static LOG_FILE: File);
//! ```
//!
//! The actual type of `LOG_FILE` variable is `Option<&File>`: that is,
//! possibly absent reference to a file. All dynamic variables have `None` as
//! their default value, unless a particular value is set for them.
//!
//! If you enable the [`"static-init"` feature](#features), it is also
//! possible to provide `'static` initialization for types that allow it:
//!
//! ```no_run
//! # use fluid_let::fluid_let;
//! #
//! # enum LogLevel { Info }
//! #
//! # #[cfg(feature = "static-init")]
//! fluid_let!(static LOG_LEVEL: LogLevel = LogLevel::Info);
//! ```
//!
//! Here `LOG_LEVEL` has `Some(&LogLevel::Info)` as its default value.
//!
//! # Setting dynamic variables
//!
//! [`set`] is used to give value to a dynamic variable:
//!
//! [`set`]: struct.DynamicVariable.html#method.set
//!
//! ```no_run
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: File);
//! #
//! # fn open(path: &str) -> File { unimplemented!() }
//! #
//! let log_file: File = open("/tmp/log.txt");
//!
//! LOG_FILE.set(&log_file, || {
//!     //
//!     // logs will be redirected to /tmp/log.txt in this block
//!     //
//! });
//! ```
//!
//! Note that you store an _immutable reference_ in the dynamic variable.
//! You can’t directly modify the dynamic variable value after setting it,
//! but you can use something like `Cell` or `RefCell` to circumvent that.
//!
//! The new value is in effect within the _dynamic extent_ of the assignment,
//! that is within the closure passed to `set`. Once the closure returns, the
//! previous value of the variable is restored.
//!
//! If you do not need precise control over the extent of the assignment, you
//! can use the [`fluid_set!`] macro to assign until the end of the scope:
//!
//! [`fluid_set!`]: macro.fluid_set.html
//!
//! ```no_run
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: File);
//! #
//! # fn open(path: &str) -> File { unimplemented!() }
//! #
//! use fluid_let::fluid_set;
//!
//! fn chatterbox_function() {
//!     fluid_set!(LOG_FILE, open("/dev/null"));
//!     //
//!     // logs will be written to /dev/null in this function
//!     //
//! }
//! ```
//!
//! Obviously, you can also nest assignments arbitrarily:
//!
//! ```no_run
//! # use std::fs::File;
//! #
//! # use fluid_let::{fluid_let, fluid_set};
//! #
//! # fluid_let!(static LOG_FILE: File);
//! #
//! # fn open(path: &str) -> File { unimplemented!() }
//! #
//! LOG_FILE.set(open("A.txt"), || {
//!     // log to A.txt here
//!     LOG_FILE.set(open("/dev/null"), || {
//!         // log to /dev/null for a bit
//!         fluid_set!(LOG_FILE, open("B.txt"));
//!         // log to B.txt starting with this line
//!         {
//!             fluid_set!(LOG_FILE, open("C.txt"));
//!             // but in this block log to C.txt
//!         }
//!         // before going back to using B.txt here
//!     });
//!     // and logging to A.txt again
//! });
//! ```
//!
//! # Accessing dynamic variables
//!
//! [`get`] is used to retrieve the current value of a dynamic variable:
//!
//! [`get`]: struct.DynamicVariable.html#method.get
//!
//! ```no_run
//! # use std::io::{self, Write};
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: File);
//! #
//! fn write_log(msg: &str) -> io::Result<()> {
//!     LOG_FILE.get(|current| {
//!         if let Some(mut log_file) = current {
//!             write!(log_file, "{}\n", msg)?;
//!         }
//!         Ok(())
//!     })
//! }
//! ```
//!
//! Current value of the dynamic variable is passed to the provided closure, and
//! the value returned by the closure becomes the value of the `get()` call.
//!
//! This somewhat weird access interface is dictated by safety requirements. The
//! dynamic variable itself is global and thus has `'static` lifetime. However,
//! its values usually have shorter lifetimes, as short as the corresponing
//! `set()` call. Therefore, access reference must have _even shorter_ lifetime.
//!
//! If the variable type implements `Clone` or `Copy` then you can use [`cloned`]
//! and [`copied`] convenience accessors to get a copy of the current value:
//!
//! [`cloned`]: struct.DynamicVariable.html#method.cloned
//! [`copied`]: struct.DynamicVariable.html#method.copied
//!
//! ```no_run
//! # use std::io::{self, Write};
//! # use std::fs::File;
//! #
//! # use fluid_let::fluid_let;
//! #
//! # fluid_let!(static LOG_FILE: File);
//! #
//! #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! enum LogLevel {
//!     Debug,
//!     Info,
//!     Error,
//! }
//!
//! # #[cfg(not(feature = "fluid-let"))]
//! # fluid_let!(static LOG_LEVEL: LogLevel);
//! # #[cfg(feature = "fluid-let")]
//! fluid_let!(static LOG_LEVEL: LogLevel = LogLevel::Info);
//!
//! fn write_log(level: LogLevel, msg: &str) -> io::Result<()> {
//!     if level < LOG_LEVEL.copied().unwrap() {
//!         return Ok(());
//!     }
//!     LOG_FILE.get(|current| {
//!         if let Some(mut log_file) = current {
//!             write!(log_file, "{}\n", msg)?;
//!         }
//!         Ok(())
//!     })
//! }
//! ```
//!
//! # Thread safety
//!
//! Dynamic variables are global and _thread-local_. That is, each thread gets
//! its own independent instance of a dynamic variable. Values set in one thread
//! are visible only in this thread. Other threads will not see any changes in
//! values of their dynamic variables and may have different configurations.
//!
//! Note, however, that this does not free you from the usual synchronization
//! concerns when shared objects are involved. Dynamic variables hold _references_
//! to objects. Therefore it is entirely possible to bind _the same_ object with
//! internal mutability to a dynamic variable and access it from multiple threads.
//! In this case you will probably need some synchronization to use the shared
//! object in a safe manner, just like you would do when using `Arc` and friends.
//!
//! # Features
//!
//! Currently, there is only one optional feature: `"static-init"`,
//! gating static initialization of dynamic variables:
//!
//! ```
//! # use fluid_let::fluid_let;
//! #
//! # enum LogLevel { Info }
//! #
//! # #[cfg(feature = "static-init")]
//! fluid_let!(static LOG_LEVEL: LogLevel = LogLevel::Info);
//! //                                    ~~~~~~~~~~~~~~~~
//! ```
//!
//! The API for accessing known-initialized variables has not stabilized yet
//! and may be subject to changes.

use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::mem;
use std::thread::LocalKey;

#[cfg(feature = "static-init")]
/// Declares global dynamic variables.
///
/// # Examples
///
/// One-line form for single declarations:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let!(static ENABLED: bool);
/// ```
///
/// If [`"static-init"` feature](index.html#features) is enabled,
/// you can provide initial value:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let!(static ENABLED: bool = true);
/// ```
///
/// Multiple declarations with attributes and visibility modifiers are also supported:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let! {
///     /// Length of `Debug` representation of hashes in characters.
///     pub static HASH_LENGTH: usize = 32;
///
///     /// If set to true then passwords will be printed to logs.
///     #[cfg(test)]
///     static DUMP_PASSWORDS: bool;
/// }
/// ```
///
/// See also [crate-level documentation](index.html) for usage examples.
#[macro_export]
macro_rules! fluid_let {
    // Simple case: a single definition with None value.
    {
        $(#[$attr:meta])*
        $pub:vis static $name:ident: $type:ty
    } => {
        $(#[$attr])*
        $pub static $name: $crate::DynamicVariable<$type> = {
            thread_local! {
                static VARIABLE: $crate::DynamicCell<$type> = $crate::DynamicCell::empty();
            }
            $crate::DynamicVariable::new(&VARIABLE)
        };
    };
    // Simple case: a single definition with Some value.
    {
        $(#[$attr:meta])*
        $pub:vis static $name:ident: $type:ty = $value:expr
    } => {
        $(#[$attr])*
        $pub static $name: $crate::DynamicVariable<$type> = {
            static DEFAULT: $type = $value;
            thread_local! {
                static VARIABLE: $crate::DynamicCell<$type> = $crate::DynamicCell::with_static(&DEFAULT);
            }
            $crate::DynamicVariable::new(&VARIABLE)
        };
    };
    // Multiple definitions (iteration), with None value.
    {
        $(#[$attr:meta])*
        $pub:vis static $name:ident: $type:ty;
        $($rest:tt)*
    } => {
        $crate::fluid_let!($(#[$attr])* $pub static $name: $type);
        $crate::fluid_let!($($rest)*);
    };
    // Multiple definitions (iteration), with Some value.
    {
        $(#[$attr:meta])*
        $pub:vis static $name:ident: $type:ty = $value:expr;
        $($rest:tt)*
    } => {
        $crate::fluid_let!($(#[$attr])* $pub static $name: $type = $value);
        $crate::fluid_let!($($rest)*);
    };
    // No definitions (recursion base).
    {} => {};
}

// FIXME(ilammy, 2021-10-12): Make "static-init" available by default
//
// Macros can't abstract out #[cfg(...)] checks in expanded code
// thus we have to duplicate this macro to insert a compiler error.

#[cfg(not(feature = "static-init"))]
/// Declares global dynamic variables.
///
/// # Examples
///
/// One-line form for single declarations:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let!(static ENABLED: bool);
/// ```
///
/// Multiple declarations with attributes and visibility modifiers are also supported:
///
/// ```
/// # use fluid_let::fluid_let;
/// fluid_let! {
///     /// Length of `Debug` representation of hashes in characters.
///     pub static HASH_LENGTH: usize;
///
///     /// If set to true then passwords will be printed to logs.
///     #[cfg(test)]
///     static DUMP_PASSWORDS: bool;
/// }
/// ```
///
/// See also [crate-level documentation](index.html) for usage examples.
#[macro_export]
macro_rules! fluid_let {
    // Simple case: a single definition with None value.
    {
        $(#[$attr:meta])*
        $pub:vis static $name:ident: $type:ty
    } => {
        $(#[$attr])*
        $pub static $name: $crate::DynamicVariable<$type> = {
            thread_local! {
                static VARIABLE: $crate::DynamicCell<$type> = $crate::DynamicCell::empty();
            }
            $crate::DynamicVariable::new(&VARIABLE)
        };
    };
    // Simple case: a single definition with Some value.
    {
        $(#[$attr:meta])*
        $pub:vis static $name:ident: $type:ty = $value:expr
    } => {
        compile_error!("Static initialization is unstable, use \"static-init\" feature to opt-in");
    };
    // Multiple definitions (iteration), with None value.
    {
        $(#[$attr:meta])*
        $pub:vis static $name:ident: $type:ty;
        $($rest:tt)*
    } => {
        $crate::fluid_let!($(#[$attr])* $pub static $name: $type);
        $crate::fluid_let!($($rest)*);
    };
    // Multiple definitions (iteration), with Some value.
    {
        $(#[$attr:meta])*
        $pub:vis static $name:ident: $type:ty = $value:expr;
        $($rest:tt)*
    } => {
        $crate::fluid_let!($(#[$attr])* $pub static $name: $type = $value);
        $crate::fluid_let!($($rest)*);
    };
    // No definitions (recursion base).
    {} => {};
}

/// Binds a value to a dynamic variable.
///
/// # Examples
///
/// If you do not need to explicitly delimit the scope of dynamic assignment then you can
/// use `fluid_set!` to assign a value until the end of the current scope:
///
/// ```no_run
/// use fluid_let::{fluid_let, fluid_set};
///
/// fluid_let!(static ENABLED: bool);
///
/// fn some_function() {
///     fluid_set!(ENABLED, true);
///
///     // function body
/// }
/// ```
///
/// This is effectively equivalent to writing
///
/// ```no_run
/// # use fluid_let::{fluid_let, fluid_set};
/// #
/// # fluid_let!(static ENABLED: bool);
/// #
/// fn some_function() {
///     ENABLED.set(true, || {
///         // function body
///     });
/// }
/// ```
///
/// See also [crate-level documentation](index.html) for usage examples.
#[macro_export]
macro_rules! fluid_set {
    ($variable:expr, $value:expr) => {
        let _value_ = $value;
        // This is safe because the users do not get direct access to the guard
        // and are not able to drop it prematurely, thus maintaining invariants.
        let _guard_ = unsafe { $variable.set_guard(&_value_) };
    };
}

/// A global dynamic variable.
///
/// Declared and initialized by the [`fluid_let!`](macro.fluid_let.html) macro.
///
/// See [crate-level documentation](index.html) for examples.
pub struct DynamicVariable<T: 'static> {
    cell: &'static LocalKey<DynamicCell<T>>,
}

/// A resettable reference.
#[doc(hidden)]
pub struct DynamicCell<T> {
    cell: UnsafeCell<Option<*const T>>,
}

/// Guard setting a new value of `DynamicCell<T>`.
#[doc(hidden)]
pub struct DynamicCellGuard<'a, T> {
    old_value: Option<*const T>,
    cell: &'a DynamicCell<T>,
}

impl<T> DynamicVariable<T> {
    /// Initialize a dynamic variable.
    ///
    /// Use [`fluid_let!`](macro.fluid_let.html) macro to do this.
    #[doc(hidden)]
    pub const fn new(cell: &'static LocalKey<DynamicCell<T>>) -> Self {
        Self { cell }
    }

    /// Access current value of the dynamic variable.
    pub fn get<R>(&self, f: impl FnOnce(Option<&T>) -> R) -> R {
        self.cell.with(|current| {
            // This is safe because the lifetime of the reference returned by get()
            // is limited to this block so it cannot outlive any value set by set()
            // in the caller frames.
            f(unsafe { current.get() })
        })
    }

    /// Bind a new value to the dynamic variable.
    pub fn set<R>(&self, value: impl Borrow<T>, f: impl FnOnce() -> R) -> R {
        self.cell.with(|current| {
            // This is safe because the guard returned by set() is guaranteed to be
            // dropped after the thunk returns and before anything else executes.
            let _guard_ = unsafe { current.set(value.borrow()) };
            f()
        })
    }

    /// Bind a new value to the dynamic variable.
    ///
    /// # Safety
    ///
    /// The value is bound for the lifetime of the returned guard. The guard must be
    /// dropped before the end of lifetime of the new and old assignment values.
    /// If the variable is assigned another value while this guard is alive, it must
    /// not be dropped until that new assignment is undone.
    #[doc(hidden)]
    pub unsafe fn set_guard(&self, value: &T) -> DynamicCellGuard<T> {
        // We use transmute to extend the lifetime or "current" to that of "value".
        // This is really the case when assignments are properly scoped.
        unsafe fn extend_lifetime<'a, 'b, T>(r: &'a T) -> &'b T {
            mem::transmute(r)
        }
        self.cell
            .with(|current| extend_lifetime(current).set(value))
    }
}

impl<T: Clone> DynamicVariable<T> {
    /// Clone current value of the dynamic variable.
    pub fn cloned(&self) -> Option<T> {
        self.get(|value| value.cloned())
    }
}

impl<T: Copy> DynamicVariable<T> {
    /// Copy current value of the dynamic variable.
    pub fn copied(&self) -> Option<T> {
        self.get(|value| value.copied())
    }
}

impl<T> DynamicCell<T> {
    /// Makes a new empty cell.
    pub fn empty() -> Self {
        DynamicCell {
            cell: UnsafeCell::new(None),
        }
    }

    /// Makes a new cell with value.
    #[cfg(feature = "static-init")]
    pub fn with_static(value: &'static T) -> Self {
        DynamicCell {
            cell: UnsafeCell::new(Some(value)),
        }
    }

    /// Access the current value of the cell, if any.
    ///
    /// # Safety
    ///
    /// The returned reference is safe to use during the lifetime of a corresponding guard
    /// returned by a `set()` call. Ensure that this reference does not outlive it.
    unsafe fn get(&self) -> Option<&T> {
        (&*self.cell.get()).map(|p| &*p)
    }

    /// Temporarily set a new value of the cell.
    ///
    /// The value will be active while the returned guard object is live. It will be reset
    /// back to the original value (at the moment of the call) when the guard is dropped.
    ///
    /// # Safety
    ///
    /// You have to ensure that the guard for the previous value is dropped after this one.
    /// That is, they must be dropped in strict LIFO order, like a call stack.
    unsafe fn set(&self, value: &T) -> DynamicCellGuard<T> {
        DynamicCellGuard {
            old_value: mem::replace(&mut *self.cell.get(), Some(value)),
            cell: self,
        }
    }
}

impl<'a, T> Drop for DynamicCellGuard<'a, T> {
    fn drop(&mut self) {
        // We can safely drop the new value of a cell and restore the old one provided that
        // get() and set() methods of DynamicCell are used correctly. That is, there must be
        // no users of the new value which is about to be destroyed.
        unsafe {
            *self.cell.cell.get() = self.old_value.take();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt;
    use std::thread;

    #[test]
    fn cell_set_get_guards() {
        // This is how properly scoped usage of DynamicCell works.
        unsafe {
            let v = DynamicCell::empty();
            assert_eq!(v.get(), None);
            {
                let _g = v.set(&5);
                assert_eq!(v.get(), Some(&5));
                {
                    let _g = v.set(&10);
                    assert_eq!(v.get(), Some(&10));
                }
                assert_eq!(v.get(), Some(&5));
            }
        }
    }

    #[test]
    fn cell_unsafe_set_get_usage() {
        // The following is safe because references to constants are 'static,
        // but it is not safe in general case allowed by the API.
        unsafe {
            let v = DynamicCell::empty();
            let g1 = v.set(&5);
            let g2 = v.set(&10);
            assert_eq!(v.get(), Some(&10));
            // Specifically, you CANNOT do this:
            drop(g1);
            // g1 *must* outlive g2 or else you'll that values are restored in
            // incorrect order. Here we observe the value before "5" was set.
            assert_eq!(v.get(), None);
            // When g2 gets dropped it restores the value set by g1, which
            // may not be a valid reference at this point.
            drop(g2);
            assert_eq!(v.get(), Some(&5));
            // And now there's no one to reset the variable to None state.
        }
    }

    #[test]
    #[cfg(feature = "static-init")]
    fn static_initializer() {
        fluid_let!(static NUMBER: i32 = 42);

        assert_eq!(NUMBER.copied(), Some(42));

        fluid_let! {
            static NUMBER_1: i32 = 100;
            static NUMBER_2: i32;
            static NUMBER_3: i32 = 200;
        }

        assert_eq!(NUMBER_1.copied(), Some(100));
        assert_eq!(NUMBER_2.copied(), None);
        assert_eq!(NUMBER_3.copied(), Some(200));
    }

    #[test]
    fn dynamic_scoping() {
        fluid_let!(static YEAR: i32);

        YEAR.get(|current| assert_eq!(current, None));

        fluid_set!(YEAR, 2019);

        YEAR.get(|current| assert_eq!(current, Some(&2019)));
        {
            fluid_set!(YEAR, 2525);

            YEAR.get(|current| assert_eq!(current, Some(&2525)));
        }
        YEAR.get(|current| assert_eq!(current, Some(&2019)));
    }

    #[test]
    fn references() {
        fluid_let!(static YEAR: i32);

        // Temporary value
        fluid_set!(YEAR, 10);
        assert_eq!(YEAR.copied(), Some(10));

        // Local reference
        let current_year = 20;
        fluid_set!(YEAR, &current_year);
        assert_eq!(YEAR.copied(), Some(20));

        // Heap reference
        let current_year = Box::new(30);
        fluid_set!(YEAR, current_year);
        assert_eq!(YEAR.copied(), Some(30));
    }

    #[test]
    fn thread_locality() {
        fluid_let!(static THREAD_ID: i8);

        THREAD_ID.set(0, || {
            THREAD_ID.get(|current| assert_eq!(current, Some(&0)));
            let t = thread::spawn(move || {
                THREAD_ID.get(|current| assert_eq!(current, None));
                THREAD_ID.set(1, || {
                    THREAD_ID.get(|current| assert_eq!(current, Some(&1)));
                });
            });
            drop(t.join());
        })
    }

    #[test]
    fn convenience_accessors() {
        fluid_let!(static ENABLED: bool);

        assert_eq!(ENABLED.cloned(), None);
        assert_eq!(ENABLED.copied(), None);

        ENABLED.set(true, || assert_eq!(ENABLED.cloned(), Some(true)));
        ENABLED.set(true, || assert_eq!(ENABLED.copied(), Some(true)));
    }

    struct Hash {
        value: [u8; 16],
    }

    fluid_let!(pub static DEBUG_FULL_HASH: bool);

    impl fmt::Debug for Hash {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let full = DEBUG_FULL_HASH.copied().unwrap_or(false);

            write!(f, "Hash(")?;
            if full {
                for byte in &self.value {
                    write!(f, "{:02X}", byte)?;
                }
            } else {
                for byte in &self.value[..4] {
                    write!(f, "{:02X}", byte)?;
                }
                write!(f, "...")?;
            }
            write!(f, ")")
        }
    }

    #[test]
    fn readme_example_code() {
        let hash = Hash { value: [0; 16] };
        assert_eq!(format!("{:?}", hash), "Hash(00000000...)");
        fluid_set!(DEBUG_FULL_HASH, true);
        assert_eq!(
            format!("{:?}", hash),
            "Hash(00000000000000000000000000000000)"
        );
    }
}
