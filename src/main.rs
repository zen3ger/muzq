use std::{env, io};

use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};

mod event;
mod player;

use crate::event::{Event, Events, EXIT_KEY};
use crate::player::{Action, Error, Player, State};

fn main() -> Result<(), Error> {
    let tracks = env::args().skip(1);

    // Initialize terminal
    let stdout = io::stdout().into_raw_mode().map_err(|e| Error::Io(e))?;
    let stdout = AlternateScreen::from(stdout);

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
            print!(
                "{}{}{:?}: {}\n",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
                player.state(),
                player.now_playing().expect("no tracks")
            );
        }

        if let Ok(Event::Input(key)) = events.next() {
            action = match key {
                EXIT_KEY => return player.execute(Action::Stop),

                Key::Char(' ') => match player.state() {
                    State::Playing => Some(Action::Pause),
                    State::Paused | State::Stopped => Some(Action::Play),
                },
                Key::Char('s') => Some(Action::Stop),
                Key::Char('>') => Some(Action::NextTrack),
                Key::Char('<') => Some(Action::PrevTrack),
                Key::Char('r') => Some(Action::Rewind),
                //Key::Char('J') => Some(Action::VolDecrease),
                //Key::Char('K') => Some(Action::VolIncrease),
                _ => None,
            };
        }
    }
}
