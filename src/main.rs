use carlog::cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    cli.command.run().await;
}
