use std::{env, io};

use termion::{raw::IntoRawMode, screen::AlternateScreen};

mod lib;

use crate::lib::{
    event::{Event, Events},
    player::Player,
    Action, Config, Error,
};

fn main() -> Result<(), Error> {
    let config = Config::open()?;
    let tracks = env::args().skip(1);

    // Initialize terminal
    let _stdout = {
        let raw = io::stdout().into_raw_mode().map_err(Error::Io)?;
        let alt = AlternateScreen::from(raw);
        termion::cursor::HideCursor::from(alt)
    };
    let mut player = Player::new()?;

    for track in tracks {
        player.enqueue(track);
    }

    // Setup event handlers
    let events = Events::new();

    loop {
        match events.next() {
            Ok(Event::Tick(delta)) => {
                player.update(delta)?;

                player.dbg_info();
            }
            Ok(Event::Input(key)) => match config.get_binding(key) {
                None => {}
                Some(&action) => match action {
                    Action::Exit => return Ok(()),
                    Action::Player(action) => player.execute(action)?,
                },
            },
            _ => {}
        }
    }
}
