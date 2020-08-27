use rodio;
use std::fs::File;
use std::io;

#[derive(Copy, Clone)]
pub(crate) enum Action {
    Play,
    Pause,
    Stop,
    Rewind,
    NextTrack,
    PrevTrack,
    //VolDecrease,
    //VolIncrease,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum State {
    Playing,
    Paused,
    Stopped,
}

pub(crate) struct Player {
    state: State,
    device: rodio::Device,
    sink: Option<rodio::Sink>,
    current: usize,
    tracks: Vec<String>, // TODO
}

#[derive(Debug)]
pub(crate) enum Error {
    NoSoundDevice,
    SinkState,
    PlayerState,
    Decoder(rodio::decoder::DecoderError),
    TrackSelect,
    Io(io::Error),
}

impl Player {
    pub fn new() -> Result<Self, Error> {
        let device = rodio::default_output_device().ok_or(Error::NoSoundDevice)?;
        Ok(Self {
            state: State::Stopped,
            sink: None,
            current: 0,
            tracks: vec![],
            device,
        })
    }

    pub fn enqueue(&mut self, track: String) {
        self.tracks.push(track);
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn now_playing(&self) -> Option<&String> {
        self.tracks.get(self.current)
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
        match self.sink.take() {
            None => Err(Error::SinkState),
            Some(sink) => {
                self.state = State::Stopped;
                sink.stop();
                Ok(())
            }
        }
    }

    // XXX: rodio does not support seeking
    fn load_current(&mut self) -> Result<(), Error> {
        let track = self
            .tracks
            .get(self.current)
            .ok_or(Error::TrackSelect)?
            .as_str();
        let file = File::open(track).map_err(|e| Error::Io(e))?;
        let source =
            rodio::Decoder::new(io::BufReader::new(file)).map_err(|e| Error::Decoder(e))?;

        let sink = rodio::Sink::new(&self.device);
        sink.append(source);
        self.sink = Some(sink);

        self.state = State::Stopped;
        Ok(())
    }

    fn load_next(&mut self) -> Result<(), Error> {
        self.current = if self.current < self.tracks.len() {
            self.current + 1
        } else {
            0
        };
        self.load_current()?;

        self.state = State::Stopped;
        Ok(())
    }

    fn load_prev(&mut self) -> Result<(), Error> {
        self.current = if self.current == 0 {
            self.tracks.len() - 1
        } else {
            self.current - 1
        };
        self.load_current()?;

        self.state = State::Stopped;
        Ok(())
    }

    pub fn execute(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::Play => match self.state {
                State::Playing => Err(Error::PlayerState),
                State::Paused => self.play(),
                State::Stopped => {
                    self.load_current()?;
                    self.play()
                }
            },
            Action::Pause => match self.state {
                State::Playing => self.pause(),
                _ => Err(Error::PlayerState),
            },
            Action::Stop => match self.state {
                State::Playing | State::Paused => self.stop(),
                State::Stopped => Ok(()),
            },
            Action::Rewind => {
                if let State::Stopped = self.state {
                    Ok(())
                } else {
                    let state = self.state;
                    self.load_current()?;
                    if let State::Playing = state {
                        self.play()?;
                    }
                    Ok(())
                }
            }
            Action::NextTrack => {
                self.load_next()?;
                match self.state {
                    State::Playing => self.play(),
                    _ => Ok(()),
                }
            }
            Action::PrevTrack => {
                self.load_prev()?;
                match self.state {
                    State::Playing => self.play(),
                    _ => Ok(()),
                }
            }
        }
    }
}
