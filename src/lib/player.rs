use serde::{Deserialize, Serialize};
use std::io;
use std::{fs::File, time::Duration};

use crate::lib::{track::Track, Error};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum Action {
    PlaybackToggle,
    PlaybackStop,
    RepeatModeCycle,

    TrackRewind,
    TrackNext,
    TrackPrevious,

    VolumeDecrease,
    VolumeIncrease,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum State {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug)]
enum Repeate {
    All,
    Single,
    Once,
    None,
}

impl Repeate {
    fn cycle(&self) -> Self {
        match self {
            Repeate::All => Repeate::Single,
            Repeate::Single => Repeate::Once,
            Repeate::Once => Repeate::None,
            Repeate::None => Repeate::All,
        }
    }
}

struct Playlist {
    current: usize,
    last: usize,
    tracks: Vec<Track>,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            current: 0,
            last: 0,
            tracks: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, track: String) {
        let track = Track::new(track.as_ref());
        self.tracks.push(track);
    }

    pub fn next(&mut self) {
        self.last = self.current;
        self.current = if self.current < self.tracks.len() {
            self.current + 1
        } else {
            0
        };
    }

    pub fn prev(&mut self) {
        self.last = self.current;
        self.current = if self.current == 0 {
            self.tracks.len() - 1
        } else {
            self.current - 1
        };
    }

    pub fn track(&self) -> Option<&Track> {
        self.tracks.get(self.current)
    }

    // FIXME: this is a horrible name
    pub fn at(&self) -> (usize, usize, usize) {
        (self.last, self.current, self.tracks.len())
    }

    pub fn at_end(&self) -> bool {
        self.current + 1 == self.tracks.len()
    }

    #[allow(dead_code)]
    pub fn at_start(&self) -> bool {
        self.current == 0
    }
}

pub struct Player {
    state: State,
    device: rodio::Device,
    sink: Option<rodio::Sink>,

    repeat: Repeate,
    playlist: Playlist,
    playback_time: Duration,
}

impl Player {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            state: State::Stopped,
            device: rodio::default_output_device().ok_or(Error::NoSoundDevice)?,
            sink: None,

            repeat: Repeate::None,
            playlist: Playlist::new(),
            playback_time: Duration::from_secs(0),
        })
    }

    pub fn enqueue(&mut self, track: String) {
        self.playlist.enqueue(track);
    }

    // temp, just for debugging
    pub fn dbg_info(&self) {
        let (_, at, all) = self.playlist.at();
        let track = self.playlist.track();

        println!(
            "{}{}[{}/{}] {} {}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            at + 1,
            all,
            match self.state {
                State::Playing => "PLAY ",
                State::Paused => "PAUSE",
                State::Stopped => "STOP ",
            },
            match self.repeat {
                Repeate::All => "ALL   ",
                Repeate::Single => "SINGLE",
                Repeate::Once => "ONCE  ",
                Repeate::None => "      ",
            },
        );
        if let Some(track) = track {
            let info = track.info();

            println!(
                "{}Artist: {}{}Album: {}{}Title: {}{}Genre: {:?}{}{} {}/{}",
                termion::cursor::Goto(3, 3),
                info.tag.artist,
                termion::cursor::Goto(3, 4),
                info.tag.album,
                termion::cursor::Goto(3, 5),
                info.tag.title,
                termion::cursor::Goto(3, 6),
                info.tag.genre,
                termion::cursor::Goto(1, 8),
                progress(&self.playback_time, &info.duration),
                format_duration(&self.playback_time),
                format_duration(&info.duration),
            );
        }
    }

    fn play(&mut self) -> Result<(), Error> {
        match self.sink {
            None => Err(Error::SinkState),
            Some(ref sink) => {
                self.state = State::Playing;
                sink.play();
                Ok(())
            }
        }
    }

    fn pause(&mut self) -> Result<(), Error> {
        match self.sink {
            None => Err(Error::SinkState),
            Some(ref sink) => {
                self.state = State::Paused;
                sink.pause();
                Ok(())
            }
        }
    }

    fn stop(&mut self) -> Result<(), Error> {
        if let Some(sink) = self.sink.take() {
            self.state = State::Stopped;
            sink.stop();
        }
        self.playback_time = Duration::from_secs(0);
        Ok(())
    }

    fn load(&mut self) -> Result<(), Error> {
        let track = self.playlist.track().ok_or(Error::TrackSelect)?;
        let file = File::open(track.path().as_str()).map_err(Error::Io)?;
        let source = rodio::Decoder::new(io::BufReader::new(file)).map_err(Error::Decoder)?;

        let sink = rodio::Sink::new(&self.device);
        sink.append(source);
        sink.pause();
        self.sink = Some(sink);

        Ok(())
    }

    fn next(&mut self) {
        self.playlist.next();
    }

    fn prev(&mut self) {
        self.playlist.prev();
    }

    pub fn update(&mut self, delta: u64) -> Result<(), Error> {
        if self.state == State::Playing {
            self.playback_time += std::time::Duration::from_millis(delta);
        }

        if self.sink.as_ref().map_or(false, |s| s.empty()) {
            match self.repeat {
                Repeate::None => self.execute(Action::PlaybackStop),
                Repeate::All => self.execute(Action::TrackNext),
                Repeate::Single => self.execute(Action::TrackRewind),
                Repeate::Once => {
                    let action = if !self.playlist.at_end() {
                        Action::TrackNext
                    } else {
                        Action::PlaybackStop
                    };
                    self.execute(action)
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn execute(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::PlaybackToggle => match self.state {
                State::Playing => self.pause(),
                State::Paused => {
                    let (last, current, _) = self.playlist.at();
                    if last != current {
                        self.load()?;
                    }
                    self.play()
                }
                State::Stopped => {
                    self.load()?;
                    self.play()
                }
            },
            Action::PlaybackStop => match self.state {
                State::Playing | State::Paused => self.stop(),
                State::Stopped => Ok(()),
            },
            Action::TrackRewind => {
                self.stop()?;
                self.load()?;
                self.play()
            }
            Action::TrackNext => {
                let state = self.state;

                self.stop()?;
                self.next();

                if state == State::Playing {
                    self.execute(Action::PlaybackToggle)
                } else {
                    Ok(())
                }
            }
            Action::TrackPrevious => {
                let state = self.state;

                self.stop()?;
                self.prev();

                if state == State::Playing {
                    self.execute(Action::PlaybackToggle)
                } else {
                    Ok(())
                }
            }
            Action::RepeatModeCycle => {
                self.repeat = self.repeat.cycle();
                Ok(())
            }
            Action::VolumeDecrease | Action::VolumeIncrease => Ok(()),
        }
    }
}

fn format_duration(duration: &Duration) -> String {
    let mut sec = duration.as_secs();
    let hour = sec / 3600;
    sec -= hour * 3600;
    let min = sec / 60;
    sec -= min * 60;

    if hour != 0 {
        format!("{:02}:{:02}:{:02}", hour, min, sec)
    } else {
        format!("{:02}:{:02}", min, sec)
    }
}

fn progress(playback: &Duration, length: &Duration) -> String {
    let psec = playback.as_secs();
    let lsec = length.as_secs();
    let ratio = ((psec as f32 / lsec as f32) * 20.0) as usize;

    let mut buf = String::with_capacity(22);

    buf.push('[');
    for i in 2..buf.capacity() {
        if i <= ratio {
            buf.push('-');
        } else {
            buf.push(' ');
        }
    }
    buf.push(']');

    assert_eq!(buf.len(), 22);
    assert_eq!(buf.capacity(), 22);

    buf
}
