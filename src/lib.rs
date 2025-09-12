//! A pulsar timing and metadata archive developed for ARGOS project
//! <https://argos-telescope.eu/>.

#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]

extern crate argos_arpa_item_macro as item_macro;

mod archivist;
pub mod config;
pub mod conveniences;
pub mod diagnostics;
mod error;
pub mod external_tools;
pub mod pipeline;

pub use archivist::{Archivist, data_types, table::Table, table::TableItem};
pub use error::ARPAError;

pub(crate) type Result<T> = std::result::Result<T, ARPAError>;
