use std::sync::mpsc::channel;

use anyhow::Result;
use server::uniq::Server;

fn wait() {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    rx.recv().expect("Could not receive from channel.");
    println!("Exiting...");
}
#[tokio::main]
async fn run() -> Result<()> {
    let tunnel = Server::new().await;
    tunnel.start().await?;
    wait();
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
    run()
}
