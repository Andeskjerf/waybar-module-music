use std::time::Duration;

use dbus::blocking::Connection;
use player_client::{PlayerClient, BASE_INTERFACE};
use utils::strip_until_match;

mod player_client;
mod utils;

fn get_players(conn: &Connection) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let proxy = conn.with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));

    let (names,): (Vec<String>,) = proxy.method_call("org.freedesktop.DBus", "ListNames", ())?;

    let players: Vec<String> = names
        .iter()
        .filter(|name| name.contains(BASE_INTERFACE))
        .fold(vec![], |mut a, elem| {
            a.push(strip_until_match(BASE_INTERFACE.to_owned(), elem));
            a
        });

    Ok(players)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_session()?;

    let players = get_players(&conn)?
        .iter()
        .map(|p| PlayerClient::new(&conn, p))
        .collect::<Vec<PlayerClient>>();

    let mut active_player: Option<&PlayerClient> = None;
    const SLEEP_MS: u64 = 100;
    loop {
        std::thread::sleep(Duration::from_millis(SLEEP_MS));

        for p in &players {
            if p.playing()? {
                active_player = Some(p);
            }
        }

        if active_player.is_none() {
            continue;
        }
        let active_player =
            active_player.expect("unable to get active_player despite it being Some?");

        let title = match active_player.title() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let artist = match active_player.artist() {
            Ok(t) => t,
            Err(_) => continue,
        };

        println!("\n\n-----");
        println!("player: {}", active_player.name());
        println!("artist: {:?}", artist);
        println!("title: {:?}", title);

        println!("\n");
        println!("{:?}", active_player.get_all_properties()?);
    }

    Ok(())
}

//  [-------          ]
// [  ] Justice - Randy

// :1.27 | 1765 feishin | _ | :1.27 | session-3.scope | 3 | -
// :1.42 | 1765 feishin | _ | :1.42 | session-3.scope | 3 | -

// org.mpris.MediaPlayer2.Feishin
