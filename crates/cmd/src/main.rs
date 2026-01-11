// Copyright 2025 Crrow
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::{Args, Parser, Subcommand};
use snafu::Whatever;

mod build_info;
use rsketch_app::AppConfig;

#[derive(Debug, Parser)]
#[clap(
name = "rsketch",
about= "rsketch-cmd",
author = build_info::AUTHOR,
version = build_info::FULL_VERSION)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Hello(HelloArgs),
    Server(ServerArgs),
}

#[derive(Debug, Clone, Args)]
#[command(flatten_help = true)]
#[command(long_about = r"

Print hello.
Examples:

rsketch hello

")]
struct HelloArgs {}

impl HelloArgs {
    fn run(&self) -> Result<(), Whatever> {
        println!("Hello, world!");
        Ok(())
    }
}

#[derive(Debug, Clone, Args)]
#[command(flatten_help = true)]
#[command(long_about = r"

Starts the rsketch server.
Examples:

rsketch server

")]
struct ServerArgs {}

impl ServerArgs {
    async fn run(&self) -> Result<(), Whatever> {
        let app = AppConfig::default().open();
        app.run().await
    }
}

#[tokio::main]
async fn main() -> Result<(), Whatever> {
    let cli = Cli::parse();
    match cli.commands {
        Commands::Hello(ha) => ha.run(),
        Commands::Server(sa) => sa.run().await,
    }
}
