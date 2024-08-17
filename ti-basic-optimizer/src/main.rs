use clap::Parser;
use deku::prelude::*;
use std::{fs, io, path::PathBuf};

use titokens::{ti_connect_file::TIProgram, Tokenizer, Tokens, Version};

use ti_basic_optimizer::parse::Program;
use ti_basic_optimizer::*;

#[derive(Debug)]
enum LoadError {
    IoError(io::Error),
    DekuError(deku::DekuError),
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Settings {
    #[arg(
        long = "txt",
        group = "in_file",
        required_unless_present = "path_to_8xp_file",
        help = "Provide a text stream of tokens to optimize. Mutually exclusive with --8xp."
    )]
    path_to_txt_file: Option<PathBuf>,
    #[arg(
        long = "8xp",
        group = "in_file",
        required_unless_present = "path_to_txt_file",
        help = "Provide a tokenized 8xp to optimize. Mutually exclusive with --txt."
    )]
    path_to_8xp_file: Option<PathBuf>,
    /*#[arg(long = "size", group = "level")]
    pub size_opts: bool,
    #[arg(long = "speed", group = "level")]
    pub speed_opts: bool,*/
}

fn optimize(tokens: Tokens, settings: Settings) {
    todo!()
}

fn parse_8xp(path_buf: PathBuf) -> Result<parse::Program, LoadError> {
    let bytes = fs::read(path_buf).map_err(LoadError::IoError)?;
    let ti_program = TIProgram::from_bytes((&bytes, 0))
        .map_err(LoadError::DekuError)?
        .1;

    let mut tokens = ti_program.read_tokens();
    Ok(Program::from_tokens(
        &mut tokens,
        &Tokenizer::new(Version::latest(), "en"),
    ))
}

fn parse_txt(path_buf: PathBuf) -> Result<parse::Program, LoadError> {
    let string = fs::read_to_string(path_buf).map_err(LoadError::IoError)?;
    Ok(Program::from_text(&string, Version::latest()))
}

fn main() {
    let settings = Settings::parse();

    let loaded = if let Some(path_buf) = settings.path_to_8xp_file {
        parse_8xp(path_buf)
    } else {
        let path_buf = settings.path_to_txt_file.unwrap();

        parse_txt(path_buf)
    };

    if loaded.is_ok() {
        print!("Loaded program successfully!");
    } else {
        loaded.unwrap();
    }
}
