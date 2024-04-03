use data::Data;
use index::Index;
use pos::PartOfSpeech;
use std::{collections::BTreeSet, path::PathBuf};

mod data;
mod index;
mod pos;
mod synset;

pub struct WordNet {
    index: Index,
    data: Data,
    database: PathBuf,
}

#[derive(Debug)]
pub struct Definition {
    pub pos: PartOfSpeech,
    pub def: String,
}

#[derive(Debug)]
pub struct Synonyms {
    pub pos: PartOfSpeech,
    pub synonyms: BTreeSet<String>,
}

impl WordNet {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            index: Index::new(),
            data: Data::new(),
            database: dir,
        }
    }

    pub fn definitions(&mut self, word: &str) -> Vec<Definition> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut vec = Vec::new();

        for i in items {
            for o in i.syn_offsets.iter() {
                let items = self.data.load(&self.database, *o, i.pos);
                vec.append(
                    &mut items
                        .iter()
                        .map(|x| Definition {
                            pos: i.pos,
                            def: x.gloss.clone(),
                        })
                        .collect(),
                )
            }
        }
        vec
    }

    pub fn synonyms(&mut self, word: &str) -> Vec<Synonyms> {
        let word = word.to_lowercase();
        let items = self.index.load(&self.database, &word);
        let mut synonyms = Vec::new();

        for item in items {
            let mut words = BTreeSet::new();
            for offset in item.syn_offsets.iter() {
                for word in self
                    .data
                    .load(&self.database, *offset, item.pos)
                    .iter()
                    .flat_map(|x| x.words.clone())
                {
                    words.insert(word);
                }
            }
            synonyms.push(Synonyms {
                pos: item.pos,
                synonyms: words,
            });
        }

        synonyms
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
        let mut wn = WordNet::new(PathBuf::from(wndir));
        let def = wn.definitions(word);
        let expected = expect![[r#"
            [
                Definition {
                    pos: Noun,
                    def: "a score in baseball made by a runner touching all four bases safely; \"the Yankees scored 3 runs in the bottom of the 9th\"; \"their first tally came in the 3rd inning\"",
                },
                Definition {
                    pos: Noun,
                    def: "the act of testing something; \"in the experimental trials the amount of carbon was measured separately\"; \"he called each flip of the coin a new trial\"",
                },
                Definition {
                    pos: Noun,
                    def: "a race run on foot; \"she broke the record for the half-mile run\"",
                },
                Definition {
                    pos: Noun,
                    def: "an unbroken series of events; \"had a streak of bad luck\"; \"Nicklaus had a run of birdies\"",
                },
                Definition {
                    pos: Noun,
                    def: "(American football) a play in which a player attempts to carry the ball through or past the opposing team; \"the defensive line braced to stop the run\"; \"the coach put great emphasis on running\"",
                },
                Definition {
                    pos: Noun,
                    def: "a regular trip; \"the ship made its run in record time\"",
                },
                Definition {
                    pos: Noun,
                    def: "the act of running; traveling on foot at a fast pace; \"he broke into a run\"; \"his daily run keeps him fit\"",
                },
                Definition {
                    pos: Noun,
                    def: "the continuous period of time during which something (a machine or a factory) operates or continues in operation; \"the assembly line was on a 12-hour run\"",
                },
                Definition {
                    pos: Noun,
                    def: "unrestricted freedom to use; \"he has the run of the house\"",
                },
                Definition {
                    pos: Noun,
                    def: "the production achieved during a continuous period of operation (of a machine or factory etc.); \"a daily run of 100,000 gallons of paint\"",
                },
                Definition {
                    pos: Noun,
                    def: "a small stream",
                },
                Definition {
                    pos: Noun,
                    def: "a race between candidates for elective office; \"I managed his campaign for governor\"; \"he is raising money for a Senate run\"",
                },
                Definition {
                    pos: Noun,
                    def: "a row of unravelled stitches; \"she got a run in her stocking\"",
                },
                Definition {
                    pos: Noun,
                    def: "the pouring forth of a fluid",
                },
                Definition {
                    pos: Noun,
                    def: "an unbroken chronological sequence; \"the play had a long run on Broadway\"; \"the team enjoyed a brief run of victories\"",
                },
                Definition {
                    pos: Noun,
                    def: "a short trip; \"take a run into town\"",
                },
                Definition {
                    pos: Verb,
                    def: "move fast by using one's feet, with one foot off the ground at any given time; \"Don't run--you'll be out of breath\"; \"The children ran to the store\"",
                },
                Definition {
                    pos: Verb,
                    def: "flee; take to one's heels; cut and run; \"If you see this man, run!\"; \"The burglars escaped before the police showed up\"",
                },
                Definition {
                    pos: Verb,
                    def: "stretch out over a distance, space, time, or scope; run or extend between two points or beyond a certain point; \"Service runs all the way to Cranbury\"; \"His knowledge doesn't go very far\"; \"My memory extends back to my fourth year of life\"; \"The facts extend beyond a consideration of her personal assets\"",
                },
                Definition {
                    pos: Verb,
                    def: "direct or control; projects, businesses, etc.; \"She is running a relief operation in the Sudan\"",
                },
                Definition {
                    pos: Verb,
                    def: "have a particular form; \"the story or argument runs as follows\"; \"as the saying goes...\"",
                },
                Definition {
                    pos: Verb,
                    def: "move along, of liquids; \"Water flowed into the cave\"; \"the Missouri feeds into the Mississippi\"",
                },
                Definition {
                    pos: Verb,
                    def: "perform as expected when applied; \"The washing machine won't go unless it's plugged in\"; \"Does this old car still run well?\"; \"This old radio doesn't work anymore\"",
                },
                Definition {
                    pos: Verb,
                    def: "change or be different within limits; \"Estimates for the losses in the earthquake range as high as $2 billion\"; \"Interest rates run from 5 to 10 percent\"; \"The instruments ranged from tuba to cymbals\"; \"My students range from very bright to dull\"",
                },
                Definition {
                    pos: Verb,
                    def: "run, stand, or compete for an office or a position; \"Who's running for treasurer this year?\"",
                },
                Definition {
                    pos: Verb,
                    def: "cause to emit recorded audio or video; \"They ran the tapes over and over again\"; \"I'll play you my favorite record\"; \"He never tires of playing that video\"",
                },
                Definition {
                    pos: Verb,
                    def: "move about freely and without restraint, or act as if running around in an uncontrolled way; \"who are these people running around in the building?\"; \"She runs around telling everyone of her troubles\"; \"let the dogs run free\"",
                },
                Definition {
                    pos: Verb,
                    def: "have a tendency or disposition to do or be something; be inclined; \"She tends to be nervous before her lectures\"; \"These dresses run small\"; \"He inclined to corpulence\"",
                },
                Definition {
                    pos: Verb,
                    def: "be operating, running or functioning; \"The car is still running--turn it off!\"",
                },
                Definition {
                    pos: Verb,
                    def: "change from one state to another; \"run amok\"; \"run rogue\"; \"run riot\"",
                },
                Definition {
                    pos: Verb,
                    def: "cause to perform; \"run a subject\"; \"run a process\"",
                },
                Definition {
                    pos: Verb,
                    def: "be affected by; be subjected to; \"run a temperature\"; \"run a risk\"",
                },
                Definition {
                    pos: Verb,
                    def: "continue to exist; \"These stories die hard\"; \"The legend of Elvis endures\"",
                },
                Definition {
                    pos: Verb,
                    def: "occur persistently; \"Musical talent runs in the family\"",
                },
                Definition {
                    pos: Verb,
                    def: "carry out a process or program, as on a computer or a machine; \"Run the dishwasher\"; \"run a new program on the Mac\"; \"the computer executed the instruction\"",
                },
                Definition {
                    pos: Verb,
                    def: "include as the content; broadcast or publicize; \"We ran the ad three times\"; \"This paper carries a restaurant review\"; \"All major networks carried the press conference\"",
                },
                Definition {
                    pos: Verb,
                    def: "carry out; \"run an errand\"",
                },
                Definition {
                    pos: Verb,
                    def: "pass over, across, or through; \"He ran his eyes over her body\"; \"She ran her fingers along the carved figurine\"; \"He drew her hair through his fingers\"",
                },
                Definition {
                    pos: Verb,
                    def: "cause something to pass or lead somewhere; \"Run the wire behind the cabinet\"",
                },
                Definition {
                    pos: Verb,
                    def: "make without a miss",
                },
                Definition {
                    pos: Verb,
                    def: "deal in illegally, such as arms or liquor",
                },
                Definition {
                    pos: Verb,
                    def: "cause an animal to move fast; \"run the dogs\"",
                },
                Definition {
                    pos: Verb,
                    def: "be diffused; \"These dyes and colors are guaranteed not to run\"",
                },
                Definition {
                    pos: Verb,
                    def: "sail before the wind",
                },
                Definition {
                    pos: Verb,
                    def: "cover by running; run a certain distance; \"She ran 10 miles that day\"",
                },
                Definition {
                    pos: Verb,
                    def: "extend or continue for a certain period of time; \"The film runs 5 hours\"",
                },
                Definition {
                    pos: Verb,
                    def: "set animals loose to graze",
                },
                Definition {
                    pos: Verb,
                    def: "keep company; \"the heifers run with the bulls to produce offspring\"",
                },
                Definition {
                    pos: Verb,
                    def: "run with the ball; in such sports as football",
                },
                Definition {
                    pos: Verb,
                    def: "travel rapidly, by any (unspecified) means; \"Run to the store!\"; \"She always runs to Italy, because she has a lover there\"",
                },
                Definition {
                    pos: Verb,
                    def: "travel a route regularly; \"Ships ply the waters near the coast\"",
                },
                Definition {
                    pos: Verb,
                    def: "pursue for food or sport (as of wild animals); \"Goering often hunted wild boars in Poland\"; \"The dogs are running deer\"; \"The Duke hunted in these woods\"",
                },
                Definition {
                    pos: Verb,
                    def: "compete in a race; \"he is running the Marathon this year\"; \"let's race and see who gets there first\"",
                },
                Definition {
                    pos: Verb,
                    def: "progress by being changed; \"The speech has to go through several more drafts\"; \"run through your presentation before the meeting\"",
                },
                Definition {
                    pos: Verb,
                    def: "reduce or cause to be reduced from a solid to a liquid state, usually by heating; \"melt butter\"; \"melt down gold\"; \"The wax melted in the sun\"",
                },
                Definition {
                    pos: Verb,
                    def: "come unraveled or undone as if by snagging; \"Her nylons were running\"",
                },
                Definition {
                    pos: Verb,
                    def: "become undone; \"the sweater unraveled\"",
                },
            ]
        "#]];
        expected.assert_debug_eq(&def);
    }

    #[test]
    fn multipos_data_synonyms() {
        let word = "run";
        let wndir = env::var("WORDNET").unwrap();
        let mut wn = WordNet::new(PathBuf::from(wndir));
        let syn = wn.synonyms(word);
        let expected = expect![[r#"
            [
                Synonyms {
                    pos: Noun,
                    synonyms: {
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
                },
                Synonyms {
                    pos: Verb,
                    synonyms: {
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
                },
            ]
        "#]];
        expected.assert_debug_eq(&syn);
    }
}
