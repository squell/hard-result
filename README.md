HardResult
==========
A Rust crate for dealing with `Result` types (and `Option` and `bool`) that
has mitigations against bit-flips induced by e.g. RowHammer attacks.

This also tries to prevent against bit-flips in the instruction set that could
alter the program flow by changing e.g. a `JNZ` instructions into `JZ`.

WORK IN PROGRESS

Demo
----
Run `cargo run --example demo` for a 99 bottles of beer using `HardBool`.

Feature flags
-------------
The `try` feature flag will enable `?` for use with `HardResult`. This
requires the nightly compiler.
