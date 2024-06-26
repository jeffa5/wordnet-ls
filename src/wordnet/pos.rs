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

    pub fn variants() -> [PartOfSpeech; 4] {
        [
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
        ]
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

pub struct PartsOfSpeech<T> {
    pub noun: T,
    pub verb: T,
    pub adjective: T,
    pub adverb: T,
}

impl<T: std::fmt::Debug> std::fmt::Debug for PartsOfSpeech<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PartsOfSpeech")
            .field("noun", &self.noun)
            .field("verb", &self.verb)
            .field("adjective", &self.adjective)
            .field("adverb", &self.adverb)
            .finish()
    }
}

impl<T> PartsOfSpeech<T> {
    pub fn with(mut f: impl FnMut(PartOfSpeech) -> T) -> Self {
        Self {
            noun: f(PartOfSpeech::Noun),
            verb: f(PartOfSpeech::Verb),
            adjective: f(PartOfSpeech::Adjective),
            adverb: f(PartOfSpeech::Adverb),
        }
    }

    pub fn try_with<E>(mut f: impl FnMut(PartOfSpeech) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            noun: f(PartOfSpeech::Noun)?,
            verb: f(PartOfSpeech::Verb)?,
            adjective: f(PartOfSpeech::Adjective)?,
            adverb: f(PartOfSpeech::Adverb)?,
        })
    }

    pub fn any(&self, mut f: impl FnMut(&T) -> bool) -> bool {
        f(&self.noun) || f(&self.verb) || f(&self.adjective) || f(&self.adverb)
    }

    pub fn all(&self, mut f: impl FnMut(&T) -> bool) -> bool {
        f(&self.noun) && f(&self.verb) && f(&self.adjective) && f(&self.adverb)
    }

    pub fn map<U>(self, mut f: impl FnMut(PartOfSpeech, T) -> U) -> PartsOfSpeech<U> {
        PartsOfSpeech {
            noun: f(PartOfSpeech::Noun, self.noun),
            verb: f(PartOfSpeech::Verb, self.verb),
            adjective: f(PartOfSpeech::Adjective, self.adjective),
            adverb: f(PartOfSpeech::Adverb, self.adverb),
        }
    }

    pub fn for_each(self, mut f: impl FnMut(PartOfSpeech, T)) {
        f(PartOfSpeech::Noun, self.noun);
        f(PartOfSpeech::Verb, self.verb);
        f(PartOfSpeech::Adjective, self.adjective);
        f(PartOfSpeech::Adverb, self.adverb);
    }

    pub fn get(&self, pos: PartOfSpeech) -> &T {
        match pos {
            PartOfSpeech::Noun => &self.noun,
            PartOfSpeech::Verb => &self.verb,
            PartOfSpeech::Adjective => &self.adjective,
            PartOfSpeech::Adverb => &self.adverb,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.noun, &self.verb, &self.adjective, &self.adverb].into_iter()
    }
}
