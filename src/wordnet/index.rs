use super::pos::PoS;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

#[derive(Default, Debug)]
pub struct Index {
    /// Map from lemma to list of items which may be different
    items: HashMap<String, Vec<IndexItem>>,
}

#[derive(Debug)]
pub struct IndexItem {
    pub pos: PoS,
    pub syn_offsets: Vec<u64>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn load(&mut self, dir: &Path, s: &str) -> &[IndexItem] {
        let mut items = Vec::new();

        if let Some(i) = self.search(dir, PoS::Noun, s) {
            items.push(i)
        }

        if let Some(i) = self.search(dir, PoS::Verb, s) {
            items.push(i)
        }

        if let Some(i) = self.search(dir, PoS::Adjective, s) {
            items.push(i)
        }

        if let Some(i) = self.search(dir, PoS::Adverb, s) {
            items.push(i)
        }

        self.items.insert(s.to_string(), items);
        self.items.get(s).unwrap()
    }

    fn search(&self, dir: &Path, pos: PoS, word: &str) -> Option<IndexItem> {
        // do a binary search later
        // for now just linear
        let p = dir.join("index").with_extension(pos.as_suffix());
        let file = match File::open(p) {
            Ok(file) => file,
            Err(_) => return None,
        };
        let reader = BufReader::new(file);
        for l in reader.lines() {
            match l {
                Err(_) => continue,
                Ok(l) => {
                    let parts: Vec<_> = l.split_whitespace().collect();
                    match parts.get(0) {
                        None => continue,
                        Some(lemma) => {
                            if *lemma == word {
                                return Some(IndexItem::from_parts(&parts).unwrap());
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

impl IndexItem {
    pub fn from_parts(ps: &[&str]) -> Option<Self> {
        match ps {
            [_lemma, pos, _synset_cnt, p_cnt, rest @ ..] => {
                let p_cnt = p_cnt.parse::<usize>().unwrap();
                let rest: Vec<_> = rest.iter().skip(p_cnt).collect();
                match &rest[..] {
                    [_sense_cnt, _tagsense_cnt, rest @ ..] => {
                        let syn_offsets = rest.iter().map(|x| x.parse().unwrap()).collect();
                        Some(Self {
                            pos: PoS::try_from_str(pos).unwrap(),
                            syn_offsets,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
