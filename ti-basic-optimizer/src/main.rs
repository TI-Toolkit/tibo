use clap::Parser;

use titokens::Tokens;

use ti_basic_optimizer::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Settings {
    #[arg(long = "size", group = "level")]
    pub size_opts: bool,
    #[arg(long = "speed", group = "level")]
    pub speed_opts: bool,
}

fn optimize(tokens: Tokens, settings: Settings) {
    todo!()
}

fn main() {
    let settings = Settings::parse();
}
