use super::pos::PartOfSpeech;
use super::synset::SynSetType;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

#[derive(Default, Debug)]
pub struct Data {
    /// Map from lemma to list of items which may be different
    items: HashMap<u64, Vec<DataItem>>,
}

#[derive(Debug, Clone)]
pub struct DataItem {
    synset_type: SynSetType,
    pub words: Vec<String>,
    pub gloss: String,
}

impl Data {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn load(&mut self, dir: &Path, o: u64, pos: PartOfSpeech) -> Vec<DataItem> {
        let item = self.items.get(&o);
        match item {
            Some(di) => di.clone(),
            None => {
                let mut items = Vec::new();

                if let Some(i) = self.search(dir, pos, o) {
                    items.push(i)
                }

                self.items.insert(o, items);
                self.items.get(&o).unwrap().clone()
            }
        }
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
                            synset_type: SynSetType::Noun,
                            words,
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
