/// SSType represents the type of a synset
#[derive(Debug, Copy, Clone)]
pub enum SSType {
    Noun,
    Verb,
    Adjective,
    AdjectiveSatallite,
    Adverb,
}

impl SSType {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "n" => Some(SSType::Noun),
            "v" => Some(SSType::Verb),
            "a" => Some(SSType::Adjective),
            "s" => Some(SSType::AdjectiveSatallite),
            "r" => Some(SSType::Adverb),
            _ => None,
        }
    }
}
