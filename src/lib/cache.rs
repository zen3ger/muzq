use std::collections::HashMap;
use std::time::Duration;

const CACHENAME: &str = "cache.ron";

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
/// Unique identifier for an imported track.
pub struct TrackId(u32);

/// Holds the path to the imported file.
pub struct Track(String);

#[derive(Debug)]
pub struct Metadata {
    duration: Option<Duration>,
    artist: String,
    title: String,
    album: String,
    genre: String,
    year: u16,
}

/// Sequence of track IDs represent a custom playlist.
pub struct Playlist {
    name: String,
    tracks: Vec<TrackId>,
}

/// Predicates that can be used for filtering tracks based on their metadata.
pub enum MetaFilterPredicate<'a> {
    Artist(&'a str),
    Title(&'a str),
    Album(&'a str),
    Genre(&'a str),
    Year(u16),
}

/// Predicate modifiers.
pub enum MetaFilter<'a> {
    Not(MetaFilterPredicate<'a>),
    Is(MetaFilterPredicate<'a>),
}

impl<'a> MetaFilterPredicate<'a> {
    pub fn matches(&self, meta: &Metadata) -> bool {
        match self {
            MetaFilterPredicate::Artist(artist) => meta.artist == *artist,
            MetaFilterPredicate::Title(title) => meta.title == *title,
            MetaFilterPredicate::Album(album) => meta.album == *album,
            MetaFilterPredicate::Genre(genre) => meta.genre == *genre,
            MetaFilterPredicate::Year(year) => meta.year == *year,
        }
    }
}

impl<'a> MetaFilter<'a> {
    pub fn matches(&self, meta: &Metadata) -> bool {
        match self {
            MetaFilter::Not(pred) => !pred.matches(meta),
            MetaFilter::Is(pred) => pred.matches(meta),
        }
    }
}

pub struct Cache {
    playlists: Vec<Playlist>,
    next_id: TrackId,
    tracks: HashMap<TrackId, Track>,
    metas: HashMap<TrackId, Metadata>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            playlists: Vec::new(),
            next_id: TrackId(0),
            tracks: HashMap::new(),
            metas: HashMap::new(),
        }
    }

    pub fn insert(&mut self, track: Track, meta: Option<Metadata>) {
        let id = self.next_id;

        self.tracks.insert(id, track);
        if let Some(meta) = meta {
            self.metas.insert(id, meta);
        }

        self.next_id = TrackId(id.0 + 1);
    }

    pub fn get_track(&self, id: TrackId) -> Option<&Track> {
        self.tracks.get(&id)
    }

    pub fn matches_filter(&self, filters: &[MetaFilter]) -> Playlist {
        let mut matches = Playlist {
            name: "Matches".to_owned(),
            tracks: Vec::new(),
        };

        for (id, meta) in self.metas.iter() {
            if filters.iter().all(|filter| filter.matches(meta)) {
                matches.tracks.push(*id)
            }
        }

        matches.tracks.sort();
        matches
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::cache::*;
    use crate::lib::cache::{MetaFilter::*, MetaFilterPredicate::*};

    fn test_cache() -> Cache {
        let mut cache = Cache::new();

        cache.insert(
            Track("/track0".to_owned()),
            Some(Metadata {
                duration: None,
                artist: "A".to_owned(),
                title: "a".to_owned(),
                album: "a".to_owned(),
                genre: "a".to_owned(),
                year: 0,
            }),
        );

        cache.insert(
            Track("/track1".to_owned()),
            Some(Metadata {
                duration: None,
                artist: "B".to_owned(),
                title: "a".to_owned(),
                album: "b".to_owned(),
                genre: "a".to_owned(),
                year: 1,
            }),
        );

        cache.insert(
            Track("/track2".to_owned()),
            Some(Metadata {
                duration: None,
                artist: "B".to_owned(),
                title: "b".to_owned(),
                album: "b".to_owned(),
                genre: "a".to_owned(),
                year: 1,
            }),
        );

        cache.insert(
            Track("/track3".to_owned()),
            Some(Metadata {
                duration: None,
                artist: "C".to_owned(),
                title: "c".to_owned(),
                album: "c".to_owned(),
                genre: "a".to_owned(),
                year: 2,
            }),
        );

        cache.insert(
            Track("/track4".to_owned()),
            Some(Metadata {
                duration: None,
                artist: "D".to_owned(),
                title: "d".to_owned(),
                album: "d".to_owned(),
                genre: "a".to_owned(),
                year: 3,
            }),
        );

        cache.insert(
            Track("/track0".to_owned()),
            Some(Metadata {
                duration: None,
                artist: "E".to_owned(),
                title: "e".to_owned(),
                album: "e".to_owned(),
                genre: "e".to_owned(),
                year: 3,
            }),
        );

        cache
    }

    #[test]
    fn cache_filter_artist() {
        let cache = test_cache();

        assert_eq!(
            cache.matches_filter(&[Is(Artist("A"))]).tracks,
            vec![TrackId(0)]
        );

        assert_eq!(
            cache.matches_filter(&[Is(Artist("B"))]).tracks,
            vec![TrackId(1), TrackId(2)]
        );

        assert_eq!(
            cache
                .matches_filter(&[Not(Artist("A")), Not(Artist("B"))])
                .tracks,
            vec![TrackId(3), TrackId(4), TrackId(5)]
        );
    }

    #[test]
    fn cache_filter_title() {
        let cache = test_cache();

        assert_eq!(
            cache.matches_filter(&[Is(Title("a"))]).tracks,
            vec![TrackId(0), TrackId(1)]
        );

        assert_eq!(
            cache.matches_filter(&[Is(Title("b"))]).tracks,
            vec![TrackId(2)]
        );

        assert_eq!(
            cache
                .matches_filter(&[Not(Title("a")), Not(Title("b"))])
                .tracks,
            vec![TrackId(3), TrackId(4), TrackId(5)]
        );
    }

    #[test]
    fn cache_filter_album() {
        let cache = test_cache();

        assert_eq!(
            cache.matches_filter(&[Is(Album("a"))]).tracks,
            vec![TrackId(0)]
        );
        assert_eq!(
            cache.matches_filter(&[Is(Album("b"))]).tracks,
            vec![TrackId(1), TrackId(2)]
        );
        assert_eq!(
            cache
                .matches_filter(&[Not(Album("a")), Not(Album("b"))])
                .tracks,
            vec![TrackId(3), TrackId(4), TrackId(5)]
        );
    }

    #[test]
    fn cache_filter_genre() {
        let cache = test_cache();
        assert_eq!(
            cache.matches_filter(&[Is(Genre("a"))]).tracks,
            vec![TrackId(0), TrackId(1), TrackId(2), TrackId(3), TrackId(4)]
        );
        assert_eq!(
            cache.matches_filter(&[Not(Genre("a"))]).tracks,
            vec![TrackId(5)]
        );
    }

    #[test]
    fn cache_complex_filter() {
        let cache = test_cache();
        assert_eq!(
            cache
                .matches_filter(&[Is(Artist("B")), Is(Album("b")), Is(Genre("a")), Is(Year(1))])
                .tracks,
            vec![TrackId(1), TrackId(2)]
        );
    }

    #[test]
    fn cache_empty_filter() {
        let cache = test_cache();
        assert_eq!(
            cache.matches_filter(&[]).tracks,
            vec![
                TrackId(0),
                TrackId(1),
                TrackId(2),
                TrackId(3),
                TrackId(4),
                TrackId(5),
            ]
        );
    }
}
