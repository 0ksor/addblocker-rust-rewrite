// use tokio::time::{Duration, sleep};
use zbus::Connection;

use spotify::SpotifyPlayerProxy;
mod spotify;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let conn = Connection::session().await?;
    let spotfy = SpotifyPlayerProxy::new(&conn).await?;

    match spotfy.playback_status().await {
        Err(e) => {
            eprintln!("Error: {e}");
            return Ok(());
        }
        Ok(s) => println!("{s}"),
    }
    // if let Err(e) = spotfy.play().await {
    //     eprintln!("Error: {e}");
    // }
    let mut a = spotfy.metadata().await.unwrap();
    let artists: Vec<String> = a.remove("xesam:artist").unwrap().try_into().unwrap();
    // .value_signature()
    // .to_string();
    println!("{}", artists[0]);

    Ok(())
}
