use calculator::{
    BetType, BetVariant, CharSiuVariant, DragonVariant, Flame7sVariant, Fortune4PairVariant,
    Heaven9Variant, Lucky6Variant, Lucky7Variant, NaturalVariant, SuperLucky7Variant,
    TigerPairVariant, TigerVariant,
};

pub struct SourceVariantBaseline {
    pub bet_type: BetType,
    pub variant: BetVariant,
    pub probability: f64,
}

pub const SOURCE_STANDARD_8_DECK_VARIANTS: [SourceVariantBaseline; 50] = [
    SourceVariantBaseline {
        bet_type: BetType::PlayerDragon,
        variant: BetVariant::Dragon(DragonVariant::Natural),
        probability: 0.16258909541220692,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point4),
        probability: 0.03736804109618025,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point5),
        probability: 0.03324448289809535,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point6),
        probability: 0.02825683139538551,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point7),
        probability: 0.01792379409066877,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point8),
        probability: 0.00682171441010088,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point9),
        probability: 0.00368306620447101,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerDragon,
        variant: BetVariant::Dragon(DragonVariant::Push),
        probability: 0.01787090650724999,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerDragon,
        variant: BetVariant::Dragon(DragonVariant::Natural),
        probability: 0.16258909541220742,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point4),
        probability: 0.04024232488097321,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point5),
        probability: 0.03146525620076208,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point6),
        probability: 0.02384765394751831,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point7),
        probability: 0.01590851606764434,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point8),
        probability: 0.00566283261637017,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerDragon,
        variant: BetVariant::Dragon(DragonVariant::Point9),
        probability: 0.00307905494153577,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerDragon,
        variant: BetVariant::Dragon(DragonVariant::Push),
        probability: 0.01787090650724999,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerNatural,
        variant: BetVariant::Natural(NaturalVariant::Win),
        probability: 0.16258909541220742,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerNatural,
        variant: BetVariant::Natural(NaturalVariant::Push),
        probability: 0.01787090650724999,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerNatural,
        variant: BetVariant::Natural(NaturalVariant::Win),
        probability: 0.16258909541220692,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerNatural,
        variant: BetVariant::Natural(NaturalVariant::Push),
        probability: 0.01787090650724999,
    },
    SourceVariantBaseline {
        bet_type: BetType::Lucky6,
        variant: BetVariant::Lucky6(Lucky6Variant::Two),
        probability: 0.037_246_719_177_343_3,
    },
    SourceVariantBaseline {
        bet_type: BetType::Lucky6,
        variant: BetVariant::Lucky6(Lucky6Variant::Three),
        probability: 0.01661699668042493,
    },
    SourceVariantBaseline {
        bet_type: BetType::Tiger,
        variant: BetVariant::Tiger(TigerVariant::Two),
        probability: 0.037_246_719_177_343_3,
    },
    SourceVariantBaseline {
        bet_type: BetType::Tiger,
        variant: BetVariant::Tiger(TigerVariant::Three),
        probability: 0.01661699668042493,
    },
    SourceVariantBaseline {
        bet_type: BetType::TigerPair,
        variant: BetVariant::TigerPair(TigerPairVariant::Perfect),
        probability: 0.00000295951677010,
    },
    SourceVariantBaseline {
        bet_type: BetType::TigerPair,
        variant: BetVariant::TigerPair(TigerPairVariant::Both),
        probability: 0.00558071849456992,
    },
    SourceVariantBaseline {
        bet_type: BetType::TigerPair,
        variant: BetVariant::TigerPair(TigerPairVariant::Single),
        probability: 0.13823615337230594,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerFortune4Pair,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30),
        probability: 0.00032437442075996,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerFortune4Pair,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15),
        probability: 0.00097312326227989,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerFortune4Pair,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune12),
        probability: 0.00389249304911956,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerFortune4Pair,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune9),
        probability: 0.01167747914735867,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerFortune4Pair,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30),
        probability: 0.00032437442075996,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerFortune4Pair,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15),
        probability: 0.00097312326227989,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerFortune4Pair,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune12),
        probability: 0.00389249304911956,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerFortune4Pair,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune9),
        probability: 0.01167747914735867,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerCharSiu,
        variant: BetVariant::CharSiu(CharSiuVariant::Point4),
        probability: 0.02692314684572595,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerCharSiu,
        variant: BetVariant::CharSiu(CharSiuVariant::Point5),
        probability: 0.01358817350710437,
    },
    SourceVariantBaseline {
        bet_type: BetType::BankerCharSiu,
        variant: BetVariant::CharSiu(CharSiuVariant::Point6),
        probability: 0.00654772767526555,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerCharSiu,
        variant: BetVariant::CharSiu(CharSiuVariant::Point4),
        probability: 0.02692314684572595,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerCharSiu,
        variant: BetVariant::CharSiu(CharSiuVariant::Point5),
        probability: 0.01182891860667444,
    },
    SourceVariantBaseline {
        bet_type: BetType::PlayerCharSiu,
        variant: BetVariant::CharSiu(CharSiuVariant::Point6),
        probability: 0.00685983059021679,
    },
    SourceVariantBaseline {
        bet_type: BetType::Lucky7,
        variant: BetVariant::Lucky7(Lucky7Variant::Two),
        probability: 0.05434667239745365,
    },
    SourceVariantBaseline {
        bet_type: BetType::Lucky7,
        variant: BetVariant::Lucky7(Lucky7Variant::Three),
        probability: 0.027_288_274_835_666_4,
    },
    SourceVariantBaseline {
        bet_type: BetType::SuperLucky7,
        variant: BetVariant::SuperLucky7(SuperLucky7Variant::Four),
        probability: 0.00897424352068278,
    },
    SourceVariantBaseline {
        bet_type: BetType::SuperLucky7,
        variant: BetVariant::SuperLucky7(SuperLucky7Variant::Five),
        probability: 0.00729903683362935,
    },
    SourceVariantBaseline {
        bet_type: BetType::SuperLucky7,
        variant: BetVariant::SuperLucky7(SuperLucky7Variant::Six),
        probability: 0.00271480406042392,
    },
    SourceVariantBaseline {
        bet_type: BetType::Flame7s,
        variant: BetVariant::Flame7s(Flame7sVariant::Two),
        probability: 0.00897063572881066,
    },
    SourceVariantBaseline {
        bet_type: BetType::Flame7s,
        variant: BetVariant::Flame7s(Flame7sVariant::Three),
        probability: 0.00231194312430109,
    },
    SourceVariantBaseline {
        bet_type: BetType::Heaven9,
        variant: BetVariant::Heaven9(Heaven9Variant::Single),
        probability: 0.068_197_658_579_716_3,
    },
    SourceVariantBaseline {
        bet_type: BetType::Heaven9,
        variant: BetVariant::Heaven9(Heaven9Variant::Both),
        probability: 0.00206210936681277,
    },
];
