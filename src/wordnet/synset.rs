use super::{relation::Relation, PartOfSpeech, WordNet};

#[derive(Debug)]
pub struct SynSet {
    /// The words for the synset, the synonyms.
    pub words: Vec<String>,
    /// Glossary entry.
    pub definition: String,
    /// What type of word it is.
    pub part_of_speech: PartOfSpeech,
    /// How it relates to other synsets.
    pub relationships: Vec<Relationship>,
}

#[derive(Debug)]
pub struct Relationship {
    /// The kind of relationship to other synsets.
    pub relation: Relation,
    /// Offset in data file for the part of speech.
    pub synset_offset: u64,
    /// File to look in.
    pub part_of_speech: PartOfSpeech,
}

impl SynSet {
    pub fn with_relationship(&self, relation: Relation) -> Vec<&Relationship> {
        self.relationships
            .iter()
            .filter(|r| r.relation == relation)
            .collect()
    }

    pub fn antonyms(&self, wn: &WordNet) -> Vec<String> {
        let mut antonyms = self
            .with_relationship(Relation::Antonym)
            .iter()
            .filter_map(|r| wn.resolve(r.part_of_speech, r.synset_offset))
            .flat_map(|s| s.words)
            .collect::<Vec<_>>();
        antonyms.sort_unstable();
        antonyms.dedup();
        antonyms
    }
}
