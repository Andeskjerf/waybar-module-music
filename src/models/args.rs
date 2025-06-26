use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Only monitor specified players, e.g "spotify firefox"
    #[arg(short, long)]
    whitelist: Vec<String>,

    /// Set play icon
    #[arg(long, default_value_t = String::from(""))]
    play_icon: String,

    /// Set pause icon
    #[arg(long, default_value_t = String::from(""))]
    pause_icon: String,

    /// Format string
    #[arg(short, long, default_value_t = String::from("[ %icon% ] %artist% - %title%"))]
    format: String,

    /// Pause before restarting marquee, in ms
    #[arg(short, long, default_value_t = 0)]
    delay_on_loop: u32,

    /// Animation update interval, in ms
    #[arg(long, default_value_t = 200)]
    effect_speed: u32,

    /// Max artist length before overflow
    #[arg(short, long, default_value_t = 0)]
    artist_width: u32,

    /// Max title length before overflow
    #[arg(short, long, default_value_t = 20)]
    title_width: u32,

    /// Enable marquee scrolling on overflow
    #[arg(short, long, default_value_t = false)]
    marquee: bool,

    /// Enable ellipsis (...) on overflow
    #[arg(long, default_value_t = false)]
    ellipsis: bool,
}
