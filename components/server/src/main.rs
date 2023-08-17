mod settings;
mod setup;
mod tcp;

use std::env;
use clap::Parser;
use settings::Settings;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long, value_name = "PATH")]
    config_path: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup::remove_started_file_indicator();
    let args = Args::parse();
    let settings = Settings::new(args.config_path)?;
    setup::setup_logger(&settings.log_level);
    setup::setup_panic_hook();
    setup::touch_started_file_indicator();


    Ok(())
}


