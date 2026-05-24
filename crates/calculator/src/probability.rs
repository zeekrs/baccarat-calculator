use serde::{Deserialize, Serialize};

use crate::{
    bet_registry::{
        bet_definitions, public_probability_definitions, BetClass, BetDefinition, BetId, BetType,
        BetVariant, Fortune4PairVariant, TigerPairVariant,
    },
    shoe::ShoeCounts,
    standard_baccarat,
    terminal::{
        aggregate_opening_probability_breakdown, aggregate_terminal_probability_breakdown,
        terminal_probability_breakdown, TerminalAccumulator, TerminalWinner,
    },
    BetOutcome, CardCount, CardRank, CardSuit,
};

/// Probability for one public variant under a canonical public bet row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariantProbability {
    /// Public variant identifier within the parent bet.
    pub variant: BetVariant,
    /// Objective probability for this variant under the supplied card counts.
    pub probability: f64,
}

/// Probability for one outcome bucket under a canonical public bet row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeProbability {
    /// Outcome bucket, such as `Monkey` or `PerfectPairBothSides`.
    pub outcome: BetOutcome,
    /// Objective probability for this outcome under the supplied card counts.
    pub probability: f64,
}

/// Probability row for one caller-facing public bet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProbabilityResult {
    /// Caller-facing public bet identifier.
    pub bet_type: BetType,
    /// Aggregate probability for this public bet.
    pub probability: f64,
    /// Variant probabilities when this public row exposes variant detail.
    pub variants: Vec<VariantProbability>,
    /// Outcome bucket probabilities when this public row has branch detail.
    pub outcomes: Vec<OutcomeProbability>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProbabilitySnapshot {
    pub(crate) standard: StandardOutcomeProbabilities,
    pub(crate) opening_two: OpeningTwoStats,
    pub(crate) terminal: TerminalAccumulator,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StandardOutcomeProbabilities {
    pub(crate) player: f64,
    pub(crate) banker: f64,
    pub(crate) tie: f64,
}

impl StandardOutcomeProbabilities {
    fn from_terminal(terminal: &TerminalAccumulator) -> Self {
        let denominator = terminal.denominator() as f64;
        let mut player = 0_u128;
        let mut banker = 0_u128;
        let mut tie = 0_u128;

        for (outcome, weight) in terminal.weighted_outcomes() {
            match outcome.winner {
                TerminalWinner::Player => player += weight,
                TerminalWinner::Banker => banker += weight,
                TerminalWinner::Tie => tie += weight,
            }
        }

        Self {
            player: player as f64 / denominator,
            banker: banker as f64 / denominator,
            tie: tie as f64 / denominator,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProbabilityRatio {
    pub(crate) numerator: u128,
    pub(crate) denominator: u128,
}

impl ProbabilityRatio {
    pub(crate) const fn zero() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }

    pub(crate) fn new(numerator: u128, denominator: u128) -> Self {
        if denominator == 0 {
            Self::zero()
        } else {
            Self {
                numerator,
                denominator,
            }
        }
    }

    pub(crate) fn as_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

impl Default for ProbabilityRatio {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Fortune4PairStats {
    fortune30: ProbabilityRatio,
    fortune15: ProbabilityRatio,
    fortune12: ProbabilityRatio,
    fortune9: ProbabilityRatio,
}

impl Fortune4PairStats {
    fn probability(self, variant: Fortune4PairVariant) -> ProbabilityRatio {
        match variant {
            Fortune4PairVariant::Fortune30 => self.fortune30,
            Fortune4PairVariant::Fortune15 => self.fortune15,
            Fortune4PairVariant::Fortune12 => self.fortune12,
            Fortune4PairVariant::Fortune9 => self.fortune9,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MonkeyStats {
    pub(crate) monkey: ProbabilityRatio,
    pub(crate) no_monkey: ProbabilityRatio,
}

impl MonkeyStats {
    fn probability(self) -> ProbabilityRatio {
        if self.monkey.denominator != self.no_monkey.denominator {
            return ProbabilityRatio::zero();
        }

        ProbabilityRatio::new(
            self.monkey.numerator + self.no_monkey.numerator,
            self.monkey.denominator,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TigerPairStats {
    perfect: ProbabilityRatio,
    both: ProbabilityRatio,
    single: ProbabilityRatio,
}

impl TigerPairStats {
    fn probability(self, variant: TigerPairVariant) -> ProbabilityRatio {
        match variant {
            TigerPairVariant::Perfect => self.perfect,
            TigerPairVariant::Both => self.both,
            TigerPairVariant::Single => self.single,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PerfectPairStats {
    any: ProbabilityRatio,
    pub(crate) single: ProbabilityRatio,
    pub(crate) both: ProbabilityRatio,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct OpeningTwoStats {
    player_pair: ProbabilityRatio,
    banker_pair: ProbabilityRatio,
    any_pair: ProbabilityRatio,
    pub(crate) perfect_pair: PerfectPairStats,
    player_red: ProbabilityRatio,
    banker_red: ProbabilityRatio,
    player_black: ProbabilityRatio,
    banker_black: ProbabilityRatio,
    pub(crate) monkey: MonkeyStats,
    tiger_pair: TigerPairStats,
    player_fortune4_pair: Fortune4PairStats,
    banker_fortune4_pair: Fortune4PairStats,
}

impl OpeningTwoStats {
    pub(crate) fn from_counts(counts: &ShoeCounts) -> Self {
        let pair = same_rank_pair_probability(&counts.rank, counts.total);
        let any_pair = any_pair_probability(&counts.rank, counts.total);

        let perfect_pair = perfect_pair_stats(&counts.card, counts.total);
        let red = color_probability(&counts.card, counts.total, CardColor::Red);
        let black = color_probability(&counts.card, counts.total, CardColor::Black);
        let monkey = monkey_stats(&counts.rank, counts.total);
        let tiger_pair = tiger_pair_stats(&counts.rank, &counts.card, counts.total);
        let fortune4_pair = fortune4_pair_stats(&counts.card, counts.total);

        Self {
            player_pair: pair,
            banker_pair: pair,
            any_pair,
            perfect_pair,
            player_red: red,
            banker_red: red,
            player_black: black,
            banker_black: black,
            monkey,
            tiger_pair,
            player_fortune4_pair: fortune4_pair,
            banker_fortune4_pair: fortune4_pair,
        }
    }

    pub(crate) fn probability_for(self, definition: &BetDefinition) -> Option<ProbabilityRatio> {
        match definition.id {
            BetId::AnyPair => Some(self.any_pair),
            BetId::PlayerPair => Some(self.player_pair),
            BetId::BankerPair => Some(self.banker_pair),
            BetId::PerfectPair => Some(self.perfect_pair.any),
            BetId::PlayerRed => Some(self.player_red),
            BetId::BankerRed => Some(self.banker_red),
            BetId::PlayerBlack => Some(self.player_black),
            BetId::BankerBlack => Some(self.banker_black),
            BetId::Monkey => Some(self.monkey.probability()),
            BetId::TigerPairPerfect | BetId::TigerPairBoth | BetId::TigerPairSingle => {
                let Some(BetVariant::TigerPair(variant)) = definition.variant else {
                    return None;
                };
                Some(self.tiger_pair.probability(variant))
            }
            BetId::BankerFortune4PairFortune30
            | BetId::BankerFortune4PairFortune15
            | BetId::BankerFortune4PairFortune12
            | BetId::BankerFortune4PairFortune9 => {
                let Some(BetVariant::Fortune4Pair(variant)) = definition.variant else {
                    return None;
                };
                Some(self.banker_fortune4_pair.probability(variant))
            }
            BetId::PlayerFortune4PairFortune30
            | BetId::PlayerFortune4PairFortune15
            | BetId::PlayerFortune4PairFortune12
            | BetId::PlayerFortune4PairFortune9 => {
                let Some(BetVariant::Fortune4Pair(variant)) = definition.variant else {
                    return None;
                };
                Some(self.player_fortune4_pair.probability(variant))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OutcomeProbabilityBreakdown {
    pub(crate) win_probability: f64,
    pub(crate) lose_probability: f64,
    pub(crate) push_probability: f64,
}

impl OutcomeProbabilityBreakdown {
    pub(crate) fn new(win_probability: f64, push_probability: f64) -> Self {
        let lose_probability = 1.0 - win_probability - push_probability;

        Self {
            win_probability,
            lose_probability,
            push_probability,
        }
    }
}

pub(crate) fn calculate_probability_snapshot(
    counts: ShoeCounts,
) -> Result<ProbabilitySnapshot, String> {
    let total_cards = u32::from(counts.total);
    const STANDARD_HAND_MAX_CARD_COUNT: u32 = 6;
    if total_cards < STANDARD_HAND_MAX_CARD_COUNT {
        return Err(format!(
            "calculator rejected cards: at least {STANDARD_HAND_MAX_CARD_COUNT} cards are required to complete a standard baccarat hand, found {total_cards}"
        ));
    }

    let opening_two = OpeningTwoStats::from_counts(&counts);
    let terminal = crate::terminal::calculate_terminal_accumulator(counts)?;
    let standard = StandardOutcomeProbabilities::from_terminal(&terminal);

    Ok(ProbabilitySnapshot {
        standard,
        opening_two,
        terminal,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BetProbabilityCore {
    pub(crate) probability: f64,
}

/// Calculates objective probabilities for every registered canonical public bet.
///
/// The only input is the remaining card counts. Callers cannot request a subset
/// of bets through this API. Use `calculate_ev` when you need caller-selected
/// per-bet odds and result rows.
pub fn calculate_probabilities(
    cards: &[CardCount],
) -> Result<Vec<ProbabilityResult>, String> {
    let counts = ShoeCounts::from_cards(cards)?;
    let probabilities = calculate_probability_snapshot(counts)?;
    let mut bets = Vec::new();

    for definition in public_probability_definitions() {
        let core = probability_for_definition(definition, &probabilities)?;

        bets.push(ProbabilityResult {
            bet_type: definition.bet_type(),
            probability: core.probability,
            variants: variant_probabilities_for_definition(definition, &probabilities)?,
            outcomes: outcome_probabilities_for_definition(definition, &probabilities),
        });
    }

    Ok(bets)
}

fn same_rank_pair_probability(counts: &[u16; 13], total_cards: u16) -> ProbabilityRatio {
    ProbabilityRatio::new(sum_falling2(counts), ordered2(total_cards))
}

fn any_pair_probability(rank_counts: &[u16; 13], total_cards: u16) -> ProbabilityRatio {
    let denominator = ordered4(total_cards);
    let pair_orders = sum_falling2(rank_counts);
    let both_pair_orders = both_pair_orders(rank_counts);
    if denominator == 0 {
        return ProbabilityRatio::zero();
    }

    let one_side_scaled = 2
        * pair_orders
        * u128::from(total_cards.saturating_sub(2))
        * u128::from(total_cards.saturating_sub(3));
    ProbabilityRatio::new(one_side_scaled - both_pair_orders, denominator)
}

fn perfect_pair_stats(card_counts: &[u16; 52], total_cards: u16) -> PerfectPairStats {
    let denominator = ordered4(total_cards);
    let pair_orders = sum_falling2(card_counts);
    let both = both_perfect_pair_orders(card_counts);
    if denominator == 0 {
        return PerfectPairStats::default();
    }

    let one_side_scaled = 2
        * pair_orders
        * u128::from(total_cards.saturating_sub(2))
        * u128::from(total_cards.saturating_sub(3));

    PerfectPairStats {
        any: ProbabilityRatio::new(one_side_scaled - both, denominator),
        single: ProbabilityRatio::new(one_side_scaled - 2 * both, denominator),
        both: ProbabilityRatio::new(both, denominator),
    }
}

fn tiger_pair_stats(
    rank_counts: &[u16; 13],
    card_counts: &[u16; 52],
    total_cards: u16,
) -> TigerPairStats {
    let denominator = ordered4(total_cards);
    if denominator == 0 {
        return TigerPairStats::default();
    }

    let both = both_pair_orders(rank_counts);
    let pair_orders = sum_falling2(rank_counts);
    let any = 2
        * pair_orders
        * u128::from(total_cards.saturating_sub(2))
        * u128::from(total_cards.saturating_sub(3))
        - both;
    let perfect = card_counts.iter().map(|count| falling4(*count)).sum();

    TigerPairStats {
        perfect: ProbabilityRatio::new(perfect, denominator),
        both: ProbabilityRatio::new(both, denominator),
        single: ProbabilityRatio::new(any - both, denominator),
    }
}

fn both_pair_orders(rank_counts: &[u16; 13]) -> u128 {
    let pair_orders = sum_falling2(rank_counts);
    let same_rank_square_sum = rank_counts
        .iter()
        .map(|count| {
            let orders = falling2(*count);
            orders * orders
        })
        .sum::<u128>();
    let same_rank_four_card_orders = rank_counts
        .iter()
        .map(|count| falling4(*count))
        .sum::<u128>();
    pair_orders * pair_orders - same_rank_square_sum + same_rank_four_card_orders
}

pub(crate) fn both_perfect_pair_orders(card_counts: &[u16; 52]) -> u128 {
    let pair_orders = sum_falling2(card_counts);
    let same_card_square_sum = card_counts
        .iter()
        .map(|count| {
            let orders = falling2(*count);
            orders * orders
        })
        .sum::<u128>();
    let same_card_four_orders = card_counts
        .iter()
        .map(|count| falling4(*count))
        .sum::<u128>();
    pair_orders * pair_orders - same_card_square_sum + same_card_four_orders
}

fn fortune4_pair_stats(card_counts: &[u16; 52], total_cards: u16) -> Fortune4PairStats {
    let denominator = ordered2(total_cards);
    if denominator == 0 {
        return Fortune4PairStats::default();
    }

    let diamond_index = CardSuit::Diamonds.index();
    let rank_four_index = CardRank::Four.index();
    let mut fortune15 = 0;
    let mut fortune12 = 0;
    let mut fortune9 = 0;
    for suit_index in 0..4 {
        for rank_index in 0..13 {
            let orders = falling2(card_counts[card_index(suit_index, rank_index)]);
            let is_diamond = suit_index == diamond_index;
            let is_rank_four = rank_index == rank_four_index;
            if is_diamond && is_rank_four {
                continue;
            } else if is_rank_four {
                fortune15 += orders;
            } else if is_diamond {
                fortune12 += orders;
            } else {
                fortune9 += orders;
            }
        }
    }

    Fortune4PairStats {
        fortune30: ProbabilityRatio::new(
            falling2(card_counts[card_index(diamond_index, rank_four_index)]),
            denominator,
        ),
        fortune15: ProbabilityRatio::new(fortune15, denominator),
        fortune12: ProbabilityRatio::new(fortune12, denominator),
        fortune9: ProbabilityRatio::new(fortune9, denominator),
    }
}

fn monkey_stats(rank_counts: &[u16; 13], total_cards: u16) -> MonkeyStats {
    let denominator = ordered4(total_cards);
    if denominator == 0 {
        return MonkeyStats::default();
    }

    let monkey_count = rank_counts[10..=12]
        .iter()
        .map(|count| u128::from(*count))
        .sum::<u128>();
    let no_monkey_count = rank_counts[0..=9]
        .iter()
        .map(|count| u128::from(*count))
        .sum::<u128>();

    MonkeyStats {
        monkey: ProbabilityRatio::new(falling4_from_count(monkey_count), denominator),
        no_monkey: ProbabilityRatio::new(falling4_from_count(no_monkey_count), denominator),
    }
}

#[derive(Clone, Copy)]
enum CardColor {
    Red,
    Black,
}

fn color_probability(
    card_counts: &[u16; 52],
    total_cards: u16,
    color: CardColor,
) -> ProbabilityRatio {
    let color_count = card_counts
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            card_color_matches(
                *index / usize::from(standard_baccarat::RANKS_PER_DECK),
                color,
            )
        })
        .map(|(_, count)| u128::from(*count))
        .sum::<u128>();
    ProbabilityRatio::new(
        color_count * color_count.saturating_sub(1),
        ordered2(total_cards),
    )
}

fn card_color_matches(suit_index: usize, color: CardColor) -> bool {
    match color {
        CardColor::Red => matches!(suit_index, 1 | 2),
        CardColor::Black => matches!(suit_index, 0 | 3),
    }
}

const fn card_index(suit_index: usize, rank_index: usize) -> usize {
    suit_index * standard_baccarat::RANKS_PER_DECK as usize + rank_index
}

pub(crate) fn sum_falling2<const N: usize>(counts: &[u16; N]) -> u128 {
    counts.iter().map(|count| falling2(*count)).sum()
}

fn falling2(count: u16) -> u128 {
    u128::from(count) * u128::from(count.saturating_sub(1))
}

fn falling4(count: u16) -> u128 {
    falling4_from_count(u128::from(count))
}

pub(crate) fn falling4_from_count(count: u128) -> u128 {
    count * count.saturating_sub(1) * count.saturating_sub(2) * count.saturating_sub(3)
}

fn ordered2(total_cards: u16) -> u128 {
    u128::from(total_cards) * u128::from(total_cards.saturating_sub(1))
}

pub(crate) fn ordered4(total_cards: u16) -> u128 {
    u128::from(total_cards)
        * u128::from(total_cards.saturating_sub(1))
        * u128::from(total_cards.saturating_sub(2))
        * u128::from(total_cards.saturating_sub(3))
}

pub(crate) fn variant_probabilities_for_definition(
    definition: &BetDefinition,
    probabilities: &ProbabilitySnapshot,
) -> Result<Vec<VariantProbability>, String> {
    let mut variants = Vec::new();

    for child in bet_definitions()
        .iter()
        .filter(|child| child.bet_type() == definition.bet_type())
    {
        let Some(variant) = child.variant else {
            continue;
        };
        let core = probability_for_definition(child, probabilities)?;

        variants.push(VariantProbability {
            variant,
            probability: core.probability,
        });
    }

    Ok(variants)
}

pub(crate) fn outcome_probabilities_for_definition(
    definition: &BetDefinition,
    probabilities: &ProbabilitySnapshot,
) -> Vec<OutcomeProbability> {
    match definition.id {
        BetId::PerfectPair => vec![
            OutcomeProbability {
                outcome: BetOutcome::PerfectPairSingleSide,
                probability: probabilities.opening_two.perfect_pair.single.as_f64(),
            },
            OutcomeProbability {
                outcome: BetOutcome::PerfectPairBothSides,
                probability: probabilities.opening_two.perfect_pair.both.as_f64(),
            },
        ],
        BetId::Monkey => vec![
            OutcomeProbability {
                outcome: BetOutcome::Monkey,
                probability: probabilities.opening_two.monkey.monkey.as_f64(),
            },
            OutcomeProbability {
                outcome: BetOutcome::NoMonkey,
                probability: probabilities.opening_two.monkey.no_monkey.as_f64(),
            },
        ],
        _ => Vec::new(),
    }
}

pub(crate) fn probability_for_definition(
    definition: &BetDefinition,
    probabilities: &ProbabilitySnapshot,
) -> Result<BetProbabilityCore, String> {
    match definition.id {
        BetId::Player => Ok(BetProbabilityCore {
            probability: probabilities.standard.player,
        }),
        BetId::Banker => Ok(BetProbabilityCore {
            probability: probabilities.standard.banker,
        }),
        BetId::Tie => Ok(BetProbabilityCore {
            probability: probabilities.standard.tie,
        }),
        _ => match definition.class {
            BetClass::OpeningTwoCombinator => {
                let Some(ratio) = probabilities.opening_two.probability_for(definition) else {
                    return Err(format!(
                        "unsupported calculator contract: opening-two bet {:?} is registered without probability coverage",
                        definition.id
                    ));
                };

                Ok(BetProbabilityCore {
                    probability: ratio.as_f64(),
                })
            }
            BetClass::TerminalPredicate => {
                let breakdown = terminal_probability_breakdown(&probabilities.terminal, definition);

                Ok(BetProbabilityCore {
                    probability: if definition.id.is_push_result() {
                        breakdown.push_probability
                    } else {
                        breakdown.win_probability
                    },
                })
            }
            BetClass::AggregateBet => {
                let breakdown = outcome_probability_breakdown(definition, probabilities)?;

                Ok(BetProbabilityCore {
                    probability: breakdown.win_probability,
                })
            }
        },
    }
}

pub(crate) fn outcome_probability_breakdown(
    definition: &BetDefinition,
    probabilities: &ProbabilitySnapshot,
) -> Result<OutcomeProbabilityBreakdown, String> {
    match definition.id {
        BetId::Player => Ok(OutcomeProbabilityBreakdown::new(
            probabilities.standard.player,
            probabilities.standard.tie,
        )),
        BetId::Banker => Ok(OutcomeProbabilityBreakdown::new(
            probabilities.standard.banker,
            probabilities.standard.tie,
        )),
        BetId::Tie => Ok(OutcomeProbabilityBreakdown::new(
            probabilities.standard.tie,
            0.0,
        )),
        _ => match definition.class {
            BetClass::OpeningTwoCombinator => {
                let Some(ratio) = probabilities.opening_two.probability_for(definition) else {
                    return Err(format!(
                        "unsupported calculator contract: opening-two bet {:?} is registered without EV coverage",
                        definition.id
                    ));
                };

                Ok(OutcomeProbabilityBreakdown::new(ratio.as_f64(), 0.0))
            }
            BetClass::TerminalPredicate => Ok(terminal_probability_breakdown(
                &probabilities.terminal,
                definition,
            )),
            BetClass::AggregateBet => {
                let evaluation = if let Some(evaluation) =
                    aggregate_terminal_probability_breakdown(&probabilities.terminal, definition)?
                {
                    evaluation
                } else if let Some(evaluation) =
                    aggregate_opening_probability_breakdown(probabilities.opening_two, definition)?
                {
                    evaluation
                } else {
                    return Err(format!(
                        "unsupported calculator contract: aggregate bet {:?} is registered without probability/EV coverage",
                        definition.id
                    ));
                };

                Ok(evaluation)
            }
        },
    }
}
