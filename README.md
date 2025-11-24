# waybar-module-music

A real-time media monitoring module for Waybar.

Built with Rust using event-driven architecture and D-Bus integration to monitor MPRIS-compatible media players (Spotify, Firefox, VLC, mpv, and more).

![Module showcase using marquee and marquee delay options](https://lmao.sh/pics/waybar-module-music.gif)
![Module showcase using marquee, marquee delay & disabled control icons with custom Waybar stylei](https://lmao.sh/pics/waybar-module-music-1.gif)

## ✨ Features

- **🔄 Real-time updates** - Instantly reflects media player state changes
- **📱 Multi-player support** - Automatically switches between active players
- **🎬 Marquee scrolling** - Long titles scroll smoothly within configurable width
- **⚡ Resource efficient** - Zero CPU usage, minimal memory footprint
- **🎨 Waybar integration** - JSON output with CSS classes for theming
- **🎛️ Highly configurable** - Custom icons, formatting, text effects, and player filtering

## 🚀 Performance

Unlike polling-based solutions, this module is **event-driven**, meaning we only do any work when we have to, like when the state of a media player changes or a text effect is due for an update.

## 📦 Installation

### Arch Linux
```bash
yay -S waybar-module-music-git
```

### NixOS
This can currently only be used with flakes.
This will build this crate from source, so be aware that that may take a bit, depending on the hardware.

To add this to your nixos configuration, you have two options:

#### The overlay
To use the overlay, just add the following to your `flake.nix`:
```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    waybar-module-music = {
      url = "github:Andeskjerf/waybar-module-music";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # other flake inputs...
  };
  outputs = {nixpkgs, waybar-module-music, ...}@inputs : {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      modules = [
        # your nixos module
        ({...}: {
          nixpkgs.overlays = [ waybar-module-music.overlays.default ];
        })

        # other nixos module imports ...
      ];
    };
  };
}
```
Now `waybar-module-music` is available in `pkgs` in your NixOS modules and you can use it however you wish.
The easiest way is to add it to `environment.systemPackages`, then it will be available on the whole system.

#### Manually accessing the package
This isn't recommended, but if you don't wanna use the overlay, here's how to do that:
```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    waybar-module-music = {
      url = "github:Andeskjerf/waybar-module-music";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # other flake inputs...
  };
  outputs = {nixpkgs, ...}@inputs : {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      specialArgs = {
        inherit inputs;
      };
      modules = [
        # your nixos module. either file or function
        ({inputs, pkgs, ...}: {
            environment.systemPackages = [
                inputs.waybar-module-music.packages.${pkgs.system}.waybar-module-music
            ];
        })
        # other nixos module imports ...
      ];
    };
  };
}
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
| `--debug` | Allow debug log events in the log file | |

### Format String

Use these placeholders in your `--format` template:
- `%icon%` - Play/pause icon
- `%artist%` - Artist name
- `%title%` - Song title
- `%album%` - Album name
- `%player%` - Player name (spotify, firefox, etc.)
- `%player-icon%` - Configurable icon to show for specific players (see `~/.config/waybar-module-music/config.toml`)

**Example:**
```bash
waybar-module-music --format "🎵 %artist% | %title%" --marquee --title-width 25
```

### Configuring per-player icons

You can find a config file at `~/.config/waybar-module-music/config.toml` where you can configure per-player icons.

```toml
[icons.players]
sample-player = "🔊"
default = ""
```

It works by doing a partial match against a player's name. So `spot` would match with Spotify, and anything else that contains `spot` in its name.

You can also configure a default value that will be shown for any players with no configured text/icon.

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

## 🔧 Troubleshooting

You can find the log file at `~/.cache/waybar-module-music/app.log`

Open an issue and include the contents of the log if you run into any problems.
