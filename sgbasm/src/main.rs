#![feature(rust_2018_preview)]

use std::env;
use std::fs::File;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

use sgbasm_lib;

#[derive(StructOpt)]
struct Options {
    /// filename to store the ROM in
    #[structopt(short="o", long="output", parse(from_os_str))]
    output: Option<PathBuf>,

    /// Create a new sgbasm project
    #[structopt(short="n", long="new", parse(from_os_str))]
    new: Option<PathBuf>,
}

fn main() {
    if let Err(error) = run() {
        println!("{}", error);
    }
}

fn run() -> Result<(), Error> {
    let options = Options::from_args();

    if let Some(new) = options.new {
        fs::create_dir_all(&new)?;
        File::create(new.join("main.asm"))?;
        println!("Created new project: {:?}", new);
    }
    else {
        let source_files = sgbasm_lib::source_files()?;
        let rom = sgbasm_lib::source_to_rom(&source_files)?;

        let output = options.output.unwrap_or(env::current_dir()?.join("out.gb"));
        let mut output_file = File::create(&output)?;
        output_file.write(&rom)?;
        if let Some(name) = output.file_name() {
            if let Some(name) = name.to_str() {
                println!("Compiled project to: {}", name);
            }
        }
    }

    Ok(())
}
