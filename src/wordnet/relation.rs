#[derive(Debug, Clone, Copy, PartialEq)]
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
