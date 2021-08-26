use crate::stats::StatGain;



enum BodyPartType {
    Arm,
    Hand,
    Leg,
    Foot,
    Torso,
    Groin,
    Neck,
    Head,
    Eye,
    Nose,
    Ear,
    Tail,
}

struct BodyPart {
    gains: StatGain,

    slot_used: bool,

    parent: usize,
    children: Vec<usize>,

    ty: BodyPartType,
}

struct Body {
    
}