use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;

use crate::lib::Error;

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

pub struct Player {
    state: State,
    device: rodio::Device,
    sink: Option<rodio::Sink>,
    repeat: Repeate,
    current: usize,
    last: usize,
    tracks: Vec<String>, // TODO
}

impl Player {
    pub fn new() -> Result<Self, Error> {
        let device = rodio::default_output_device().ok_or(Error::NoSoundDevice)?;
        Ok(Self {
            state: State::Stopped,
            sink: None,
            repeat: Repeate::None,
            current: 0,
            last: 0,
            tracks: vec![],
            device,
        })
    }

    pub fn enqueue(&mut self, track: String) {
        self.tracks.push(track);
    }

    // temp, just for debugging
    pub fn info(&self) -> String {
        format!(
            "[{}/{}] {} {}: {}",
            self.current + 1,
            self.tracks.len(),
            match self.state {
                State::Playing => '>',
                State::Paused => '=',
                State::Stopped => 'x',
            },
            match self.repeat {
                Repeate::All => "@all",
                Repeate::Single => "@1  ",
                Repeate::Once => "--> ",
                Repeate::None => "    ",
            },
            self.tracks
                .get(self.current)
                .unwrap_or(&String::from("No track in queue..."))
        )
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
        Ok(())
    }

    fn load(&mut self) -> Result<(), Error> {
        let track = self
            .tracks
            .get(self.current)
            .ok_or(Error::TrackSelect)?
            .as_str();
        let file = File::open(track).map_err(Error::Io)?;
        let source = rodio::Decoder::new(io::BufReader::new(file)).map_err(Error::Decoder)?;

        let sink = rodio::Sink::new(&self.device);
        sink.append(source);
        sink.pause();
        self.sink = Some(sink);

        Ok(())
    }

    fn next(&mut self) {
        self.last = self.current;
        self.current = if self.current < self.tracks.len() - 1 {
            self.current + 1
        } else {
            0
        };
    }

    fn prev(&mut self) {
        self.last = self.current;
        self.current = if self.current == 0 {
            self.tracks.len() - 1
        } else {
            self.current - 1
        };
    }

    pub fn update(&mut self) -> Result<(), Error> {
        if self.sink.as_ref().map_or(false, |s| s.empty()) {
            match self.repeat {
                Repeate::None => self.execute(Action::PlaybackStop),
                Repeate::All => self.execute(Action::TrackNext),
                Repeate::Single => self.execute(Action::TrackRewind),
                Repeate::Once => {
                    let action = if self.current + 1 != self.tracks.len() {
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
                    if self.current != self.last {
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
