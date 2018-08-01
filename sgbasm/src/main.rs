#![feature(rust_2018_preview)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use failure::Error;
use failure::bail;
use structopt::StructOpt;

use sgbasm_lib::SourceFile;
use sgbasm_lib;

#[derive(StructOpt)]
struct Options {
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    if let Err(error) = run() {
        println!("{}", error);
    }
}

fn run() -> Result<(), Error> {
    let options = Options::from_args();

    let source_files = sgbasm_lib::source_files()?;

    if let Some(SourceFile::Asm(_)) = source_files.get("main") {
        let output = options.output.unwrap_or(env::current_dir()?.join("out.gb"));
        let mut output_file = File::create(output)?;

        let rom: Vec<u8> = vec!();
        output_file.write(&rom)?;
        Ok(())
    } else {
        bail!("No main.asm file");
    }
}
