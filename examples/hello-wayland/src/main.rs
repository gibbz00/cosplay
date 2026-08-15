//! `cosplay` counterpart of <https://github.com/emersion/hello-wayland>

use cosplay_protocols_wayland::{wl_compositor::WlCompositor, wl_shm::WlShm};
use cosplay_protocols_xdg_shell::xdg_wm_base::XdgWmBase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = cosplay_client::Client::setup(None).await?;

    let wl_shm_id = client.bind::<WlShm>().await?;

    let wl_compositor_id = client.bind::<WlCompositor>().await?;

    let xdg_base_id = client.bind::<XdgWmBase>().await?;

    Ok(())
}
