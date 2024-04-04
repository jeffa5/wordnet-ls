use std::fmt;

/// PoS represents a part of speech
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
}

impl fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PartOfSpeech::Noun => "noun",
                PartOfSpeech::Verb => "verb",
                PartOfSpeech::Adjective => "adjective",
                PartOfSpeech::Adverb => "adverb",
            }
        )
    }
}

impl PartOfSpeech {
    pub fn as_suffix(&self) -> String {
        match self {
            PartOfSpeech::Noun => "noun".to_string(),
            PartOfSpeech::Verb => "verb".to_string(),
            PartOfSpeech::Adjective => "adj".to_string(),
            PartOfSpeech::Adverb => "adv".to_string(),
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "n" => Some(PartOfSpeech::Noun),
            "v" => Some(PartOfSpeech::Verb),
            "a" => Some(PartOfSpeech::Adjective),
            // not strictly a part of speech but a ss_type (synset type) but it fits here well
            // enough for now
            "s" => Some(PartOfSpeech::Adjective),
            "r" => Some(PartOfSpeech::Adverb),
            _ => None,
        }
    }

    pub fn iter() -> impl Iterator<Item = PartOfSpeech> {
        [
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
        ]
        .into_iter()
    }
}
