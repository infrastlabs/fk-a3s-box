//! A3S Box Docker Engine API server binary.

use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use a3s_box_api::ApiServer;

/// A3S Box Docker Engine API
#[derive(Parser, Debug)]
#[command(name = "a3s-box-api", about = "A3S Box Docker Engine API Server")]
struct Args {
    /// Path to the Unix domain socket for Docker API communication.
    #[arg(long, default_value = "/var/run/a3s-box/docker.sock")]
    socket: PathBuf,

    /// Also listen on the standard Docker socket path.
    #[arg(long)]
    docker_compat: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    tracing::info!(
        socket = %args.socket.display(),
        docker_compat = args.docker_compat,
        "Starting A3S Box Docker Engine API Server"
    );

    // Create and start API server
    let server = ApiServer::new(args.socket);
    server.serve().await?;

    Ok(())
}
