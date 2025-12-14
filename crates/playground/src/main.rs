#![allow(missing_docs)]

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resolved_path =
        async_wayland_common::socket_path::SocketPath::resolve(None).context("failed to resolve server socket path")?;

    let connection = tokio::net::UnixStream::connect(resolved_path)
        .await
        .context("failed to connect to server")?;

    Ok(())
}
