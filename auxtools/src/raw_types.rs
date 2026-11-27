//! Unchecked bindings to BYOND's internal data layouts and function entry
//! points for supported builds.
//!
//! Everything in this module is for BYOND's thread only. In `funcs`, each
//! `extern "C"` wrapper calls BYOND and returns `1` on success or `0` if BYOND
//! raised a runtime error, without taking or releasing references beyond what
//! BYOND itself does. The `*_byond` function pointers and table pointers remain
//! null until the library's first initialization.
//!
//! `procs::ExecutionContext` varies in layout across BYOND builds and must be
//! read through its accessor methods rather than direct field access.

pub mod funcs;
pub mod lists;
pub mod misc;
pub mod procs;
pub mod strings;
pub mod values;
pub mod variables;
