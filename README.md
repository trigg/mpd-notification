# mpd-notification

A simple daemon to show 'Now Playing' on track start

## Requirements - Running

* A running notification manager
* MPD Server

## Requirements - Compiling

Compiling is done by [Cargo](https://doc.rust-lang.org/cargo/)

## Building from Source

```
git clone https://github.com/trigg/mpd-notification.git
cd mpd-notification
cargo run
```

## Artwork

`mpd-notification` Assumes your MPD music location is set by [xdg-user-dirs](https://wiki.archlinux.org/title/XDG_user_directories). It will pick the first file it finds ending in `.png` `.jpeg` `.jpg` `.tiff` `.webp` `.avif` or `.tiff` and ignoring case.