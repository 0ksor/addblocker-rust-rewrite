use std::process::Command;
use tokio::time::{Duration, sleep};
use zbus::Connection;

mod spotify;
use spotify::SpotifyPlayerProxy;
use spotify::SpotifyRootProxy;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    Command::new("spotify").spawn().ok();
    sleep(Duration::from_secs(4)).await;

    let conn = Connection::session().await?;
    // wait until spotify launches
    // then create the proxys
    let (spotify, root) = loop {
        if let Ok(s) = SpotifyPlayerProxy::new(&conn).await {
            let root = SpotifyRootProxy::new(&conn).await?;
            break (s, root);
        } else {
            sleep(Duration::from_millis(200)).await;
        }
    };
    loop {
        log(&spotify).await;
        sleep(Duration::from_secs(4)).await;
        root.quit().await?;
    }
}

async fn log(spotify: &SpotifyPlayerProxy<'_>) {
    let mut meta = spotify.metadata().await.unwrap();
    let artists: Vec<String> = meta.remove("xesam:artist").unwrap().try_into().unwrap();
    let album: String = meta.remove("xesam:album").unwrap().try_into().unwrap();
    let title: String = meta.remove("xesam:title").unwrap().try_into().unwrap();
    println!(
        "artist: {}\n Album: {}\n title: {}",
        artists.join(""),
        album,
        title,
    );
}
