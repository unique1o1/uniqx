use anyhow::Result;
use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use shared::{events, structs::Protocol};
use std::net::TcpListener;
use tokio::main;
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// The local port to expose.
    #[clap(short = 'p', long, value_name = "port")]
    local_port: u16,

    /// The protocol to use for the tunnel.
    #[clap(long, value_enum)]
    protocol: Protocol,
    /// Subdomain for public access
    #[clap(short, long, value_name = "subdomain", env = "USER")]
    subdomain: String,

    /// The local host to expose.
    #[clap(short, long, value_name = "HOST", default_value = "localhost")]
    local_host: String,

    /// Address of the remote server to expose local ports to.
    #[clap(short, long, env = "UNIQ_SERVER")]
    remote_host: String,
}
mod uniq;
use uniq::*;

#[tokio::main]
async fn run(args: Args) -> Result<()> {
    let mut uniq_client = UniqClient::new(args).await?;

    uniq_client.start().await?;
    Ok(())
}
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(false)
        // Build the subscriber
        .init();
    run(Args::parse())
}
