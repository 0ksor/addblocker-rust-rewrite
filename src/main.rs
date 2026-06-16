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
use zbus::names::BusName;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let conn = Connection::session().await.unwrap();
    let (mut spotify, mut root) = launch_spotify(&conn).await;
    try_play(&spotify).await;
    log(&spotify).await;
    let mut changes = pin!(spotify.receive_metadata_changed().await);

    loop {
        poll_fn(|cx| changes.as_mut().poll_next(cx)).await;
        log(&spotify).await;
        if is_artist_empty(&spotify).await {
            println!("artist is empty");
            let _ = root.quit().await;
            wait_spotify_dead(&conn).await;
            // NOTE: This code works without this reassingment.

            // my guess is zbus is using names insted of ids to seperate busses,
            // but I am reassinging just in case somethig failes.
            (spotify, root) = launch_spotify(&conn).await; // we reassign the streams to the newly created ones.
            try_play(&spotify).await;
        }
    }
}

async fn launch_spotify(
    conn: &Connection,
) -> (SpotifyPlayerProxy<'static>, SpotifyRootProxy<'static>) {
    Command::new("spotify")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .ok();
    // wait until spotify launches
    // then create the proxys
    let (spotify, root) = loop {
        if let Ok(s) = SpotifyPlayerProxy::new(conn).await
            && let Ok(r) = SpotifyRootProxy::new(conn).await
        {
            break (s, r);
        }
        sleep(Duration::from_millis(200)).await;
    };
    (spotify, root)
}

async fn is_artist_empty(spotify: &SpotifyPlayerProxy<'static>) -> bool {
    let mut meta = spotify.metadata().await.unwrap();
    let artists: Vec<String> = meta.remove("xesam:artist").unwrap().try_into().unwrap();
    // spotify dbus implemantation has a bug. It always retuns 1 artst even when there are multiple
    let artist = artists.join("");
    artist.is_empty()
}

async fn log(spotify: &SpotifyPlayerProxy<'static>) {
    let mut meta = spotify.metadata().await.unwrap();
    let artists: Vec<String> = meta.remove("xesam:artist").unwrap().try_into().unwrap();
    let title: String = meta.remove("xesam:title").unwrap().try_into().unwrap();
    let album: String = meta.remove("xesam:album").unwrap().try_into().unwrap();
    println!(
        "artist: {}\n Album: {}\n title: {}\n",
        artists.join(""),
        album,
        title,
    );
}

async fn try_play(spotify: &SpotifyPlayerProxy<'static>) {
    loop {
        if spotify.play().await.is_ok()
            && spotify.playback_status().await.as_deref() == Ok("Playing")
        {
            println!("broken");
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
}

async fn wait_spotify_dead(conn: &Connection) {
    while zbus::fdo::DBusProxy::new(conn)
        .await
        .unwrap()
        .name_has_owner(BusName::from_static_str("org.mpris.MediaPlayer2.spotify").unwrap())
        .await
        .unwrap()
    {
        sleep(Duration::from_millis(200)).await;
    }
}
