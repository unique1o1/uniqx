use anyhow::Result;
use clap::{Parser, Subcommand};
use shared::Protocol;
use std::sync::mpsc::channel;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}
#[derive(Subcommand, Debug)]
enum Command {
    Client {
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
        // Enable request console UI
        #[clap(short, long, default_value = "false")]
        console: bool,
    },
    Server {
        /// Domain being used for public access
        #[clap(short, long)]
        domain: String,
        #[clap(long, default_value_t = 80)]
        http_port: u16,
    },
}
// mod console;
use client::uniqx::UniqxClient;

#[tokio::main]
async fn run(command: Command) -> Result<()> {
    match command {
        Command::Client {
            protocol,
            local_port,
            port,
            remote_host,
            subdomain,
            local_host,
            console,
        } => {
            let client = UniqxClient::new(
                protocol,
                local_port,
                port,
                remote_host,
                subdomain,
                local_host,
                console,
            )
            .await?;
            client.start().await?;
        }
        Command::Server { domain, http_port } => {
            let tunnel = server::uniqx::UniqxServer::new(domain, http_port).await;
            tunnel.start().await?;
            let (tx, rx) = channel();
            ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
                .expect("Error setting Ctrl-C handler");
            rx.recv().expect("Could not receive from channel.");
        }
    }

    Ok(())
}
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("server=info,client=info")
        .with_level(true)
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(true)
        // Build the subscriber
        .init();
    run(Args::parse().command)
}
