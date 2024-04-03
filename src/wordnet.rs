use data::Data;
use index::Index;
pub use pos::PartOfSpeech;
pub use relation::Relation;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
pub use synset::SynSet;

mod data;
mod index;
mod pos;
mod relation;
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

    /// Directly resolve a reference, this should only be used with part_of_speech, offset pairs
    /// from the returned results, such as the relationships in synsets.
    pub fn resolve(&self, part_of_speech: PartOfSpeech, offset: u64) -> Option<SynSet> {
        self.data.load(&self.database, offset, part_of_speech)
    }

    pub fn iter_words(&self, part_of_speech: PartOfSpeech) -> Vec<String> {
        self.index.words_for(&self.database, part_of_speech)
    }

    pub fn all_words(&self) -> Vec<String> {
        let mut result = Vec::new();
        for pos in [
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adverb,
            PartOfSpeech::Adjective,
        ] {
            result.append(&mut self.index.words_for(&self.database, pos))
        }
        result.sort_unstable();
        result.dedup();
        result
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
        self.with_relationship(word, Relation::Antonym)
    }

    pub fn with_relationship(
        &self,
        word: &str,
        relationship: Relation,
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
                                .flat_map(|r| {
                                    self.data.load(
                                        &self.database,
                                        r.synset_offset,
                                        r.part_of_speech,
                                    )
                                })
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
    fn woman_data_synset_resolve() {
        let word = "woman";
        let wndir = env::var("WORDNET").unwrap();
        let wn = WordNet::new(PathBuf::from(wndir));
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
                        words: [
                            "female",
                            "female_person",
                        ],
                        definition: "a person who belongs to the sex that can have babies",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 7846,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1484451,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: Antonym,
                                synset_offset: 9624168,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: PartMeronym,
                                synset_offset: 5219923,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10084043,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10084295,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10106995,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10131151,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "adult",
                            "grownup",
                        ],
                        definition: "a fully developed person from maturity onward",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 7846,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1488616,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1488616,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 15152817,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 14425103,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Antonym,
                                synset_offset: 9622049,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: PartMeronym,
                                synset_offset: 5219561,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9872464,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9895561,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9900981,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9904837,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9909060,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9957156,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10024025,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10048218,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10074249,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10187130,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10200781,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10218164,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10256756,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10257084,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10287213,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10316013,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10376523,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10390199,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10407105,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10441534,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10441694,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10480253,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10618146,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10625285,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10653388,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10658867,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10709358,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "womanly",
                            "feminine",
                        ],
                        definition: "befitting or characteristic of a woman especially a mature woman; \"womanly virtues of gentleness and compassion\"",
                        part_of_speech: Adjective,
                        relationships: [
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 5008943,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 4667660,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Antonym,
                                synset_offset: 1485502,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: SimilarTo,
                                synset_offset: 1485254,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: SimilarTo,
                                synset_offset: 1485401,
                                part_of_speech: Adjective,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "womanhood",
                            "muliebrity",
                        ],
                        definition: "the state of being an adult woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 14425103,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 8477634,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
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
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "womanhood",
                        ],
                        definition: "the status of a woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 586262,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 8477634,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "feminize",
                            "feminise",
                            "effeminize",
                            "effeminise",
                            "womanize",
                        ],
                        definition: "to give a (more) feminine, effeminate, or womanly quality or appearance to; \"This hairdo feminizes the man\"",
                        part_of_speech: Verb,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 126264,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "philander",
                            "womanize",
                            "womanise",
                        ],
                        definition: "have amorous affairs; of men; \"He has been womanizing for years\"",
                        part_of_speech: Verb,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 2376958,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Antonym,
                    SynSet {
                        words: [
                            "man",
                            "adult_male",
                        ],
                        definition: "an adult person who is male (as opposed to a woman); \"there were two women and six men on the bus\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 9624168,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 9605289,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1483677,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1475831,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: Antonym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: PartMeronym,
                                synset_offset: 5220306,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 9586553,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 9586994,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 9587217,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 9587409,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9637211,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9641002,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9643545,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9827246,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9829923,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9852289,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9870926,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9871364,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9871580,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9878702,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9991026,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10048117,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10064046,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10066452,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10081670,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10081842,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10083358,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10118301,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10123711,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10127273,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10144338,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10153414,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 10158222,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10173771,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10184822,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10188369,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10192839,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10194231,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10202225,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10216560,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10216839,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 10219778,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10288964,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10314054,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10329675,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10375214,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10375506,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10406765,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10419785,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10452752,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10456391,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10578952,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10586903,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 10588860,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10597745,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10598013,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10601451,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10656120,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10665587,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10692090,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10780506,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10789415,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10804287,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    PartMeronym,
                    SynSet {
                        words: [
                            "adult_female_body",
                            "woman's_body",
                        ],
                        definition: "the body of an adult woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 5219561,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 5219923,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: PartHolonym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: PartMeronym,
                                synset_offset: 5402576,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "Eve",
                        ],
                        definition: "(Old Testament) Adam's wife in Judeo-Christian mythology: the first woman and mother of the human race; God created Eve from Adam's rib and placed Adam and Eve in the Garden of Eden",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: InstanceHypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "Black_woman",
                        ],
                        definition: "a woman who is Black",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "white_woman",
                        ],
                        definition: "a woman who is White",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "yellow_woman",
                        ],
                        definition: "offensive term for an Asian woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 9643078,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "amazon",
                            "virago",
                        ],
                        definition: "a large strong and aggressive woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "maenad",
                        ],
                        definition: "(Greek mythology) a woman participant in the orgiastic rites of Dionysus",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "bachelor_girl",
                            "bachelorette",
                        ],
                        definition: "a young unmarried woman who lives alone",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "baggage",
                        ],
                        definition: "a worthless or immoral woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "ball-buster",
                            "ball-breaker",
                        ],
                        definition: "a demanding woman who destroys men's confidence",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "B-girl",
                            "bar_girl",
                        ],
                        definition: "a woman employed by a bar to act as a companion to men customers",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "bluestocking",
                            "bas_bleu",
                        ],
                        definition: "a woman having literary or intellectual interests",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "bridesmaid",
                            "maid_of_honor",
                        ],
                        definition: "an unmarried woman who attends the bride at a wedding",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 9821831,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "broad",
                        ],
                        definition: "slang term for a woman; \"a broad is a woman who can throw a mean punch\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "cat",
                        ],
                        definition: "a spiteful woman gossip; \"what a cat she is!\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10139347,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 225912,
                                part_of_speech: Adjective,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        words: [
                            "Cinderella",
                        ],
                        definition: "a woman whose merits were not been recognized but who then achieves sudden success and recognition",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "coquette",
                            "flirt",
                            "vamp",
                            "vamper",
                            "minx",
                            "tease",
                            "prickteaser",
                        ],
                        definition: "a seductive woman who uses her sex appeal to exploit men",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 852506,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1803641,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1038538,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1038538,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 2131958,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1037910,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1037910,
                                part_of_speech: Verb,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        words: [
                            "dame",
                            "madam",
                            "ma'am",
                            "lady",
                            "gentlewoman",
                        ],
                        definition: "a woman of refinement; \"a chauffeur opened the door of the limousine for the grand lady\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10142166,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "debutante",
                            "deb",
                        ],
                        definition: "a young woman making her debut into society",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "divorcee",
                            "grass_widow",
                        ],
                        definition: "a divorced woman or a woman who is separated from her husband",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 2490634,
                                part_of_speech: Verb,
                            },
                            Relationship {
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
                        words: [
                            "ex-wife",
                            "ex",
                        ],
                        definition: "a woman who was formerly a particular man's wife; \"all his exes live in Texas\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "dominatrix",
                        ],
                        definition: "a dominating woman (especially one who plays that role in a sadomasochistic sexual relationship)",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "donna",
                        ],
                        definition: "an Italian woman of rank",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "enchantress",
                            "temptress",
                            "siren",
                            "Delilah",
                            "femme_fatale",
                        ],
                        definition: "a woman who is considered to be dangerously seductive",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "eyeful",
                        ],
                        definition: "a strikingly beautiful woman; \"she was a statuesque redheaded eyeful\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "geisha",
                            "geisha_girl",
                        ],
                        definition: "a Japanese woman trained to entertain men with conversation and singing and dancing",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "girl",
                            "miss",
                            "missy",
                            "young_lady",
                            "young_woman",
                            "fille",
                        ],
                        definition: "a young woman; \"a young lady of 18\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 15147330,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9827363,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9849012,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9854708,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9905530,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9919451,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9936825,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9989045,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10095420,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10117851,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10119609,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10129338,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10247358,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10282482,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10304160,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10318193,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10403366,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10416364,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10531694,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10539160,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10584729,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10592049,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10626994,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10682599,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10715030,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10745770,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "girl",
                        ],
                        definition: "a friendly informal reference to a grown woman; \"Mrs. Smith was just one of the girls\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "girlfriend",
                            "girl",
                            "lady_friend",
                        ],
                        definition: "a girl or young woman with whom a man is romantically involved; \"his girlfriend kicked him out\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "girlfriend",
                        ],
                        definition: "any female friend; \"Mary and her girlfriend organized the party\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "gold_digger",
                        ],
                        definition: "a woman who associates with or marries a rich man in order to get valuables from him through gifts or a divorce settlement",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "gravida",
                        ],
                        definition: "a pregnant woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10472129,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10574723,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "heroine",
                        ],
                        definition: "a woman possessing heroic qualities or a woman who has performed heroic deeds",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 10226219,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "inamorata",
                        ],
                        definition: "a woman with whom you are in love or have an intimate relationship",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "jezebel",
                        ],
                        definition: "a shameless impudent scheming woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "jilt",
                        ],
                        definition: "a woman who jilts a lover",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 613248,
                                part_of_speech: Verb,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        words: [
                            "lady",
                        ],
                        definition: "a polite name for any woman; \"a nice lady at the library helped me\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "maenad",
                        ],
                        definition: "an unnaturally frenzied or distraught woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "matriarch",
                            "materfamilias",
                        ],
                        definition: "a female head of a family or tribe",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "matriarch",
                        ],
                        definition: "a feisty older woman with a big bosom (as drawn in cartoons)",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "matron",
                        ],
                        definition: "a woman in charge of nursing in a medical institution",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10366966,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "mestiza",
                        ],
                        definition: "a woman of mixed racial ancestry (especially mixed European and Native American ancestry)",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "mistress",
                            "kept_woman",
                            "fancy_woman",
                        ],
                        definition: "an adulterous woman; a woman who has an ongoing extramarital sexual relationship with a man",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 9622745,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9952393,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 10863440,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "mother_figure",
                        ],
                        definition: "a woman who evokes the feelings usually reserved for a mother",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "nanny",
                            "nursemaid",
                            "nurse",
                        ],
                        definition: "a woman who is the custodian of children",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10229498,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1186428,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10038119,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10287082,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "nullipara",
                        ],
                        definition: "(obstetrics) a woman who has never give birth to a child",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "nymph",
                            "houri",
                        ],
                        definition: "a voluptuously beautiful young woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "nymphet",
                        ],
                        definition: "a sexually attractive young woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "old_woman",
                        ],
                        definition: "a woman who is old",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10376523,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9847543,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10143530,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10155485,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        definition: "a woman who engages in sexual intercourse for money",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 424787,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 748155,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 2580577,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1427695,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 2554066,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9889065,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9890296,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9941172,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10001882,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10663315,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "shiksa",
                            "shikse",
                        ],
                        definition: "a derogatory term used by Jews to refer to non-Jewish women",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DomainOfSynsetUsage,
                                synset_offset: 6717170,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        definition: "a very attractive or seductive looking woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 218673,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 2115430,
                                part_of_speech: Verb,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        words: [
                            "sylph",
                        ],
                        definition: "a slender graceful young woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "unmarried_woman",
                        ],
                        definition: "a woman who is not married",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10598181,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "vestal",
                        ],
                        definition: "a chaste woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 361125,
                                part_of_speech: Adjective,
                            },
                        ],
                    },
                ),
                (
                    Hyponym,
                    SynSet {
                        words: [
                            "Wac",
                        ],
                        definition: "a member of the Women's Army Corps",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10622053,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "Wave",
                        ],
                        definition: "a member of the women's reserve of the United States Navy; originally organized during World War II but now no longer a separate branch",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10523341,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "widow",
                            "widow_woman",
                        ],
                        definition: "a woman whose husband is dead especially one who has not remarried",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 15153667,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 13967970,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 360337,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10028289,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "wife",
                            "married_woman",
                        ],
                        definition: "a married woman; a man's partner in marriage",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10640620,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1735475,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: Antonym,
                                synset_offset: 10193967,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9844356,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9981278,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10092794,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10137498,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10183347,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10189776,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10293773,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10303037,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10304086,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10323529,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10375314,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10588519,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10597889,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10730820,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10743941,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10752020,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10756061,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 10838288,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 10887593,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 11034485,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 11251384,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 11255775,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: InstanceHyponym,
                                synset_offset: 11275952,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "wonder_woman",
                        ],
                        definition: "a woman who can be a successful wife and have a professional career at the same time",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
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
                        words: [
                            "female",
                            "female_person",
                        ],
                        definition: "a person who belongs to the sex that can have babies",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 7846,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1484451,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: Antonym,
                                synset_offset: 9624168,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: PartMeronym,
                                synset_offset: 5219923,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10084043,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10084295,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10106995,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10131151,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "colloquialism",
                        ],
                        definition: "a colloquial expression; characteristic of spoken or written communication that seeks to imitate informal speech",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 7069948,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4817,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 19505,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 20647,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 22437,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 33077,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 92136,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 115094,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 115193,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 115906,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 167520,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 218673,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 228876,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 265314,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 294463,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 341655,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 437223,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 439905,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 452114,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 456929,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 458266,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 459953,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 476819,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 478311,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 532560,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 653617,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 680156,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 680634,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 694773,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 733632,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 735882,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 748563,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 750054,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 765289,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 799401,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 806243,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 811248,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 850875,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 855309,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 886448,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 901650,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 919984,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 933415,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 971660,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 972354,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 972501,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 975011,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 975778,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 976016,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1019450,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1054367,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1075524,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1086213,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1123879,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1127440,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1127782,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1129021,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1138450,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1201298,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1267339,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1274741,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1276872,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1280908,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1281874,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1307571,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1332907,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1388062,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1389022,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1391074,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1392633,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1392896,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1403632,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1423187,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1462461,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1468850,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1510628,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1520908,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1533659,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1538583,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1586194,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1677200,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1678586,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1709681,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1766958,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1777662,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1795353,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1816525,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1836766,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1850446,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1850742,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1863442,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1871349,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1880071,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1914250,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1917594,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2076234,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2077904,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2081114,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2204580,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2227485,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2257601,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2298642,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2337558,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2341864,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2342463,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2346013,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2347742,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2358650,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2358762,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2358898,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2383564,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2407346,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2421364,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2432154,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2434473,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2509710,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2512044,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2571536,
                                part_of_speech: Adjective,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3846,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 8007,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9541,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15471,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 25290,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 25559,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 25728,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 32598,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 33809,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 39318,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 54950,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 56916,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 57042,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 57388,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 58033,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 89076,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 91032,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 101752,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 104661,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 118727,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 144722,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 147876,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 148139,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 161630,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 164676,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 168564,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 200614,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 277585,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 354033,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 355080,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 426140,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 426278,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 472323,
                                part_of_speech: Adverb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 104088,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 323262,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 580190,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 583089,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 797468,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 802785,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 854393,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1096674,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1172598,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1260556,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1260731,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1503976,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2324587,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2834506,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3173142,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3404012,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3436182,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3655838,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3688192,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 3711603,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4027820,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4055595,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4056491,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4066023,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4095109,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4165811,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4201992,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4355115,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4552097,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4761960,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4815177,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 4931267,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5032351,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5084733,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5096294,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5105009,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5206445,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5210820,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5311054,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5312040,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5599084,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5601357,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5687832,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5705484,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5786372,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5812485,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5828102,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5828263,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5831939,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 5921685,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6322357,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6397645,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6547832,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6610436,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6716796,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 6947658,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7141537,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7142924,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7144416,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7246215,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7292418,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7319399,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7320894,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7519983,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7539962,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7677860,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7907037,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 7960666,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 8245425,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 8590719,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9125984,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9270508,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9605110,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9853881,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9879144,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9890662,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9891300,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 9976283,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10011785,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10013114,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10022908,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10026367,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10085101,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10097477,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10114662,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10117851,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10123711,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10162780,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10163593,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10194566,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10240235,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10251329,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10288516,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10314182,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10335801,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10397142,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10505459,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10531557,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10591347,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10627899,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10628368,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10638136,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10642845,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10666615,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10709876,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10719395,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10762342,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13259797,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13272712,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13366428,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13371190,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13372123,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13772313,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13927112,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13937727,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 13988498,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14018055,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14038027,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14360742,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14405061,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14485673,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14485811,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14485990,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 14521954,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15048623,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15060569,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15170178,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15243202,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 15244200,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 590241,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 1906322,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2010698,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2167210,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2532458,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: MemberOfThisDomainUsage,
                                synset_offset: 2600082,
                                part_of_speech: Verb,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "philander",
                            "womanize",
                            "womanise",
                        ],
                        definition: "have amorous affairs; of men; \"He has been womanizing for years\"",
                        part_of_speech: Verb,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 2376958,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "philander",
                            "womanize",
                            "womanise",
                        ],
                        definition: "have amorous affairs; of men; \"He has been womanizing for years\"",
                        part_of_speech: Verb,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 2376958,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10789118,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Antonym,
                    SynSet {
                        words: [
                            "man",
                        ],
                        definition: "a male person who plays a significant role (husband or lover or boyfriend) in the life of a particular woman; \"she takes good care of her man\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 9624168,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 9622745,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DomainOfSynsetUsage,
                                synset_offset: 7075172,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Antonym,
                                synset_offset: 10788852,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    Hypernym,
                    SynSet {
                        words: [
                            "cleaner",
                        ],
                        definition: "someone whose occupation is cleaning",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 10241300,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1533442,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 1532589,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9911226,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 9919061,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10562645,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 10662474,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "class",
                            "stratum",
                            "social_class",
                            "socio-economic_class",
                        ],
                        definition: "people having the same social, economic, or educational status; \"the working class\"; \"an emerging professional class\"",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 7942152,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: PartHolonym,
                                synset_offset: 7966140,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberHolonym,
                                synset_offset: 8378555,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 739662,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 654625,
                                part_of_speech: Verb,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 7965937,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 7974766,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8075287,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8075388,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8167365,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8180639,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8181540,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8181658,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8181820,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8181930,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8182283,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8244895,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8245059,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8246502,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8306047,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8306194,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8386365,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8389094,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8415983,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8416137,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8416523,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8417572,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8424951,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: Hyponym,
                                synset_offset: 8436562,
                                part_of_speech: Noun,
                            },
                            Relationship {
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
                        words: [
                            "womankind",
                        ],
                        definition: "women as distinguished from men",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 7942152,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: MemberMeronym,
                                synset_offset: 8477634,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "womanhood",
                            "muliebrity",
                        ],
                        definition: "the state of being an adult woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 14425103,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 8477634,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
                    SynSet {
                        words: [
                            "womanhood",
                        ],
                        definition: "the status of a woman",
                        part_of_speech: Noun,
                        relationships: [
                            Relationship {
                                relation: Hypernym,
                                synset_offset: 586262,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 10787470,
                                part_of_speech: Noun,
                            },
                            Relationship {
                                relation: DerivationallyRelatedForm,
                                synset_offset: 8477634,
                                part_of_speech: Noun,
                            },
                        ],
                    },
                ),
                (
                    DerivationallyRelatedForm,
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
                ),
            ]
        "#]];
        expected.assert_debug_eq(&resolved_related);
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

    #[test]
    fn all_words() {
        let wndir = env::var("WORDNET").unwrap();
        let wn = WordNet::new(PathBuf::from(wndir));
        let len = wn.all_words().len();
        let expected = expect![[r#"
            147306
        "#]];
        expected.assert_debug_eq(&len);
    }

    #[test]
    fn all_words_cause() {
        let wndir = env::var("WORDNET").unwrap();
        let wn = WordNet::new(PathBuf::from(wndir));
        let words = wn
            .all_words()
            .into_iter()
            .map(|w| wn.with_relationship(&w, Relation::Cause))
            .count();
        let expected = expect![[]];
        expected.assert_debug_eq(&words);
    }
}
