#![feature(rust_2018_preview)]
#![recursion_limit="1024"] // Used for large nom parsers

pub mod constants;
pub mod header;
pub mod instruction;
pub mod parser;
pub mod rom_builder;
