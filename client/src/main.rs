use anyhow::Result;
use clap::Parser;
use shared::Protocol;
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// The protocol to use for the tunnel.
    // #[clap(long, value_enum)]
    protocol: Protocol,
    /// The local port to expose.
    #[clap(long)]
    local_port: u16,
    /// Optional port on the remote server to select.
    #[clap(short, long, required_if_eq("protocol", "tcp"))]
    port: Option<u16>,

    /// Address of the remote server to expose local ports to.
    #[clap(short, long, env = "UNIQ_SERVER")]
    remote_host: String,
    /// Subdomain for public access
    #[clap(short, long, env = "USER")]
    subdomain: String,

    /// The local host to expose.
    #[clap(short, long, value_name = "HOST", default_value = "localhost")]
    local_host: String,
}
// mod console;
use client::uniq::UniqClient;

#[tokio::main]
async fn run(args: Args) -> Result<()> {
    let uniq_client = UniqClient::new(
        args.protocol,
        args.local_port,
        args.port,
        args.remote_host,
        args.subdomain,
        args.local_host,
    )
    .await?;

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
