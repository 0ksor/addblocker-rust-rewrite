use tokio::time::{Duration, sleep};
use zbus::Connection;

use spotify::SpotifyPlayerProxy;
mod spotify;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let conn = Connection::session().await?;
    let spotfy = SpotifyPlayerProxy::new(&conn).await?;
    let st = spotfy.playback_status().await.unwrap();
    spotfy.play().await.unwrap();
    sleep(Duration::from_millis(20)).await;
    let sta = spotfy.playback_status().await.unwrap();
    println!("{st}\n{sta}");

    Ok(())
}
