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
    
}