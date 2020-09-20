use mp3_metadata::{self, AudioTag, Genre};
use std::time::Duration;

pub struct TrackInfo {
    pub duration: Duration,
    pub tag: AudioTag,
}

impl TrackInfo {
    pub fn new(duration: Duration, tag: AudioTag) -> Self {
        Self { duration, tag }
    }

    pub fn no_metadata() -> Self {
        TrackInfo::no_tag(Duration::from_secs(0))
    }

    pub fn no_tag(duration: Duration) -> Self {
        let dummy = "Unknown";
        let tag = AudioTag {
            title: dummy.to_owned(),
            artist: dummy.to_owned(),
            album: dummy.to_owned(),
            year: 0,
            comment: String::new(),
            genre: Genre::Unknown,
        };
        TrackInfo::new(duration, tag)
    }
}

pub struct Track {
    info: TrackInfo,
    path: String,
}

impl Track {
    pub fn new(path: &str) -> Self {
        let info = if let Ok(meta) = mp3_metadata::read_from_file(&path) {
            match meta.tag {
                Some(tag) => TrackInfo::new(meta.duration, tag),
                None => TrackInfo::no_tag(meta.duration),
            }
        } else {
            TrackInfo::no_metadata()
        };

        Self {
            info,
            path: path.to_owned(),
        }
    }

    pub fn info(&self) -> &TrackInfo {
        &self.info
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}
