// #![feature(specialization)] - in nightly builds we can remove the hard dependency on Gluon and use the traits to implement other scripting languages
#![deny(bare_trait_objects)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate gluon_vm;
#[macro_use] extern crate gluon_codegen;

mod symbol;
mod editor;
mod notebook;
mod host;
mod error;

pub use self::symbol::*;
pub use self::editor::*;
pub use self::notebook::*;
pub use self::host::*;
pub use self::error::*;

pub mod gluon_host;
pub mod streams;
