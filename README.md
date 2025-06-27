# waybar-module-music

A **blazingly fast**, real-time media monitoring module for Waybar.

Built with Rust using event-driven architecture and D-Bus integration to monitor MPRIS-compatible media players (Spotify, Firefox, VLC, mpv, and more).

## ✨ Features

- **🔄 Real-time updates** - Instantly reflects media player state changes
- **📱 Multi-player support** - Automatically switches between active players
- **🎬 Marquee scrolling** - Long titles scroll smoothly within configurable width
- **⚡ Resource efficient** - Zero CPU usage, minimal memory footprint
- **🎨 Waybar integration** - JSON output with CSS classes for theming
- **🎛️ Highly configurable** - Custom icons, formatting, text effects, and player filtering
- **🔧 Media controls** - Play/pause/next/previous support *(coming soon)*

## 🚀 Performance

Unlike polling-based solutions, this module is **event-driven**:
- **~0% CPU usage** - we only update when needed
- **~1MB memory usage**

## 📦 Installation

### Arch Linux
```bash
yay -S waybar-module-music-git
```

### From Source
```bash
# Clone and build
git clone https://github.com/Andeskjerf/waybar-module-music.git
cd waybar-module-music
cargo build --release

cp target/release/waybar-module-music ~/.local/bin/
```

## ⚙️ Configuration

### Basic Waybar Setup

Add to your Waybar config (`~/.config/waybar/config`):
```json
{
  "custom/music": {
    "format": "{}",
    "return-type": "json",
    "exec": "waybar-module-music",
    "restart-interval": 0
  }
}
```

Include in your modules list:
```json
{
  "modules-left": ["custom/music", "..."]
}
```

### Advanced Configuration

```bash
waybar-module-music [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `-h, --help` | Show help message | |
| `-v, --version` | Show version | |
| `-w, --whitelist "player1 player2"` | Only monitor specified players | All players |
| `--play-icon <icon>` | Set play icon | `` |
| `--pause-icon <icon>` | Set pause icon | `` |
| `-f, --format <template>` | Format string (see below) | `[ %icon% ] %artist% - %title%` |
| `-d, --delay-marquee <ms>` | Pause before restarting marquee | `0` |
| `--effect-speed <ms>` | Animation update interval | `200` |
| `-a, --artist-width <chars>` | Max artist length before overflow | Unlimited |
| `-t, --title-width <chars>` | Max title length before overflow | `20` |
| `-m, --marquee` | Enable marquee scrolling on overflow | |
| `--ellipsis` | Enable ellipsis (...) on overflow | |

### Format String

Use these placeholders in your `--format` template:
- `%icon%` - Play/pause icon
- `%artist%` - Artist name
- `%title%` - Song title
- `%album%` - Album name
- `%player%` - Player name (spotify, firefox, etc.)

**Example:**
```bash
waybar-module-music --format "🎵 %artist% | %title%" --marquee --title-width 25
```

### Example Configurations

**Minimal setup:**
```bash
waybar-module-music
```

**Spotify-only with custom icons:**
```bash
waybar-module-music --whitelist "spotify" --play-icon "▶" --pause-icon "⏸"
```

**Compact scrolling display:**
```bash
waybar-module-music --marquee --title-width 15 --effect-speed 150
```

## 🎨 Styling

The module provides CSS classes for theming in your Waybar stylesheet:

```css
#custom-music {
  padding: 0 10px;
  margin: 0 5px;
}

#custom-music.playing {
  color: #a6e3a1;
  background: #1e1e2e;
}

#custom-music.paused {
  color: #f9e2af;
  background: #1e1e2e;
}

#custom-music.stopped {
  color: #6c7086;
  background: #1e1e2e;
}
```

**Available states:**
- `.playing` - Media is currently playing
- `.paused` - Media is paused
- `.stopped` - No active players or media

---

<sub>**Why Rust?** Because when you want zero-cost abstractions, memory safety, and performance that doesn't compromise on features, Rust delivers. Also, because we can. 🦀</sub>
