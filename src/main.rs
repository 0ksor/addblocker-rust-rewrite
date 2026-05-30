use tokio::time::{Duration, sleep};
use zbus::Connection;

use spotify::SpotifyPlayerProxy;
mod spotify;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let conn = Connection::session().await?;
    let spotfy = SpotifyPlayerProxy::new(&conn).await?;
    loop {
        let artist = get_artist(&spotfy).await?;
        println!("{artist}");
        sleep(Duration::from_secs(2)).await;
    }
}

async fn get_artist(spotify: &SpotifyPlayerProxy<'_>) -> zbus::Result<String> {
    match spotify.playback_status().await {
        Err(e) => {
            eprintln!("Error: {e}");
            return Err(e);
        }
        Ok(s) => println!("{s}"),
    }

    let mut a = spotify.metadata().await?;
    let artists: Vec<String> = a.remove("xesam:artist").unwrap().try_into().unwrap();
    Ok(artists.into_iter().next().unwrap())
}
