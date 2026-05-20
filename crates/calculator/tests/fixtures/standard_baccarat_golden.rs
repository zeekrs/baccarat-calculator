use calculator::BetType;

pub const PROBABILITY_ABS_TOLERANCE: f64 = 1e-12;
pub const PROBABILITY_SUM_ABS_TOLERANCE: f64 = 1e-12;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutcomeGoldenConstants {
    pub player_probability: f64,
    pub banker_probability: f64,
    pub tie_probability: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EvGoldenConstants {
    pub player_default_base_ev: f64,
    pub banker_default_base_ev: f64,
    pub tie_default_base_ev: f64,
    pub player_total_stake_effective_probability: f64,
    pub player_non_refund_effective_probability: f64,
    pub player_losing_only_effective_probability: f64,
}

pub const STANDARD_8_DECK_GOLDEN: OutcomeGoldenConstants = OutcomeGoldenConstants {
    player_probability: 0.446246609343597,
    banker_probability: 0.458597422632763,
    tie_probability: 0.095155968023641,
};

pub const STANDARD_8_DECK_EV_GOLDEN: EvGoldenConstants = EvGoldenConstants {
    player_default_base_ev: -0.012350813289166,
    banker_default_base_ev: -0.010579057842472,
    tie_default_base_ev: -0.143596287787231,
    player_total_stake_effective_probability: 1.0,
    player_non_refund_effective_probability: 0.904844031976359,
    player_losing_only_effective_probability: 0.458597422632763,
};

pub struct FixtureBet {
    pub bet_type: BetType,
    pub probability: f64,
}

pub struct BaccaratFixture {
    pub name: &'static str,
    pub ranks: [(&'static str, u32); 13],
    pub expected_bets: [FixtureBet; 5],
}

pub const STANDARD_8_DECK_FIXTURE: BaccaratFixture = BaccaratFixture {
    name: "standard-8-deck",
    ranks: [
        ("A", 32),
        ("2", 32),
        ("3", 32),
        ("4", 32),
        ("5", 32),
        ("6", 32),
        ("7", 32),
        ("8", 32),
        ("9", 32),
        ("10", 32),
        ("J", 32),
        ("Q", 32),
        ("K", 32),
    ],
    expected_bets: [
        FixtureBet {
            bet_type: BetType::Player,
            probability: 0.446246609343597,
        },
        FixtureBet {
            bet_type: BetType::Banker,
            probability: 0.458597422632763,
        },
        FixtureBet {
            bet_type: BetType::Tie,
            probability: 0.095155968023641,
        },
        FixtureBet {
            bet_type: BetType::PlayerPair,
            probability: 0.074698795180723,
        },
        FixtureBet {
            bet_type: BetType::BankerPair,
            probability: 0.074698795180723,
        },
    ],
};

pub const DEPLETED_SHOE_FIXTURE: BaccaratFixture = BaccaratFixture {
    name: "depleted-8-deck-rank-mix",
    ranks: [
        ("A", 28),
        ("2", 31),
        ("3", 32),
        ("4", 30),
        ("5", 29),
        ("6", 31),
        ("7", 32),
        ("8", 30),
        ("9", 31),
        ("10", 20),
        ("J", 29),
        ("Q", 30),
        ("K", 28),
    ],
    expected_bets: [
        FixtureBet {
            bet_type: BetType::Player,
            probability: 0.445448531614868,
        },
        FixtureBet {
            bet_type: BetType::Banker,
            probability: 0.458160849668332,
        },
        FixtureBet {
            bet_type: BetType::Tie,
            probability: 0.096390618716800,
        },
        FixtureBet {
            bet_type: BetType::PlayerPair,
            probability: 0.075286641801354,
        },
        FixtureBet {
            bet_type: BetType::BankerPair,
            probability: 0.075286641801354,
        },
    ],
};
