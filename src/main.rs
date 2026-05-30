use tokio::time::{Duration, sleep};
use zbus::Connection;

use spotify::SpotifyPlayerProxy;
mod spotify;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let conn = Connection::session().await?;
    let spotify = SpotifyPlayerProxy::new(&conn).await?;
    log(&spotify).await;
    Ok(())
}

enum Data {
    Artist,
    Album,
    Title,
}
impl Data {
    fn turn(&self) -> String {
        match self {
            Data::Artist => "xesam:artist".to_string(),
            Data::Album => "xesam:album".to_string(),
            Data::Title => "xesam:title".to_string(),
        }
    }
}

async fn get_metadata(spotify: &SpotifyPlayerProxy<'_>, data: Data) -> String {
    let key = data.turn();
    let mut metadata = spotify.metadata().await.unwrap();
    match data {
        Data::Artist => {
            let artists: Vec<String> = metadata.remove(&key).unwrap().try_into().unwrap();
            artists.into_iter().next().unwrap_or_default()
        }
        Data::Album => {
            let album: String = metadata.remove(&key).unwrap().try_into().unwrap();
            album
        }
        Data::Title => {
            let title: String = metadata.remove(&key).unwrap().try_into().unwrap();
            title
        }
    }
}

async fn log(spotify: &SpotifyPlayerProxy<'_>) {
    let artist = get_metadata(spotify, Data::Artist).await;
    let title = get_metadata(spotify, Data::Title).await;
    let album = get_metadata(spotify, Data::Album).await;
    println!("artist: {artist}\n Album: {album}\n title: {title}");
}
