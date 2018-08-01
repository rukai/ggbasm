#![feature(rust_2018_preview)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

use sgbasm_lib;

#[derive(StructOpt)]
struct Options {
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
    #[structopt(parse(from_os_str))]
    new_project: Option<PathBuf>,
}

fn main() {
    if let Err(error) = run() {
        println!("{}", error);
    }
}

fn run() -> Result<(), Error> {
    let options = Options::from_args();

    let source_files = sgbasm_lib::source_files()?;
    let rom = sgbasm_lib::source_to_rom(&source_files)?;

    let output = options.output.unwrap_or(env::current_dir()?.join("out.gb"));
    let mut output_file = File::create(output)?;
    output_file.write(&rom)?;

    Ok(())
}
