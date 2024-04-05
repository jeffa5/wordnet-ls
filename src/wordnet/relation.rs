use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Relation {
    // !    Antonym
    Antonym,
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
    // +    Derivationally related form
    DerivationallyRelatedForm,
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
    // ^    Also see
    AlsoSee,
    // $    Verb Group
    VerbGroup,
    // &    Similar to
    SimilarTo,
    // <    Participle of verb
    ParticipleOfVerb,
    // \    Pertainym (pertains to noun)
    Pertainym,
    // \    Derived from adjective
    // DerivedFromAdjective,
}

impl Relation {
    pub fn try_from_str(s: &str) -> Option<Relation> {
        match s {
            "!" => Some(Relation::Antonym),
            "@" => Some(Relation::Hypernym),
            "@i" => Some(Relation::InstanceHypernym),
            "~" => Some(Relation::Hyponym),
            "~i" => Some(Relation::InstanceHyponym),
            "#m" => Some(Relation::MemberHolonym),
            "#s" => Some(Relation::SubstanceHolonym),
            "#p" => Some(Relation::PartHolonym),
            "%m" => Some(Relation::MemberMeronym),
            "%s" => Some(Relation::SubstanceMeronym),
            "%p" => Some(Relation::PartMeronym),
            "=" => Some(Relation::Attribute),
            "+" => Some(Relation::DerivationallyRelatedForm),
            ";c" => Some(Relation::DomainOfSynsetTopic),
            "-c" => Some(Relation::MemberOfThisDomainTopic),
            ";r" => Some(Relation::DomainOfSynsetRegion),
            "-r" => Some(Relation::MemberOfThisDomainRegion),
            ";u" => Some(Relation::DomainOfSynsetUsage),
            "-u" => Some(Relation::MemberOfThisDomainUsage),
            "*" => Some(Relation::Entailment),
            ">" => Some(Relation::Cause),
            "^" => Some(Relation::AlsoSee),
            "$" => Some(Relation::VerbGroup),
            "&" => Some(Relation::SimilarTo),
            "<" => Some(Relation::ParticipleOfVerb),
            "\\" => Some(Relation::Pertainym),
            _ => None,
        }
    }
}

impl Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Relation::Antonym => "antonym",
            Relation::Hypernym => "hypernym",
            Relation::InstanceHypernym => "instance hypernym",
            Relation::Hyponym => "hyponym",
            Relation::InstanceHyponym => "instance hyponym",
            Relation::MemberHolonym => "member holonym",
            Relation::SubstanceHolonym => "substance holonym",
            Relation::PartHolonym => "part holonym",
            Relation::MemberMeronym => "member meronym",
            Relation::SubstanceMeronym => "substance meronym",
            Relation::PartMeronym => "part meronym",
            Relation::Attribute => "attribute",
            Relation::DerivationallyRelatedForm => "derivationally related form",
            Relation::DomainOfSynsetTopic => "domain of synset topic",
            Relation::MemberOfThisDomainTopic => "member of this domain topic",
            Relation::DomainOfSynsetRegion => "domain of synset region",
            Relation::MemberOfThisDomainRegion => "member of this domain region",
            Relation::DomainOfSynsetUsage => "domain of synset usage",
            Relation::MemberOfThisDomainUsage => "member of this domain usage",
            Relation::Entailment => "entailment",
            Relation::Cause => "cause",
            Relation::AlsoSee => "also see",
            Relation::VerbGroup => "verb group",
            Relation::SimilarTo => "similar to",
            Relation::ParticipleOfVerb => "participle of verb",
            Relation::Pertainym => "pertainym",
        };
        f.write_str(s)
    }
}

#[derive(Debug)]
pub enum RelationKind {
    /// A semantic relationship between synsets.
    Semantic,
    /// A lexical relationship between words in synsets.
    Lexical(u32, u32)
}
