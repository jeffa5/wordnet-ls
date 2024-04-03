use data::Data;
use index::Index;
pub use pos::PartOfSpeech;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use self::{pointer::PointerType, synset::SynSet};

mod data;
mod index;
mod pointer;
mod pos;
mod synset;

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
                    .extend(items.iter().map(|x| x.definition.clone()))
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
        let mut filtered: BTreeMap<PartOfSpeech, BTreeSet<String>> = BTreeMap::new();

        for item in items {
            for offset in item.syn_offsets.iter() {
                filtered.entry(item.pos).or_default().extend(
                    self.data
                        .load(&self.database, *offset, item.pos)
                        .iter()
                        .flat_map(|x| {
                            x.relationships
                                .iter()
                                .filter(|r| r.relation == relationship)
                                .flat_map(|r| self.data.load(&self.database, r.synset_offset, r.part_of_speech))
                                .flat_map(|di| di.words)
                        }),
                );
            }
        }

        filtered
    }

    pub fn synsets(&self, word: &str) -> Vec<SynSet> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut synsets = Vec::new();

        for item in items {
            for offset in item.syn_offsets.iter() {
                let synset = self.data.load(&self.database, *offset, item.pos);
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
                            relation: Hypernym,
                            synset_offset: 9619168,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hypernym,
                            synset_offset: 9605289,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 1484987,
                            part_of_speech: Adjective,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 14425715,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 8477634,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 606006,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 566322,
                            part_of_speech: Verb,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 2590910,
                            part_of_speech: Verb,
                        },
                        Relationship {
                            relation: Antonym,
                            synset_offset: 10287213,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: PartMeronym,
                            synset_offset: 5220126,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: InstanceHyponym,
                            synset_offset: 9586743,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9637339,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9641130,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9643670,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9787293,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9787390,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9830080,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9832456,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9834258,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9852430,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9861599,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9874862,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9875663,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9900153,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9923263,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9965134,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9989290,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 9997834,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10020366,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10020533,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10024784,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10025635,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10055410,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10075063,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10122858,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10129825,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10130447,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10130686,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10130877,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10136283,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10144838,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10173410,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10202085,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10222170,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10222259,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10243137,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10280034,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10302576,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10302700,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10303186,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10311661,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10323752,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10333044,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10345100,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10366145,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10368528,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10368624,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10377021,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10485440,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10589243,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10613996,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10685398,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10739512,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10748804,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10761962,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10771066,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10780284,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10780632,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: Hyponym,
                            synset_offset: 10789820,
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
                            relation: Hypernym,
                            synset_offset: 9619168,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DomainOfSynsetUsage,
                            synset_offset: 7075172,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 2590910,
                            part_of_speech: Verb,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 2590910,
                            part_of_speech: Verb,
                        },
                        Relationship {
                            relation: Antonym,
                            synset_offset: 10288516,
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
                            relation: Hypernym,
                            synset_offset: 9927089,
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
                            relation: Hypernym,
                            synset_offset: 7974025,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: MemberHolonym,
                            synset_offset: 8477912,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 14425715,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 606006,
                            part_of_speech: Noun,
                        },
                        Relationship {
                            relation: DerivationallyRelatedForm,
                            synset_offset: 10787470,
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
