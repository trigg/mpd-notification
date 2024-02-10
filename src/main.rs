use config::Config;
use directories::{BaseDirs, UserDirs};
use lazy_regex::regex_is_match;
use mpd::idle::Subsystem;
use mpd::Client;
use mpd::Idle;
use mpd::Song;
use notify_rust::{Hint, Notification, Timeout};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;

/**
 * Return a PathBuf of the album art for the given Album
 *
 * File basename should be removed and this will assume the first file
 * with an image-like extension is the album art
 */
fn get_album_art(path: PathBuf) -> Option<String> {
    match fs::read_dir(path){
        Ok(results) => {
            for file in results {
                match file {
                    Ok(dir_entry) => {
                        let path = dir_entry.path();
                        let string_form = path.to_str().unwrap();
                        if regex_is_match!(r"\.(jpeg|jpg|gif|png|webp|avif|tiff)$"i, string_form) {
                            return Some(string_form.to_string());
                        }
                    }
                    Err(_e) => {}
                }
            }
        }
        Err(_e)=>{
            // Either the Music directory doesn't match MPD or MPD is on an entirely different machine
        }
    }

    return None;
}

/**
 * Decant information into notification
 */
fn notify_song(now_playing: Song, music_directory: &Path) {
    // Relative path of current playing song
    let file = Path::new(&now_playing.file);
    // Relative path of current album
    let relative_directory = file.parent().unwrap();
    // Join music dir & relative album path
    let path = Path::new(music_directory);
    let path = path.join(relative_directory);
    // Iterate files in directory until one is an image
    let album_art_path = get_album_art(path);
    // Get name of album from MPD
    let mut album_name: String = "".to_string();
    for tag in now_playing.tags {
        if tag.0 == "Album" {
            album_name = tag.1.to_string();
        }
    }
    // Get title from MPD
    let title = now_playing.title.unwrap_or("No Title".to_string());
    // Get artist from MPD
    let artist = now_playing.artist.unwrap_or("".to_string());
    // Join artist & album name
    let body = format!("{}\n{}", artist, album_name);

    let mut notify = Notification::new();
    notify.summary(&title);
    notify.body(&body);
    notify.hint(Hint::Custom(
        "x-canonical-private-synchronous".to_string(),
        "mpd-notification".to_string(),
    ));
    println!("{} {} {} {:?}", title, artist, album_name, album_art_path);

    if let Some(album_string) = album_art_path {
        notify.icon(&album_string);
    } else {
        notify.icon("audio-x-generic");
    }

    notify.timeout(Timeout::Milliseconds(6000)).show().unwrap();
}

fn main() {
    // Get directory default locations
    let base_dirs = BaseDirs::new().unwrap();
    let user_dirs = UserDirs::new().unwrap();
    let directory = base_dirs.config_dir();
    let music_directory = user_dirs.audio_dir().unwrap();
    println!("Music Dir: {:?}", music_directory);
    // Read config file
    let settings = match Config::builder()
        .add_source(config::File::with_name(directory.to_str().unwrap()).required(false))
        .build()
    {
        Ok(settings) => settings,
        Err(e) => {
            println!("Unable to find config file: {:?}", e);
            process::exit(1);
        }
    };
    // Get hostname from config file
    // TODO Get from ENV if set
    let host_name = settings
        .get_string("host_name")
        .unwrap_or("127.0.0.1:6600".to_string());
    println!("Connecting to server at : {}", host_name);
    let con = Client::connect(host_name);
    match con {
        Ok(mut con) => {
            // Main loop, await a change message from MPD
            loop {
                let value = con.idle(&[Subsystem::Player]).unwrap();
                // Not using this causes it to helpfully compile out the wait...
                let value = value.get().unwrap();
                println!("{:?}", value);
                match con.status() {
                    Ok(status) => {
                        if status.state == mpd::status::State::Play {
                            // We're in playing state, grab the song and notify it
                            let now_playing = con.currentsong().unwrap();
                            match now_playing {
                                Some(now_playing) => {
                                    notify_song(now_playing, music_directory);
                                }
                                None => {
                                    // Nothing is playing
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Unknown status : {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("No connection {:?}", e);
        }
    }
}
