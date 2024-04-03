use super::pointer::PointerType;
use super::pos::PartOfSpeech;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

#[derive(Default, Debug)]
pub struct Data;

#[derive(Debug, Clone)]
pub struct DataItem {
    pub words: Vec<String>,
    pub relationships: Vec<(PointerType, u64, PartOfSpeech)>,
    pub gloss: String,
}

impl Data {
    pub fn load(&self, dir: &Path, offset: u64, pos: PartOfSpeech) -> Vec<DataItem> {
        let mut items = Vec::new();
        if let Some(i) = self.search(dir, pos, offset) {
            items.push(i)
        }
        items
    }

    fn search(&self, dir: &Path, pos: PartOfSpeech, offset: u64) -> Option<DataItem> {
        // do a binary search later
        // for now just linear
        let p = dir.join("data").with_extension(pos.as_suffix());
        let mut file = match File::open(p) {
            Ok(file) => file,
            Err(_) => return None,
        };

        if file.seek(SeekFrom::Start(offset)).is_err() {
            return None;
        };

        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let parts: Vec<_> = line.split_whitespace().collect();
        Some(DataItem::from_parts(&parts).unwrap())
    }
}

impl DataItem {
    pub fn from_parts(ps: &[&str]) -> Option<Self> {
        match ps {
            [_synset_offset, _lex_filenum, _ss_type, w_cnt, rest @ ..] => {
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
                            let pointer_type = PointerType::try_from_str(pointer_symbol).unwrap();
                            let synset_offset = synset_offset.parse::<u64>().unwrap();
                            let part_of_speech =
                                PartOfSpeech::try_from_str(part_of_speech).unwrap();
                            relationships.push((pointer_type, synset_offset, part_of_speech));
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
                            gloss,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
