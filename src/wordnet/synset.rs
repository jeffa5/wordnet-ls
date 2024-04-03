/// SSType represents the type of a synset
#[derive(Debug, Copy, Clone)]
pub enum SynSetType {
    Noun,
    Verb,
    Adjective,
    AdjectiveSatallite,
    Adverb,
}

impl SynSetType {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "n" => Some(SynSetType::Noun),
            "v" => Some(SynSetType::Verb),
            "a" => Some(SynSetType::Adjective),
            "s" => Some(SynSetType::AdjectiveSatallite),
            "r" => Some(SynSetType::Adverb),
            _ => None,
        }
    }
}
