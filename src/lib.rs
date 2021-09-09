#![allow(dead_code, unused_variables)]
//! See README.md

// TODO: reorg pub
#[macro_use]
pub mod core;
pub mod dom;
pub mod sfc;
mod ssr;

pub use crate::core::base_compile;
