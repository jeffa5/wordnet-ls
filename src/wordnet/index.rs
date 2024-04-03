use super::pos::PartOfSpeech;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

#[derive(Default, Debug)]
pub struct Index;

#[derive(Debug)]
pub struct IndexItem {
    pub pos: PartOfSpeech,
    pub syn_offsets: Vec<u64>,
}

impl Index {
    pub fn load(&self, dir: &Path, word: &str) -> Vec<IndexItem> {
        let mut items = Vec::new();

        for pos in [
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
        ] {
            if let Some(i) = self.search(dir, pos, word) {
                items.push(i)
            }
        }

        items
    }

    fn search(&self, dir: &Path, pos: PartOfSpeech, word: &str) -> Option<IndexItem> {
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
                    match parts.first() {
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

    pub fn words_for(&self, dir: &Path, pos: PartOfSpeech) -> Vec<String> {
        let p = dir.join("index").with_extension(pos.as_suffix());
        let file = match File::open(p) {
            Ok(file) => file,
            Err(_) => return Vec::new(),
        };
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        for l in reader.lines() {
            match l {
                Err(_) => continue,
                Ok(l) => {
                    let parts: Vec<_> = l.split_whitespace().collect();
                    match parts.first() {
                        None => continue,
                        Some(&lemma) => {
                            results.push(lemma.to_owned());
                        }
                    }
                }
            }
        }
        results.sort_unstable();
        results
    }
}

impl IndexItem {
    pub fn from_parts(ps: &[&str]) -> Option<Self> {
        // line example: computer n 2 7 @ ~ #p %p + ; - 2 1 03082979 09887034
        match ps {
            [_lemma, pos, _synset_cnt, p_cnt, rest @ ..] => {
                let p_cnt = p_cnt.parse::<usize>().unwrap();
                let rest: Vec<_> = rest.iter().skip(p_cnt).collect();
                match &rest[..] {
                    [_sense_cnt, _tagsense_cnt, rest @ ..] => {
                        let syn_offsets = rest.iter().map(|x| x.parse().unwrap()).collect();
                        Some(Self {
                            pos: PartOfSpeech::try_from_str(pos).unwrap(),
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
