use std::{env, io};

use termion::{raw::IntoRawMode, screen::AlternateScreen};

mod lib;

use crate::lib::{
    event::{Event, Events},
    player::{Action, Player, State},
    Binding, Config, Error,
};

fn main() -> Result<(), Error> {
    let config = Config::open()?;
    let tracks = env::args().skip(1);

    // Initialize terminal
    let _stdout = {
        let raw = io::stdout().into_raw_mode().map_err(Error::Io)?;
        AlternateScreen::from(raw)
    };
    let mut player = Player::new()?;
    let mut action = None;

    for track in tracks {
        player.enqueue(track);
    }

    // Setup event handlers
    let events = Events::new();

    loop {
        if let Some(action) = action.take() {
            player.execute(action)?;
            println!(
                "{}{}{:?}: {}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
                player.state(),
                player.now_playing().expect("no tracks")
            );
        }

        if let Ok(Event::Input(key)) = events.next() {
            if let Some(binding) = config.get_binding(key) {
                action = match binding {
                    Binding::Exit => return Ok(()),
                    Binding::TogglePlayback => match player.state() {
                        State::Playing => Some(Action::Pause),
                        State::Paused | State::Stopped => Some(Action::Play),
                    },
                    Binding::StopPlayback => Some(Action::Stop),
                    Binding::NextTrack => Some(Action::NextTrack),
                    Binding::PreviousTrack => Some(Action::PrevTrack),
                    Binding::RewindTrack => Some(Action::Rewind),
                }
            } else {
                action = None;
            }
        }
    }
}
