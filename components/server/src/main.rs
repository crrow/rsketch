mod settings;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long, value_name = "PATH")]
    config_path: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    Ok(())
}
