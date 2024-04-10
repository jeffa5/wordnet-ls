use memmap::Mmap;

use super::pos::{PartOfSpeech, PartsOfSpeech};
use super::utils;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;

#[derive(Debug)]
pub struct Index {
    maps: PartsOfSpeech<Mmap>,
}

#[derive(Debug)]
pub struct IndexItem {
    pub pos: PartOfSpeech,
    pub syn_offsets: Vec<u64>,
}

impl Index {
    pub fn new(dir: &Path) -> std::io::Result<Self> {
        let maps = PartsOfSpeech::try_with(|pos| unsafe { Mmap::map(&Self::get_file(dir, pos)?) })?;
        Ok(Index { maps })
    }

    pub fn load(&self, word: &str) -> PartsOfSpeech<Option<IndexItem>> {
        PartsOfSpeech::with(|pos| self.search(pos, word))
    }

    pub fn contains(&self, word: &str, pos: PartOfSpeech) -> bool {
        self.search(pos, word).is_some()
    }

    fn get_file(dir: &Path, pos: PartOfSpeech) -> std::io::Result<File> {
        let p = dir.join("index").with_extension(pos.as_suffix());
        File::open(p)
    }

    fn search(&self, pos: PartOfSpeech, word: &str) -> Option<IndexItem> {
        let map = self.maps.get(pos);
        let line = utils::binary_search_file(map, word)?;
        IndexItem::from_parts(line.split_whitespace())
    }

    pub fn words_for(&self, pos: PartOfSpeech) -> Vec<String> {
        let map = self.maps.get(pos);
        let mut results = Vec::new();
        for l in map.lines() {
            match l {
                Err(_) => continue,
                Ok(l) => {
                    if l.starts_with("  ") {
                        // license part
                        continue;
                    }
                    let word = l.split_whitespace().next();
                    match word {
                        None => continue,
                        Some(lemma) => {
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
    pub fn from_parts<'a>(mut ps: impl Iterator<Item = &'a str>) -> Option<Self> {
        // line example: computer n 2 7 @ ~ #p %p + ; - 2 1 03082979 09887034
        let _lemma = ps.next()?;
        let pos = ps.next()?;
        let _synset_cnt = ps.next()?;
        let p_cnt = ps.next()?;
        let p_cnt = p_cnt.parse::<usize>().unwrap();
        let mut ps = ps.skip(p_cnt);
        let _sens_cnt = ps.next()?;
        let _tagsense_cnt = ps.next()?;
        let syn_offsets = ps.map(|x| x.parse().unwrap()).collect();
        Some(Self {
            pos: PartOfSpeech::try_from_str(pos).unwrap(),
            syn_offsets,
        })
    }
}
