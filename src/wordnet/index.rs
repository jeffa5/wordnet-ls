use memmap::Mmap;

use super::pos::PartOfSpeech;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

#[derive(Default, Debug)]
pub struct Index {
    #[allow(dead_code)]
    files: BTreeMap<PartOfSpeech, File>,
    mmaps: BTreeMap<PartOfSpeech, Mmap>,
}

#[derive(Debug)]
pub struct IndexItem {
    pub pos: PartOfSpeech,
    pub syn_offsets: Vec<u64>,
}

impl Index {
    pub fn new(dir: &Path) -> Self {
        let mut files = BTreeMap::new();
        let mut mmaps = BTreeMap::new();
        for pos in PartOfSpeech::iter() {
            let file = Self::get_file(dir, pos);
            mmaps.insert(pos, unsafe { Mmap::map(&file).unwrap() });
            files.insert(pos, file);
        }
        Index { files, mmaps }
    }

    pub fn load(&self, word: &str, pos: Option<PartOfSpeech>) -> Vec<IndexItem> {
        let mut items = Vec::new();

        let poses = pos
            .map(|p| vec![p])
            .unwrap_or_else(|| PartOfSpeech::iter().collect());
        for pos in poses {
            if let Some(i) = self.search(pos, word) {
                items.push(i)
            }
        }

        items
    }

    fn get_file(dir: &Path, pos: PartOfSpeech) -> File {
        let p = dir.join("index").with_extension(pos.as_suffix());
        File::open(p).unwrap()
    }

    fn search(&self, pos: PartOfSpeech, word: &str) -> Option<IndexItem> {
        let map = self.mmaps.get(&pos)?;

        let mut start = 0_usize;
        let mut end = map.len();

        let mut iword = String::new();

        while start < end {
            iword.clear();
            let mut mid = (start + end) / 2;
            // scan forwards to a newline
            while mid < end && map[mid] != b'\n' {
                mid += 1;
            }
            let line_end = mid;
            mid -= 1;
            while mid > start && map[mid] != b'\n' {
                mid -= 1;
            }
            let line_start = mid;

            // mid now points to a newline character so bump it by one to get the start of the line
            mid += 1;

            // now we extract the word from the line
            while mid < end && map[mid] != b' ' {
                iword.push(map[mid] as char);
                mid += 1;
            }
            if mid == end {
                // gone too far
                end = line_start;
                continue;
            }
            if iword.is_empty() {
                // may have been a license line
                start = line_end;
                continue;
            }

            // and check how this word compares to the one we are searching for
            match word.cmp(&iword) {
                std::cmp::Ordering::Less => {
                    end = line_start;
                }
                std::cmp::Ordering::Equal => {
                    // read the rest of the line into iword
                    while map[mid] != b'\n' {
                        iword.push(map[mid] as char);
                        mid += 1;
                    }
                    // and return the parsed parts
                    return Some(
                        IndexItem::from_parts(&iword.split_whitespace().collect::<Vec<_>>())
                            .unwrap(),
                    );
                }
                std::cmp::Ordering::Greater => {
                    start = line_end;
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
                    if l.starts_with("  ") {
                        // license part
                        continue;
                    }
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
