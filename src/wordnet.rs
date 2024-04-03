use data::Data;
use index::Index;
use pos::PartOfSpeech;
use std::collections::HashSet;
use std::path::PathBuf;

mod data;
mod index;
mod pos;
mod synset;

pub struct WordNet {
    index: Index,
    data: Data,
    database: PathBuf,
}

pub struct Definition {
    pub pos: PartOfSpeech,
    pub def: String,
}

impl WordNet {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            index: Index::new(),
            data: Data::new(),
            database: dir,
        }
    }

    pub fn definitions(&mut self, word: &str) -> Vec<Definition> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut vec = Vec::new();

        for i in items {
            for o in i.syn_offsets.iter() {
                let items = self.data.load(&self.database, *o, i.pos);
                vec.append(
                    &mut items
                        .iter()
                        .map(|x| Definition {
                            pos: i.pos,
                            def: x.gloss.clone(),
                        })
                        .collect(),
                )
            }
        }
        vec
    }

    pub fn synonyms(&mut self, word: &str) -> Vec<String> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut set: HashSet<String> = HashSet::new();

        for i in items {
            for o in i.syn_offsets.iter() {
                for i in self
                    .data
                    .load(&self.database, *o, i.pos)
                    .iter()
                    .flat_map(|x| x.words.clone())
                {
                    set.insert(i);
                }
            }
        }

        let mut vec = Vec::new();
        for s in set {
            vec.push(s.clone())
        }
        vec.sort();
        vec
    }
}
