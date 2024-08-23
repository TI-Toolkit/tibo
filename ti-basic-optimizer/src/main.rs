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
pub struct Args {
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
}

fn parse_8xp(path_buf: PathBuf) -> Result<parse::Program, LoadError> {
    let bytes = fs::read(path_buf).map_err(LoadError::IoError)?;
    let ti_program = TIProgram::from_bytes((&bytes, 0))
        .map_err(LoadError::DekuError)?
        .1;

    let mut tokens = ti_program.read_tokens();
    Ok(Program::from_tokens(
        &mut tokens,
        &Tokenizer::new(titokens::version::LATEST.clone(), "en"),
    ))
}

fn parse_txt(path_buf: PathBuf) -> Result<parse::Program, LoadError> {
    let string = fs::read_to_string(path_buf).map_err(LoadError::IoError)?;
    Ok(Program::from_text(
        &string,
        titokens::version::LATEST.clone(),
    ))
}

fn main() {
    let settings = Args::parse();

    let loaded = if let Some(path_buf) = settings.path_to_8xp_file {
        parse_8xp(path_buf)
    } else {
        let path_buf = settings.path_to_txt_file.unwrap();

        parse_txt(path_buf)
    };

    if let Ok(program) = loaded {
        print!("Loaded program successfully!");
        if cfg!(feature = "round-trip") {
            let version = titokens::version::LATEST.clone();

            let tokenizer = Tokenizer::new(version.clone(), "en");
            let a = program.reconstruct(&version);
            let a_program =
                Program::from_tokens(&mut Tokens::from_vec(a.clone(), Some(version.clone())), &tokenizer);
            let b = a_program.reconstruct(&version);

            if a != b {
                print!("{}", tokenizer.stringify(&a));
                print!("{}", tokenizer.stringify(&b));
                panic!("test failed");
            }

            print!("{}", tokenizer.stringify(&b));
        }
    } else {
        loaded.unwrap();
    }
}
