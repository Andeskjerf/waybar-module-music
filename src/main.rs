use std::{error::Error, time::Duration};

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
            a.push(strip_until_match(BASE_INTERFACE, elem));
            a
        });

    Ok(players)
}

/// If the artist name is leading the title, we remove the artist from the title
fn sanitize_title(title: &str, artist: &str) -> String {
    if title.to_lowercase().contains(&artist.to_lowercase()) {
        return strip_until_match(&format!("{} -", artist), title).to_owned();
    }
    title.to_owned()
}

fn print(player: &PlayerClient) -> Result<(), Box<dyn Error>> {
    let icon = match player.playing()? {
        true => "",
        false => "",
    };

    let artist = match player.artist() {
        Ok(t) => t,
        Err(err) => return Err(format!("unable to get artist, err == {err}").into()),
    };

    let title = match player.title() {
        Ok(t) => sanitize_title(&t, &artist),
        Err(err) => return Err(format!("unable to get title, err == {err}").into()),
    };

    println!("[ {icon} ] {} - {}", artist, title);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_session()?;

    // TODO: arg handling
    // TODO: events, like sending signal to play/pause active player
    // TODO: logging

    // TODO: thread to monitor players being opened or exited
    let players = get_players(&conn)?
        .iter()
        .map(|p| PlayerClient::new(&conn, p))
        .collect::<Vec<PlayerClient>>();

    let mut active_player: Option<&PlayerClient> = None;
    const SLEEP_MS: u64 = 100;
    loop {
        std::thread::sleep(Duration::from_millis(SLEEP_MS));

        // TODO: smarter check for last playing?
        // currently, the positioning of elements in the vec is static
        // so certain players will always take priority for printing if played at the same time
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

        // TODO: add character limit for printing
        // consider making it act like marquee
        print(active_player).expect("failed to print");
    }
}
