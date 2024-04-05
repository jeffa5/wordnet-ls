use super::pos::PartOfSpeech;
use super::relation::LexicalRelation;
use super::relation::SemanticRelation;
use super::synset::Lemma;
use super::synset::LexicalRelationship;
use super::synset::SemanticRelationship;
use super::synset::SynSet;
use memmap::Mmap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufRead as _;
use std::path::Path;

#[derive(Default, Debug)]
pub struct Data {
    #[allow(dead_code)]
    files: BTreeMap<PartOfSpeech, File>,
    mmaps: BTreeMap<PartOfSpeech, Mmap>,
}

impl Data {
    pub fn new(dir: &Path) -> Self {
        let mut files = BTreeMap::new();
        let mut mmaps = BTreeMap::new();
        for pos in PartOfSpeech::iter() {
            let file = Self::get_file(dir, pos);
            mmaps.insert(pos, unsafe { Mmap::map(&file).unwrap() });
            files.insert(pos, file);
        }
        Self { files, mmaps }
    }

    /// Load a synset from the given offset in a particular part of speech file.
    pub(super) fn load(&self, offset: u64, pos: PartOfSpeech) -> Option<SynSet> {
        // do a binary search later
        // for now just linear
        let map = self.mmaps.get(&pos)?;

        let mut line = String::new();
        (&map[offset as usize..]).read_line(&mut line).unwrap();

        Some(SynSet::from_parts(line.split_whitespace()).unwrap())
    }

    fn get_file(dir: &Path, pos: PartOfSpeech) -> File {
        let p = dir.join("data").with_extension(pos.as_suffix());
        File::open(p).unwrap()
    }
}

impl SynSet {
    pub fn from_parts<'a>(mut ps: impl Iterator<Item = &'a str>) -> Option<Self> {
        let _synset_offset = ps.next()?;
        let _lex_filenum = ps.next()?;
        let ss_type = ps.next()?;
        let part_of_speech = PartOfSpeech::try_from_str(ss_type).unwrap();
        let w_cnt = ps.next()?;
        let mut w_cnt = usize::from_str_radix(w_cnt, 16).unwrap();

        let mut lemmas = Vec::new();
        while w_cnt > 0 {
            w_cnt -= 1;
            let word = ps.next()?;
            let _lex_id = ps.next()?;
            lemmas.push(Lemma {
                word: word.to_string(),
                part_of_speech,
                relationships: Vec::new(),
            });
        }

        let p_cnt = ps.next()?;
        let mut p_cnt = p_cnt.parse::<usize>().unwrap();

        let mut relationships = Vec::new();
        while p_cnt > 0 {
            p_cnt -= 1;

            let pointer_symbol = ps.next()?;
            let synset_offset = ps.next()?;
            let synset_offset = synset_offset.parse::<u64>().unwrap();
            let part_of_speech = ps.next()?;
            let part_of_speech = PartOfSpeech::try_from_str(part_of_speech).unwrap();
            let source_target = ps.next()?;
            if source_target == "0000" {
                let pointer_type = SemanticRelation::try_from_str(pointer_symbol).unwrap();
                relationships.push(SemanticRelationship {
                    relation: pointer_type,
                    synset_offset,
                    part_of_speech,
                });
            } else {
                let pointer_type = LexicalRelation::try_from_str(pointer_symbol).unwrap();
                let (source, target) = source_target.split_at(2);
                let source = usize::from_str_radix(source, 16).unwrap();
                let target = usize::from_str_radix(target, 16).unwrap();
                lemmas[source - 1].relationships.push(LexicalRelationship {
                    relation: pointer_type,
                    synset_offset,
                    part_of_speech,
                    target: target - 1,
                })
            };
        }

        let gloss = ps
            .skip_while(|x| *x != "|")
            .skip(1)
            .fold(String::new(), |mut a, s| {
                a.push_str(s);
                a.push(' ');
                a
            })
            .trim()
            .to_string();
        Some(Self {
            lemmas,
            relationships,
            definition: gloss,
            part_of_speech,
        })
    }
}
