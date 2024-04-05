// https://wordnet.princeton.edu/documentation/morphy7wn

use std::path::Path;

use super::{index::Index, PartOfSpeech};

pub struct Lemmatizer {}

impl Lemmatizer {
    pub fn new(_dir: &Path) -> Self {
        Self {}
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
            PartOfSpeech::Adverb => Vec::new(),
        }
    }

    fn lemmatize_noun(&self, word: &str, index: &Index) -> Vec<String> {
        let mut results = Vec::new();
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
        results
    }

    fn lemmatize_verb(&self, word: &str, index: &Index) -> Vec<String> {
        let mut results = Vec::new();
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
        results
    }

    fn lemmatize_adjective(&self, word: &str, index: &Index) -> Vec<String> {
        let mut results = Vec::new();
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
                    "axe",
                    "ax",
                ]
            "#]],
        );
    }

    #[test]
    fn lemmatize_none() {
        check("unknownword", PartOfSpeech::Noun, expect![[r#"
            []
        "#]]);
    }
}
