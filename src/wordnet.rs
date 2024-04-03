use data::Data;
use index::Index;
pub use pos::PartOfSpeech;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use self::pointer::PointerType;

mod data;
mod index;
mod pointer;
mod pos;

pub struct WordNet {
    index: Index,
    data: Data,
    database: PathBuf,
}

impl WordNet {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            index: Index,
            data: Data,
            database: dir,
        }
    }

    pub fn definitions(&self, word: &str) -> BTreeMap<PartOfSpeech, BTreeSet<String>> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut defs: BTreeMap<PartOfSpeech, BTreeSet<String>> = BTreeMap::new();

        for i in items {
            for o in i.syn_offsets.iter() {
                let items = self.data.load(&self.database, *o, i.pos);
                defs.entry(i.pos)
                    .or_default()
                    .extend(items.iter().map(|x| x.gloss.clone()))
            }
        }
        defs
    }

    pub fn synonyms(&self, word: &str) -> BTreeMap<PartOfSpeech, BTreeSet<String>> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut synonyms: BTreeMap<PartOfSpeech, BTreeSet<String>> = BTreeMap::new();

        for item in items {
            for offset in item.syn_offsets.iter() {
                synonyms.entry(item.pos).or_default().extend(
                    self.data
                        .load(&self.database, *offset, item.pos)
                        .iter()
                        .flat_map(|x| x.words.clone()),
                );
            }
        }

        synonyms
    }

    pub fn antonyms(&self, word: &str) -> BTreeMap<PartOfSpeech, BTreeSet<String>> {
        self.with_relationship(word, PointerType::Antonym)
    }

    pub fn with_relationship(
        &self,
        word: &str,
        relationship: PointerType,
    ) -> BTreeMap<PartOfSpeech, BTreeSet<String>> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut antonyms: BTreeMap<PartOfSpeech, BTreeSet<String>> = BTreeMap::new();

        for item in items {
            for offset in item.syn_offsets.iter() {
                antonyms.entry(item.pos).or_default().extend(
                    self.data
                        .load(&self.database, *offset, item.pos)
                        .iter()
                        .flat_map(|x| {
                            x.relationships
                                .iter()
                                .filter(|r| r.0 == relationship)
                                .flat_map(|r| self.data.load(&self.database, r.1, r.2))
                                .flat_map(|di| di.words)
                        }),
                );
            }
        }

        antonyms
    }

    pub fn synsets(&self, word: &str) -> Vec<SynSet> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut synsets = Vec::new();

        for item in items {
            for offset in item.syn_offsets.iter() {
                let dataitems = self.data.load(&self.database, *offset, item.pos);
                for synset in dataitems {
                    let full_synset = SynSet {
                        words: synset.words,
                        definition: synset.gloss,
                        part_of_speech: item.pos,
                        relationships: synset
                            .relationships
                            .into_iter()
                            .map(|r| Relationship {
                                relationship_kind: r.0,
                                words: self
                                    .data
                                    .load(&self.database, r.1, r.2)
                                    .iter()
                                    .flat_map(|di| di.words.clone())
                                    .collect(),
                                part_of_speech: r.2,
                            })
                            .collect(),
                    };
                    synsets.push(full_synset);
                }
            }
        }

        synsets
    }
}

#[derive(Debug)]
pub struct SynSet {
    /// The words for the synset.
    pub words: Vec<String>,
    /// Glossary entry.
    pub definition: String,
    /// What type of word it is.
    pub part_of_speech: PartOfSpeech,
    /// How it relates to other words.
    pub relationships: Vec<Relationship>,
}

#[derive(Debug)]
pub struct Relationship {
    pub relationship_kind: PointerType,
    pub words: Vec<String>,
    pub part_of_speech: PartOfSpeech,
}

#[cfg(test)]

mod tests {
    use std::env;

    use super::*;
    use expect_test::expect;

    #[test]
    fn multipos_data_definition() {
        let word = "run";
        let wndir = env::var("WORDNET").unwrap();
        let wn = WordNet::new(PathBuf::from(wndir));
        let def = wn.definitions(word);
        let expected = expect![[r#"
            {
                Noun: {
                    "(American football) a play in which a player attempts to carry the ball through or past the opposing team; \"the defensive line braced to stop the run\"; \"the coach put great emphasis on running\"",
                    "a race between candidates for elective office; \"I managed his campaign for governor\"; \"he is raising money for a Senate run\"",
                    "a race run on foot; \"she broke the record for the half-mile run\"",
                    "a regular trip; \"the ship made its run in record time\"",
                    "a row of unravelled stitches; \"she got a run in her stocking\"",
                    "a score in baseball made by a runner touching all four bases safely; \"the Yankees scored 3 runs in the bottom of the 9th\"; \"their first tally came in the 3rd inning\"",
                    "a short trip; \"take a run into town\"",
                    "a small stream",
                    "an unbroken chronological sequence; \"the play had a long run on Broadway\"; \"the team enjoyed a brief run of victories\"",
                    "an unbroken series of events; \"had a streak of bad luck\"; \"Nicklaus had a run of birdies\"",
                    "the act of running; traveling on foot at a fast pace; \"he broke into a run\"; \"his daily run keeps him fit\"",
                    "the act of testing something; \"in the experimental trials the amount of carbon was measured separately\"; \"he called each flip of the coin a new trial\"",
                    "the continuous period of time during which something (a machine or a factory) operates or continues in operation; \"the assembly line was on a 12-hour run\"",
                    "the pouring forth of a fluid",
                    "the production achieved during a continuous period of operation (of a machine or factory etc.); \"a daily run of 100,000 gallons of paint\"",
                    "unrestricted freedom to use; \"he has the run of the house\"",
                },
                Verb: {
                    "be affected by; be subjected to; \"run a temperature\"; \"run a risk\"",
                    "be diffused; \"These dyes and colors are guaranteed not to run\"",
                    "be operating, running or functioning; \"The car is still running--turn it off!\"",
                    "become undone; \"the sweater unraveled\"",
                    "carry out a process or program, as on a computer or a machine; \"Run the dishwasher\"; \"run a new program on the Mac\"; \"the computer executed the instruction\"",
                    "carry out; \"run an errand\"",
                    "cause an animal to move fast; \"run the dogs\"",
                    "cause something to pass or lead somewhere; \"Run the wire behind the cabinet\"",
                    "cause to emit recorded audio or video; \"They ran the tapes over and over again\"; \"I'll play you my favorite record\"; \"He never tires of playing that video\"",
                    "cause to perform; \"run a subject\"; \"run a process\"",
                    "change from one state to another; \"run amok\"; \"run rogue\"; \"run riot\"",
                    "change or be different within limits; \"Estimates for the losses in the earthquake range as high as $2 billion\"; \"Interest rates run from 5 to 10 percent\"; \"The instruments ranged from tuba to cymbals\"; \"My students range from very bright to dull\"",
                    "come unraveled or undone as if by snagging; \"Her nylons were running\"",
                    "compete in a race; \"he is running the Marathon this year\"; \"let's race and see who gets there first\"",
                    "continue to exist; \"These stories die hard\"; \"The legend of Elvis endures\"",
                    "cover by running; run a certain distance; \"She ran 10 miles that day\"",
                    "deal in illegally, such as arms or liquor",
                    "direct or control; projects, businesses, etc.; \"She is running a relief operation in the Sudan\"",
                    "extend or continue for a certain period of time; \"The film runs 5 hours\"",
                    "flee; take to one's heels; cut and run; \"If you see this man, run!\"; \"The burglars escaped before the police showed up\"",
                    "have a particular form; \"the story or argument runs as follows\"; \"as the saying goes...\"",
                    "have a tendency or disposition to do or be something; be inclined; \"She tends to be nervous before her lectures\"; \"These dresses run small\"; \"He inclined to corpulence\"",
                    "include as the content; broadcast or publicize; \"We ran the ad three times\"; \"This paper carries a restaurant review\"; \"All major networks carried the press conference\"",
                    "keep company; \"the heifers run with the bulls to produce offspring\"",
                    "make without a miss",
                    "move about freely and without restraint, or act as if running around in an uncontrolled way; \"who are these people running around in the building?\"; \"She runs around telling everyone of her troubles\"; \"let the dogs run free\"",
                    "move along, of liquids; \"Water flowed into the cave\"; \"the Missouri feeds into the Mississippi\"",
                    "move fast by using one's feet, with one foot off the ground at any given time; \"Don't run--you'll be out of breath\"; \"The children ran to the store\"",
                    "occur persistently; \"Musical talent runs in the family\"",
                    "pass over, across, or through; \"He ran his eyes over her body\"; \"She ran her fingers along the carved figurine\"; \"He drew her hair through his fingers\"",
                    "perform as expected when applied; \"The washing machine won't go unless it's plugged in\"; \"Does this old car still run well?\"; \"This old radio doesn't work anymore\"",
                    "progress by being changed; \"The speech has to go through several more drafts\"; \"run through your presentation before the meeting\"",
                    "pursue for food or sport (as of wild animals); \"Goering often hunted wild boars in Poland\"; \"The dogs are running deer\"; \"The Duke hunted in these woods\"",
                    "reduce or cause to be reduced from a solid to a liquid state, usually by heating; \"melt butter\"; \"melt down gold\"; \"The wax melted in the sun\"",
                    "run with the ball; in such sports as football",
                    "run, stand, or compete for an office or a position; \"Who's running for treasurer this year?\"",
                    "sail before the wind",
                    "set animals loose to graze",
                    "stretch out over a distance, space, time, or scope; run or extend between two points or beyond a certain point; \"Service runs all the way to Cranbury\"; \"His knowledge doesn't go very far\"; \"My memory extends back to my fourth year of life\"; \"The facts extend beyond a consideration of her personal assets\"",
                    "travel a route regularly; \"Ships ply the waters near the coast\"",
                    "travel rapidly, by any (unspecified) means; \"Run to the store!\"; \"She always runs to Italy, because she has a lover there\"",
                },
            }
        "#]];
        expected.assert_debug_eq(&def);
    }

    #[test]
    fn multipos_data_synonyms() {
        let word = "run";
        let wndir = env::var("WORDNET").unwrap();
        let wn = WordNet::new(PathBuf::from(wndir));
        let syn = wn.synonyms(word);
        let expected = expect![[r#"
            {
                Noun: {
                    "campaign",
                    "discharge",
                    "foot_race",
                    "footrace",
                    "ladder",
                    "outpouring",
                    "political_campaign",
                    "ravel",
                    "rill",
                    "rivulet",
                    "run",
                    "runnel",
                    "running",
                    "running_game",
                    "running_play",
                    "streak",
                    "streamlet",
                    "tally",
                    "test",
                    "trial",
                },
                Verb: {
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
                    "draw",
                    "endure",
                    "escape",
                    "execute",
                    "extend",
                    "feed",
                    "flow",
                    "fly_the_coop",
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
                    "pass",
                    "persist",
                    "play",
                    "ply",
                    "prevail",
                    "race",
                    "range",
                    "run",
                    "run_away",
                    "run_for",
                    "scarper",
                    "scat",
                    "take_to_the_woods",
                    "tend",
                    "track_down",
                    "turn_tail",
                    "unravel",
                    "work",
                },
            }
        "#]];
        expected.assert_debug_eq(&syn);
    }

    #[test]
    fn woman_data_synset() {
        let word = "woman";
        let wndir = env::var("WORDNET").unwrap();
        let wn = WordNet::new(PathBuf::from(wndir));
        let syn = wn.synsets(word);
        let expected = expect![[r#"
            [
                SynSet {
                    words: [
                        "woman",
                        "adult_female",
                    ],
                    definition: "an adult female person (as opposed to a man); \"the woman kept house while the man hunted\"",
                    part_of_speech: Noun,
                    relationships: [
                        Relationship {
                            relationship_kind: Hypernym,
                            words: [
                                "female",
                                "female_person",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hypernym,
                            words: [
                                "adult",
                                "grownup",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "womanly",
                                "feminine",
                            ],
                            part_of_speech: Adjective,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "womanhood",
                                "muliebrity",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "womanhood",
                                "woman",
                                "fair_sex",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "womanhood",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "feminize",
                                "feminise",
                                "effeminize",
                                "effeminise",
                                "womanize",
                            ],
                            part_of_speech: Verb,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "philander",
                                "womanize",
                                "womanise",
                            ],
                            part_of_speech: Verb,
                        },
                        Relationship {
                            relationship_kind: Antonym,
                            words: [
                                "man",
                                "adult_male",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: PartMeronym,
                            words: [
                                "adult_female_body",
                                "woman's_body",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: InstanceHyponym,
                            words: [
                                "Eve",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "Black_woman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "white_woman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "yellow_woman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "amazon",
                                "virago",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "maenad",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "bachelor_girl",
                                "bachelorette",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "baggage",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "ball-buster",
                                "ball-breaker",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "B-girl",
                                "bar_girl",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "bluestocking",
                                "bas_bleu",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "bridesmaid",
                                "maid_of_honor",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "broad",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "cat",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "Cinderella",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "coquette",
                                "flirt",
                                "vamp",
                                "vamper",
                                "minx",
                                "tease",
                                "prickteaser",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "dame",
                                "madam",
                                "ma'am",
                                "lady",
                                "gentlewoman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "debutante",
                                "deb",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "divorcee",
                                "grass_widow",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "ex-wife",
                                "ex",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "dominatrix",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "donna",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "enchantress",
                                "temptress",
                                "siren",
                                "Delilah",
                                "femme_fatale",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "eyeful",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "geisha",
                                "geisha_girl",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "girl",
                                "miss",
                                "missy",
                                "young_lady",
                                "young_woman",
                                "fille",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "girl",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "girlfriend",
                                "girl",
                                "lady_friend",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "girlfriend",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "gold_digger",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "gravida",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "heroine",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "inamorata",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "jezebel",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "jilt",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "lady",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "maenad",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "matriarch",
                                "materfamilias",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "matriarch",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "matron",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "mestiza",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "mistress",
                                "kept_woman",
                                "fancy_woman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "mother_figure",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "nanny",
                                "nursemaid",
                                "nurse",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "nullipara",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "nymph",
                                "houri",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "nymphet",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "old_woman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "prostitute",
                                "cocotte",
                                "whore",
                                "harlot",
                                "bawd",
                                "tart",
                                "cyprian",
                                "fancy_woman",
                                "working_girl",
                                "sporting_lady",
                                "lady_of_pleasure",
                                "woman_of_the_street",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "shiksa",
                                "shikse",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "smasher",
                                "stunner",
                                "knockout",
                                "beauty",
                                "ravisher",
                                "sweetheart",
                                "peach",
                                "lulu",
                                "looker",
                                "mantrap",
                                "dish",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "sylph",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "unmarried_woman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "vestal",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "Wac",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "Wave",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "widow",
                                "widow_woman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "wife",
                                "married_woman",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: Hyponym,
                            words: [
                                "wonder_woman",
                            ],
                            part_of_speech: Noun,
                        },
                    ],
                },
                SynSet {
                    words: [
                        "woman",
                    ],
                    definition: "a female person who plays a significant role (wife or mistress or girlfriend) in the life of a particular man; \"he was faithful to his woman\"",
                    part_of_speech: Noun,
                    relationships: [
                        Relationship {
                            relationship_kind: Hypernym,
                            words: [
                                "female",
                                "female_person",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DomainOfSynsetUsage,
                            words: [
                                "colloquialism",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "philander",
                                "womanize",
                                "womanise",
                            ],
                            part_of_speech: Verb,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "philander",
                                "womanize",
                                "womanise",
                            ],
                            part_of_speech: Verb,
                        },
                        Relationship {
                            relationship_kind: Antonym,
                            words: [
                                "man",
                            ],
                            part_of_speech: Noun,
                        },
                    ],
                },
                SynSet {
                    words: [
                        "charwoman",
                        "char",
                        "cleaning_woman",
                        "cleaning_lady",
                        "woman",
                    ],
                    definition: "a human female employed to do housework; \"the char will clean the carpet\"; \"I have a woman who comes in four hours a day while I write\"",
                    part_of_speech: Noun,
                    relationships: [
                        Relationship {
                            relationship_kind: Hypernym,
                            words: [
                                "cleaner",
                            ],
                            part_of_speech: Noun,
                        },
                    ],
                },
                SynSet {
                    words: [
                        "womanhood",
                        "woman",
                        "fair_sex",
                    ],
                    definition: "women as a class; \"it's an insult to American womanhood\"; \"woman is the glory of creation\"; \"the fair sex gathered on the veranda\"",
                    part_of_speech: Noun,
                    relationships: [
                        Relationship {
                            relationship_kind: Hypernym,
                            words: [
                                "class",
                                "stratum",
                                "social_class",
                                "socio-economic_class",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: MemberHolonym,
                            words: [
                                "womankind",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "womanhood",
                                "muliebrity",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "womanhood",
                            ],
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relationship_kind: DerivationallyRelatedForm,
                            words: [
                                "woman",
                                "adult_female",
                            ],
                            part_of_speech: Noun,
                        },
                    ],
                },
            ]
        "#]];
        expected.assert_debug_eq(&syn);
    }

    #[test]
    fn woman_data_synonyms() {
        let word = "woman";
        let wndir = env::var("WORDNET").unwrap();
        let wn = WordNet::new(PathBuf::from(wndir));
        let syn = wn.synonyms(word);
        let expected = expect![[r#"
            {
                Noun: {
                    "adult_female",
                    "char",
                    "charwoman",
                    "cleaning_lady",
                    "cleaning_woman",
                    "fair_sex",
                    "woman",
                    "womanhood",
                },
            }
        "#]];
        expected.assert_debug_eq(&syn);
    }

    #[test]
    fn woman_data_antonyms() {
        let word = "woman";
        let wndir = env::var("WORDNET").unwrap();
        let wn = WordNet::new(PathBuf::from(wndir));
        let syn = wn.antonyms(word);
        let expected = expect![[r#"
            {
                Noun: {
                    "adult_male",
                    "man",
                },
            }
        "#]];
        expected.assert_debug_eq(&syn);
    }
}
