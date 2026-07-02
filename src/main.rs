use std::future::poll_fn;
use std::pin::pin;
use std::process::Command;
use std::process::Stdio;
use std::process::exit;
use std::sync::OnceLock;

use futures_core::stream::Stream;
use tokio::time::{Duration, sleep};
use zbus::Connection;

mod spotify;
use spotify::SpotifyPlayerProxy;
use spotify::SpotifyRootProxy;
use zbus::names::BusName;
type Metadata = std::collections::HashMap<String, zbus::zvariant::OwnedValue>;
static SPOTIFY_CMD: OnceLock<String> = OnceLock::new();

struct Counter {
    count: u64,
}
impl Counter {
    fn new() -> Self {
        let count = 0;
        Self { count }
    }

    fn count(&mut self) {
        self.count += 1;
        println!("\n{} ads has been skiped\n", self.count);
    }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    check_spotify();
    let mut counter = Counter::new();
    let conn = Connection::session().await.unwrap();
    let (mut spotify, mut root) = launch_spotify(&conn).await;
    let mut changes = pin!(spotify.receive_metadata_changed().await);
    log(&spotify.metadata().await.unwrap());

    loop {
        let meta = poll_fn(|cx| changes.as_mut().poll_next(cx))
            .await
            .unwrap()
            .get()
            .await
            .unwrap();
        log(&meta);
        if is_artist_empty(&meta) {
            counter.count();
            let _ = root.quit().await;
            wait_spotify_dead(&conn).await;
            // NOTE: This code works without this reassingment.
            // my guess is zbus is using names insted of ids to tell apart busses,
            // but I am reassigning just in case something failes.
            (spotify, root) = launch_spotify(&conn).await; // we reassign the streams to the newly created ones.
            let _ = spotify.play().await.is_ok();
        }
    }
}

async fn launch_spotify(
    conn: &Connection,
) -> (SpotifyPlayerProxy<'static>, SpotifyRootProxy<'static>) {
    let _ = Command::new(SPOTIFY_CMD.get().unwrap())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn();
    // wait until spotify launches
    // then create the proxys
    let (spotify, root) = loop {
        if let Ok(s) = SpotifyPlayerProxy::new(conn).await
            && let Ok(r) = SpotifyRootProxy::new(conn).await
            && s.playback_status().await.is_ok()
        {
            break (s, r);
        }
        sleep(Duration::from_millis(200)).await;
    };
    (spotify, root)
}

fn is_artist_empty(meta: &Metadata) -> bool {
    // Get a reference, clone it via try_clone, and unwrap (since we know it's cloneable)
    let artists = meta
        .get("xesam:artist")
        .unwrap()
        .try_clone()
        .expect("artist value should be cloneable");

    let artists: Vec<String> = artists.try_into().unwrap();
    // there is a bug in dbus implemantation of Spotify,
    // it returns only one artist even when there are multiple.
    let artist = artists.join("");
    artist.is_empty()
}

fn log(meta: &Metadata) {
    let artists = meta
        .get("xesam:artist")
        .unwrap()
        .try_clone()
        .expect("artist value should be cloneable");

    let artists: Vec<String> = artists.try_into().unwrap();
    let title: String = meta.get("xesam:title").unwrap().to_string();
    let album: String = meta.get("xesam:album").unwrap().to_string();
    if !is_artist_empty(meta) {
        println!(
            "artist: {}\n Album: {}\n title: {}\n",
            artists.join(""),
            album,
            title,
        );
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

fn has_cmd(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn check_spotify() {
    let cmd = ["spotify-launcher", "spotify"]
        .into_iter()
        .find(|c| has_cmd(c))
        .unwrap_or_else(|| {
            eprintln!("ERROR: spotify not found");
            exit(1);
        });
    SPOTIFY_CMD.set(cmd.to_string()).ok();
}
