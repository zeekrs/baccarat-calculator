use crate::{
    bet_registry::{
        public_bet_id_for_type, variant_bet_id, BetDefinition, BetId, BetVariant, CharSiuVariant,
        DragonVariant, Flame7sVariant, Fortune4PairVariant, Heaven9Variant, Lucky6Variant,
        Lucky7Variant, NaturalVariant, SuperLucky7Variant, TigerPairVariant, TigerVariant,
    },
    BetOutcome, BetType,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Cow;

/// Settlement behavior for a variant odds entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OddsSettlement {
    /// Winning outcome pays net odds as profit.
    Net,
    /// Outcome is a refund or push with no profit.
    Refund,
}

/// Simple net odds for one public bet.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimpleOddsSpec {
    /// Public bet this odds value applies to.
    pub bet_type: BetType,
    /// Net odds paid on a winning unit stake.
    pub odds: f64,
}

/// Net odds for one outcome bucket inside a public bet.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeOdds {
    /// Outcome bucket this odds value applies to.
    pub outcome: BetOutcome,
    /// Net odds paid when this outcome wins.
    pub odds: f64,
}

/// Outcome-based odds for one public bet.
///
/// Use this when a bet needs branch-specific odds, such as `Monkey` or
/// `PerfectPair` with an explicit mode.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OutcomeOddsSpec {
    /// Public bet these outcome odds apply to.
    pub bet_type: BetType,
    /// Compatibility odds used by modes that accept a single fallback value.
    pub odds: f64,
    /// Outcome odds supplied by the caller or default table.
    pub outcomes: Cow<'static, [OutcomeOdds]>,
}

impl<'de> Deserialize<'de> for OutcomeOddsSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            bet_type: BetType,
            odds: f64,
            outcomes: Vec<OutcomeOdds>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(Self {
            bet_type: helper.bet_type,
            odds: helper.odds,
            outcomes: Cow::Owned(helper.outcomes),
        })
    }
}

/// Odds for one registry variant under an aggregate public bet.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariantOddsSpec {
    /// Registry identifier for the precise variant row.
    pub bet_id: BetId,
    /// Public variant label under the aggregate bet.
    pub variant: BetVariant,
    /// Net odds paid when this variant wins.
    pub odds: f64,
    /// Whether this variant pays net odds or refunds stake.
    pub settlement: OddsSettlement,
}

/// Odds family made from variant-level child odds.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AggregateOddsSpec {
    /// Registry identifier for the aggregate public bet.
    pub bet_id: BetId,
    /// Variant children that make up this aggregate odds contract.
    pub children: Cow<'static, [VariantOddsSpec]>,
}

impl<'de> Deserialize<'de> for AggregateOddsSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            bet_id: BetId,
            children: Vec<VariantOddsSpec>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(Self {
            bet_id: helper.bet_id,
            children: Cow::Owned(helper.children),
        })
    }
}

/// Caller-facing EV odds specification.
///
/// `Simple` covers one public bet with one net odds value. `ByOutcome` covers a
/// public bet with outcome-specific odds. `Variant` and `Aggregate` keep
/// registry precision for default odds and aggregate families.
///
/// ### Aggregate bet types
///
/// Aggregate canonical bets (such as `BetType::Lucky6`, `BetType::SuperLucky7`,
/// `BetType::PlayerDragon`) accept two distinct odds representations:
///
/// - `OddsSpec::Aggregate`: every winning variant pays its own odds. This is the
///   default-table representation and yields the math-correct per-variant EV
///   calculation. Aggregate odds must list every calculator variant (including
///   push variants where applicable) — `calculate_ev` rejects mismatched lists.
/// - `OddsSpec::Simple` on the aggregate bet type: every winning variant pays
///   the same net odds value. This is intentionally supported so that callers
///   can model promo / flat-odds offers without splitting children. The EV
///   reduces to `win_probability × net_odds − lose_probability` and will differ
///   from the per-variant `Aggregate` form unless every variant happens to
///   share that single net odds value.
///
/// Pick `Aggregate` for accurate house-edge analysis and `Simple` only when the
/// platform actually pays a single net odds for every winning variant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OddsSpec {
    /// One public bet with one net odds value.
    Simple(SimpleOddsSpec),
    /// One public bet with outcome-specific odds.
    ByOutcome(OutcomeOddsSpec),
    /// One variant-level odds row.
    Variant(VariantOddsSpec),
    /// One aggregate public bet made from variant-level children.
    Aggregate(AggregateOddsSpec),
}

impl OddsSpec {
    /// Builds simple net odds for one public bet.
    pub const fn simple(bet_type: BetType, odds: f64) -> Self {
        Self::Simple(SimpleOddsSpec { bet_type, odds })
    }

    /// Builds outcome-specific odds for one public bet.
    ///
    /// Default `PerfectPair` uses only `PerfectPairSingleSide` at 25. To pay a
    /// separate both-sides branch such as 200, callers must use
    /// `BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth)` and provide both
    /// `PerfectPairSingleSide` and `PerfectPairBothSides` odds.
    pub const fn by_outcome(
        bet_type: BetType,
        odds: f64,
        outcomes: &'static [OutcomeOdds],
    ) -> Self {
        Self::ByOutcome(OutcomeOddsSpec {
            bet_type,
            odds,
            outcomes: Cow::Borrowed(outcomes),
        })
    }

    /// Builds net odds for one variant under a public aggregate family.
    pub const fn with_variant(bet_type: BetType, variant: BetVariant, odds: f64) -> Self {
        Self::Variant(VariantOddsSpec {
            bet_id: variant_bet_id(bet_type, variant),
            variant,
            odds,
            settlement: OddsSettlement::Net,
        })
    }

    /// Builds a refunded variant under a public aggregate family.
    pub const fn refunded_variant(bet_type: BetType, variant: BetVariant) -> Self {
        Self::Variant(VariantOddsSpec {
            bet_id: variant_bet_id(bet_type, variant),
            variant,
            odds: 0.0,
            settlement: OddsSettlement::Refund,
        })
    }

    /// Builds an aggregate odds family from variant children.
    pub const fn aggregate(bet_id: BetId, children: &'static [VariantOddsSpec]) -> Self {
        Self::Aggregate(AggregateOddsSpec {
            bet_id,
            children: Cow::Borrowed(children),
        })
    }

    /// Builds an aggregate odds family from owned variant children.
    ///
    /// Useful when callers construct aggregate odds dynamically (e.g. from a
    /// platform config or in tests). For static default odds prefer
    /// [`OddsSpec::aggregate`].
    pub fn aggregate_owned(bet_id: BetId, children: Vec<VariantOddsSpec>) -> Self {
        Self::Aggregate(AggregateOddsSpec {
            bet_id,
            children: Cow::Owned(children),
        })
    }

    /// Returns the caller-facing public bet for this odds spec.
    pub const fn bet_type(&self) -> BetType {
        match self {
            Self::Simple(spec) => spec.bet_type,
            Self::ByOutcome(spec) => spec.bet_type,
            Self::Variant(spec) => spec.bet_id.bet_type(),
            Self::Aggregate(spec) => spec.bet_id.bet_type(),
        }
    }

    /// Returns the registry identifier represented by this odds spec.
    pub const fn bet_id(&self) -> BetId {
        match self {
            Self::Simple(spec) => public_bet_id_for_type(spec.bet_type),
            Self::ByOutcome(spec) => public_bet_id_for_type(spec.bet_type),
            Self::Variant(spec) => spec.bet_id,
            Self::Aggregate(spec) => spec.bet_id,
        }
    }

    /// Returns the single net odds value when this spec has one.
    pub const fn odds(&self) -> Option<f64> {
        match self {
            Self::Simple(spec) => Some(spec.odds),
            Self::ByOutcome(spec) => Some(spec.odds),
            Self::Variant(spec) => Some(spec.odds),
            Self::Aggregate(_) => None,
        }
    }

    /// Returns variant settlement behavior when this is a variant spec.
    pub const fn settlement(&self) -> Option<OddsSettlement> {
        match self {
            Self::Simple(_) | Self::ByOutcome(_) => None,
            Self::Variant(spec) => Some(spec.settlement),
            Self::Aggregate(_) => None,
        }
    }

    /// Returns the public variant when this is a variant spec.
    pub const fn bet_variant(&self) -> Option<BetVariant> {
        match self {
            Self::Simple(_) | Self::ByOutcome(_) | Self::Aggregate(_) => None,
            Self::Variant(spec) => Some(spec.variant),
        }
    }

    /// Returns outcome odds when this is an outcome-based spec.
    pub fn outcome_odds(&self) -> Option<&[OutcomeOdds]> {
        match self {
            Self::ByOutcome(spec) => Some(spec.outcomes.as_ref()),
            _ => None,
        }
    }

    /// Returns the odds for one outcome bucket when present.
    pub fn odds_for_outcome(&self, outcome: BetOutcome) -> Option<f64> {
        self.outcome_odds()?
            .iter()
            .find(|candidate| candidate.outcome == outcome)
            .map(|candidate| candidate.odds)
    }

    /// Returns aggregate children when this is an aggregate spec.
    pub fn children(&self) -> Option<&[VariantOddsSpec]> {
        match self {
            Self::Aggregate(spec) => Some(spec.children.as_ref()),
            _ => None,
        }
    }

    /// Returns whether this odds spec maps to a registry definition.
    pub fn matches_definition(&self, definition: BetDefinition) -> bool {
        self.bet_id() == definition.id
    }
}

/// Static lookup table for default odds specs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OddsTable {
    specs: &'static [OddsSpec],
}

impl OddsTable {
    /// Creates a lookup table over static odds specs.
    pub const fn new(specs: &'static [OddsSpec]) -> Self {
        Self { specs }
    }

    /// Returns all specs backing this table.
    pub fn specs(self) -> &'static [OddsSpec] {
        self.specs
    }

    /// Looks up odds by registry identifier, including aggregate children.
    pub fn get(&self, bet_id: BetId) -> Option<OddsSpec> {
        for spec in self.specs.iter() {
            if spec.bet_id() == bet_id {
                return Some(spec.clone());
            }

            if let OddsSpec::Aggregate(aggregate) = spec {
                if let Some(child) = aggregate
                    .children
                    .iter()
                    .cloned()
                    .find(|child| child.bet_id == bet_id)
                {
                    return Some(OddsSpec::Variant(child));
                }
            }
        }

        None
    }

    /// Returns true when the table contains odds for the registry identifier.
    pub fn contains(&self, bet_id: BetId) -> bool {
        self.get(bet_id).is_some()
    }
}

impl Default for OddsTable {
    fn default() -> Self {
        Self::new(DEFAULT_ODDS_SPECS)
    }
}

const PLAYER_DRAGON_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::PlayerDragonNatural,
        variant: BetVariant::Dragon(DragonVariant::Natural),
        odds: 1.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerDragonPoint4,
        variant: BetVariant::Dragon(DragonVariant::Point4),
        odds: 1.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerDragonPoint5,
        variant: BetVariant::Dragon(DragonVariant::Point5),
        odds: 2.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerDragonPoint6,
        variant: BetVariant::Dragon(DragonVariant::Point6),
        odds: 4.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerDragonPoint7,
        variant: BetVariant::Dragon(DragonVariant::Point7),
        odds: 6.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerDragonPoint8,
        variant: BetVariant::Dragon(DragonVariant::Point8),
        odds: 10.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerDragonPoint9,
        variant: BetVariant::Dragon(DragonVariant::Point9),
        odds: 30.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerDragonPush,
        variant: BetVariant::Dragon(DragonVariant::Push),
        odds: 0.0,
        settlement: OddsSettlement::Refund,
    },
];

const BANKER_DRAGON_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::BankerDragonNatural,
        variant: BetVariant::Dragon(DragonVariant::Natural),
        odds: 1.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerDragonPoint4,
        variant: BetVariant::Dragon(DragonVariant::Point4),
        odds: 1.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerDragonPoint5,
        variant: BetVariant::Dragon(DragonVariant::Point5),
        odds: 2.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerDragonPoint6,
        variant: BetVariant::Dragon(DragonVariant::Point6),
        odds: 4.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerDragonPoint7,
        variant: BetVariant::Dragon(DragonVariant::Point7),
        odds: 6.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerDragonPoint8,
        variant: BetVariant::Dragon(DragonVariant::Point8),
        odds: 10.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerDragonPoint9,
        variant: BetVariant::Dragon(DragonVariant::Point9),
        odds: 30.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerDragonPush,
        variant: BetVariant::Dragon(DragonVariant::Push),
        odds: 0.0,
        settlement: OddsSettlement::Refund,
    },
];

const BANKER_NATURAL_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::BankerNaturalWin,
        variant: BetVariant::Natural(NaturalVariant::Win),
        odds: 4.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerNaturalPush,
        variant: BetVariant::Natural(NaturalVariant::Push),
        odds: 0.0,
        settlement: OddsSettlement::Refund,
    },
];

const PLAYER_NATURAL_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::PlayerNaturalWin,
        variant: BetVariant::Natural(NaturalVariant::Win),
        odds: 4.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerNaturalPush,
        variant: BetVariant::Natural(NaturalVariant::Push),
        odds: 0.0,
        settlement: OddsSettlement::Refund,
    },
];

const LUCKY6_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::Lucky6Two,
        variant: BetVariant::Lucky6(Lucky6Variant::Two),
        odds: 12.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::Lucky6Three,
        variant: BetVariant::Lucky6(Lucky6Variant::Three),
        odds: 20.0,
        settlement: OddsSettlement::Net,
    },
];

const TIGER_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::TigerTwo,
        variant: BetVariant::Tiger(TigerVariant::Two),
        odds: 12.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::TigerThree,
        variant: BetVariant::Tiger(TigerVariant::Three),
        odds: 20.0,
        settlement: OddsSettlement::Net,
    },
];

const TIGER_PAIR_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::TigerPairPerfect,
        variant: BetVariant::TigerPair(TigerPairVariant::Perfect),
        odds: 100.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::TigerPairBoth,
        variant: BetVariant::TigerPair(TigerPairVariant::Both),
        odds: 20.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::TigerPairSingle,
        variant: BetVariant::TigerPair(TigerPairVariant::Single),
        odds: 4.0,
        settlement: OddsSettlement::Net,
    },
];

const BANKER_FORTUNE4_PAIR_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::BankerFortune4PairFortune30,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30),
        odds: 30.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerFortune4PairFortune15,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15),
        odds: 15.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerFortune4PairFortune12,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune12),
        odds: 12.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerFortune4PairFortune9,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune9),
        odds: 9.0,
        settlement: OddsSettlement::Net,
    },
];

const PLAYER_FORTUNE4_PAIR_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::PlayerFortune4PairFortune30,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30),
        odds: 30.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerFortune4PairFortune15,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15),
        odds: 15.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerFortune4PairFortune12,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune12),
        odds: 12.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerFortune4PairFortune9,
        variant: BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune9),
        odds: 9.0,
        settlement: OddsSettlement::Net,
    },
];

const BANKER_CHAR_SIU_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::BankerCharSiuPoint4,
        variant: BetVariant::CharSiu(CharSiuVariant::Point4),
        odds: 10.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerCharSiuPoint5,
        variant: BetVariant::CharSiu(CharSiuVariant::Point5),
        odds: 15.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::BankerCharSiuPoint6,
        variant: BetVariant::CharSiu(CharSiuVariant::Point6),
        odds: 50.0,
        settlement: OddsSettlement::Net,
    },
];

const PLAYER_CHAR_SIU_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::PlayerCharSiuPoint4,
        variant: BetVariant::CharSiu(CharSiuVariant::Point4),
        odds: 10.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerCharSiuPoint5,
        variant: BetVariant::CharSiu(CharSiuVariant::Point5),
        odds: 15.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::PlayerCharSiuPoint6,
        variant: BetVariant::CharSiu(CharSiuVariant::Point6),
        odds: 50.0,
        settlement: OddsSettlement::Net,
    },
];

const LUCKY7_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::Lucky7Two,
        variant: BetVariant::Lucky7(Lucky7Variant::Two),
        odds: 6.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::Lucky7Three,
        variant: BetVariant::Lucky7(Lucky7Variant::Three),
        odds: 15.0,
        settlement: OddsSettlement::Net,
    },
];

const SUPER_LUCKY7_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::SuperLucky7Four,
        variant: BetVariant::SuperLucky7(SuperLucky7Variant::Four),
        odds: 30.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::SuperLucky7Five,
        variant: BetVariant::SuperLucky7(SuperLucky7Variant::Five),
        odds: 40.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::SuperLucky7Six,
        variant: BetVariant::SuperLucky7(SuperLucky7Variant::Six),
        odds: 100.0,
        settlement: OddsSettlement::Net,
    },
];

const FLAME7S_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::Flame7sTwo,
        variant: BetVariant::Flame7s(Flame7sVariant::Two),
        odds: 40.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::Flame7sThree,
        variant: BetVariant::Flame7s(Flame7sVariant::Three),
        odds: 150.0,
        settlement: OddsSettlement::Net,
    },
];

const HEAVEN9_CHILDREN: &[VariantOddsSpec] = &[
    VariantOddsSpec {
        bet_id: BetId::Heaven9Single,
        variant: BetVariant::Heaven9(Heaven9Variant::Single),
        odds: 8.0,
        settlement: OddsSettlement::Net,
    },
    VariantOddsSpec {
        bet_id: BetId::Heaven9Both,
        variant: BetVariant::Heaven9(Heaven9Variant::Both),
        odds: 60.0,
        settlement: OddsSettlement::Net,
    },
];

const PERFECT_PAIR_OUTCOMES: &[OutcomeOdds] = &[OutcomeOdds {
    outcome: BetOutcome::PerfectPairSingleSide,
    odds: 25.0,
}];

const MONKEY_OUTCOMES: &[OutcomeOdds] = &[
    OutcomeOdds {
        outcome: BetOutcome::Monkey,
        odds: 50.0,
    },
    OutcomeOdds {
        outcome: BetOutcome::NoMonkey,
        odds: 1.0,
    },
];

/// Canonical default odds specs for every public EV bet.
///
/// `PerfectPair` defaults to single-side only with 25 odds. Both-sides odds,
/// such as 200, are an explicit `PerfectPairMode::SinglePlusBoth` caller
/// contract and are not part of the default table.
pub const DEFAULT_ODDS_SPECS: &[OddsSpec] = &[
    OddsSpec::simple(BetType::Player, 1.0),
    OddsSpec::simple(BetType::Banker, 0.95),
    OddsSpec::simple(BetType::Tie, 8.0),
    OddsSpec::simple(BetType::AnyPair, 5.0),
    OddsSpec::simple(BetType::PlayerPair, 11.0),
    OddsSpec::simple(BetType::BankerPair, 11.0),
    OddsSpec::by_outcome(BetType::PerfectPair, 25.0, PERFECT_PAIR_OUTCOMES),
    OddsSpec::by_outcome(BetType::Monkey, 0.0, MONKEY_OUTCOMES),
    OddsSpec::aggregate(BetId::PlayerDragonAggregate, PLAYER_DRAGON_CHILDREN),
    OddsSpec::aggregate(BetId::BankerDragonAggregate, BANKER_DRAGON_CHILDREN),
    OddsSpec::aggregate(BetId::BankerNaturalAggregate, BANKER_NATURAL_CHILDREN),
    OddsSpec::aggregate(BetId::PlayerNaturalAggregate, PLAYER_NATURAL_CHILDREN),
    OddsSpec::aggregate(BetId::Lucky6Aggregate, LUCKY6_CHILDREN),
    OddsSpec::aggregate(BetId::TigerAggregate, TIGER_CHILDREN),
    OddsSpec::simple(BetType::SmallTiger, 22.0),
    OddsSpec::simple(BetType::BigTiger, 50.0),
    OddsSpec::simple(BetType::TigerTie, 35.0),
    OddsSpec::aggregate(BetId::TigerPairAggregate, TIGER_PAIR_CHILDREN),
    OddsSpec::simple(BetType::Banker4Fortune, 20.0),
    OddsSpec::simple(BetType::Player4Fortune, 35.0),
    OddsSpec::aggregate(
        BetId::BankerFortune4PairAggregate,
        BANKER_FORTUNE4_PAIR_CHILDREN,
    ),
    OddsSpec::aggregate(
        BetId::PlayerFortune4PairAggregate,
        PLAYER_FORTUNE4_PAIR_CHILDREN,
    ),
    OddsSpec::simple(BetType::PlayerRed, 2.0),
    OddsSpec::simple(BetType::BankerRed, 2.0),
    OddsSpec::simple(BetType::PlayerBlack, 2.0),
    OddsSpec::simple(BetType::BankerBlack, 2.0),
    OddsSpec::simple(BetType::Invincible6, 4.0),
    OddsSpec::simple(BetType::Big, 0.54),
    OddsSpec::simple(BetType::Small, 1.5),
    OddsSpec::aggregate(BetId::BankerCharSiuAggregate, BANKER_CHAR_SIU_CHILDREN),
    OddsSpec::aggregate(BetId::PlayerCharSiuAggregate, PLAYER_CHAR_SIU_CHILDREN),
    OddsSpec::simple(BetType::SmallBull, 20.0),
    OddsSpec::simple(BetType::BigBull, 35.0),
    OddsSpec::simple(BetType::TigerBull, 4.0),
    OddsSpec::simple(BetType::WuDaLang, 150.0),
    OddsSpec::simple(BetType::BigLucky7, 30.0),
    OddsSpec::simple(BetType::SmallLucky7, 15.0),
    OddsSpec::simple(BetType::Dragon7, 40.0),
    OddsSpec::simple(BetType::Panda8, 25.0),
    OddsSpec::simple(BetType::SuperTie0, 150.0),
    OddsSpec::simple(BetType::SuperTie1, 215.0),
    OddsSpec::simple(BetType::SuperTie2, 220.0),
    OddsSpec::simple(BetType::SuperTie3, 200.0),
    OddsSpec::simple(BetType::SuperTie4, 120.0),
    OddsSpec::simple(BetType::SuperTie5, 110.0),
    OddsSpec::simple(BetType::SuperTie6, 45.0),
    OddsSpec::simple(BetType::SuperTie7, 45.0),
    OddsSpec::simple(BetType::SuperTie8, 80.0),
    OddsSpec::simple(BetType::SuperTie9, 80.0),
    OddsSpec::aggregate(BetId::Lucky7Aggregate, LUCKY7_CHILDREN),
    OddsSpec::aggregate(BetId::SuperLucky7Aggregate, SUPER_LUCKY7_CHILDREN),
    OddsSpec::aggregate(BetId::Flame7sAggregate, FLAME7S_CHILDREN),
    OddsSpec::aggregate(BetId::Heaven9Aggregate, HEAVEN9_CHILDREN),
    OddsSpec::simple(BetType::TreasureAll, 5.0),
];

/// Static lookup table over `DEFAULT_ODDS_SPECS`.
pub const DEFAULT_ODDS_TABLE: OddsTable = OddsTable::new(DEFAULT_ODDS_SPECS);

/// Returns the canonical static default odds specs.
pub const fn default_odds_specs() -> &'static [OddsSpec] {
    DEFAULT_ODDS_SPECS
}

/// Returns the canonical default odds lookup table.
pub const fn default_odds_table() -> OddsTable {
    DEFAULT_ODDS_TABLE
}
