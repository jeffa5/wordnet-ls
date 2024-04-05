use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticRelation {
    // @    Hypernym
    Hypernym,
    // @i    Instance Hypernym
    InstanceHypernym,
    //  ~    Hyponym
    Hyponym,
    //  ~i    Instance Hyponym
    InstanceHyponym,
    // #m    Member holonym
    MemberHolonym,
    // #s    Substance holonym
    SubstanceHolonym,
    // #p    Part holonym
    PartHolonym,
    // %m    Member meronym
    MemberMeronym,
    // %s    Substance meronym
    SubstanceMeronym,
    // %p    Part meronym
    PartMeronym,
    // =    Attribute
    Attribute,
    // ;c    Domain of synset - TOPIC
    DomainOfSynsetTopic,
    // -c    Member of this domain - TOPIC
    MemberOfThisDomainTopic,
    // ;r    Domain of synset - REGION
    DomainOfSynsetRegion,
    // -r    Member of this domain - REGION
    MemberOfThisDomainRegion,
    // ;u    Domain of synset - USAGE
    DomainOfSynsetUsage,
    // -u    Member of this domain - USAGE
    MemberOfThisDomainUsage,
    // *    Entailment
    Entailment,
    // >    Cause
    Cause,
    // $    Verb Group
    VerbGroup,
    // &    Similar to
    SimilarTo,
    // \    Derived from adjective
    DerivedFromAdjective,
    // ^    Also see
    AlsoSee,
}

impl SemanticRelation {
    pub fn try_from_str(s: &str) -> Option<SemanticRelation> {
        match s {
            "@" => Some(SemanticRelation::Hypernym),
            "@i" => Some(SemanticRelation::InstanceHypernym),
            "~" => Some(SemanticRelation::Hyponym),
            "~i" => Some(SemanticRelation::InstanceHyponym),
            "#m" => Some(SemanticRelation::MemberHolonym),
            "#s" => Some(SemanticRelation::SubstanceHolonym),
            "#p" => Some(SemanticRelation::PartHolonym),
            "%m" => Some(SemanticRelation::MemberMeronym),
            "%s" => Some(SemanticRelation::SubstanceMeronym),
            "%p" => Some(SemanticRelation::PartMeronym),
            "=" => Some(SemanticRelation::Attribute),
            ";c" => Some(SemanticRelation::DomainOfSynsetTopic),
            "-c" => Some(SemanticRelation::MemberOfThisDomainTopic),
            ";r" => Some(SemanticRelation::DomainOfSynsetRegion),
            "-r" => Some(SemanticRelation::MemberOfThisDomainRegion),
            ";u" => Some(SemanticRelation::DomainOfSynsetUsage),
            "-u" => Some(SemanticRelation::MemberOfThisDomainUsage),
            "*" => Some(SemanticRelation::Entailment),
            ">" => Some(SemanticRelation::Cause),
            "$" => Some(SemanticRelation::VerbGroup),
            "&" => Some(SemanticRelation::SimilarTo),
            "\\" => Some(SemanticRelation::DerivedFromAdjective),
            "^" => Some(SemanticRelation::AlsoSee),
            _ => None,
        }
    }
}

impl Display for SemanticRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SemanticRelation::Hypernym => "hypernym",
            SemanticRelation::InstanceHypernym => "instance hypernym",
            SemanticRelation::Hyponym => "hyponym",
            SemanticRelation::InstanceHyponym => "instance hyponym",
            SemanticRelation::MemberHolonym => "member holonym",
            SemanticRelation::SubstanceHolonym => "substance holonym",
            SemanticRelation::PartHolonym => "part holonym",
            SemanticRelation::MemberMeronym => "member meronym",
            SemanticRelation::SubstanceMeronym => "substance meronym",
            SemanticRelation::PartMeronym => "part meronym",
            SemanticRelation::Attribute => "attribute",
            SemanticRelation::DomainOfSynsetTopic => "domain of synset topic",
            SemanticRelation::MemberOfThisDomainTopic => "member of this domain topic",
            SemanticRelation::DomainOfSynsetRegion => "domain of synset region",
            SemanticRelation::MemberOfThisDomainRegion => "member of this domain region",
            SemanticRelation::DomainOfSynsetUsage => "domain of synset usage",
            SemanticRelation::MemberOfThisDomainUsage => "member of this domain usage",
            SemanticRelation::Entailment => "entailment",
            SemanticRelation::Cause => "cause",
            SemanticRelation::VerbGroup => "verb group",
            SemanticRelation::SimilarTo => "similar to",
            SemanticRelation::DerivedFromAdjective => "derived from adjective",
            SemanticRelation::AlsoSee => "also see",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LexicalRelation {
    // !    Antonym
    Antonym,
    // +    Derivationally related form
    DerivationallyRelatedForm,
    // ^    Also see
    AlsoSee,
    // <    Participle of verb
    ParticipleOfVerb,
    // \    Pertainym (pertains to noun)
    Pertainym,
    // ;u    Domain of synset - USAGE
    DomainOfSynsetUsage,
    // ;r    Domain of synset - REGION
    DomainOfSynsetRegion,
    // -r    Member of this domain - REGION
    MemberOfThisDomainRegion,
    // -u    Member of this domain - USAGE
    MemberOfThisDomainUsage,
    // $    Verb Group
    VerbGroup,
    // -c    Member of this domain - TOPIC
    MemberOfThisDomainTopic,
    // ;c    Domain of synset - TOPIC
    DomainOfSynsetTopic,
}

impl LexicalRelation {
    pub fn try_from_str(s: &str) -> Option<LexicalRelation> {
        match s {
            "!" => Some(LexicalRelation::Antonym),
            "+" => Some(LexicalRelation::DerivationallyRelatedForm),
            "^" => Some(LexicalRelation::AlsoSee),
            "<" => Some(LexicalRelation::ParticipleOfVerb),
            "\\" => Some(LexicalRelation::Pertainym),
            ";u" => Some(LexicalRelation::DomainOfSynsetUsage),
            ";r" => Some(LexicalRelation::DomainOfSynsetRegion),
            "-r" => Some(LexicalRelation::MemberOfThisDomainRegion),
            "-u" => Some(LexicalRelation::MemberOfThisDomainRegion),
            "$" => Some(LexicalRelation::VerbGroup),
            "-c" => Some(LexicalRelation::MemberOfThisDomainTopic),
            ";c" => Some(LexicalRelation::DomainOfSynsetTopic),
            _ => None,
        }
    }
}

impl Display for LexicalRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LexicalRelation::Antonym => "antonym",
            LexicalRelation::DerivationallyRelatedForm => "derivationally related form",
            LexicalRelation::AlsoSee => "also see",
            LexicalRelation::ParticipleOfVerb => "participle of verb",
            LexicalRelation::Pertainym => "pertainym",
            LexicalRelation::DomainOfSynsetUsage => "domain of synset usage",
            LexicalRelation::DomainOfSynsetRegion => "domain of synset region",
            LexicalRelation::MemberOfThisDomainRegion => "member of this domain region",
            LexicalRelation::MemberOfThisDomainUsage => "member of this domain usage",
            LexicalRelation::VerbGroup => "verb group",
            LexicalRelation::MemberOfThisDomainTopic => "member of this domain topic",
            LexicalRelation::DomainOfSynsetTopic => "domain of synset topic",
        };
        f.write_str(s)
    }
}
