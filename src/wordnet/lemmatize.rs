// https://wordnet.princeton.edu/documentation/morphy7wn

use std::{collections::BTreeMap, fs::File, io::BufRead, path::Path};

use memmap::Mmap;

use super::{index::Index, PartOfSpeech};

pub struct Lemmatizer {
    exceptions: BTreeMap<PartOfSpeech, File>,
    maps: BTreeMap<PartOfSpeech, Mmap>,
}

impl Lemmatizer {
    pub fn new(dir: &Path) -> Self {
        let mut exception_files = BTreeMap::new();
        let mut maps = BTreeMap::new();
        for pos in PartOfSpeech::variants() {
            let file = Self::get_file(dir, pos);
            maps.insert(pos, unsafe { Mmap::map(&file).unwrap() });
            exception_files.insert(pos, file);
        }
        Self {
            exceptions: exception_files,
            maps,
        }
    }

    fn get_file(dir: &Path, pos: PartOfSpeech) -> File {
        let p = dir.join(pos.as_suffix()).with_extension("exc");
        File::open(p).unwrap()
    }

    fn exceptions_for(&self, index: &Index, word: &str, pos: PartOfSpeech) -> Vec<String> {
        // TODO: use a binary search in the mmap for this
        let mut results = Vec::new();
        for line in self.maps.get(&pos).unwrap().lines() {
            let line = line.unwrap();
            let mut parts = line.split_whitespace();
            let first = parts.next().unwrap();
            if first == word {
                for base_form in parts {
                    // not all base forms exist in word net so don't include them
                    if index.contains(base_form, pos) {
                        results.push(base_form.to_owned());
                    }
                }
            }
        }
        results
    }

    pub fn lemmatize(
        &self,
        word: &str,
        part_of_speech: PartOfSpeech,
        index: &Index,
    ) -> Vec<String> {
        match part_of_speech {
            PartOfSpeech::Noun => self.lemmatize_noun(word, index),
            PartOfSpeech::Verb => self.lemmatize_verb(word, index),
            PartOfSpeech::Adjective => self.lemmatize_adjective(word, index),
            PartOfSpeech::Adverb => self.lemmatize_adverb(word, index),
        }
    }

    fn lemmatize_noun(&self, word: &str, index: &Index) -> Vec<String> {
        let mut results = self.exceptions_for(index, word, PartOfSpeech::Noun);
        if index.contains(word, PartOfSpeech::Noun) {
            results.push(word.to_owned());
        }
        macro_rules! strip_add_search {
            ($suffix:expr, $ending:expr) => {
                if let Some(detached) = word.strip_suffix($suffix) {
                    let mut detached = detached.to_owned();
                    detached.push_str($ending);
                    if index.contains(&detached, PartOfSpeech::Noun) {
                        results.push(detached);
                    }
                }
            };
        }
        strip_add_search!("s", "");
        strip_add_search!("ses", "s");
        strip_add_search!("xes", "x");
        strip_add_search!("zes", "z");
        strip_add_search!("ches", "ch");
        strip_add_search!("shes", "sh");
        strip_add_search!("men", "man");
        strip_add_search!("ies", "y");
        results.sort_unstable();
        results.dedup();
        results
    }

    fn lemmatize_verb(&self, word: &str, index: &Index) -> Vec<String> {
        let mut results = self.exceptions_for(index, word, PartOfSpeech::Verb);
        if index.contains(word, PartOfSpeech::Verb) {
            results.push(word.to_owned());
        }
        macro_rules! strip_add_search {
            ($suffix:expr, $ending:expr) => {
                if let Some(detached) = word.strip_suffix($suffix) {
                    let mut detached = detached.to_owned();
                    detached.push_str($ending);
                    if index.contains(&detached, PartOfSpeech::Verb) {
                        results.push(detached);
                    }
                }
            };
        }
        strip_add_search!("s", "");
        strip_add_search!("ies", "y");
        strip_add_search!("es", "e");
        strip_add_search!("es", "");
        strip_add_search!("ed", "e");
        strip_add_search!("ed", "");
        strip_add_search!("ing", "e");
        strip_add_search!("ing", "");
        results.sort_unstable();
        results.dedup();
        results
    }

    fn lemmatize_adjective(&self, word: &str, index: &Index) -> Vec<String> {
        let mut results = self.exceptions_for(index, word, PartOfSpeech::Adjective);
        if index.contains(word, PartOfSpeech::Adjective) {
            results.push(word.to_owned());
        }
        macro_rules! strip_add_search {
            ($suffix:expr, $ending:expr) => {
                if let Some(detached) = word.strip_suffix($suffix) {
                    let mut detached = detached.to_owned();
                    detached.push_str($ending);
                    if index.contains(&detached, PartOfSpeech::Adjective) {
                        results.push(detached);
                    }
                }
            };
        }
        strip_add_search!("er", "");
        strip_add_search!("est", "");
        strip_add_search!("er", "e");
        strip_add_search!("est", "e");
        results.sort_unstable();
        results.dedup();
        results
    }

    fn lemmatize_adverb(&self, word: &str, index: &Index) -> Vec<String> {
        let mut results = self.exceptions_for(index, word, PartOfSpeech::Adverb);
        if index.contains(word, PartOfSpeech::Adverb) {
            results.push(word.to_owned());
        }
        results.sort_unstable();
        results.dedup();
        results
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use expect_test::{expect, Expect};

    use super::*;

    fn check(word: &str, pos: PartOfSpeech, expected: Expect) {
        let wndir = PathBuf::from(env::var("WNSEARCHDIR").unwrap());
        let index = Index::new(&wndir);
        let lemmatizer = Lemmatizer::new(&wndir);
        let shortened = lemmatizer.lemmatize(word, pos, &index);
        expected.assert_debug_eq(&shortened);
    }

    #[test]
    fn noun_s() {
        check(
            "dogs",
            PartOfSpeech::Noun,
            expect![[r#"
                [
                    "dog",
                ]
            "#]],
        );
    }

    #[test]
    fn noun_ses() {
        check(
            "classes",
            PartOfSpeech::Noun,
            expect![[r#"
                [
                    "class",
                ]
            "#]],
        );
    }

    #[test]
    fn noun_ies() {
        check(
            "families",
            PartOfSpeech::Noun,
            expect![[r#"
                [
                    "family",
                ]
            "#]],
        );
    }

    #[test]
    fn noun_axes() {
        check(
            "axes",
            PartOfSpeech::Noun,
            expect![[r#"
                [
                    "ax",
                    "axe",
                    "axis",
                ]
            "#]],
        );
    }

    #[test]
    fn lemmatize_none() {
        check(
            "unknownword",
            PartOfSpeech::Noun,
            expect![[r#"
            []
        "#]],
        );
    }

    #[test]
    fn exception_flamingoes() {
        check(
            "flamingoes",
            PartOfSpeech::Noun,
            expect![[r#"
                [
                    "flamingo",
                ]
            "#]],
        );
    }
}
