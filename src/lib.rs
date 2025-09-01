//! A pulsar timing and metadata archive developed for argos.

#![warn(
    // missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo,
)]
#![allow(clippy::must_use_candidate)]

mod error;
mod archivist;
// mod worker;
pub mod external_tools;
pub mod conveniences;
pub mod config;
pub mod diagnostics;

pub use archivist::{
    Archivist,
    table::Table,
    table::TableItem,
    data_types,
};
// pub use worker::Worker;
pub use error::ARPAError;

pub type Result<T> = std::result::Result<T, ARPAError>;
