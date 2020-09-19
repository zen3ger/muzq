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
        AlternateScreen::from(raw)
    };
    let mut player = Player::new()?;

    for track in tracks {
        player.enqueue(track);
    }

    // Setup event handlers
    let events = Events::new();

    loop {
        match events.next() {
            Ok(Event::Tick) => {
                player.update()?;

                println!(
                    "{}{}{}",
                    termion::clear::All,
                    termion::cursor::Goto(1, 1),
                    player.info()
                );
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
