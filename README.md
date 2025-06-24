# What is this?

A dynamic, currently playing module intended for Waybar (might work fine on other status bars).

It uses DBus to communicate with MPRIS compatible players.

## Features

- Written in Rust!
- Marquee effect on long titles
- Controls media player state, like playing and pausing (TODO)

# Installing 

## Arch

You can find it on the AUR by the name `waybar-module-music-git`.

## Other / from source

- compile & install binary

```shell
git clone git@githib.com:Andeskjerf/waybar-module-music.git
cd waybar-module-music
cargo build --release

# move the binary to a directory in your $PATH
cp target/release/waybar-module-music ~/.local/bin
```

- add to your Waybar config

```
"custom/music": {
	"format": "{}",
	"return-type": "json",
	"exec": "waybar-module-music",
},
```

- include the module in your bar!
