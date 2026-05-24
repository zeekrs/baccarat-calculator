pub use types::baccarat::BetType;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BetId {
    Player,
    Banker,
    Tie,
    AnyPair,
    PlayerPair,
    BankerPair,
    PerfectPair,
    Monkey,
    PlayerDragonNatural,
    PlayerDragonPoint4,
    PlayerDragonPoint5,
    PlayerDragonPoint6,
    PlayerDragonPoint7,
    PlayerDragonPoint8,
    PlayerDragonPoint9,
    PlayerDragonPush,
    PlayerDragonAggregate,
    BankerDragonNatural,
    BankerDragonPoint4,
    BankerDragonPoint5,
    BankerDragonPoint6,
    BankerDragonPoint7,
    BankerDragonPoint8,
    BankerDragonPoint9,
    BankerDragonPush,
    BankerDragonAggregate,
    BankerNaturalWin,
    BankerNaturalPush,
    BankerNaturalAggregate,
    PlayerNaturalWin,
    PlayerNaturalPush,
    PlayerNaturalAggregate,
    Lucky6Two,
    Lucky6Three,
    Lucky6Aggregate,
    TigerTwo,
    TigerThree,
    TigerAggregate,
    SmallTiger,
    BigTiger,
    TigerTie,
    TigerPairPerfect,
    TigerPairBoth,
    TigerPairSingle,
    TigerPairAggregate,
    Banker4Fortune,
    Player4Fortune,
    BankerFortune4PairFortune30,
    BankerFortune4PairFortune15,
    BankerFortune4PairFortune12,
    BankerFortune4PairFortune9,
    BankerFortune4PairAggregate,
    PlayerFortune4PairFortune30,
    PlayerFortune4PairFortune15,
    PlayerFortune4PairFortune12,
    PlayerFortune4PairFortune9,
    PlayerFortune4PairAggregate,
    PlayerRed,
    BankerRed,
    PlayerBlack,
    BankerBlack,
    Invincible6,
    Big,
    Small,
    BankerCharSiuPoint4,
    BankerCharSiuPoint5,
    BankerCharSiuPoint6,
    BankerCharSiuAggregate,
    PlayerCharSiuPoint4,
    PlayerCharSiuPoint5,
    PlayerCharSiuPoint6,
    PlayerCharSiuAggregate,
    SmallBull,
    BigBull,
    TigerBull,
    WuDaLang,
    Dragon7,
    Panda8,
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
    Lucky7Two,
    Lucky7Three,
    Lucky7Aggregate,
    BigLucky7,
    SmallLucky7,
    SuperLucky7Four,
    SuperLucky7Five,
    SuperLucky7Six,
    SuperLucky7Aggregate,
    Flame7sTwo,
    Flame7sThree,
    Flame7sAggregate,
    Heaven9Single,
    Heaven9Both,
    Heaven9Aggregate,
    TreasureAll,
}

impl BetId {
    pub const fn bet_type(self) -> BetType {
        match self {
            Self::Player => BetType::Player,
            Self::Banker => BetType::Banker,
            Self::Tie => BetType::Tie,
            Self::AnyPair => BetType::AnyPair,
            Self::PlayerPair => BetType::PlayerPair,
            Self::BankerPair => BetType::BankerPair,
            Self::PerfectPair => BetType::PerfectPair,
            Self::Monkey => BetType::Monkey,
            Self::PlayerDragonNatural
            | Self::PlayerDragonPoint4
            | Self::PlayerDragonPoint5
            | Self::PlayerDragonPoint6
            | Self::PlayerDragonPoint7
            | Self::PlayerDragonPoint8
            | Self::PlayerDragonPoint9
            | Self::PlayerDragonPush
            | Self::PlayerDragonAggregate => BetType::PlayerDragon,
            Self::BankerDragonNatural
            | Self::BankerDragonPoint4
            | Self::BankerDragonPoint5
            | Self::BankerDragonPoint6
            | Self::BankerDragonPoint7
            | Self::BankerDragonPoint8
            | Self::BankerDragonPoint9
            | Self::BankerDragonPush
            | Self::BankerDragonAggregate => BetType::BankerDragon,
            Self::BankerNaturalWin | Self::BankerNaturalPush | Self::BankerNaturalAggregate => {
                BetType::BankerNatural
            }
            Self::PlayerNaturalWin | Self::PlayerNaturalPush | Self::PlayerNaturalAggregate => {
                BetType::PlayerNatural
            }
            Self::Lucky6Two | Self::Lucky6Three | Self::Lucky6Aggregate => BetType::Lucky6,
            Self::TigerTwo | Self::TigerThree | Self::TigerAggregate => BetType::Tiger,
            Self::SmallTiger => BetType::SmallTiger,
            Self::BigTiger => BetType::BigTiger,
            Self::TigerTie => BetType::TigerTie,
            Self::TigerPairPerfect
            | Self::TigerPairBoth
            | Self::TigerPairSingle
            | Self::TigerPairAggregate => BetType::TigerPair,
            Self::Banker4Fortune => BetType::Banker4Fortune,
            Self::Player4Fortune => BetType::Player4Fortune,
            Self::BankerFortune4PairFortune30
            | Self::BankerFortune4PairFortune15
            | Self::BankerFortune4PairFortune12
            | Self::BankerFortune4PairFortune9
            | Self::BankerFortune4PairAggregate => BetType::BankerFortune4Pair,
            Self::PlayerFortune4PairFortune30
            | Self::PlayerFortune4PairFortune15
            | Self::PlayerFortune4PairFortune12
            | Self::PlayerFortune4PairFortune9
            | Self::PlayerFortune4PairAggregate => BetType::PlayerFortune4Pair,
            Self::PlayerRed => BetType::PlayerRed,
            Self::BankerRed => BetType::BankerRed,
            Self::PlayerBlack => BetType::PlayerBlack,
            Self::BankerBlack => BetType::BankerBlack,
            Self::Invincible6 => BetType::Invincible6,
            Self::Big => BetType::Big,
            Self::Small => BetType::Small,
            Self::BankerCharSiuPoint4
            | Self::BankerCharSiuPoint5
            | Self::BankerCharSiuPoint6
            | Self::BankerCharSiuAggregate => BetType::BankerCharSiu,
            Self::PlayerCharSiuPoint4
            | Self::PlayerCharSiuPoint5
            | Self::PlayerCharSiuPoint6
            | Self::PlayerCharSiuAggregate => BetType::PlayerCharSiu,
            Self::SmallBull => BetType::SmallBull,
            Self::BigBull => BetType::BigBull,
            Self::TigerBull => BetType::TigerBull,
            Self::WuDaLang => BetType::WuDaLang,
            Self::Dragon7 => BetType::Dragon7,
            Self::Panda8 => BetType::Panda8,
            Self::SuperTie0 => BetType::SuperTie0,
            Self::SuperTie1 => BetType::SuperTie1,
            Self::SuperTie2 => BetType::SuperTie2,
            Self::SuperTie3 => BetType::SuperTie3,
            Self::SuperTie4 => BetType::SuperTie4,
            Self::SuperTie5 => BetType::SuperTie5,
            Self::SuperTie6 => BetType::SuperTie6,
            Self::SuperTie7 => BetType::SuperTie7,
            Self::SuperTie8 => BetType::SuperTie8,
            Self::SuperTie9 => BetType::SuperTie9,
            Self::Lucky7Two | Self::Lucky7Three | Self::Lucky7Aggregate => BetType::Lucky7,
            Self::BigLucky7 => BetType::BigLucky7,
            Self::SmallLucky7 => BetType::SmallLucky7,
            Self::SuperLucky7Four
            | Self::SuperLucky7Five
            | Self::SuperLucky7Six
            | Self::SuperLucky7Aggregate => BetType::SuperLucky7,
            Self::Flame7sTwo | Self::Flame7sThree | Self::Flame7sAggregate => BetType::Flame7s,
            Self::Heaven9Single | Self::Heaven9Both | Self::Heaven9Aggregate => BetType::Heaven9,
            Self::TreasureAll => BetType::TreasureAll,
        }
    }

    pub const fn is_push_result(self) -> bool {
        matches!(
            self,
            Self::PlayerDragonPush
                | Self::BankerDragonPush
                | Self::BankerNaturalPush
                | Self::PlayerNaturalPush
        )
    }

    pub const fn is_public_probability_result(self) -> bool {
        matches!(
            self,
            Self::Player
                | Self::Banker
                | Self::Tie
                | Self::AnyPair
                | Self::PlayerPair
                | Self::BankerPair
                | Self::PerfectPair
                | Self::Monkey
                | Self::PlayerDragonAggregate
                | Self::BankerDragonAggregate
                | Self::BankerNaturalAggregate
                | Self::PlayerNaturalAggregate
                | Self::Lucky6Aggregate
                | Self::TigerAggregate
                | Self::SmallTiger
                | Self::BigTiger
                | Self::TigerTie
                | Self::TigerPairAggregate
                | Self::Banker4Fortune
                | Self::Player4Fortune
                | Self::BankerFortune4PairAggregate
                | Self::PlayerFortune4PairAggregate
                | Self::PlayerRed
                | Self::BankerRed
                | Self::PlayerBlack
                | Self::BankerBlack
                | Self::Invincible6
                | Self::Big
                | Self::Small
                | Self::BankerCharSiuAggregate
                | Self::PlayerCharSiuAggregate
                | Self::SmallBull
                | Self::BigBull
                | Self::TigerBull
                | Self::WuDaLang
                | Self::Dragon7
                | Self::Panda8
                | Self::SuperTie0
                | Self::SuperTie1
                | Self::SuperTie2
                | Self::SuperTie3
                | Self::SuperTie4
                | Self::SuperTie5
                | Self::SuperTie6
                | Self::SuperTie7
                | Self::SuperTie8
                | Self::SuperTie9
                | Self::Lucky7Aggregate
                | Self::BigLucky7
                | Self::SmallLucky7
                | Self::SuperLucky7Aggregate
                | Self::Flame7sAggregate
                | Self::Heaven9Aggregate
                | Self::TreasureAll
        )
    }
}

/// Returns the canonical public registry row for an external [`BetType`].
///
/// `BetType` is the caller-facing bet family/row, while `BetId` is the
/// registry's precise internal identifier. Aggregate public rows intentionally
/// map to their aggregate `BetId` here.
pub const fn public_bet_id_for_type(bet_type: BetType) -> BetId {
    match bet_type {
        BetType::Player => BetId::Player,
        BetType::Banker => BetId::Banker,
        BetType::Tie => BetId::Tie,
        BetType::AnyPair => BetId::AnyPair,
        BetType::PlayerPair => BetId::PlayerPair,
        BetType::BankerPair => BetId::BankerPair,
        BetType::PerfectPair => BetId::PerfectPair,
        BetType::Monkey => BetId::Monkey,
        BetType::PlayerDragon => BetId::PlayerDragonAggregate,
        BetType::BankerDragon => BetId::BankerDragonAggregate,
        BetType::BankerNatural => BetId::BankerNaturalAggregate,
        BetType::PlayerNatural => BetId::PlayerNaturalAggregate,
        BetType::Lucky6 => BetId::Lucky6Aggregate,
        BetType::Tiger => BetId::TigerAggregate,
        BetType::SmallTiger => BetId::SmallTiger,
        BetType::BigTiger => BetId::BigTiger,
        BetType::TigerTie => BetId::TigerTie,
        BetType::TigerPair => BetId::TigerPairAggregate,
        BetType::Banker4Fortune => BetId::Banker4Fortune,
        BetType::Player4Fortune => BetId::Player4Fortune,
        BetType::BankerFortune4Pair => BetId::BankerFortune4PairAggregate,
        BetType::PlayerFortune4Pair => BetId::PlayerFortune4PairAggregate,
        BetType::PlayerRed => BetId::PlayerRed,
        BetType::BankerRed => BetId::BankerRed,
        BetType::PlayerBlack => BetId::PlayerBlack,
        BetType::BankerBlack => BetId::BankerBlack,
        BetType::Invincible6 => BetId::Invincible6,
        BetType::Big => BetId::Big,
        BetType::Small => BetId::Small,
        BetType::BankerCharSiu => BetId::BankerCharSiuAggregate,
        BetType::PlayerCharSiu => BetId::PlayerCharSiuAggregate,
        BetType::SmallBull => BetId::SmallBull,
        BetType::BigBull => BetId::BigBull,
        BetType::TigerBull => BetId::TigerBull,
        BetType::WuDaLang => BetId::WuDaLang,
        BetType::Dragon7 => BetId::Dragon7,
        BetType::Panda8 => BetId::Panda8,
        BetType::Lucky7 => BetId::Lucky7Aggregate,
        BetType::BigLucky7 => BetId::BigLucky7,
        BetType::SmallLucky7 => BetId::SmallLucky7,
        BetType::SuperLucky7 => BetId::SuperLucky7Aggregate,
        BetType::Flame7s => BetId::Flame7sAggregate,
        BetType::SuperTie0 => BetId::SuperTie0,
        BetType::SuperTie1 => BetId::SuperTie1,
        BetType::SuperTie2 => BetId::SuperTie2,
        BetType::SuperTie3 => BetId::SuperTie3,
        BetType::SuperTie4 => BetId::SuperTie4,
        BetType::SuperTie5 => BetId::SuperTie5,
        BetType::SuperTie6 => BetId::SuperTie6,
        BetType::SuperTie7 => BetId::SuperTie7,
        BetType::SuperTie8 => BetId::SuperTie8,
        BetType::SuperTie9 => BetId::SuperTie9,
        BetType::Heaven9 => BetId::Heaven9Aggregate,
        BetType::TreasureAll => BetId::TreasureAll,
    }
}

/// Returns the precise internal `BetId` for a variant inside an external
/// caller-facing [`BetType`] family.
pub const fn variant_bet_id(bet_type: BetType, variant: BetVariant) -> BetId {
    match (bet_type, variant) {
        (BetType::PlayerDragon, BetVariant::Dragon(DragonVariant::Natural)) => {
            BetId::PlayerDragonNatural
        }
        (BetType::PlayerDragon, BetVariant::Dragon(DragonVariant::Point4)) => {
            BetId::PlayerDragonPoint4
        }
        (BetType::PlayerDragon, BetVariant::Dragon(DragonVariant::Point5)) => {
            BetId::PlayerDragonPoint5
        }
        (BetType::PlayerDragon, BetVariant::Dragon(DragonVariant::Point6)) => {
            BetId::PlayerDragonPoint6
        }
        (BetType::PlayerDragon, BetVariant::Dragon(DragonVariant::Point7)) => {
            BetId::PlayerDragonPoint7
        }
        (BetType::PlayerDragon, BetVariant::Dragon(DragonVariant::Point8)) => {
            BetId::PlayerDragonPoint8
        }
        (BetType::PlayerDragon, BetVariant::Dragon(DragonVariant::Point9)) => {
            BetId::PlayerDragonPoint9
        }
        (BetType::PlayerDragon, BetVariant::Dragon(DragonVariant::Push)) => BetId::PlayerDragonPush,
        (BetType::BankerDragon, BetVariant::Dragon(DragonVariant::Natural)) => {
            BetId::BankerDragonNatural
        }
        (BetType::BankerDragon, BetVariant::Dragon(DragonVariant::Point4)) => {
            BetId::BankerDragonPoint4
        }
        (BetType::BankerDragon, BetVariant::Dragon(DragonVariant::Point5)) => {
            BetId::BankerDragonPoint5
        }
        (BetType::BankerDragon, BetVariant::Dragon(DragonVariant::Point6)) => {
            BetId::BankerDragonPoint6
        }
        (BetType::BankerDragon, BetVariant::Dragon(DragonVariant::Point7)) => {
            BetId::BankerDragonPoint7
        }
        (BetType::BankerDragon, BetVariant::Dragon(DragonVariant::Point8)) => {
            BetId::BankerDragonPoint8
        }
        (BetType::BankerDragon, BetVariant::Dragon(DragonVariant::Point9)) => {
            BetId::BankerDragonPoint9
        }
        (BetType::BankerDragon, BetVariant::Dragon(DragonVariant::Push)) => BetId::BankerDragonPush,
        (BetType::BankerNatural, BetVariant::Natural(NaturalVariant::Win)) => {
            BetId::BankerNaturalWin
        }
        (BetType::BankerNatural, BetVariant::Natural(NaturalVariant::Push)) => {
            BetId::BankerNaturalPush
        }
        (BetType::PlayerNatural, BetVariant::Natural(NaturalVariant::Win)) => {
            BetId::PlayerNaturalWin
        }
        (BetType::PlayerNatural, BetVariant::Natural(NaturalVariant::Push)) => {
            BetId::PlayerNaturalPush
        }
        (BetType::Lucky6, BetVariant::Lucky6(Lucky6Variant::Two)) => BetId::Lucky6Two,
        (BetType::Lucky6, BetVariant::Lucky6(Lucky6Variant::Three)) => BetId::Lucky6Three,
        (BetType::Tiger, BetVariant::Tiger(TigerVariant::Two)) => BetId::TigerTwo,
        (BetType::Tiger, BetVariant::Tiger(TigerVariant::Three)) => BetId::TigerThree,
        (BetType::TigerPair, BetVariant::TigerPair(TigerPairVariant::Perfect)) => {
            BetId::TigerPairPerfect
        }
        (BetType::TigerPair, BetVariant::TigerPair(TigerPairVariant::Both)) => BetId::TigerPairBoth,
        (BetType::TigerPair, BetVariant::TigerPair(TigerPairVariant::Single)) => {
            BetId::TigerPairSingle
        }
        (BetType::BankerFortune4Pair, BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30)) => {
            BetId::BankerFortune4PairFortune30
        }
        (BetType::BankerFortune4Pair, BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15)) => {
            BetId::BankerFortune4PairFortune15
        }
        (BetType::BankerFortune4Pair, BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune12)) => {
            BetId::BankerFortune4PairFortune12
        }
        (BetType::BankerFortune4Pair, BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune9)) => {
            BetId::BankerFortune4PairFortune9
        }
        (BetType::PlayerFortune4Pair, BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30)) => {
            BetId::PlayerFortune4PairFortune30
        }
        (BetType::PlayerFortune4Pair, BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15)) => {
            BetId::PlayerFortune4PairFortune15
        }
        (BetType::PlayerFortune4Pair, BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune12)) => {
            BetId::PlayerFortune4PairFortune12
        }
        (BetType::PlayerFortune4Pair, BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune9)) => {
            BetId::PlayerFortune4PairFortune9
        }
        (BetType::BankerCharSiu, BetVariant::CharSiu(CharSiuVariant::Point4)) => {
            BetId::BankerCharSiuPoint4
        }
        (BetType::BankerCharSiu, BetVariant::CharSiu(CharSiuVariant::Point5)) => {
            BetId::BankerCharSiuPoint5
        }
        (BetType::BankerCharSiu, BetVariant::CharSiu(CharSiuVariant::Point6)) => {
            BetId::BankerCharSiuPoint6
        }
        (BetType::PlayerCharSiu, BetVariant::CharSiu(CharSiuVariant::Point4)) => {
            BetId::PlayerCharSiuPoint4
        }
        (BetType::PlayerCharSiu, BetVariant::CharSiu(CharSiuVariant::Point5)) => {
            BetId::PlayerCharSiuPoint5
        }
        (BetType::PlayerCharSiu, BetVariant::CharSiu(CharSiuVariant::Point6)) => {
            BetId::PlayerCharSiuPoint6
        }
        (BetType::Lucky7, BetVariant::Lucky7(Lucky7Variant::Two)) => BetId::Lucky7Two,
        (BetType::Lucky7, BetVariant::Lucky7(Lucky7Variant::Three)) => BetId::Lucky7Three,
        (BetType::SuperLucky7, BetVariant::SuperLucky7(SuperLucky7Variant::Four)) => {
            BetId::SuperLucky7Four
        }
        (BetType::SuperLucky7, BetVariant::SuperLucky7(SuperLucky7Variant::Five)) => {
            BetId::SuperLucky7Five
        }
        (BetType::SuperLucky7, BetVariant::SuperLucky7(SuperLucky7Variant::Six)) => {
            BetId::SuperLucky7Six
        }
        (BetType::Flame7s, BetVariant::Flame7s(Flame7sVariant::Two)) => BetId::Flame7sTwo,
        (BetType::Flame7s, BetVariant::Flame7s(Flame7sVariant::Three)) => BetId::Flame7sThree,
        (BetType::Heaven9, BetVariant::Heaven9(Heaven9Variant::Single)) => BetId::Heaven9Single,
        (BetType::Heaven9, BetVariant::Heaven9(Heaven9Variant::Both)) => BetId::Heaven9Both,
        _ => public_bet_id_for_type(bet_type),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetClass {
    TerminalPredicate,
    OpeningTwoCombinator,
    AggregateBet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragonVariant {
    Natural,
    Point4,
    Point5,
    Point6,
    Point7,
    Point8,
    Point9,
    Push,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NaturalVariant {
    Win,
    Push,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lucky6Variant {
    Two,
    Three,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TigerVariant {
    Two,
    Three,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TigerPairVariant {
    Perfect,
    Both,
    Single,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fortune4PairVariant {
    Fortune30,
    Fortune15,
    Fortune12,
    Fortune9,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharSiuVariant {
    Point4,
    Point5,
    Point6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lucky7Variant {
    Two,
    Three,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuperLucky7Variant {
    Four,
    Five,
    Six,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Flame7sVariant {
    Two,
    Three,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heaven9Variant {
    Single,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetVariant {
    Dragon(DragonVariant),
    Natural(NaturalVariant),
    Lucky6(Lucky6Variant),
    Tiger(TigerVariant),
    TigerPair(TigerPairVariant),
    Fortune4Pair(Fortune4PairVariant),
    CharSiu(CharSiuVariant),
    Lucky7(Lucky7Variant),
    SuperLucky7(SuperLucky7Variant),
    Flame7s(Flame7sVariant),
    Heaven9(Heaven9Variant),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BetDefinition {
    pub id: BetId,
    pub class: BetClass,
    pub suit_dependent: bool,
    pub variant: Option<BetVariant>,
}

impl BetDefinition {
    pub const fn bet_type(self) -> BetType {
        self.id.bet_type()
    }

    pub const fn is_public_probability_result(self) -> bool {
        self.id.is_public_probability_result()
    }
}

macro_rules! bet_definition {
    ($id:ident, $class:expr, $suit:expr, $variant:expr $(,)?) => {
        BetDefinition {
            id: BetId::$id,
            class: $class,
            suit_dependent: $suit,
            variant: $variant,
        }
    };
}

const BET_DEFINITIONS: &[BetDefinition] = &[
    bet_definition!(Player, BetClass::TerminalPredicate, false, None,),
    bet_definition!(Banker, BetClass::TerminalPredicate, false, None,),
    bet_definition!(Tie, BetClass::TerminalPredicate, false, None,),
    bet_definition!(AnyPair, BetClass::OpeningTwoCombinator, false, None,),
    bet_definition!(PlayerPair, BetClass::OpeningTwoCombinator, false, None,),
    bet_definition!(BankerPair, BetClass::OpeningTwoCombinator, false, None,),
    bet_definition!(PerfectPair, BetClass::OpeningTwoCombinator, true, None,),
    bet_definition!(Monkey, BetClass::OpeningTwoCombinator, false, None,),
    bet_definition!(
        PlayerDragonNatural,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Natural)),
    ),
    bet_definition!(
        PlayerDragonPoint4,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point4)),
    ),
    bet_definition!(
        PlayerDragonPoint5,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point5)),
    ),
    bet_definition!(
        PlayerDragonPoint6,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point6)),
    ),
    bet_definition!(
        PlayerDragonPoint7,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point7)),
    ),
    bet_definition!(
        PlayerDragonPoint8,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point8)),
    ),
    bet_definition!(
        PlayerDragonPoint9,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point9)),
    ),
    bet_definition!(
        PlayerDragonPush,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Push)),
    ),
    bet_definition!(PlayerDragonAggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(
        BankerDragonNatural,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Natural)),
    ),
    bet_definition!(
        BankerDragonPoint4,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point4)),
    ),
    bet_definition!(
        BankerDragonPoint5,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point5)),
    ),
    bet_definition!(
        BankerDragonPoint6,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point6)),
    ),
    bet_definition!(
        BankerDragonPoint7,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point7)),
    ),
    bet_definition!(
        BankerDragonPoint8,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point8)),
    ),
    bet_definition!(
        BankerDragonPoint9,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Point9)),
    ),
    bet_definition!(
        BankerDragonPush,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Dragon(DragonVariant::Push)),
    ),
    bet_definition!(BankerDragonAggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(
        BankerNaturalWin,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Natural(NaturalVariant::Win)),
    ),
    bet_definition!(
        BankerNaturalPush,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Natural(NaturalVariant::Push)),
    ),
    bet_definition!(BankerNaturalAggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(
        PlayerNaturalWin,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Natural(NaturalVariant::Win)),
    ),
    bet_definition!(
        PlayerNaturalPush,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Natural(NaturalVariant::Push)),
    ),
    bet_definition!(PlayerNaturalAggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(
        Lucky6Two,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Lucky6(Lucky6Variant::Two)),
    ),
    bet_definition!(
        Lucky6Three,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Lucky6(Lucky6Variant::Three)),
    ),
    bet_definition!(Lucky6Aggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(
        TigerTwo,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Tiger(TigerVariant::Two)),
    ),
    bet_definition!(
        TigerThree,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Tiger(TigerVariant::Three)),
    ),
    bet_definition!(TigerAggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(SmallTiger, BetClass::TerminalPredicate, false, None,),
    bet_definition!(BigTiger, BetClass::TerminalPredicate, false, None,),
    bet_definition!(TigerTie, BetClass::TerminalPredicate, false, None,),
    bet_definition!(
        TigerPairPerfect,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::TigerPair(TigerPairVariant::Perfect)),
    ),
    bet_definition!(
        TigerPairBoth,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::TigerPair(TigerPairVariant::Both)),
    ),
    bet_definition!(
        TigerPairSingle,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::TigerPair(TigerPairVariant::Single)),
    ),
    bet_definition!(TigerPairAggregate, BetClass::AggregateBet, true, None,),
    bet_definition!(Banker4Fortune, BetClass::TerminalPredicate, false, None,),
    bet_definition!(Player4Fortune, BetClass::TerminalPredicate, false, None,),
    bet_definition!(
        BankerFortune4PairFortune30,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30)),
    ),
    bet_definition!(
        BankerFortune4PairFortune15,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15)),
    ),
    bet_definition!(
        BankerFortune4PairFortune12,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune12)),
    ),
    bet_definition!(
        BankerFortune4PairFortune9,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune9)),
    ),
    bet_definition!(
        BankerFortune4PairAggregate,
        BetClass::AggregateBet,
        true,
        None,
    ),
    bet_definition!(
        PlayerFortune4PairFortune30,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30)),
    ),
    bet_definition!(
        PlayerFortune4PairFortune15,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15)),
    ),
    bet_definition!(
        PlayerFortune4PairFortune12,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune12)),
    ),
    bet_definition!(
        PlayerFortune4PairFortune9,
        BetClass::OpeningTwoCombinator,
        true,
        Some(BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune9)),
    ),
    bet_definition!(
        PlayerFortune4PairAggregate,
        BetClass::AggregateBet,
        true,
        None,
    ),
    bet_definition!(PlayerRed, BetClass::OpeningTwoCombinator, true, None,),
    bet_definition!(BankerRed, BetClass::OpeningTwoCombinator, true, None,),
    bet_definition!(PlayerBlack, BetClass::OpeningTwoCombinator, true, None,),
    bet_definition!(BankerBlack, BetClass::OpeningTwoCombinator, true, None,),
    bet_definition!(Invincible6, BetClass::TerminalPredicate, false, None,),
    bet_definition!(Big, BetClass::TerminalPredicate, false, None,),
    bet_definition!(Small, BetClass::TerminalPredicate, false, None,),
    bet_definition!(
        BankerCharSiuPoint4,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::CharSiu(CharSiuVariant::Point4)),
    ),
    bet_definition!(
        BankerCharSiuPoint5,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::CharSiu(CharSiuVariant::Point5)),
    ),
    bet_definition!(
        BankerCharSiuPoint6,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::CharSiu(CharSiuVariant::Point6)),
    ),
    bet_definition!(BankerCharSiuAggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(
        PlayerCharSiuPoint4,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::CharSiu(CharSiuVariant::Point4)),
    ),
    bet_definition!(
        PlayerCharSiuPoint5,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::CharSiu(CharSiuVariant::Point5)),
    ),
    bet_definition!(
        PlayerCharSiuPoint6,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::CharSiu(CharSiuVariant::Point6)),
    ),
    bet_definition!(PlayerCharSiuAggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(SmallBull, BetClass::TerminalPredicate, false, None,),
    bet_definition!(BigBull, BetClass::TerminalPredicate, false, None,),
    bet_definition!(TigerBull, BetClass::TerminalPredicate, false, None,),
    bet_definition!(WuDaLang, BetClass::TerminalPredicate, false, None,),
    bet_definition!(Dragon7, BetClass::TerminalPredicate, false, None,),
    bet_definition!(Panda8, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie0, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie1, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie2, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie3, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie4, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie5, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie6, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie7, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie8, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SuperTie9, BetClass::TerminalPredicate, false, None,),
    bet_definition!(
        Lucky7Two,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Lucky7(Lucky7Variant::Two)),
    ),
    bet_definition!(
        Lucky7Three,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Lucky7(Lucky7Variant::Three)),
    ),
    bet_definition!(Lucky7Aggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(BigLucky7, BetClass::TerminalPredicate, false, None,),
    bet_definition!(SmallLucky7, BetClass::TerminalPredicate, false, None,),
    bet_definition!(
        SuperLucky7Four,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::SuperLucky7(SuperLucky7Variant::Four)),
    ),
    bet_definition!(
        SuperLucky7Five,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::SuperLucky7(SuperLucky7Variant::Five)),
    ),
    bet_definition!(
        SuperLucky7Six,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::SuperLucky7(SuperLucky7Variant::Six)),
    ),
    bet_definition!(SuperLucky7Aggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(
        Flame7sTwo,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Flame7s(Flame7sVariant::Two)),
    ),
    bet_definition!(
        Flame7sThree,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Flame7s(Flame7sVariant::Three)),
    ),
    bet_definition!(Flame7sAggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(
        Heaven9Single,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Heaven9(Heaven9Variant::Single)),
    ),
    bet_definition!(
        Heaven9Both,
        BetClass::TerminalPredicate,
        false,
        Some(BetVariant::Heaven9(Heaven9Variant::Both)),
    ),
    bet_definition!(Heaven9Aggregate, BetClass::AggregateBet, false, None,),
    bet_definition!(TreasureAll, BetClass::AggregateBet, false, None,),
];

pub fn bet_definitions() -> &'static [BetDefinition] {
    BET_DEFINITIONS
}

pub fn public_probability_definitions() -> impl Iterator<Item = &'static BetDefinition> {
    BET_DEFINITIONS
        .iter()
        .filter(|definition| definition.is_public_probability_result())
}
