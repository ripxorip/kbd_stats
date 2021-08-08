# Keyboard Statistics
A program used to log/measure keyboard metrics

![screenshot](misc/screenshots/1.png)

# Install
## Arch Linux
Install from the AUR: [kbd_stats](https://aur.archlinux.org/packages/kbd_stats-git/)

# Build
Build with cargo: `cargo build`

# Usage
## evdev
The keyboard events are captured by reading events using evdev. This way the tool is able to
process each key independent of which application the user is using.

## Command line
`kbd_stats --input_file /dev/input/by-path/platform-i8042-serio-0-event-kbd` where
input file points to the event to listen to. (Make sure that the user that runs the tool has
rights to read from the file be e.g. being part of the input group).

## Persistent logging
By passing the `--output_file` flag the char histogram is continuously written to the specified file.
