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

    let players = get_players(&conn)?;

    for p in players {
        let player = PlayerClient::new(&conn, &p);
        println!("{:?}", player.get_all_properties());
    }

    Ok(())
}

//  [-------          ]
// [  ] Justice - Randy

// :1.27 | 1765 feishin | _ | :1.27 | session-3.scope | 3 | -
// :1.42 | 1765 feishin | _ | :1.42 | session-3.scope | 3 | -

// org.mpris.MediaPlayer2.Feishin
