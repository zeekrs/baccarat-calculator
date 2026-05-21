use serde::{Deserialize, Serialize};

pub const STANDARD_DECK_COUNT: u8 = 8;
pub const RANKS_PER_DECK: u8 = 13;
pub const SUITS_PER_DECK: u8 = 4;
pub const CARDS_PER_DECK: u16 = 52;
pub const STANDARD_SHOE_CARD_COUNT: u16 = STANDARD_DECK_COUNT as u16 * CARDS_PER_DECK;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardSuit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl CardSuit {
    pub const ALL: [Self; 4] = [Self::Clubs, Self::Diamonds, Self::Hearts, Self::Spades];

    pub const fn index(self) -> usize {
        match self {
            Self::Clubs => 0,
            Self::Diamonds => 1,
            Self::Hearts => 2,
            Self::Spades => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardRank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl CardRank {
    pub const ALL: [Self; 13] = [
        Self::Ace,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
    ];

    pub fn from_label(label: &str) -> Option<Self> {
        match label.trim().to_ascii_uppercase().as_str() {
            "A" | "ACE" => Some(Self::Ace),
            "2" | "TWO" => Some(Self::Two),
            "3" | "THREE" => Some(Self::Three),
            "4" | "FOUR" => Some(Self::Four),
            "5" | "FIVE" => Some(Self::Five),
            "6" | "SIX" => Some(Self::Six),
            "7" | "SEVEN" => Some(Self::Seven),
            "8" | "EIGHT" => Some(Self::Eight),
            "9" | "NINE" => Some(Self::Nine),
            "10" | "T" | "TEN" => Some(Self::Ten),
            "J" | "JACK" => Some(Self::Jack),
            "Q" | "QUEEN" => Some(Self::Queen),
            "K" | "KING" => Some(Self::King),
            _ => None,
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Self::Ace => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
            Self::Five => 4,
            Self::Six => 5,
            Self::Seven => 6,
            Self::Eight => 7,
            Self::Nine => 8,
            Self::Ten => 9,
            Self::Jack => 10,
            Self::Queen => 11,
            Self::King => 12,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Ace => "A",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
        }
    }

    pub const fn baccarat_value(self) -> u8 {
        match self {
            Self::Ace => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
            Self::Ten | Self::Jack | Self::Queen | Self::King => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub suit: CardSuit,
    pub rank: CardRank,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardCount {
    pub card: Card,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BetType {
    Player,
    Banker,
    Tie,
    AnyPair,
    PlayerPair,
    BankerPair,
    PerfectPair,
    PlayerDragon,
    BankerDragon,
    BankerNatural,
    PlayerNatural,
    Lucky6,
    Tiger,
    SmallTiger,
    BigTiger,
    TigerTie,
    TigerPair,
    Banker4Fortune,
    Player4Fortune,
    BankerFortune4Pair,
    PlayerFortune4Pair,
    PlayerRed,
    BankerRed,
    PlayerBlack,
    BankerBlack,
    Invincible6,
    Big,
    Small,
    BankerCharSiu,
    PlayerCharSiu,
    SmallBull,
    BigBull,
    TigerBull,
    WuDaLang,
    Dragon7,
    Panda8,
    Lucky7,
    BigLucky7,
    SmallLucky7,
    SuperLucky7,
    Flame7s,
    Monkey,
    SuperTie0,
    SuperTie1,
    SuperTie2,
    SuperTie3,
    SuperTie4,
    SuperTie5,
    SuperTie6,
    SuperTie7,
    SuperTie8,
    SuperTie9,
    Heaven9,
    TreasureAll,
}
