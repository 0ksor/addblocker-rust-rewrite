use tokio::time::{Duration, sleep};
use zbus::Connection;

use spotify::SpotifyPlayerProxy;
mod spotify;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let conn = Connection::session().await?;
    let spotfy = SpotifyPlayerProxy::new(&conn).await?;

    if let Err(e) = spotfy.playback_status().await {
        print!("Error: {e}");
    }
    if let Err(e) = spotfy.play().await {
        eprintln!("Error: {e}");
    }

    Ok(())
}
