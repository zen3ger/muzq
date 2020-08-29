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
    last: usize,
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
            last: 0,
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
        self.current = if self.current < self.tracks.len() {
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

    pub fn execute(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::Play => match self.state {
                State::Playing => Err(Error::PlayerState),
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
            Action::Pause => match self.state {
                State::Playing => self.pause(),
                _ => Err(Error::PlayerState),
            },
            Action::Stop => match self.state {
                State::Playing | State::Paused => self.stop(),
                State::Stopped => Ok(()),
            },
            Action::Rewind => {
                self.stop()?;
                self.load()?;
                self.play()
            }
            Action::NextTrack => {
                let action = if let State::Playing = self.state {
                    Some(Action::Play)
                } else {
                    None
                };
                self.stop()?;
                self.next();
                if action.is_some() {
                    self.execute(Action::Play)
                } else {
                    Ok(())
                }
            }
            Action::PrevTrack => {
                let action = if let State::Playing = self.state {
                    Some(Action::Play)
                } else {
                    None
                };
                self.stop()?;
                self.prev();
                if action.is_some() {
                    self.execute(Action::Play)
                } else {
                    Ok(())
                }
            }
        }
    }
}
