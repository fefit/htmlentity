//! # htmlentity
//!
//! `htmlentity` encode and decode for html entity.
pub mod data;
pub mod entity;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
