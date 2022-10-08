// FIXME: re-enable missing doc checks
//#![deny(missing_docs)]
//#![deny(clippy::missing_docs_in_private_items)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]

pub mod camera;
pub mod config;
pub mod diffusion;
pub mod generation;
pub mod organisms;
pub mod pathfinding;
pub mod position;
pub mod signals;
pub mod structures;
pub mod terrain;
pub mod tiles;
pub mod units;
pub mod utils;
