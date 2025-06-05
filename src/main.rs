use std::{
    error::Error,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use dbus::blocking::Connection;
use effects::marquee::Marquee;
use player_client::{PlayerClient, BASE_INTERFACE};
use text_effect::TextEffect;
use utils::strip_until_match;

mod effects;
mod player_client;
mod text_effect;
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
fn sanitize_title(title: String, artist: &str) -> String {
    if title
        .to_lowercase()
        .contains(&format!("{} -", &artist.to_lowercase()))
    {
        return strip_until_match(&format!("{} -", artist), &title).to_owned();
    }
    title
}

fn print(player: &PlayerClient, title_effect: &mut TextEffect) -> Result<(), Box<dyn Error>> {
    let icon = match player.playing()? {
        true => "",
        false => "",
    };

    let artist = match player.artist() {
        Ok(t) => t,
        Err(err) => return Err(format!("unable to get artist, err == {err}").into()),
    };

    let title = match player.title() {
        Ok(t) => sanitize_title(t, &artist),
        Err(err) => return Err(format!("unable to get title, err == {err}").into()),
    };

    title_effect.set_content(&title);

    let formatted = if title.is_empty() && artist.is_empty() {
        "No data".to_owned()
    } else {
        format!("{} - {}", artist, title_effect.draw())
    };

    println!("[ {icon} ] {formatted}",);
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

    // hmmm... maybe hacky?
    // we need to hold onto when the effect was previously run, so we can time it
    // easy and maybe hacky solution for now is to simply lift state up here
    let max_width = 20;
    let mut title_effect =
        TextEffect::new(150).with_effect(Box::new(Marquee::new(max_width, true)));

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
        print(active_player, &mut title_effect).expect("failed to print");
    }
}
