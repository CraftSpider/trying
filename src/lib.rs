#![feature(try_trait_v2)]
#![cfg_attr(feature = "yeet", feature(try_trait_v2_yeet))]
#![warn(clippy::std_instead_of_alloc, clippy::std_instead_of_core, clippy::match_same_arms)]

pub mod assert;
pub mod early;
pub mod warn_result;
