use std::fmt;

/// PoS represents a part of speech
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PoS {
    Noun,
    Verb,
    Adjective,
    Adverb,
}

impl fmt::Display for PoS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PoS::Noun => "noun",
                PoS::Verb => "verb",
                PoS::Adjective => "adjective",
                PoS::Adverb => "adverb",
            }
        )
    }
}

impl PoS {
    pub fn as_suffix(&self) -> String {
        match self {
            PoS::Noun => "noun".to_string(),
            PoS::Verb => "verb".to_string(),
            PoS::Adjective => "adj".to_string(),
            PoS::Adverb => "adv".to_string(),
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "n" => Some(PoS::Noun),
            "v" => Some(PoS::Verb),
            "a" => Some(PoS::Adjective),
            "r" => Some(PoS::Adverb),
            _ => None,
        }
    }
}
