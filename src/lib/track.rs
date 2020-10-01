use metaflac;
use mp3_metadata;

use std::{convert::From, default::Default, time::Duration};

pub struct Metadata {
    duration: Duration,
    artist: String,
    title: String,
    album: String,
    genre: String,
    year: u16,
}

impl Metadata {
    pub fn new(path: &str) -> Option<Self> {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(std::ffi::OsStr::to_str);
        match extension {
            Some("mp3") => mp3_metadata::read_from_file(path).map(Metadata::from).ok(),
            Some("flac") => metaflac::Tag::read_from_path(path).map(Metadata::from).ok(),
            _ => None,
        }
    }
}

impl From<mp3_metadata::MP3Metadata> for Metadata {
    fn from(mp3: mp3_metadata::MP3Metadata) -> Self {
        if let Some(tag) = &mp3.tag {
            Self {
                duration: mp3.duration,
                artist: tag.artist.clone(),
                title: tag.title.clone(),
                album: tag.album.clone(),
                // TODO: Debug format probably not the same as
                // what's found in a Vorbis comment
                genre: format!("{:?}", tag.genre),
                year: tag.year,
            }
        } else {
            let mut meta = Self::default();
            meta.duration = mp3.duration;

            meta
        }
    }
}

impl From<metaflac::Tag> for Metadata {
    fn from(tag: metaflac::Tag) -> Self {
        let mut meta = Self::default();

        let mut vorbis_comment = tag.get_blocks(metaflac::BlockType::VorbisComment);
        let mut stream_info = tag.get_blocks(metaflac::BlockType::StreamInfo);

        if let Some(metaflac::Block::VorbisComment(vc)) = vorbis_comment.next() {
            if let Some(Some(artist)) = vc.artist().map(|a| a.first()) {
                meta.artist.replace_range(.., artist.as_str());
            }

            if let Some(Some(title)) = vc.title().map(|a| a.first()) {
                meta.title.replace_range(.., title.as_str());
            }

            if let Some(Some(album)) = vc.album().map(|a| a.first()) {
                meta.album.replace_range(.., album.as_str());
            }

            if let Some(Some(genre)) = vc.genre().map(|a| a.first()) {
                meta.genre.replace_range(.., genre.as_str());
            }

            if let Some(date) = vc.get("DATE") {
                meta.year = match date.first() {
                    None => 0,
                    Some(date) => date.parse::<u16>().unwrap_or(0),
                };
            }
        }

        if let Some(metaflac::Block::StreamInfo(si)) = stream_info.next() {
            meta.duration = Duration::from_secs(si.total_samples / si.sample_rate as u64);
        }

        meta
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            duration: Duration::new(0, 0),
            artist: "Unknown".to_owned(),
            title: "Unknown".to_owned(),
            album: "Unknown".to_owned(),
            genre: "Unknown".to_owned(),
            year: 0,
        }
    }
}

pub struct Track {
    meta: Metadata,
    path: String,
}

impl Track {
    pub fn new(path: &str) -> Self {
        Self {
            meta: Metadata::new(path).unwrap_or_default(),
            path: path.to_owned(),
        }
    }

    pub fn duration(&self) -> &Duration {
        &self.meta.duration
    }

    pub fn artist(&self) -> &String {
        &self.meta.artist
    }

    pub fn title(&self) -> &String {
        &self.meta.title
    }

    pub fn album(&self) -> &String {
        &self.meta.album
    }

    pub fn genre(&self) -> &String {
        &self.meta.genre
    }

    pub fn year(&self) -> u16 {
        self.meta.year
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}
