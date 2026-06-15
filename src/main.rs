use std::future::poll_fn;
use std::pin::pin;
use std::process::Command;
use std::process::Stdio;

use futures_core::stream::Stream;
use tokio::time::{Duration, sleep};
use zbus::Connection;

mod spotify;
use spotify::SpotifyPlayerProxy;
use spotify::SpotifyRootProxy;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    Command::new("spotify")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .ok();
    let conn = Connection::session().await?;
    // wait until spotify launches
    // then create the proxys
    let (spotify, root) = loop {
        if let Ok(s) = SpotifyPlayerProxy::new(&conn).await
            && let Ok(r) = SpotifyRootProxy::new(&conn).await
        {
            break (s, r);
        }
        sleep(Duration::from_millis(200)).await;
    };
    let mut changes = pin!(spotify.receive_metadata_changed().await);

    loop {
        poll_fn(|cx| changes.as_mut().poll_next(cx)).await;
        log(&spotify).await;
    }
}

async fn log(spotify: &SpotifyPlayerProxy<'_>) {
    let mut meta = spotify.metadata().await.unwrap();
    let artists: Vec<String> = meta.remove("xesam:artist").unwrap().try_into().unwrap();
    let album: String = meta.remove("xesam:album").unwrap().try_into().unwrap();
    let title: String = meta.remove("xesam:title").unwrap().try_into().unwrap();
    println!(
        "artist: {}\n Album: {}\n title: {}\n",
        artists.join(""),
        album,
        title,
    );
}
