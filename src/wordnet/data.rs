use super::relation::Relation;
use super::pos::PartOfSpeech;
use super::synset::Relationship;
use super::synset::SynSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

#[derive(Default, Debug)]
pub struct Data;

impl Data {
    /// Load a synset from the given offset in a particular part of speech file.
    pub(super) fn load(&self, dir: &Path, offset: u64, pos: PartOfSpeech) -> Option<SynSet> {
        // do a binary search later
        // for now just linear
        let p = dir.join("data").with_extension(pos.as_suffix());
        let mut file = File::open(p).ok()?;

        if file.seek(SeekFrom::Start(offset)).is_err() {
            return None;
        };

        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let parts: Vec<_> = line.split_whitespace().collect();
        Some(SynSet::from_parts(&parts).unwrap())
    }
}

impl SynSet {
    pub fn from_parts(ps: &[&str]) -> Option<Self> {
        match ps {
            [_synset_offset, _lex_filenum, ss_type, w_cnt, rest @ ..] => {
                let w_cnt = usize::from_str_radix(w_cnt, 16).unwrap();
                let word_lex_id = &rest[..w_cnt * 2];
                let mut words = Vec::new();
                for (i, w) in word_lex_id.iter().enumerate() {
                    if i % 2 == 0 {
                        words.push(w.to_string())
                    }
                }
                let rest: Vec<_> = rest.iter().skip(w_cnt * 2).collect();
                match &rest[..] {
                    [p_cnt, rest @ ..] => {
                        let p_cnt = p_cnt.parse::<usize>().unwrap();
                        let pointers = rest.iter().take(p_cnt * 4).collect::<Vec<_>>();
                        let mut relationships = Vec::new();
                        for chunk in pointers.chunks(4) {
                            let [pointer_symbol, synset_offset, part_of_speech, _source_target] =
                                chunk
                            else {
                                panic!("invalid chunk")
                            };
                            let pointer_type = Relation::try_from_str(pointer_symbol).unwrap();
                            let synset_offset = synset_offset.parse::<u64>().unwrap();
                            let part_of_speech =
                                PartOfSpeech::try_from_str(part_of_speech).unwrap();
                            relationships.push(Relationship {
                                relation: pointer_type,
                                synset_offset,
                                part_of_speech,
                            });
                        }
                        let rest: Vec<_> = rest.iter().skip(p_cnt * 4).collect();
                        let gloss = rest
                            .iter()
                            .map(|x| ***x)
                            .skip_while(|x| *x != "|")
                            .skip(1)
                            .fold(String::new(), |a, s| format!("{} {}", a, s))
                            .trim()
                            .to_string();
                        Some(Self {
                            words,
                            relationships,
                            definition: gloss,
                            part_of_speech: PartOfSpeech::try_from_str(ss_type).unwrap(),
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
