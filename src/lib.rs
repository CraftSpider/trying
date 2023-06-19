//! Various potential implementors for the `Try` trait - typed assertions,
//! early-return values, and more.

#![feature(try_trait_v2)]
#![cfg_attr(feature = "yeet", feature(try_trait_v2_yeet))]
#![warn(
    missing_docs,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    missing_abi,
    noop_method_call,
    pointer_structural_match,
    semicolon_in_expressions_from_macros,
    unused_import_braces,
    unused_lifetimes,
    unsafe_op_in_unsafe_fn,
    // clippy::cargo,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::ptr_as_ptr,
    clippy::cloned_instead_of_copied,
    clippy::unreadable_literal,
    clippy::undocumented_unsafe_blocks,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,

    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::match_same_arms
)]

extern crate alloc;

pub mod assert;
pub mod early;
pub mod warn_result;
