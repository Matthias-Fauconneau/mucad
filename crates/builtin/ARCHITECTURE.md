# `builtin` Crate & Standard Library Architecture

The standard library is split between Rust compiler built-ins in `crates/builtin` and the `mucad_std` `.mu` source library:

- **Built-in definitions:** `crates/builtin/src/mu.rs`
  - Main Rust implementations for `core`, `debug`, `math`, `string`, `list`, `color`, `geo2d`, `geo3d`, and `ops`.
  - Declared using attributes: `#[builtin_fn]`, `#[builtin_primitive]`, `#[builtin_constant]`, and `#[builtin_mod]`.
- **Built-in runtime items:** `crates/builtin/src/item.rs`
  - Defines `BuiltinFunction`, `BuiltinPrimitive`, `BuiltinConstant`, and `BuiltinItem`.
- **Proc-macro registration:** `crates/macros/src/`
  - `builtin_fn.rs`, `builtin_primitive.rs`, `builtin_constant.rs`, `builtin_mod.rs` (exported via `lib.rs`).
- **µcad stdlib `.mu` sources:** `crates/std/lib/std/`
  - User-facing `.mu` definitions embedded and loaded via `crates/std/lib.rs`.
- **Standard-library loader:** `crates/driver/src/prelude.rs`
  - Exposes `mucad_std` and loads `crates/std/lib/std` via `stdlib::load(...)`.
- **Tests:**
  - **Low-level Rust builtin tests:** `crates/builtin/tests/` using `mucad_macros::test_builtin_fn` (defined in `crates/macros/src/`, NOT `crates/builtin/src/`).
  - **High-level `.mu` stdlib tests:** `crates/driver/tests/stdlib.rs` (dedicated non-snapshot `.mu` evaluation tests—do not add to `compiler.rs`).

Key entry points: `crates/builtin/src/mu.rs` (Rust implementations), `crates/std/lib.rs` (`.mu` stdlib loader), and `crates/builtin/tests/` (Rust `test_builtin_fn` tests).
