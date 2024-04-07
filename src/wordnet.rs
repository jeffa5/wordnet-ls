use data::Data;
use index::Index;
pub use pos::PartOfSpeech;
use rayon::prelude::*;
pub use relation::LexicalRelation;
pub use relation::SemanticRelation;
use std::path::Path;
pub use synset::SynSet;

use self::lemmatize::Lemmatizer;
use self::pos::PartsOfSpeech;

mod data;
mod index;
mod lemmatize;
mod pos;
mod relation;
mod synset;
mod utils;

pub struct WordNet {
    index: Index,
    data: Data,
    lemmatizer: Lemmatizer,
}

impl WordNet {
    pub fn new(dir: &Path) -> Self {
        Self {
            index: Index::new(dir),
            data: Data::new(dir),
            lemmatizer: Lemmatizer::new(dir),
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        PartOfSpeech::variants()
            .into_iter()
            .any(|pos| self.index.contains(word, pos))
    }

    /// Directly resolve a reference, this should only be used with part_of_speech, offset pairs
    /// from the returned results, such as the relationships in synsets.
    pub fn resolve(&self, part_of_speech: PartOfSpeech, offset: u64) -> Option<SynSet> {
        self.data.load(offset, part_of_speech)
    }

    pub fn all_words(&self) -> Vec<String> {
        let mut result = Vec::new();
        result.par_extend(
            PartOfSpeech::variants()
                .into_par_iter()
                .flat_map(|pos| self.index.words_for(pos)),
        );
        result.par_sort_unstable();
        result.dedup();
        result
    }

    pub fn lemmatize(&self, word: &str) -> PartsOfSpeech<Vec<String>> {
        PartsOfSpeech::with(|pos| {
            let mut lemmas = self.lemmatizer.lemmatize(word, pos, &self.index);
            lemmas.sort_unstable();
            lemmas.dedup();
            lemmas
        })
    }

    pub fn lemmatize_for(&self, word: &str, pos: PartOfSpeech) -> Vec<String> {
        self.lemmatizer.lemmatize(word, pos, &self.index)
    }

    pub fn synsets(&self, word: &str) -> Vec<SynSet> {
        let word = word.to_lowercase();
        let items = self.index.load(&word, None);
        let mut synsets = Vec::new();

        for item in items {
            for offset in item.syn_offsets.iter() {
                let synset = self.data.load(*offset, item.pos);
                if let Some(synset) = synset {
                    synsets.push(synset);
                }
            }
        }

        synsets
    }

    pub fn synsets_for(&self, word: &str, pos: PartOfSpeech) -> Vec<SynSet> {
        let word = word.to_lowercase();
        let items = self.index.load(&word, Some(pos));
        let mut synsets = Vec::new();

        for item in items {
            for offset in item.syn_offsets.iter() {
                let synset = self.data.load(*offset, item.pos);
                if let Some(synset) = synset {
                    synsets.push(synset);
                }
            }
        }

        synsets
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, env, path::PathBuf};

    use super::*;
    use expect_test::expect;

    #[test]
    fn multipos_data_definition() {
        let word = "run";
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let def = wn
            .synsets(word)
            .into_iter()
            .map(|ss| ss.definition)
            .collect::<Vec<_>>();
        let expected = expect![[r#"
            [
                "a score in baseball made by a runner touching all four bases safely",
                "the act of testing something",
                "a race run on foot",
                "an unbroken series of events",
                "(American football) a play in which a player attempts to carry the ball through or past the opposing team",
                "a regular trip",
                "the act of running; traveling on foot at a fast pace",
                "the continuous period of time during which something (a machine or a factory) operates or continues in operation",
                "unrestricted freedom to use",
                "the production achieved during a continuous period of operation (of a machine or factory etc.)",
                "a small stream",
                "a race between candidates for elective office",
                "a row of unravelled stitches",
                "the pouring forth of a fluid",
                "an unbroken chronological sequence",
                "a short trip",
                "move fast by using one's feet, with one foot off the ground at any given time",
                "flee; take to one's heels; cut and run",
                "stretch out over a distance, space, time, or scope; run or extend between two points or beyond a certain point",
                "direct or control; projects, businesses, etc.",
                "have a particular form",
                "move along, of liquids",
                "perform as expected when applied",
                "change or be different within limits",
                "run, stand, or compete for an office or a position",
                "cause to emit recorded audio or video",
                "move about freely and without restraint, or act as if running around in an uncontrolled way",
                "have a tendency or disposition to do or be something; be inclined",
                "be operating, running or functioning",
                "change from one state to another",
                "cause to perform",
                "be affected by; be subjected to",
                "continue to exist",
                "occur persistently",
                "carry out a process or program, as on a computer or a machine",
                "include as the content; broadcast or publicize",
                "carry out",
                "pass over, across, or through",
                "cause something to pass or lead somewhere",
                "make without a miss",
                "deal in illegally, such as arms or liquor",
                "cause an animal to move fast",
                "be diffused",
                "sail before the wind",
                "cover by running; run a certain distance",
                "extend or continue for a certain period of time",
                "set animals loose to graze",
                "keep company",
                "run with the ball; in such sports as football",
                "travel rapidly, by any (unspecified) means",
                "travel a route regularly",
                "pursue for food or sport (as of wild animals)",
                "compete in a race",
                "progress by being changed",
                "reduce or cause to be reduced from a solid to a liquid state, usually by heating",
                "come unraveled or undone as if by snagging",
                "become undone",
            ]
        "#]];
        expected.assert_debug_eq(&def);
    }

    #[test]
    fn multipos_data_examples() {
        let word = "run";
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let def = wn
            .synsets(word)
            .into_iter()
            .map(|ss| ss.examples)
            .collect::<Vec<_>>();
        let expected = expect![[r#"
            [
                [
                    "the Yankees scored 3 runs in the bottom of the 9th",
                    "their first tally came in the 3rd inning",
                ],
                [
                    "in the experimental trials the amount of carbon was measured separately",
                    "he called each flip of the coin a new trial",
                ],
                [
                    "she broke the record for the half-mile run",
                ],
                [
                    "had a streak of bad luck",
                    "Nicklaus had a run of birdies",
                ],
                [
                    "the defensive line braced to stop the run",
                    "the coach put great emphasis on running",
                ],
                [
                    "the ship made its run in record time",
                ],
                [
                    "he broke into a run",
                    "his daily run keeps him fit",
                ],
                [
                    "the assembly line was on a 12-hour run",
                ],
                [
                    "he has the run of the house",
                ],
                [
                    "a daily run of 100,000 gallons of paint",
                ],
                [],
                [
                    "I managed his campaign for governor",
                    "he is raising money for a Senate run",
                ],
                [
                    "she got a run in her stocking",
                ],
                [],
                [
                    "the play had a long run on Broadway",
                    "the team enjoyed a brief run of victories",
                ],
                [
                    "take a run into town",
                ],
                [
                    "Don't run--you'll be out of breath",
                    "The children ran to the store",
                ],
                [
                    "If you see this man, run!",
                    "The burglars escaped before the police showed up",
                ],
                [
                    "Service runs all the way to Cranbury",
                    "His knowledge doesn't go very far",
                    "My memory extends back to my fourth year of life",
                    "The facts extend beyond a consideration of her personal assets",
                ],
                [
                    "She is running a relief operation in the Sudan",
                ],
                [
                    "the story or argument runs as follows",
                    "as the saying goes...",
                ],
                [
                    "Water flowed into the cave",
                    "the Missouri feeds into the Mississippi",
                ],
                [
                    "The washing machine won't go unless it's plugged in",
                    "Does this old car still run well?",
                    "This old radio doesn't work anymore",
                ],
                [
                    "Estimates for the losses in the earthquake range as high as $2 billion",
                    "Interest rates run from 5 to 10 percent",
                    "The instruments ranged from tuba to cymbals",
                    "My students range from very bright to dull",
                ],
                [
                    "Who's running for treasurer this year?",
                ],
                [
                    "They ran the tapes over and over again",
                    "I'll play you my favorite record",
                    "He never tires of playing that video",
                ],
                [
                    "who are these people running around in the building?",
                    "She runs around telling everyone of her troubles",
                    "let the dogs run free",
                ],
                [
                    "She tends to be nervous before her lectures",
                    "These dresses run small",
                    "He inclined to corpulence",
                ],
                [
                    "The car is still running--turn it off!",
                ],
                [
                    "run amok",
                    "run rogue",
                    "run riot",
                ],
                [
                    "run a subject",
                    "run a process",
                ],
                [
                    "run a temperature",
                    "run a risk",
                ],
                [
                    "These stories die hard",
                    "The legend of Elvis endures",
                ],
                [
                    "Musical talent runs in the family",
                ],
                [
                    "Run the dishwasher",
                    "run a new program on the Mac",
                    "the computer executed the instruction",
                ],
                [
                    "We ran the ad three times",
                    "This paper carries a restaurant review",
                    "All major networks carried the press conference",
                ],
                [
                    "run an errand",
                ],
                [
                    "He ran his eyes over her body",
                    "She ran her fingers along the carved figurine",
                    "He drew her hair through his fingers",
                ],
                [
                    "Run the wire behind the cabinet",
                ],
                [],
                [],
                [
                    "run the dogs",
                ],
                [
                    "These dyes and colors are guaranteed not to run",
                ],
                [],
                [
                    "She ran 10 miles that day",
                ],
                [
                    "The film runs 5 hours",
                ],
                [],
                [
                    "the heifers run with the bulls to produce offspring",
                ],
                [],
                [
                    "Run to the store!",
                    "She always runs to Italy, because she has a lover there",
                ],
                [
                    "Ships ply the waters near the coast",
                ],
                [
                    "Goering often hunted wild boars in Poland",
                    "The dogs are running deer",
                    "The Duke hunted in these woods",
                ],
                [
                    "he is running the Marathon this year",
                    "let's race and see who gets there first",
                ],
                [
                    "The speech has to go through several more drafts",
                    "run through your presentation before the meeting",
                ],
                [
                    "melt butter",
                    "melt down gold",
                    "The wax melted in the sun",
                ],
                [
                    "Her nylons were running",
                ],
                [
                    "the sweater unraveled",
                ],
            ]
        "#]];
        expected.assert_debug_eq(&def);
    }

    #[test]
    fn multipos_data_synonyms() {
        let word = "run";
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let mut syn = wn
            .synsets(word)
            .into_iter()
            .flat_map(|ss| {
                ss.synonyms()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        syn.sort();
        syn.dedup();
        let expected = expect![[r#"
            [
                "be_given",
                "black_market",
                "bleed",
                "break_away",
                "bunk",
                "campaign",
                "carry",
                "consort",
                "course",
                "die_hard",
                "discharge",
                "draw",
                "endure",
                "escape",
                "execute",
                "extend",
                "feed",
                "flow",
                "fly_the_coop",
                "foot_race",
                "footrace",
                "function",
                "go",
                "guide",
                "head_for_the_hills",
                "hightail_it",
                "hunt",
                "hunt_down",
                "incline",
                "ladder",
                "lam",
                "lead",
                "lean",
                "melt",
                "melt_down",
                "move",
                "operate",
                "outpouring",
                "pass",
                "persist",
                "play",
                "ply",
                "political_campaign",
                "prevail",
                "race",
                "range",
                "ravel",
                "rill",
                "rivulet",
                "run",
                "run_away",
                "run_for",
                "runnel",
                "running",
                "running_game",
                "running_play",
                "scarper",
                "scat",
                "streak",
                "streamlet",
                "take_to_the_woods",
                "tally",
                "tend",
                "test",
                "track_down",
                "trial",
                "turn_tail",
                "unravel",
                "work",
            ]
        "#]];
        expected.assert_debug_eq(&syn);
    }

    #[test]
    fn woman_data_synset() {
        let word = "woman";
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let syn = wn.synsets(word);
        let expected = expect![[r#"
            [
                SynSet {
                    lemmas: [
                        Lemma {
                            word: "woman",
                            part_of_speech: Noun,
                            relationships: [
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 1484987,
                                    part_of_speech: Adjective,
                                    target: 0,
                                },
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 14425715,
                                    part_of_speech: Noun,
                                    target: 0,
                                },
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 8477634,
                                    part_of_speech: Noun,
                                    target: 0,
                                },
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 606006,
                                    part_of_speech: Noun,
                                    target: 0,
                                },
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 566322,
                                    part_of_speech: Verb,
                                    target: 4,
                                },
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 2590910,
                                    part_of_speech: Verb,
                                    target: 2,
                                },
                                LexicalRelationship {
                                    relation: Antonym,
                                    synset_offset: 10287213,
                                    part_of_speech: Noun,
                                    target: 0,
                                },
                            ],
                        },
                        Lemma {
                            word: "adult_female",
                            part_of_speech: Noun,
                            relationships: [],
                        },
                    ],
                    definition: "an adult female person (as opposed to a man)",
                    examples: [
                        "the woman kept house while the man hunted",
                    ],
                    part_of_speech: Noun,
                    relationships: [
                        SemanticRelationship {
                            relation: Hypernym,
                            synset_offset: 9619168,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hypernym,
                            synset_offset: 9605289,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: PartMeronym,
                            synset_offset: 5220126,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: InstanceHyponym,
                            synset_offset: 9586743,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9637339,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9641130,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9643670,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9787293,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9787390,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9830080,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9832456,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9834258,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9852430,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9861599,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9874862,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9875663,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9900153,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9923263,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9965134,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9989290,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 9997834,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10020366,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10020533,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10024784,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10025635,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10055410,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10075063,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10122858,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10129825,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10130447,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10130686,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10130877,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10136283,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10144838,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10173410,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10202085,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10222170,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10222259,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10243137,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10280034,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10302576,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10302700,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10303186,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10311661,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10323752,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10333044,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10345100,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10366145,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10368528,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10368624,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10377021,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10485440,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10589243,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10613996,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10685398,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10739512,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10748804,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10761962,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10771066,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10780284,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10780632,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: Hyponym,
                            synset_offset: 10789820,
                            part_of_speech: Noun,
                        },
                    ],
                },
                SynSet {
                    lemmas: [
                        Lemma {
                            word: "woman",
                            part_of_speech: Noun,
                            relationships: [
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 2590910,
                                    part_of_speech: Verb,
                                    target: 1,
                                },
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 2590910,
                                    part_of_speech: Verb,
                                    target: 2,
                                },
                                LexicalRelationship {
                                    relation: Antonym,
                                    synset_offset: 10288516,
                                    part_of_speech: Noun,
                                    target: 0,
                                },
                            ],
                        },
                    ],
                    definition: "a female person who plays a significant role (wife or mistress or girlfriend) in the life of a particular man",
                    examples: [
                        "he was faithful to his woman",
                    ],
                    part_of_speech: Noun,
                    relationships: [
                        SemanticRelationship {
                            relation: Hypernym,
                            synset_offset: 9619168,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: DomainOfSynsetUsage,
                            synset_offset: 7075172,
                            part_of_speech: Noun,
                        },
                    ],
                },
                SynSet {
                    lemmas: [
                        Lemma {
                            word: "charwoman",
                            part_of_speech: Noun,
                            relationships: [],
                        },
                        Lemma {
                            word: "char",
                            part_of_speech: Noun,
                            relationships: [],
                        },
                        Lemma {
                            word: "cleaning_woman",
                            part_of_speech: Noun,
                            relationships: [],
                        },
                        Lemma {
                            word: "cleaning_lady",
                            part_of_speech: Noun,
                            relationships: [],
                        },
                        Lemma {
                            word: "woman",
                            part_of_speech: Noun,
                            relationships: [],
                        },
                    ],
                    definition: "a human female employed to do housework",
                    examples: [
                        "the char will clean the carpet",
                        "I have a woman who comes in four hours a day while I write",
                    ],
                    part_of_speech: Noun,
                    relationships: [
                        SemanticRelationship {
                            relation: Hypernym,
                            synset_offset: 9927089,
                            part_of_speech: Noun,
                        },
                    ],
                },
                SynSet {
                    lemmas: [
                        Lemma {
                            word: "womanhood",
                            part_of_speech: Noun,
                            relationships: [
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 10787470,
                                    part_of_speech: Noun,
                                    target: 0,
                                },
                            ],
                        },
                        Lemma {
                            word: "woman",
                            part_of_speech: Noun,
                            relationships: [
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 14425715,
                                    part_of_speech: Noun,
                                    target: 0,
                                },
                                LexicalRelationship {
                                    relation: DerivationallyRelatedForm,
                                    synset_offset: 606006,
                                    part_of_speech: Noun,
                                    target: 0,
                                },
                            ],
                        },
                        Lemma {
                            word: "fair_sex",
                            part_of_speech: Noun,
                            relationships: [],
                        },
                    ],
                    definition: "women as a class",
                    examples: [
                        "it's an insult to American womanhood",
                        "woman is the glory of creation",
                        "the fair sex gathered on the veranda",
                    ],
                    part_of_speech: Noun,
                    relationships: [
                        SemanticRelationship {
                            relation: Hypernym,
                            synset_offset: 7974025,
                            part_of_speech: Noun,
                        },
                        SemanticRelationship {
                            relation: MemberHolonym,
                            synset_offset: 8477912,
                            part_of_speech: Noun,
                        },
                    ],
                },
            ]
        "#]];
        expected.assert_debug_eq(&syn);
    }

    #[test]
    fn woman_data_synset_resolve() {
        let word = "woman";
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let syn = wn.synsets(word);
        let resolved_related = syn
            .iter()
            .flat_map(|s| {
                s.relationships.iter().filter_map(|r| {
                    wn.resolve(r.part_of_speech, r.synset_offset)
                        .map(|s| (r.relation, s))
                })
            })
            .collect::<Vec<_>>();
        let expected = expect![[r#"
            [
                (
                    Hypernym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "female",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1484451,
                                        part_of_speech: Adjective,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: Antonym,
                                        synset_offset: 9624168,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "female_person",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a person who belongs to the sex that can have babies",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 7846,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: PartMeronym,
                                synset_offset: 5219923,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10084043,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10084295,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10106995,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10131151,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hypernym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "adult",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1488616,
                                        part_of_speech: Adjective,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 15152817,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 14425103,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: Antonym,
                                        synset_offset: 9622049,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "grownup",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1488616,
                                        part_of_speech: Adjective,
                                        target: 5,
                                    },
                                ],
                            },
                        ],
                        definition: "a fully developed person from maturity onward",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 7846,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: PartMeronym,
                                synset_offset: 5219561,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9872464,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9895561,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9900981,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9904837,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9909060,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9957156,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10024025,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10048218,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10074249,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10187130,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10200781,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10218164,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10256756,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10257084,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10287213,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10316013,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10376523,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10390199,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10407105,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10441534,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10441694,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10480253,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10618146,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10625285,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10653388,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10658867,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10709358,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    PartMeronym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "adult_female_body",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "woman's_body",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "the body of an adult woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 5219561,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 5219923,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: PartHolonym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: PartMeronym,
                                synset_offset: 5402576,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: PartMeronym,
                                synset_offset: 5554405,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    InstanceHyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "Eve",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "(Old Testament) Adam's wife in Judeo-Christian mythology: the first woman and mother of the human race; God created Eve from Adam's rib and placed Adam and Eve in the Garden of Eden",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: InstanceHypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: DomainOfSynsetTopic,
                                synset_offset: 6449735,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "Black_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who is Black",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 9636339,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "white_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who is White",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 9638875,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "yellow_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "offensive term for an Asian woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 9643078,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: DomainOfSynsetUsage,
                                synset_offset: 6717170,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "amazon",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "virago",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a large strong and aggressive woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "maenad",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "(Greek mythology) a woman participant in the orgiastic rites of Dionysus",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: DomainOfSynsetTopic,
                                synset_offset: 7979425,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "bachelor_girl",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "bachelorette",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a young unmarried woman who lives alone",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "baggage",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a worthless or immoral woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "ball-buster",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "ball-breaker",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a demanding woman who destroys men's confidence",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "B-girl",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "bar_girl",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman employed by a bar to act as a companion to men customers",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "bluestocking",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "bas_bleu",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman having literary or intellectual interests",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "bridesmaid",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "maid_of_honor",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "an unmarried woman who attends the bride at a wedding",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 9821831,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberHolonym,
                                synset_offset: 8256735,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "broad",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "slang term for a woman",
                        examples: [
                            "a broad is a woman who can throw a mean punch",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "cat",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 225912,
                                        part_of_speech: Adjective,
                                        target: 1,
                                    },
                                ],
                            },
                        ],
                        definition: "a spiteful woman gossip",
                        examples: [
                            "what a cat she is!",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10139347,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "Cinderella",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman whose merits were not been recognized but who then achieves sudden success and recognition",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "coquette",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1037910,
                                        part_of_speech: Verb,
                                        target: 5,
                                    },
                                ],
                            },
                            Lemma {
                                word: "flirt",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 2131958,
                                        part_of_speech: Adjective,
                                        target: 1,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1037910,
                                        part_of_speech: Verb,
                                        target: 1,
                                    },
                                ],
                            },
                            Lemma {
                                word: "vamp",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1038538,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "vamper",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1038538,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "minx",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "tease",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 852506,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1803641,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "prickteaser",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a seductive woman who uses her sex appeal to exploit men",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "dame",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "madam",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "ma'am",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "lady",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "gentlewoman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman of refinement",
                        examples: [
                            "a chauffeur opened the door of the limousine for the grand lady",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10142166,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10279778,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "debutante",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "deb",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a young woman making her debut into society",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "divorcee",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 2490634,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "grass_widow",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a divorced woman or a woman who is separated from her husband",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 11301809,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "ex-wife",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "ex",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who was formerly a particular man's wife",
                        examples: [
                            "all his exes live in Texas",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "dominatrix",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a dominating woman (especially one who plays that role in a sadomasochistic sexual relationship)",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "donna",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "an Italian woman of rank",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: DomainOfSynsetTopic,
                                synset_offset: 6964247,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "enchantress",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "temptress",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "siren",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "Delilah",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "femme_fatale",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who is considered to be dangerously seductive",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "eyeful",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a strikingly beautiful woman",
                        examples: [
                            "she was a statuesque redheaded eyeful",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "geisha",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "geisha_girl",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a Japanese woman trained to entertain men with conversation and singing and dancing",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 9718217,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "girl",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 15147330,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "miss",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "missy",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "young_lady",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "young_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "fille",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a young woman",
                        examples: [
                            "a young lady of 18",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9827363,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9849012,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9854708,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9905530,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9919451,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9936825,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9989045,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10095420,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10117851,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10119609,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10129338,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10247358,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10282482,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10304160,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10318193,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10403366,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10416364,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10531694,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10539160,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10584729,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10592049,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10626994,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10682599,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10715030,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10745770,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10791115,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "girl",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a friendly informal reference to a grown woman",
                        examples: [
                            "Mrs. Smith was just one of the girls",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "girlfriend",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "girl",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "lady_friend",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a girl or young woman with whom a man is romantically involved",
                        examples: [
                            "his girlfriend kicked him out",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 9622302,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "girlfriend",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "any female friend",
                        examples: [
                            "Mary and her girlfriend organized the party",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10112591,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "gold_digger",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who associates with or marries a rich man in order to get valuables from him through gifts or a divorce settlement",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "gravida",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a pregnant woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10472129,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10574723,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10703221,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "heroine",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman possessing heroic qualities or a woman who has performed heroic deeds",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 10226219,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 11168218,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "inamorata",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman with whom you are in love or have an intimate relationship",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 9622302,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "jezebel",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a shameless impudent scheming woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "jilt",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 613248,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                        ],
                        definition: "a woman who jilts a lover",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "lady",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a polite name for any woman",
                        examples: [
                            "a nice lady at the library helped me",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9847425,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "maenad",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "an unnaturally frenzied or distraught woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "matriarch",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "materfamilias",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a female head of a family or tribe",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10164605,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "matriarch",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a feisty older woman with a big bosom (as drawn in cartoons)",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "matron",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman in charge of nursing in a medical institution",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10366966,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "mestiza",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman of mixed racial ancestry (especially mixed European and Native American ancestry)",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: DomainOfSynsetRegion,
                                synset_offset: 9044862,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "mistress",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "kept_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "fancy_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "an adulterous woman; a woman who has an ongoing extramarital sexual relationship with a man",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 9622745,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9952393,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 10863440,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 10929116,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "mother_figure",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who evokes the feelings usually reserved for a mother",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "nanny",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "nursemaid",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "nurse",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1186428,
                                        part_of_speech: Verb,
                                        target: 3,
                                    },
                                ],
                            },
                        ],
                        definition: "a woman who is the custodian of children",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10229498,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10038119,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10287082,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10774870,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "nullipara",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "(obstetrics) a woman who has never give birth to a child",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: DomainOfSynsetTopic,
                                synset_offset: 6053439,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "nymph",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "houri",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a voluptuously beautiful young woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "nymphet",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a sexually attractive young woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "old_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who is old",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10376523,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9847543,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10143530,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10155485,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10332953,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "prostitute",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 2554066,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "cocotte",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "whore",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 748155,
                                        part_of_speech: Noun,
                                        target: 2,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 2580577,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1427695,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "harlot",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "bawd",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 424787,
                                        part_of_speech: Adjective,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "tart",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "cyprian",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "fancy_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "working_girl",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "sporting_lady",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "lady_of_pleasure",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "woman_of_the_street",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who engages in sexual intercourse for money",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9889065,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9890296,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9941172,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10001882,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10663315,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10779416,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "shiksa",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "shikse",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a derogatory term used by Jews to refer to non-Jewish women",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: DomainOfSynsetUsage,
                                synset_offset: 6717170,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: DomainOfSynsetUsage,
                                synset_offset: 6951067,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "smasher",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "stunner",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 2115430,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "knockout",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "beauty",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "ravisher",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "sweetheart",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "peach",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "lulu",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "looker",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "mantrap",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "dish",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 218673,
                                        part_of_speech: Adjective,
                                        target: 0,
                                    },
                                ],
                            },
                        ],
                        definition: "a very attractive or seductive looking woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "sylph",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a slender graceful young woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "unmarried_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who is not married",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10598181,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10636488,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "vestal",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 361125,
                                        part_of_speech: Adjective,
                                        target: 1,
                                    },
                                ],
                            },
                        ],
                        definition: "a chaste woman",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "Wac",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a member of the Women's Army Corps",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10622053,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "Wave",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a member of the women's reserve of the United States Navy; originally organized during World War II but now no longer a separate branch",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10523341,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "widow",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 15153667,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 13967970,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 360337,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "widow_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman whose husband is dead especially one who has not remarried",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10028289,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10768810,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "wife",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1735475,
                                        part_of_speech: Adjective,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: Antonym,
                                        synset_offset: 10193967,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "married_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a married woman; a man's partner in marriage",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10640620,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9844356,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9981278,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10092794,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10137498,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10183347,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10189776,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10293773,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10303037,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10304086,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10323529,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10375314,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10588519,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10597889,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10730820,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10743941,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10752020,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10756061,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 10838288,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 10887593,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 11034485,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 11251384,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 11255775,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 11275952,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: InstanceHyponym,
                                synset_offset: 11281555,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "wonder_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a woman who can be a successful wife and have a professional career at the same time",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hypernym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "female",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1484451,
                                        part_of_speech: Adjective,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: Antonym,
                                        synset_offset: 9624168,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "female_person",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "a person who belongs to the sex that can have babies",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 7846,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: PartMeronym,
                                synset_offset: 5219923,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10084043,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10084295,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10106995,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10131151,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DomainOfSynsetUsage,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "colloquialism",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: MemberOfThisDomainRegion,
                                        synset_offset: 6947658,
                                        part_of_speech: Noun,
                                        target: 7,
                                    },
                                    LexicalRelationship {
                                        relation: MemberOfThisDomainRegion,
                                        synset_offset: 2010698,
                                        part_of_speech: Verb,
                                        target: 1,
                                    },
                                ],
                            },
                        ],
                        definition: "a colloquial expression; characteristic of spoken or written communication that seeks to imitate informal speech",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 7069948,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4817,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 19505,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 20647,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 22437,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 33077,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 92136,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 115094,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 115193,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 115906,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 167520,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 218673,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 228876,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 265314,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 294463,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 341655,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 437223,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 439905,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 452114,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 456929,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 458266,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 459953,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 476819,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 478311,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 532560,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 653617,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 680156,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 680634,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 694773,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 733632,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 735882,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 748563,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 750054,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 765289,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 799401,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 806243,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 811248,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 850875,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 855309,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 886448,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 901650,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 919984,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 933415,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 971660,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 972354,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 972501,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 975011,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 975778,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 976016,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1019450,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1054367,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1075524,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1086213,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1123879,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1127440,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1127782,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1129021,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1138450,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1201298,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1267339,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1274741,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1276872,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1280908,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1281874,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1307571,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1332907,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1388062,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1389022,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1391074,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1392633,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1392896,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1403632,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1423187,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1462461,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1468850,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1510628,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1520908,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1533659,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1538583,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1586194,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1677200,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1678586,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1709681,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1766958,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1777662,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1795353,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1816525,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1836766,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1850446,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1850742,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1863442,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1871349,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1880071,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1914250,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1917594,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2076234,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2077904,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2081114,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2204580,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2227485,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2257601,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2298642,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2337558,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2341864,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2342463,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2346013,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2347742,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2358650,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2358762,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2358898,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2383564,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2407346,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2421364,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2432154,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2434473,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2509710,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2512044,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2571536,
                                part_of_speech: Adjective,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3846,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 8007,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9541,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15471,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 25290,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 25559,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 25728,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 32598,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 33809,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 39318,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 54950,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 56916,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 57042,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 57388,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 58033,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 89076,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 91032,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 101752,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 104661,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 118727,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 144722,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 147876,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 148139,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 161630,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 164676,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 168564,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 200614,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 277585,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 354033,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 355080,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 426140,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 426278,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 472323,
                                part_of_speech: Adverb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 104088,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 323262,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 580190,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 583089,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 797468,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 802785,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 854393,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1096674,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1172598,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1260556,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1260731,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1503976,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2324587,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2834506,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3173142,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3404012,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3436182,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3655838,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3688192,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3711603,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4027820,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4055595,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4056491,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4066023,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4095109,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4165811,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4201992,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4355115,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4552097,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4761960,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4815177,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4931267,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5032351,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5084733,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5096294,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5105009,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5206445,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5210820,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5311054,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5312040,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5599084,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5601357,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5687832,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5705484,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5786372,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5812485,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5828102,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5828263,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5831939,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5921685,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6322357,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6397645,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6547832,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6610436,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6716796,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7141537,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7142924,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7144416,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7246215,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7292418,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7319399,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7320894,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7519983,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7539962,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7677860,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7907037,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7960666,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 8245425,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 8590719,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9125984,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9270508,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9605110,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9853881,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9879144,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9890662,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9891300,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9976283,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10011785,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10013114,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10022908,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10026367,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10085101,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10097477,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10114662,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10117851,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10123711,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10162780,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10163593,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10194566,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10240235,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10251329,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10288516,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10314182,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10335801,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10397142,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10505459,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10531557,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10591347,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10627899,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10628368,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10638136,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10642845,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10666615,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10709876,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10719395,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10762342,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13259797,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13272712,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13366428,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13371190,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13372123,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13772313,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13927112,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13937727,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13988498,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14018055,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14038027,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14360742,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14405061,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14485673,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14485811,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14485990,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14521954,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15048623,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15060569,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15170178,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15243202,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15244200,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 590241,
                                part_of_speech: Verb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1906322,
                                part_of_speech: Verb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2167210,
                                part_of_speech: Verb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2532458,
                                part_of_speech: Verb,
                            },
                            SemanticRelationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2600082,
                                part_of_speech: Verb,
                            },
                        ],
                    },
                ),
                (
                    Hypernym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "cleaner",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1533442,
                                        part_of_speech: Verb,
                                        target: 2,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1532589,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                        ],
                        definition: "someone whose occupation is cleaning",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 10241300,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9911226,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 9919061,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10562645,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10662474,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 10783145,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hypernym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "class",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 739662,
                                        part_of_speech: Verb,
                                        target: 1,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 654625,
                                        part_of_speech: Verb,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "stratum",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "social_class",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "socio-economic_class",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "people having the same social, economic, or educational status",
                        examples: [
                            "the working class",
                            "an emerging professional class",
                        ],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 7942152,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: PartHolonym,
                                synset_offset: 7966140,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberHolonym,
                                synset_offset: 8378555,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 7965937,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 7974766,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8075287,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8075388,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8167365,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8180639,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8181540,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8181658,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8181820,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8181930,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8182283,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8244895,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8245059,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8246502,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8306047,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8306194,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8386365,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8389094,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8415983,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8416137,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8416523,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8417572,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8424951,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8436562,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: Hyponym,
                                synset_offset: 8477634,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    MemberHolonym,
                    SynSet {
                        lemmas: [
                            Lemma {
                                word: "womankind",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                        definition: "women as distinguished from men",
                        examples: [],
                        part_of_speech: Noun,
                        relationships: [
                            SemanticRelationship {
                                relation: Hypernym,
                                synset_offset: 7942152,
                                part_of_speech: Noun,
                            },
                            SemanticRelationship {
                                relation: MemberMeronym,
                                synset_offset: 8477634,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
            ]
        "#]];
        expected.assert_debug_eq(&resolved_related);
    }

    #[test]
    fn woman_data_synonyms() {
        let word = "woman";
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let mut syn = wn
            .synsets(word)
            .into_iter()
            .flat_map(|ss| ss.synonyms())
            .collect::<Vec<_>>();
        syn.sort();
        syn.dedup();
        let expected = expect![[r#"
            [
                "adult_female",
                "char",
                "charwoman",
                "cleaning_lady",
                "cleaning_woman",
                "fair_sex",
                "woman",
                "womanhood",
            ]
        "#]];
        expected.assert_debug_eq(&syn);
    }

    #[test]
    fn woman_data_antonyms() {
        let word = "woman";
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let antonyms = wn
            .synsets(word)
            .into_iter()
            .map(|ss| {
                ss.lemmas
                    .into_iter()
                    .map(|l| (l.word.clone(), l.antonyms(&wn)))
            })
            .collect::<Vec<_>>();
        let expected = expect![[r#"
            [
                Map {
                    iter: IntoIter(
                        [
                            Lemma {
                                word: "woman",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 1484987,
                                        part_of_speech: Adjective,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 14425715,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 8477634,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 606006,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 566322,
                                        part_of_speech: Verb,
                                        target: 4,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 2590910,
                                        part_of_speech: Verb,
                                        target: 2,
                                    },
                                    LexicalRelationship {
                                        relation: Antonym,
                                        synset_offset: 10287213,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "adult_female",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                    ),
                },
                Map {
                    iter: IntoIter(
                        [
                            Lemma {
                                word: "woman",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 2590910,
                                        part_of_speech: Verb,
                                        target: 1,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 2590910,
                                        part_of_speech: Verb,
                                        target: 2,
                                    },
                                    LexicalRelationship {
                                        relation: Antonym,
                                        synset_offset: 10288516,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                        ],
                    ),
                },
                Map {
                    iter: IntoIter(
                        [
                            Lemma {
                                word: "charwoman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "char",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "cleaning_woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "cleaning_lady",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                            Lemma {
                                word: "woman",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                    ),
                },
                Map {
                    iter: IntoIter(
                        [
                            Lemma {
                                word: "womanhood",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 10787470,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "woman",
                                part_of_speech: Noun,
                                relationships: [
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 14425715,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                    LexicalRelationship {
                                        relation: DerivationallyRelatedForm,
                                        synset_offset: 606006,
                                        part_of_speech: Noun,
                                        target: 0,
                                    },
                                ],
                            },
                            Lemma {
                                word: "fair_sex",
                                part_of_speech: Noun,
                                relationships: [],
                            },
                        ],
                    ),
                },
            ]
        "#]];
        expected.assert_debug_eq(&antonyms);
    }

    #[test]
    fn all_words() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let len = wn.all_words().len();
        let expected = expect![[r#"
            147306
        "#]];
        expected.assert_debug_eq(&len);
    }

    #[test]
    fn all_words_cause() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let words = wn
            .all_words()
            .into_iter()
            .map(|w| {
                let synsets = wn.synsets(&w);
                synsets
                    .iter()
                    .map(|ss| ss.with_relationship(SemanticRelation::Cause).len())
                    .sum::<usize>()
            })
            .sum::<usize>();
        let expected = expect![[r#"
            466
        "#]];
        expected.assert_debug_eq(&words);
    }

    #[test]
    fn underscore_counts() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let words = wn
            .all_words()
            .into_iter()
            .map(|w| (w.chars().filter(|c| *c == '_').count(), w))
            .fold(BTreeMap::new(), |mut acc, (c, w)| {
                acc.entry(c).or_insert((0, w)).0 += 1;
                acc
            });
        let expected = expect![[r#"
            {
                0: (
                    83118,
                    "'hood",
                ),
                1: (
                    54533,
                    "'s_gravenhage",
                ),
                2: (
                    7766,
                    "15_august_1945",
                ),
                3: (
                    1454,
                    "1st_earl_of_balfour",
                ),
                4: (
                    298,
                    "1st_earl_baldwin_of_bewdley",
                ),
                5: (
                    80,
                    "academy_of_television_arts_and_sciences",
                ),
                6: (
                    28,
                    "abu_ali_al-husain_ibn_abdallah_ibn_sina",
                ),
                7: (
                    20,
                    "armenian_secret_army_for_the_liberation_of_armenia",
                ),
                8: (
                    9,
                    "american_federation_of_labor_and_congress_of_industrial_organizations",
                ),
            }
        "#]];
        expected.assert_debug_eq(&words);
    }

    #[test]
    fn punctuation_counts() {
        let wndir = env::var("WNSEARCHDIR").unwrap();
        let wn = WordNet::new(&PathBuf::from(wndir));
        let words = wn
            .all_words()
            .into_iter()
            .filter_map(|w| {
                let mut non_alpha_chars =
                    w.chars().filter(|c| !c.is_alphabetic()).collect::<Vec<_>>();
                non_alpha_chars.sort();
                non_alpha_chars.dedup();
                if non_alpha_chars.is_empty() {
                    None
                } else {
                    Some((non_alpha_chars, w))
                }
            })
            .fold(BTreeMap::new(), |mut acc, (c, w)| {
                acc.entry(c).or_insert((0, w)).0 += 1;
                acc
            });
        let expected = expect![[r#"
            {
                [
                    '\'',
                ]: (
                    56,
                    "'hood",
                ),
                [
                    '\'',
                    '-',
                ]: (
                    80,
                    "al-ma'unah",
                ),
                [
                    '\'',
                    '-',
                    '.',
                ]: (
                    1,
                    "st.-bruno's-lily",
                ),
                [
                    '\'',
                    '-',
                    '_',
                ]: (
                    43,
                    "adam's_needle-and-thread",
                ),
                [
                    '\'',
                    '.',
                    '_',
                ]: (
                    8,
                    "st._andrew's_cross",
                ),
                [
                    '\'',
                    '1',
                    '7',
                    '_',
                ]: (
                    1,
                    "brodmann's_area_17",
                ),
                [
                    '\'',
                    '_',
                ]: (
                    1102,
                    "'s_gravenhage",
                ),
                [
                    '-',
                ]: (
                    5093,
                    "a-bomb",
                ),
                [
                    '-',
                    '.',
                    '2',
                ]: (
                    2,
                    ".22-caliber",
                ),
                [
                    '-',
                    '.',
                    '3',
                    '8',
                ]: (
                    2,
                    ".38-caliber",
                ),
                [
                    '-',
                    '.',
                    '4',
                    '5',
                ]: (
                    2,
                    ".45-caliber",
                ),
                [
                    '-',
                    '.',
                    '_',
                ]: (
                    2,
                    "st._christopher-nevis",
                ),
                [
                    '-',
                    '0',
                    '1',
                ]: (
                    1,
                    "10-membered",
                ),
                [
                    '-',
                    '0',
                    '1',
                    '4',
                ]: (
                    1,
                    "401-k",
                ),
                [
                    '-',
                    '0',
                    '1',
                    '4',
                    '_',
                ]: (
                    1,
                    "401-k_plan",
                ),
                [
                    '-',
                    '0',
                    '2',
                    '_',
                ]: (
                    1,
                    "k-dur_20",
                ),
                [
                    '-',
                    '1',
                ]: (
                    9,
                    "1-dodecanol",
                ),
                [
                    '-',
                    '1',
                    '2',
                    '5',
                ]: (
                    1,
                    "iodine-125",
                ),
                [
                    '-',
                    '1',
                    '2',
                    '_',
                ]: (
                    2,
                    "12-tone_music",
                ),
                [
                    '-',
                    '1',
                    '3',
                ]: (
                    1,
                    "iodine-131",
                ),
                [
                    '-',
                    '1',
                    '4',
                    '_',
                ]: (
                    1,
                    "carbon-14_dating",
                ),
                [
                    '-',
                    '1',
                    '8',
                    '_',
                ]: (
                    1,
                    "18-karat_gold",
                ),
                [
                    '-',
                    '1',
                    '9',
                ]: (
                    1,
                    "9-11",
                ),
                [
                    '-',
                    '1',
                    '_',
                ]: (
                    3,
                    "1st-class_mail",
                ),
                [
                    '-',
                    '2',
                ]: (
                    6,
                    "2-dimensional",
                ),
                [
                    '-',
                    '2',
                    '4',
                    '_',
                ]: (
                    2,
                    "24-hour_interval",
                ),
                [
                    '-',
                    '2',
                    '5',
                ]: (
                    1,
                    "b-52",
                ),
                [
                    '-',
                    '2',
                    '8',
                ]: (
                    3,
                    "blu-82",
                ),
                [
                    '-',
                    '2',
                    '8',
                    '_',
                ]: (
                    1,
                    "guided_bomb_unit-28",
                ),
                [
                    '-',
                    '2',
                    '_',
                ]: (
                    4,
                    "2-hydroxybenzoic_acid",
                ),
                [
                    '-',
                    '3',
                ]: (
                    5,
                    "3-d",
                ),
                [
                    '-',
                    '3',
                    '5',
                    '_',
                ]: (
                    1,
                    "5-hydroxy-3-methylglutaryl-coenzyme_a_reductase",
                ),
                [
                    '-',
                    '3',
                    '_',
                ]: (
                    1,
                    "omega-3_fatty_acid",
                ),
                [
                    '-',
                    '4',
                ]: (
                    3,
                    "4-dimensional",
                ),
                [
                    '-',
                    '5',
                ]: (
                    3,
                    "5-hitter",
                ),
                [
                    '-',
                    '6',
                ]: (
                    2,
                    "6-membered",
                ),
                [
                    '-',
                    '6',
                    '_',
                ]: (
                    1,
                    "omega-6_fatty_acid",
                ),
                [
                    '-',
                    '7',
                ]: (
                    1,
                    "7-membered",
                ),
                [
                    '-',
                    '8',
                ]: (
                    1,
                    "8-membered",
                ),
                [
                    '-',
                    '8',
                    '_',
                ]: (
                    1,
                    "v-8_juice",
                ),
                [
                    '-',
                    '9',
                ]: (
                    1,
                    "9-membered",
                ),
                [
                    '-',
                    '_',
                ]: (
                    1743,
                    "a-scan_ultrasonography",
                ),
                [
                    '.',
                ]: (
                    70,
                    "a.d.",
                ),
                [
                    '.',
                    '1',
                    '_',
                ]: (
                    1,
                    "sept._11",
                ),
                [
                    '.',
                    '2',
                ]: (
                    1,
                    ".22",
                ),
                [
                    '.',
                    '2',
                    '_',
                ]: (
                    2,
                    ".22_caliber",
                ),
                [
                    '.',
                    '3',
                    '8',
                    '_',
                ]: (
                    2,
                    ".38_caliber",
                ),
                [
                    '.',
                    '4',
                    '5',
                    '_',
                ]: (
                    2,
                    ".45_caliber",
                ),
                [
                    '.',
                    '_',
                ]: (
                    333,
                    "a._a._michelson",
                ),
                [
                    '/',
                ]: (
                    7,
                    "counts/minute",
                ),
                [
                    '/',
                    '0',
                    '2',
                ]: (
                    1,
                    "20/20",
                ),
                [
                    '/',
                    '1',
                    '9',
                ]: (
                    1,
                    "9/11",
                ),
                [
                    '/',
                    '2',
                    '4',
                    '7',
                ]: (
                    1,
                    "24/7",
                ),
                [
                    '/',
                    '_',
                ]: (
                    6,
                    "on/off_switch",
                ),
                [
                    '0',
                ]: (
                    1,
                    "0",
                ),
                [
                    '0',
                    '1',
                ]: (
                    15,
                    "10",
                ),
                [
                    '0',
                    '1',
                    '2',
                ]: (
                    2,
                    "120",
                ),
                [
                    '0',
                    '1',
                    '2',
                    '8',
                ]: (
                    1,
                    "1820s",
                ),
                [
                    '0',
                    '1',
                    '2',
                    '9',
                ]: (
                    1,
                    "1920s",
                ),
                [
                    '0',
                    '1',
                    '2',
                    '_',
                ]: (
                    1,
                    "atomic_number_102",
                ),
                [
                    '0',
                    '1',
                    '3',
                ]: (
                    2,
                    "130",
                ),
                [
                    '0',
                    '1',
                    '3',
                    '5',
                ]: (
                    1,
                    "1530s",
                ),
                [
                    '0',
                    '1',
                    '3',
                    '8',
                ]: (
                    1,
                    "1830s",
                ),
                [
                    '0',
                    '1',
                    '3',
                    '9',
                ]: (
                    1,
                    "1930s",
                ),
                [
                    '0',
                    '1',
                    '3',
                    '_',
                ]: (
                    1,
                    "atomic_number_103",
                ),
                [
                    '0',
                    '1',
                    '4',
                ]: (
                    2,
                    "140",
                ),
                [
                    '0',
                    '1',
                    '4',
                    '8',
                ]: (
                    1,
                    "1840s",
                ),
                [
                    '0',
                    '1',
                    '4',
                    '9',
                ]: (
                    1,
                    "1940s",
                ),
                [
                    '0',
                    '1',
                    '4',
                    '_',
                ]: (
                    2,
                    "atomic_number_104",
                ),
                [
                    '0',
                    '1',
                    '5',
                ]: (
                    4,
                    "105",
                ),
                [
                    '0',
                    '1',
                    '5',
                    '7',
                ]: (
                    1,
                    "1750s",
                ),
                [
                    '0',
                    '1',
                    '5',
                    '8',
                ]: (
                    1,
                    "1850s",
                ),
                [
                    '0',
                    '1',
                    '5',
                    '9',
                ]: (
                    1,
                    "1950s",
                ),
                [
                    '0',
                    '1',
                    '5',
                    '_',
                ]: (
                    2,
                    "atomic_number_105",
                ),
                [
                    '0',
                    '1',
                    '6',
                ]: (
                    2,
                    "160",
                ),
                [
                    '0',
                    '1',
                    '6',
                    '7',
                ]: (
                    1,
                    "1760s",
                ),
                [
                    '0',
                    '1',
                    '6',
                    '8',
                ]: (
                    1,
                    "1860s",
                ),
                [
                    '0',
                    '1',
                    '6',
                    '9',
                ]: (
                    1,
                    "1960s",
                ),
                [
                    '0',
                    '1',
                    '6',
                    '_',
                ]: (
                    2,
                    "atomic_number_106",
                ),
                [
                    '0',
                    '1',
                    '7',
                ]: (
                    3,
                    "170",
                ),
                [
                    '0',
                    '1',
                    '7',
                    '8',
                ]: (
                    2,
                    "1780s",
                ),
                [
                    '0',
                    '1',
                    '7',
                    '9',
                ]: (
                    2,
                    "1790s",
                ),
                [
                    '0',
                    '1',
                    '7',
                    '_',
                ]: (
                    2,
                    "atomic_number_107",
                ),
                [
                    '0',
                    '1',
                    '8',
                ]: (
                    3,
                    "180",
                ),
                [
                    '0',
                    '1',
                    '8',
                    '9',
                ]: (
                    2,
                    "1890s",
                ),
                [
                    '0',
                    '1',
                    '8',
                    '_',
                ]: (
                    2,
                    "atomic_number_108",
                ),
                [
                    '0',
                    '1',
                    '9',
                ]: (
                    4,
                    "190",
                ),
                [
                    '0',
                    '1',
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_109",
                ),
                [
                    '0',
                    '1',
                    '_',
                ]: (
                    5,
                    "atomic_number_10",
                ),
                [
                    '0',
                    '2',
                ]: (
                    4,
                    "20",
                ),
                [
                    '0',
                    '2',
                    '_',
                ]: (
                    2,
                    "atomic_number_20",
                ),
                [
                    '0',
                    '3',
                ]: (
                    4,
                    "30",
                ),
                [
                    '0',
                    '3',
                    '8',
                ]: (
                    1,
                    "u308",
                ),
                [
                    '0',
                    '3',
                    '_',
                ]: (
                    2,
                    "30_minutes",
                ),
                [
                    '0',
                    '4',
                ]: (
                    4,
                    "40",
                ),
                [
                    '0',
                    '4',
                    '_',
                ]: (
                    2,
                    "440_yards",
                ),
                [
                    '0',
                    '5',
                ]: (
                    4,
                    "50",
                ),
                [
                    '0',
                    '5',
                    '_',
                ]: (
                    1,
                    "atomic_number_50",
                ),
                [
                    '0',
                    '6',
                ]: (
                    2,
                    "60",
                ),
                [
                    '0',
                    '6',
                    '_',
                ]: (
                    3,
                    "60_minutes",
                ),
                [
                    '0',
                    '7',
                ]: (
                    2,
                    "70",
                ),
                [
                    '0',
                    '7',
                    '_',
                ]: (
                    1,
                    "atomic_number_70",
                ),
                [
                    '0',
                    '8',
                ]: (
                    2,
                    "80",
                ),
                [
                    '0',
                    '8',
                    '_',
                ]: (
                    2,
                    "880_yards",
                ),
                [
                    '0',
                    '9',
                ]: (
                    2,
                    "90",
                ),
                [
                    '0',
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_90",
                ),
                [
                    '1',
                ]: (
                    7,
                    "1",
                ),
                [
                    '1',
                    '2',
                ]: (
                    4,
                    "12",
                ),
                [
                    '1',
                    '2',
                    '5',
                ]: (
                    2,
                    "125",
                ),
                [
                    '1',
                    '2',
                    '7',
                    '8',
                ]: (
                    1,
                    "1728",
                ),
                [
                    '1',
                    '2',
                    '8',
                    '_',
                ]: (
                    1,
                    "war_of_1812",
                ),
                [
                    '1',
                    '2',
                    '_',
                ]: (
                    9,
                    "atomic_number_112",
                ),
                [
                    '1',
                    '3',
                ]: (
                    4,
                    "13",
                ),
                [
                    '1',
                    '3',
                    '5',
                ]: (
                    2,
                    "135",
                ),
                [
                    '1',
                    '3',
                    '7',
                    '_',
                ]: (
                    1,
                    "cesium_137",
                ),
                [
                    '1',
                    '3',
                    '_',
                ]: (
                    5,
                    "atomic_number_113",
                ),
                [
                    '1',
                    '4',
                ]: (
                    5,
                    "14",
                ),
                [
                    '1',
                    '4',
                    '5',
                ]: (
                    2,
                    "145",
                ),
                [
                    '1',
                    '4',
                    '5',
                    '8',
                    '9',
                    '_',
                ]: (
                    1,
                    "8_may_1945",
                ),
                [
                    '1',
                    '4',
                    '5',
                    '9',
                    '_',
                ]: (
                    1,
                    "15_august_1945",
                ),
                [
                    '1',
                    '4',
                    '6',
                    '9',
                    '_',
                ]: (
                    1,
                    "6_june_1944",
                ),
                [
                    '1',
                    '4',
                    '_',
                ]: (
                    9,
                    "14_july",
                ),
                [
                    '1',
                    '5',
                ]: (
                    7,
                    "115",
                ),
                [
                    '1',
                    '5',
                    '6',
                ]: (
                    2,
                    "165",
                ),
                [
                    '1',
                    '5',
                    '7',
                ]: (
                    2,
                    "175",
                ),
                [
                    '1',
                    '5',
                    '_',
                ]: (
                    7,
                    "15_may_organization",
                ),
                [
                    '1',
                    '6',
                ]: (
                    3,
                    "16",
                ),
                [
                    '1',
                    '6',
                    '_',
                ]: (
                    5,
                    "16_pf",
                ),
                [
                    '1',
                    '7',
                ]: (
                    3,
                    "17",
                ),
                [
                    '1',
                    '7',
                    '_',
                ]: (
                    8,
                    "17_november",
                ),
                [
                    '1',
                    '8',
                ]: (
                    3,
                    "18",
                ),
                [
                    '1',
                    '8',
                    '_',
                ]: (
                    2,
                    "atomic_number_18",
                ),
                [
                    '1',
                    '9',
                ]: (
                    3,
                    "19",
                ),
                [
                    '1',
                    '9',
                    '_',
                ]: (
                    4,
                    "atomic_number_19",
                ),
                [
                    '1',
                    '_',
                ]: (
                    33,
                    "11_november",
                ),
                [
                    '2',
                ]: (
                    11,
                    "2",
                ),
                [
                    '2',
                    '3',
                ]: (
                    4,
                    "23",
                ),
                [
                    '2',
                    '3',
                    '5',
                    '_',
                ]: (
                    1,
                    "uranium_235",
                ),
                [
                    '2',
                    '3',
                    '8',
                    '_',
                ]: (
                    1,
                    "uranium_238",
                ),
                [
                    '2',
                    '3',
                    '9',
                    '_',
                ]: (
                    1,
                    "plutonium_239",
                ),
                [
                    '2',
                    '3',
                    '_',
                ]: (
                    3,
                    "atomic_number_23",
                ),
                [
                    '2',
                    '4',
                ]: (
                    4,
                    "24",
                ),
                [
                    '2',
                    '4',
                    '_',
                ]: (
                    7,
                    "atomic_number_24",
                ),
                [
                    '2',
                    '5',
                ]: (
                    3,
                    "25",
                ),
                [
                    '2',
                    '5',
                    '_',
                ]: (
                    5,
                    "atomic_number_25",
                ),
                [
                    '2',
                    '6',
                ]: (
                    4,
                    "26",
                ),
                [
                    '2',
                    '6',
                    '_',
                ]: (
                    2,
                    "atomic_number_26",
                ),
                [
                    '2',
                    '7',
                ]: (
                    3,
                    "27",
                ),
                [
                    '2',
                    '7',
                    '_',
                ]: (
                    2,
                    "atomic_number_27",
                ),
                [
                    '2',
                    '8',
                ]: (
                    3,
                    "28",
                ),
                [
                    '2',
                    '8',
                    '_',
                ]: (
                    2,
                    "atomic_number_28",
                ),
                [
                    '2',
                    '9',
                ]: (
                    3,
                    "29",
                ),
                [
                    '2',
                    '9',
                    '_',
                ]: (
                    5,
                    "atomic_number_29",
                ),
                [
                    '2',
                    '_',
                ]: (
                    19,
                    "2_chronicles",
                ),
                [
                    '3',
                ]: (
                    7,
                    "3",
                ),
                [
                    '3',
                    '4',
                ]: (
                    4,
                    "34",
                ),
                [
                    '3',
                    '4',
                    '_',
                ]: (
                    2,
                    "atomic_number_34",
                ),
                [
                    '3',
                    '5',
                ]: (
                    3,
                    "35",
                ),
                [
                    '3',
                    '5',
                    '6',
                    '_',
                ]: (
                    1,
                    "365_days",
                ),
                [
                    '3',
                    '5',
                    '_',
                ]: (
                    2,
                    "atomic_number_35",
                ),
                [
                    '3',
                    '6',
                ]: (
                    3,
                    "36",
                ),
                [
                    '3',
                    '6',
                    '_',
                ]: (
                    3,
                    "366_days",
                ),
                [
                    '3',
                    '7',
                ]: (
                    3,
                    "37",
                ),
                [
                    '3',
                    '7',
                    '_',
                ]: (
                    2,
                    "atomic_number_37",
                ),
                [
                    '3',
                    '8',
                ]: (
                    3,
                    "38",
                ),
                [
                    '3',
                    '8',
                    '_',
                ]: (
                    2,
                    "atomic_number_38",
                ),
                [
                    '3',
                    '9',
                ]: (
                    3,
                    "39",
                ),
                [
                    '3',
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_39",
                ),
                [
                    '3',
                    '_',
                ]: (
                    7,
                    "3d_radar",
                ),
                [
                    '4',
                ]: (
                    7,
                    "4",
                ),
                [
                    '4',
                    '5',
                ]: (
                    3,
                    "45",
                ),
                [
                    '4',
                    '5',
                    '_',
                ]: (
                    2,
                    "atomic_number_45",
                ),
                [
                    '4',
                    '6',
                ]: (
                    4,
                    "46",
                ),
                [
                    '4',
                    '6',
                    '8',
                    '_',
                ]: (
                    1,
                    "ru_486",
                ),
                [
                    '4',
                    '6',
                    '_',
                ]: (
                    2,
                    "atomic_number_46",
                ),
                [
                    '4',
                    '7',
                ]: (
                    3,
                    "47",
                ),
                [
                    '4',
                    '7',
                    '_',
                ]: (
                    2,
                    "atomic_number_47",
                ),
                [
                    '4',
                    '8',
                ]: (
                    3,
                    "48",
                ),
                [
                    '4',
                    '8',
                    '_',
                ]: (
                    2,
                    "atomic_number_48",
                ),
                [
                    '4',
                    '9',
                ]: (
                    3,
                    "49",
                ),
                [
                    '4',
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_49",
                ),
                [
                    '4',
                    '_',
                ]: (
                    7,
                    "atomic_number_4",
                ),
                [
                    '5',
                ]: (
                    4,
                    "5",
                ),
                [
                    '5',
                    '6',
                ]: (
                    3,
                    "56",
                ),
                [
                    '5',
                    '6',
                    '_',
                ]: (
                    2,
                    "atomic_number_56",
                ),
                [
                    '5',
                    '7',
                ]: (
                    3,
                    "57",
                ),
                [
                    '5',
                    '7',
                    '_',
                ]: (
                    2,
                    "atomic_number_57",
                ),
                [
                    '5',
                    '8',
                ]: (
                    3,
                    "58",
                ),
                [
                    '5',
                    '8',
                    '_',
                ]: (
                    2,
                    "atomic_number_58",
                ),
                [
                    '5',
                    '9',
                ]: (
                    3,
                    "59",
                ),
                [
                    '5',
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_59",
                ),
                [
                    '5',
                    '_',
                ]: (
                    4,
                    "atomic_number_5",
                ),
                [
                    '6',
                ]: (
                    3,
                    "6",
                ),
                [
                    '6',
                    '7',
                ]: (
                    2,
                    "67",
                ),
                [
                    '6',
                    '7',
                    '_',
                ]: (
                    2,
                    "atomic_number_67",
                ),
                [
                    '6',
                    '8',
                ]: (
                    2,
                    "68",
                ),
                [
                    '6',
                    '8',
                    '_',
                ]: (
                    2,
                    "atomic_number_68",
                ),
                [
                    '6',
                    '9',
                ]: (
                    2,
                    "69",
                ),
                [
                    '6',
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_69",
                ),
                [
                    '6',
                    '_',
                ]: (
                    6,
                    "atomic_number_6",
                ),
                [
                    '7',
                ]: (
                    3,
                    "7",
                ),
                [
                    '7',
                    '8',
                ]: (
                    2,
                    "78",
                ),
                [
                    '7',
                    '8',
                    '_',
                ]: (
                    2,
                    "atomic_number_78",
                ),
                [
                    '7',
                    '9',
                ]: (
                    2,
                    "79",
                ),
                [
                    '7',
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_79",
                ),
                [
                    '7',
                    '_',
                ]: (
                    2,
                    "atomic_number_7",
                ),
                [
                    '8',
                ]: (
                    5,
                    "8",
                ),
                [
                    '8',
                    '9',
                ]: (
                    2,
                    "89",
                ),
                [
                    '8',
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_89",
                ),
                [
                    '8',
                    '_',
                ]: (
                    6,
                    "atomic_number_8",
                ),
                [
                    '9',
                ]: (
                    3,
                    "9",
                ),
                [
                    '9',
                    '_',
                ]: (
                    2,
                    "atomic_number_9",
                ),
                [
                    '_',
                ]: (
                    60675,
                    "a_battery",
                ),
            }
        "#]];
        expected.assert_debug_eq(&words);
    }
}
