use super::{
    relation::{LexicalRelation, SemanticRelation},
    PartOfSpeech, WordNet,
};

#[derive(Debug)]
pub struct SynSet {
    /// Lemmas within the synset.
    pub lemmas: Vec<Lemma>,
    /// Glossary entry.
    pub definition: String,
    /// Example uses.
    pub examples: Vec<String>,
    /// What type of word it is.
    pub part_of_speech: PartOfSpeech,
    /// How it relates to other synsets.
    pub relationships: Vec<SemanticRelationship>,
}

#[derive(Debug)]
pub struct SemanticRelationship {
    /// The kind of relationship to other synsets.
    pub relation: SemanticRelation,
    /// Offset in data file for the part of speech.
    pub synset_offset: u64,
    /// File to look in.
    pub part_of_speech: PartOfSpeech,
}

#[derive(Debug)]
pub struct LexicalRelationship {
    /// The kind of relationship to other synsets.
    pub relation: LexicalRelation,
    /// Offset in data file for the part of speech.
    pub synset_offset: u64,
    /// File to look in.
    pub part_of_speech: PartOfSpeech,
    /// Word index in target synset.
    pub target: usize,
}

impl SynSet {
    pub fn with_relationship(&self, relation: SemanticRelation) -> Vec<&SemanticRelationship> {
        self.relationships
            .iter()
            .filter(|r| r.relation == relation)
            .collect()
    }

    pub fn synonyms(&self) -> Vec<String> {
        self.lemmas.iter().map(|l| l.word.to_owned()).collect()
    }
}

#[derive(Debug)]
pub struct Lemma {
    pub word: String,
    pub part_of_speech: PartOfSpeech,
    /// Lexical relationships with other synsets.
    pub relationships: Vec<LexicalRelationship>,
}

impl Lemma {
    pub fn with_relationship(&self, relation: LexicalRelation) -> Vec<&LexicalRelationship> {
        self.relationships
            .iter()
            .filter(|r| r.relation == relation)
            .collect()
    }

    pub fn antonyms(&self, wn: &WordNet) -> Vec<String> {
        let mut antonyms = self
            .with_relationship(LexicalRelation::Antonym)
            .iter()
            .map(|r| {
                (
                    r.target,
                    wn.resolve(r.part_of_speech, r.synset_offset)
                        .expect("Failed to resolve word from lemma relationship"),
                )
            })
            .map(|(target, mut ss)| ss.lemmas.remove(target).word)
            .collect::<Vec<_>>();
        antonyms.sort_unstable();
        antonyms.dedup();
        antonyms
    }
}
