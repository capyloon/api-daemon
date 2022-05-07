[Unreleased]
============

The version currently under development.

fluid-let 1.0.0 — 2021-10-12
============================

New features:

- Convenience getters `copied()` and `cloned()` for copyable types.
- Convenience setter `fluid_set!` for scoped assignment.

Unstable features:

- `"static-init"` Cargo feature
  - `fluid_let!` allows `'static` initializers:
    ```rust
    fluid_let!(static VARIABLE: Type = initial_value);
    ```

fluid-let 0.1.0 — 2019-03-12
============================

Initial release of fluid-let.

- `fluid_let!` macro for defining global dynamically-scoped variables.
- `get()` and `set()` methods with closure-based interface for querying
  and modifying the dynamic environment.
