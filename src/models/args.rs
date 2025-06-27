use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// Only monitor specified players, e.g "spotify firefox"
    #[arg(short, long)]
    whitelist: Vec<String>,

    /// Set play icon
    #[arg(long, default_value_t = String::from(""))]
    pub play_icon: String,

    /// Set pause icon
    #[arg(long, default_value_t = String::from(""))]
    pub pause_icon: String,

    /// Format string
    #[arg(short, long, default_value_t = String::from("[ %icon% ] %artist% - %title%"))]
    pub format: String,

    /// Pause before restarting marquee, in ms
    #[arg(short, long, default_value_t = 0)]
    pub delay_on_loop: u32,

    /// Animation update interval, in ms
    #[arg(long, default_value_t = 200)]
    pub effect_speed: u16,

    /// Max artist length before overflow
    #[arg(short, long, default_value_t = 0)]
    pub artist_width: u16,

    /// Max title length before overflow
    #[arg(short, long, default_value_t = 20)]
    pub title_width: u16,

    /// Enable marquee scrolling on overflow
    #[arg(short, long, default_value_t = false)]
    pub marquee: bool,

    /// Enable ellipsis (...) on overflow
    #[arg(long, default_value_t = false)]
    pub ellipsis: bool,
}
