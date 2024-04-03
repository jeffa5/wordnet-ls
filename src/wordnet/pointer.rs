#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerType {
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

impl PointerType {
    pub fn try_from_str(s: &str) -> Option<PointerType> {
        match s {
            "!" => Some(PointerType::Antonym),
            "@" => Some(PointerType::Hypernym),
            "@i" => Some(PointerType::InstanceHypernym),
            "~" => Some(PointerType::Hyponym),
            "~i" => Some(PointerType::InstanceHyponym),
            "#m" => Some(PointerType::MemberHolonym),
            "#s" => Some(PointerType::SubstanceHolonym),
            "#p" => Some(PointerType::PartHolonym),
            "%m" => Some(PointerType::MemberMeronym),
            "%s" => Some(PointerType::SubstanceMeronym),
            "%p" => Some(PointerType::PartMeronym),
            "=" => Some(PointerType::Attribute),
            "+" => Some(PointerType::DerivationallyRelatedForm),
            ";c" => Some(PointerType::DomainOfSynsetTopic),
            "-c" => Some(PointerType::MemberOfThisDomainTopic),
            ";r" => Some(PointerType::DomainOfSynsetRegion),
            "-r" => Some(PointerType::MemberOfThisDomainRegion),
            ";u" => Some(PointerType::DomainOfSynsetUsage),
            "-u" => Some(PointerType::MemberOfThisDomainUsage),
            "*" => Some(PointerType::Entailment),
            ">" => Some(PointerType::Cause),
            "^" => Some(PointerType::AlsoSee),
            "$" => Some(PointerType::VerbGroup),
            "&" => Some(PointerType::SimilarTo),
            "<" => Some(PointerType::ParticipleOfVerb),
            "\\" => Some(PointerType::Pertainym),
            _ => None,
        }
    }
}
