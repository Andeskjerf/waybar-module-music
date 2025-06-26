# What is this?

A dynamic, currently playing module intended for Waybar (might work fine on other status bars).

It uses DBus to communicate with MPRIS compatible players.

## Features

- Marquee effect on long titles
- Controls media player state, like playing and pausing (TODO)

# How to use

## Installing

### Arch

You can find it on the AUR by the name `waybar-module-music-git`.

### Other / from source

- compile & install binary

```shell
git clone git@githib.com:Andeskjerf/waybar-module-music.git
cd waybar-module-music
cargo build --release

# move the binary to a directory in your $PATH
cp target/release/waybar-module-music ~/.local/bin
```

## Adding it to your config

- add to your Waybar config

```
"custom/music": {
	"format": "{}",
	"return-type": "json",
	"exec": "waybar-module-music",
},
```

- include the module in your bar

# Configuring

```
usage: waybar-module-music [options]
    options:
        -h, --help                  Prints this help message
        -v, --version               Get the version

        -w, --whitelist             Only monitor specified players. Can be used multiple times or invoked like "spotify firefox"

        --play-icon <value>         Set the play icon. default: 
        --pause-icon <value>        Set the play icon. default: 
        --flip-controls             Draw the player controls on the right side instead of left
        --controls-format <value>   How to format the player controls, e.g '[ %icon% ]'
        --no-controls               Disable the play / pause text

        --artist-width <value>      Set the max artist length before overflow. default: unconstrained
        --title-width <value>       Set the max title length before overflow. default: 10
        --marquee                   Marquee effect on text overflow
        --ellipsis                  Ellipsis effect on text overflow
```

## Styling

The module has the following states for CSS styling in Waybar

```
"playing"       Something is playing
"paused"        Something is paused
"stopped"       Nothing has begun playing yet, no players
```


<sub>why rust?</sub>

<sub>because i can</sub>
