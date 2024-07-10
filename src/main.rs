use clap::Parser;

mod cli;
use cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    cli.command.run().await;
}
