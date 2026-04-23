use tokio::time::{Duration, sleep};
use zbus::Connection;

async fn play_pause(conn: &Connection, command: Command) -> zbus::Result<()> {
    let result = conn
        .call_method(
            Some("org.mpris.MediaPlayer2.spotify"),
            "/org/mpris/MediaPlayer2",
            Some("org.mpris.MediaPlayer2.Player"),
            command.as_method(),
            &(),
        )
        .await;

    match result {
        Ok(_) => {
            println!("{} OK", command.as_method());
            return Ok(());
        }
        Err(e) => {
            println!("{} failed: {e}", command.as_method());
            return Err(e);
        }
    }
}

enum Command {
    Play,
    Pause,
}
impl Command {
    fn as_method(&self) -> &'static str {
        match self {
            Self::Play => "Play",
            Self::Pause => "Pause",
        }
    }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let conn = Connection::session().await?;
    play_pause(&conn, Command::Play).await?;
    sleep(Duration::from_secs(5)).await;
    play_pause(&conn, Command::Pause).await?;

    Ok(())
}
