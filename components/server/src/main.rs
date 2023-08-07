mod settings;
mod setup;

use std::env;
use clap::Parser;
use settings::Settings;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long, value_name = "PATH")]
    config_path: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let settings = Settings::new(args.config_path)?;
    setup::setup_logger(&settings.log_level);

    Ok(())
}


