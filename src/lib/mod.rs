use ron::{
    self, de,
    ser::{self, PrettyConfig},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    io::Write,
    path,
};
use termion::event::Key;

pub mod event;
pub mod player;

#[derive(Debug)]
pub enum Error {
    NoSoundDevice,
    SinkState,
    PlayerState,
    Decoder(rodio::decoder::DecoderError),
    TrackSelect,

    Config(ron::error::Error),

    Io(std::io::Error),
    Env(std::env::VarError),
}

const CONFDIRPATH: &str = "/.config/rmus/";
const CONFNAME: &str = "config.ron";

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum Binding {
    Exit,
    TogglePlayback,
    StopPlayback,
    NextTrack,
    PreviousTrack,
    RewindTrack,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    bindings: BTreeMap<char, Binding>,
}

impl Config {
    fn default() -> Self {
        let mut bindings = BTreeMap::new();

        bindings.insert('q', Binding::Exit);
        bindings.insert(' ', Binding::TogglePlayback);
        bindings.insert('s', Binding::StopPlayback);
        bindings.insert('>', Binding::NextTrack);
        bindings.insert('<', Binding::PreviousTrack);
        bindings.insert('r', Binding::RewindTrack);

        Self { bindings }
    }

    pub fn open() -> Result<Self, Error> {
        let mut path = env::var("HOME").map_err(Error::Env)?;
        path.push_str(CONFDIRPATH);

        let confdirpath = path::Path::new(&path);
        if !confdirpath.exists() {
            fs::create_dir_all(&confdirpath).map_err(Error::Io)?;
        }

        let confpath = confdirpath.join(CONFNAME);
        if confpath.exists() {
            let file = File::open(&confpath).map_err(Error::Io)?;
            de::from_reader(file).map_err(Error::Config)
        } else {
            let config = Config::default();
            let pretty =
                ser::to_string_pretty(&config, PrettyConfig::default()).map_err(Error::Config)?;
            let mut file = File::create(&confpath).map_err(Error::Io)?;
            file.write(&pretty.as_bytes()).map_err(Error::Io)?;
            Ok(config)
        }
    }

    pub fn get_binding(&self, key: Key) -> Option<&Binding> {
        match key {
            Key::Char(c) => self.bindings.get(&c),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_key(&self, binding: &Binding) -> Option<char> {
        for (key, bind) in self.bindings.iter() {
            if bind == binding {
                return Some(*key);
            }
        }
        None
    }
}
