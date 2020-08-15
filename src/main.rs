use std::fs::File;
use std::io;

use rodio;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};

const TRACK2: &str =
    "/usr/home/zen3ger/music/Falling In Reverse - The Drug In Me Is You/01. Raised By Wolves.mp3";
const TRACK1: &str =
    "/usr/home/zen3ger/music/Falling In Reverse - The Drug In Me Is You/02. Tragic Magic.mp3";

mod event;

use crate::event::{Event, Events, EXIT_KEY};

#[derive(Debug)]
enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Rewind,
    NextTrack,
    PrevTrack,
    VolDecrease,
    VolIncrease,
    ScrubForwards,
    ScrubBackwards,
}

#[derive(Debug)]
enum PlayerState {
    Playing,
    Paused,
    Stopped,
}

fn mksink(device: &rodio::Device, name: &str) -> rodio::Sink {
    let file = File::open(name).unwrap();
    let source = rodio::Decoder::new(io::BufReader::new(file)).unwrap();

    let sink = rodio::Sink::new(device);
    sink.append(source);

    sink
}

fn main() -> Result<(), io::Error> {
    // Initialize terminal
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);

    // Initialize sound device
    let device = rodio::default_output_device().unwrap(); // TODO

    // Setup event handlers
    let events = Events::new();

    let mut cmd = None;
    let mut state = PlayerState::Stopped;
    let mut track = &TRACK1;

    let mut sink: Option<rodio::Sink> = None;

    loop {
        // TODO: rodio does not support seeking
        if let Some(c) = cmd.take() {
            match c {
                PlayerCommand::Play => match state {
                    PlayerState::Playing => {}
                    PlayerState::Paused => {
                        state = PlayerState::Playing;
                        match sink {
                            None => unreachable!("expected sink"),
                            Some(ref s) => s.play(),
                        }
                    }
                    PlayerState::Stopped => {
                        state = PlayerState::Playing;
                        match sink {
                            None => {
                                let s = mksink(&device, *track);
                                s.play();
                                sink = Some(s);
                            }
                            Some(_) => unreachable!("unexpected sink"),
                        }
                    }
                },
                PlayerCommand::Pause => match state {
                    PlayerState::Playing => {
                        state = PlayerState::Paused;
                        if let Some(ref s) = sink {
                            s.pause();
                        } else {
                            unreachable!("expected sink");
                        }
                    }
                    PlayerState::Paused | PlayerState::Stopped => {}
                },
                PlayerCommand::Stop => match state {
                    PlayerState::Playing | PlayerState::Paused => {
                        state = PlayerState::Stopped;
                        match sink.take() {
                            None => unreachable!("expected sink"),
                            Some(s) => s.stop(),
                        }
                    }
                    PlayerState::Stopped => {}
                },
                PlayerCommand::Rewind => {
                    if let PlayerState::Stopped = state {
                        // nothing to be done
                    } else {
                        sink = Some(mksink(&device, *track));

                        match state {
                            PlayerState::Playing => sink.as_ref().unwrap().play(),
                            PlayerState::Paused => {}
                            PlayerState::Stopped => unreachable!("unexpected state"),
                        }
                    }
                }
                PlayerCommand::NextTrack => {
                    // TODO
                    track = &TRACK2;

                    match state {
                        PlayerState::Playing => {
                            sink = Some(mksink(&device, *track));
                            sink.as_ref().unwrap().play();
                        }
                        PlayerState::Paused | PlayerState::Stopped => {
                            state = PlayerState::Stopped;
                        }
                    }
                }
                PlayerCommand::PrevTrack => {
                    // TODO
                    track = &TRACK1;

                    match state {
                        PlayerState::Playing => {
                            sink = Some(mksink(&device, *track));
                            sink.as_ref().unwrap().play();
                        }
                        PlayerState::Paused | PlayerState::Stopped => state = PlayerState::Stopped,
                    }
                }
                cmd => eprintln!("unhandled command: {:?}", cmd),
            }

            print!("{}{}{:?}: {}\n", termion::clear::All, termion::cursor::Goto(1,1), state, *track);
        }

        if let Ok(Event::Input(key)) = events.next() {
            match key {
                EXIT_KEY => break,
                Key::Char(' ') => match state {
                    PlayerState::Playing => cmd = Some(PlayerCommand::Pause),
                    PlayerState::Paused | PlayerState::Stopped => cmd = Some(PlayerCommand::Play),
                },
                Key::Char('s') => cmd = Some(PlayerCommand::Stop),
                Key::Char('>') => cmd = Some(PlayerCommand::NextTrack),
                Key::Char('<') => cmd = Some(PlayerCommand::PrevTrack),
                Key::Char('r') => cmd = Some(PlayerCommand::Rewind),
                Key::Char('J') => cmd = Some(PlayerCommand::VolDecrease),
                Key::Char('K') => cmd = Some(PlayerCommand::VolIncrease),
                Key::Char('L') => cmd = Some(PlayerCommand::ScrubForwards),
                Key::Char('H') => cmd = Some(PlayerCommand::ScrubBackwards),
                _ => (),
            }
        }
    }

    Ok(())
}
