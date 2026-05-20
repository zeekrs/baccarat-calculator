use calculator::BetType;

pub struct SourceBaselineBet {
    pub bet_type: BetType,
    pub probability: f64,
}

pub const SOURCE_STANDARD_8_DECK_BETS: [SourceBaselineBet; 41] = [
    SourceBaselineBet {
        bet_type: BetType::AnyPair,
        probability: 0.14381687186687586,
    },
    SourceBaselineBet {
        bet_type: BetType::Banker,
        probability: 0.4585974226322891,
    },
    SourceBaselineBet {
        bet_type: BetType::Banker4Fortune,
        probability: 0.03268242767565334,
    },
    SourceBaselineBet {
        bet_type: BetType::BankerBlack,
        probability: 0.2493975903614458,
    },
    SourceBaselineBet {
        bet_type: BetType::BankerCharSiu,
        probability: 0.047059048028095865,
    },
    SourceBaselineBet {
        bet_type: BetType::BankerDragon,
        probability: 0.2827947340670113,
    },
    SourceBaselineBet {
        bet_type: BetType::BankerFortune4Pair,
        probability: 0.016867469879518072,
    },
    SourceBaselineBet {
        bet_type: BetType::BankerNatural,
        probability: 0.16258909541220742,
    },
    SourceBaselineBet {
        bet_type: BetType::BankerPair,
        probability: 0.0746987951807229,
    },
    SourceBaselineBet {
        bet_type: BetType::BankerRed,
        probability: 0.2493975903614458,
    },
    SourceBaselineBet {
        bet_type: BetType::Big,
        probability: 0.6211315091167852,
    },
    SourceBaselineBet {
        bet_type: BetType::BigBull,
        probability: 0.02186888709468297,
    },
    SourceBaselineBet {
        bet_type: BetType::BigTiger,
        probability: 0.01661699668042493,
    },
    SourceBaselineBet {
        bet_type: BetType::Flame7s,
        probability: 0.01128257885311176,
    },
    SourceBaselineBet {
        bet_type: BetType::Panda8,
        probability: 0.03454321839641624,
    },
    SourceBaselineBet {
        bet_type: BetType::Heaven9,
        probability: 0.07025976794652906,
    },
    SourceBaselineBet {
        bet_type: BetType::Invincible6,
        probability: 0.1356556968017422,
    },
    SourceBaselineBet {
        bet_type: BetType::Lucky6,
        probability: 0.05386371585776824,
    },
    SourceBaselineBet {
        bet_type: BetType::Dragon7,
        probability: 0.02253382086037812,
    },
    SourceBaselineBet {
        bet_type: BetType::PerfectPair,
        probability: 0.03345023424575235,
    },
    SourceBaselineBet {
        bet_type: BetType::Player,
        probability: 0.44624660934317073,
    },
    SourceBaselineBet {
        bet_type: BetType::Player4Fortune,
        probability: 0.01723867654131342,
    },
    SourceBaselineBet {
        bet_type: BetType::PlayerBlack,
        probability: 0.2493975903614458,
    },
    SourceBaselineBet {
        bet_type: BetType::PlayerCharSiu,
        probability: 0.04561189604261718,
    },
    SourceBaselineBet {
        bet_type: BetType::PlayerDragon,
        probability: 0.2898870255071087,
    },
    SourceBaselineBet {
        bet_type: BetType::PlayerFortune4Pair,
        probability: 0.016867469879518072,
    },
    SourceBaselineBet {
        bet_type: BetType::Lucky7,
        probability: 0.08163494723312005,
    },
    SourceBaselineBet {
        bet_type: BetType::PlayerNatural,
        probability: 0.16258909541220692,
    },
    SourceBaselineBet {
        bet_type: BetType::PlayerPair,
        probability: 0.0746987951807229,
    },
    SourceBaselineBet {
        bet_type: BetType::PlayerRed,
        probability: 0.2493975903614458,
    },
    SourceBaselineBet {
        bet_type: BetType::SuperLucky7,
        probability: 0.01898808441473606,
    },
    SourceBaselineBet {
        bet_type: BetType::Small,
        probability: 0.3788684908802761,
    },
    SourceBaselineBet {
        bet_type: BetType::SmallBull,
        probability: 0.0406829300993091,
    },
    SourceBaselineBet {
        bet_type: BetType::SmallTiger,
        probability: 0.037246719177343304,
    },
    SourceBaselineBet {
        bet_type: BetType::Tie,
        probability: 0.09515596802363396,
    },
    SourceBaselineBet {
        bet_type: BetType::Tiger,
        probability: 0.05386371585776824,
    },
    SourceBaselineBet {
        bet_type: BetType::TigerBull,
        probability: 0.11641553305175217,
    },
    SourceBaselineBet {
        bet_type: BetType::TigerPair,
        probability: 0.14381983138364596,
    },
    SourceBaselineBet {
        bet_type: BetType::TigerTie,
        probability: 0.01924016375000266,
    },
    SourceBaselineBet {
        bet_type: BetType::TreasureAll,
        probability: 0.1386193860563475,
    },
    SourceBaselineBet {
        bet_type: BetType::WuDaLang,
        probability: 0.004929417821567907,
    },
];
