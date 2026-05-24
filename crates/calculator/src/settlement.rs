use crate::{
    mode_contract, BetMode, BetOutcome, BetType, Card, CardRank, MonkeyMode, PerfectPairMode,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Resolved player and banker hands used for settlement.
///
/// Cards are final dealt cards, not remaining shoe counts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DealtHands {
    /// Final player hand in deal order.
    pub player: Vec<Card>,
    /// Final banker hand in deal order.
    pub banker: Vec<Card>,
}

/// Decimal odds for one settlement outcome bucket.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementOutcomeOdds {
    /// Outcome bucket this odds value applies to.
    pub outcome: BetOutcome,
    /// Net odds paid when this outcome wins.
    pub odds: Decimal,
}

/// Settlement odds using Decimal money precision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementOddsSpec {
    /// One net odds value for the bet.
    Simple(Decimal),
    /// Outcome-specific odds for a bet with branch outcomes.
    ByOutcome(Vec<SettlementOutcomeOdds>),
}

/// Caller-supplied settlement request for one placed bet.
///
/// `amount` and all odds use `Decimal`. Settlement validates positive amount,
/// amount scale, odds scale, mode compatibility, duplicate outcome odds,
/// irrelevant outcome odds, and missing required outcome odds fail-closed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementSpec {
    /// Caller-defined nonblank identifier copied into the result row.
    pub id: String,
    /// Public bet to settle.
    pub bet_type: BetType,
    /// Stake amount with at most two decimal places.
    pub amount: Decimal,
    /// Optional branch interpretation for outcome odds.
    pub mode: Option<BetMode>,
    /// Odds used to calculate payout and profit.
    pub odds: SettlementOddsSpec,
}

/// Odds actually applied to a winning settlement result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedOdds {
    /// Matched outcome for outcome-based bets, or `None` for single odds.
    pub outcome: Option<BetOutcome>,
    /// Decimal net odds applied to the stake.
    pub odds: Decimal,
}

/// Final win, lose, or push state for one settled bet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementStatus {
    /// Bet won and pays profit plus stake.
    Win,
    /// Bet lost and returns no stake.
    Lose,
    /// Bet pushed and returns stake without profit.
    Push,
}

/// Settlement result for one `SettlementSpec`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementResult {
    /// Caller-defined settlement identifier.
    pub id: String,
    /// Public bet settled for this row.
    pub bet_type: BetType,
    /// Mode used for branch interpretation, when supplied.
    pub mode: Option<BetMode>,
    /// Final settlement state.
    pub status: SettlementStatus,
    /// Outcome buckets that matched the dealt cards.
    pub matched_outcomes: Vec<BetOutcome>,
    /// Original stake amount.
    pub stake: Decimal,
    /// Returned amount after settlement, rounded to two decimal places.
    pub payout: Decimal,
    /// Profit or loss after settlement, rounded to two decimal places.
    pub profit: Decimal,
    /// Odds applied on a win. Empty for lose and push results.
    pub applied_odds: Vec<ResolvedOdds>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Winner {
    Player,
    Banker,
    Tie,
}

/// Settles a batch of bet specs against the same final cards.
///
/// Results preserve input order. Any invalid spec or unsupported settlement
/// contract fails the whole batch.
pub fn settle_bets(
    cards: &DealtHands,
    specs: &[SettlementSpec],
) -> Result<Vec<SettlementResult>, String> {
    specs.iter().map(|spec| settle_bet(cards, spec)).collect()
}

/// Settles one bet spec against final player and banker cards.
///
/// Money and odds use `Decimal`. Invalid amount or odds scale, incompatible
/// modes, duplicate or irrelevant outcome odds, and missing required outcome
/// odds all return errors instead of guessing a payout.
pub fn settle_bet(
    cards: &DealtHands,
    spec: &SettlementSpec,
) -> Result<SettlementResult, String> {
    validate_settlement_input(cards, spec)?;

    let matched_outcomes = matched_outcomes(cards, spec.bet_type, spec.mode)?;
    let status = settlement_status(cards, spec.bet_type, &matched_outcomes)?;
    let applied_odds = match status {
        SettlementStatus::Win => applied_odds(spec, &matched_outcomes)?,
        SettlementStatus::Lose | SettlementStatus::Push => Vec::new(),
    };
    let total_applied_odds = applied_odds.iter().map(|entry| entry.odds).sum::<Decimal>();
    let profit = round_money(match status {
        SettlementStatus::Win => spec.amount * total_applied_odds,
        SettlementStatus::Lose => -spec.amount,
        SettlementStatus::Push => Decimal::ZERO,
    });
    let payout = round_money(match status {
        SettlementStatus::Win => spec.amount + profit,
        SettlementStatus::Lose => Decimal::ZERO,
        SettlementStatus::Push => spec.amount,
    });

    Ok(SettlementResult {
        id: spec.id.clone(),
        bet_type: spec.bet_type,
        mode: spec.mode,
        status,
        matched_outcomes,
        stake: spec.amount,
        payout,
        profit,
        applied_odds,
    })
}

fn validate_settlement_input(cards: &DealtHands, spec: &SettlementSpec) -> Result<(), String> {
    if spec.id.trim().is_empty() {
        return Err(String::from("settlement spec id cannot be blank"));
    }
    if spec.amount <= Decimal::ZERO {
        return Err(format!(
            "settlement spec {} amount must be greater than zero",
            spec.id
        ));
    }
    if spec.amount.scale() > 2 {
        return Err(format!(
            "settlement spec {} amount must have at most two decimal places",
            spec.id
        ));
    }
    if cards.player.len() < 2 || cards.banker.len() < 2 {
        return Err(String::from(
            "settlement cards require at least two player and banker cards",
        ));
    }
    mode_contract::validate_mode_compatibility(spec.bet_type, spec.mode)
        .map_err(|error| format!("settlement spec {} {error}", spec.id))?;
    validate_odds(&spec.odds, &spec.id)?;
    validate_outcome_odds(spec)
}

fn validate_odds(odds: &SettlementOddsSpec, id: &str) -> Result<(), String> {
    match odds {
        SettlementOddsSpec::Simple(odds) if *odds < Decimal::ZERO => {
            Err(format!("settlement spec {id} has negative odds"))
        }
        SettlementOddsSpec::Simple(odds) if odds.scale() > 2 => Err(format!(
            "settlement spec {id} odds must have at most two decimal places"
        )),
        SettlementOddsSpec::Simple(_) => Ok(()),
        SettlementOddsSpec::ByOutcome(outcomes) => {
            for outcome in outcomes {
                if outcome.odds < Decimal::ZERO {
                    return Err(format!("settlement spec {id} has negative odds"));
                }
                if outcome.odds.scale() > 2 {
                    return Err(format!(
                        "settlement spec {id} odds must have at most two decimal places"
                    ));
                }
            }
            Ok(())
        }
    }
}

fn validate_outcome_odds(spec: &SettlementSpec) -> Result<(), String> {
    let SettlementOddsSpec::ByOutcome(outcomes) = &spec.odds else {
        return Ok(());
    };
    let allowed_outcomes = mode_contract::settlement_allowed_outcomes(spec.bet_type, spec.mode);
    if allowed_outcomes.is_empty() {
        return Err(format!(
            "settlement spec {} does not support outcome odds",
            spec.id
        ));
    }
    for (index, outcome_odds) in outcomes.iter().enumerate() {
        if !allowed_outcomes.contains(&outcome_odds.outcome) {
            return Err(format!(
                "settlement spec {} has irrelevant outcome odds",
                spec.id
            ));
        }
        if outcomes
            .iter()
            .skip(index + 1)
            .any(|candidate| candidate.outcome == outcome_odds.outcome)
        {
            return Err(format!(
                "settlement spec {} has duplicate outcome odds",
                spec.id
            ));
        }
    }
    Ok(())
}

fn round_money(amount: Decimal) -> Decimal {
    amount.round_dp(2)
}

fn settlement_status(
    cards: &DealtHands,
    bet_type: BetType,
    matched_outcomes: &[BetOutcome],
) -> Result<SettlementStatus, String> {
    match bet_type {
        BetType::Player => match winner(cards) {
            Winner::Player => Ok(SettlementStatus::Win),
            Winner::Banker => Ok(SettlementStatus::Lose),
            Winner::Tie => Ok(SettlementStatus::Push),
        },
        BetType::Banker => match winner(cards) {
            Winner::Banker => Ok(SettlementStatus::Win),
            Winner::Player => Ok(SettlementStatus::Lose),
            Winner::Tie => Ok(SettlementStatus::Push),
        },
        BetType::Tie => Ok(if winner(cards) == Winner::Tie {
            SettlementStatus::Win
        } else {
            SettlementStatus::Lose
        }),
        BetType::PerfectPair | BetType::Monkey | BetType::Tiger => {
            Ok(if matched_outcomes.is_empty() {
                SettlementStatus::Lose
            } else {
                SettlementStatus::Win
            })
        }
        BetType::SuperTie0
        | BetType::SuperTie1
        | BetType::SuperTie2
        | BetType::SuperTie3
        | BetType::SuperTie4
        | BetType::SuperTie5
        | BetType::SuperTie6
        | BetType::SuperTie7
        | BetType::SuperTie8
        | BetType::SuperTie9 => Ok(
            if super_tie_total(bet_type)
                .map(|total| player_total(cards) == total && banker_total(cards) == total)
                .unwrap_or(false)
            {
                SettlementStatus::Win
            } else {
                SettlementStatus::Lose
            },
        ),
        _ => Err(format!("unsupported settlement bet type {bet_type:?}")),
    }
}

fn matched_outcomes(
    cards: &DealtHands,
    bet_type: BetType,
    mode: Option<BetMode>,
) -> Result<Vec<BetOutcome>, String> {
    match bet_type {
        BetType::PerfectPair => perfect_pair_outcomes(cards, mode),
        BetType::Monkey => monkey_outcomes(cards, mode),
        BetType::Tiger => tiger_outcomes(cards),
        _ => Ok(Vec::new()),
    }
}

fn perfect_pair_outcomes(
    cards: &DealtHands,
    mode: Option<BetMode>,
) -> Result<Vec<BetOutcome>, String> {
    let mode = match mode {
        Some(BetMode::PerfectPair(mode)) => mode,
        Some(mode) => return Err(format!("mode {mode:?} is incompatible with PerfectPair")),
        None => PerfectPairMode::Standard,
    };
    let player_pair = is_perfect_pair(&cards.player);
    let banker_pair = is_perfect_pair(&cards.banker);

    let outcome = match (player_pair, banker_pair) {
        (true, true) => Some(BetOutcome::PerfectPairBothSides),
        (true, false) | (false, true) => Some(BetOutcome::PerfectPairSingleSide),
        (false, false) => None,
    };

    Ok(match (mode, outcome) {
        (_, Some(outcome)) => vec![outcome],
        (_, None) => Vec::new(),
    })
}

fn monkey_outcomes(cards: &DealtHands, mode: Option<BetMode>) -> Result<Vec<BetOutcome>, String> {
    let mode = match mode {
        Some(BetMode::Monkey(mode)) => mode,
        Some(mode) => return Err(format!("mode {mode:?} is incompatible with Monkey")),
        None => MonkeyMode::Standard,
    };
    let Some(initial) = initial_four_cards(cards) else {
        return Ok(Vec::new());
    };

    let outcome = if initial.iter().all(|card| is_monkey_rank(card.rank)) {
        Some(BetOutcome::Monkey)
    } else if initial.iter().all(|card| !is_monkey_rank(card.rank)) {
        Some(BetOutcome::NoMonkey)
    } else {
        None
    };

    Ok(match (mode, outcome) {
        (MonkeyMode::Standard, Some(outcome)) => vec![outcome],
        (MonkeyMode::NoMonkeyOnly, Some(BetOutcome::NoMonkey)) => vec![BetOutcome::NoMonkey],
        _ => Vec::new(),
    })
}

fn tiger_outcomes(cards: &DealtHands) -> Result<Vec<BetOutcome>, String> {
    if banker_total(cards) != 6 || winner(cards) != Winner::Banker {
        return Ok(Vec::new());
    }

    Ok(match cards.banker.len() {
        2 => vec![BetOutcome::TigerTwoCards],
        3 => vec![BetOutcome::TigerThreeCards],
        _ => Vec::new(),
    })
}

fn applied_odds(
    spec: &SettlementSpec,
    matched_outcomes: &[BetOutcome],
) -> Result<Vec<ResolvedOdds>, String> {
    match &spec.odds {
        SettlementOddsSpec::Simple(odds) => Ok(vec![ResolvedOdds {
            outcome: matched_outcomes.first().copied(),
            odds: *odds,
        }]),
        SettlementOddsSpec::ByOutcome(outcomes) => matched_outcomes
            .iter()
            .map(|matched| {
                outcomes
                    .iter()
                    .find(|candidate| candidate.outcome == *matched)
                    .map(|candidate| ResolvedOdds {
                        outcome: Some(*matched),
                        odds: candidate.odds,
                    })
                    .ok_or_else(|| format!("settlement spec {} is missing outcome odds", spec.id))
            })
            .collect(),
    }
}

fn is_perfect_pair(cards: &[Card]) -> bool {
    cards
        .first()
        .zip(cards.get(1))
        .map(|(first, second)| first == second)
        .unwrap_or(false)
}

fn initial_four_cards(cards: &DealtHands) -> Option<[Card; 4]> {
    Some([
        *cards.player.first()?,
        *cards.player.get(1)?,
        *cards.banker.first()?,
        *cards.banker.get(1)?,
    ])
}

const fn is_monkey_rank(rank: CardRank) -> bool {
    matches!(rank, CardRank::Jack | CardRank::Queen | CardRank::King)
}

fn winner(cards: &DealtHands) -> Winner {
    match player_total(cards).cmp(&banker_total(cards)) {
        std::cmp::Ordering::Greater => Winner::Player,
        std::cmp::Ordering::Less => Winner::Banker,
        std::cmp::Ordering::Equal => Winner::Tie,
    }
}

fn player_total(cards: &DealtHands) -> u8 {
    hand_total(&cards.player)
}

fn banker_total(cards: &DealtHands) -> u8 {
    hand_total(&cards.banker)
}

fn hand_total(cards: &[Card]) -> u8 {
    cards
        .iter()
        .map(|card| card.rank.baccarat_value())
        .sum::<u8>()
        % 10
}

const fn super_tie_total(bet_type: BetType) -> Option<u8> {
    match bet_type {
        BetType::SuperTie0 => Some(0),
        BetType::SuperTie1 => Some(1),
        BetType::SuperTie2 => Some(2),
        BetType::SuperTie3 => Some(3),
        BetType::SuperTie4 => Some(4),
        BetType::SuperTie5 => Some(5),
        BetType::SuperTie6 => Some(6),
        BetType::SuperTie7 => Some(7),
        BetType::SuperTie8 => Some(8),
        BetType::SuperTie9 => Some(9),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CardSuit;

    fn card(rank: CardRank, suit: CardSuit) -> Card {
        Card { rank, suit }
    }

    fn player_win_cards() -> DealtHands {
        DealtHands {
            player: vec![
                card(CardRank::Nine, CardSuit::Clubs),
                card(CardRank::King, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::Seven, CardSuit::Diamonds),
                card(CardRank::King, CardSuit::Diamonds),
            ],
        }
    }

    fn banker_win_cards() -> DealtHands {
        DealtHands {
            player: vec![
                card(CardRank::Seven, CardSuit::Clubs),
                card(CardRank::King, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::Nine, CardSuit::Diamonds),
                card(CardRank::King, CardSuit::Diamonds),
            ],
        }
    }

    fn tie_cards() -> DealtHands {
        DealtHands {
            player: vec![
                card(CardRank::Eight, CardSuit::Clubs),
                card(CardRank::King, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::Eight, CardSuit::Diamonds),
                card(CardRank::King, CardSuit::Diamonds),
            ],
        }
    }

    fn single_odds_spec(
        id: &str,
        bet_type: BetType,
        amount: Decimal,
        odds: Decimal,
    ) -> SettlementSpec {
        SettlementSpec {
            id: String::from(id),
            bet_type,
            amount,
            mode: None,
            odds: SettlementOddsSpec::Simple(odds),
        }
    }

    fn assert_single_odds_result(
        result: &SettlementResult,
        status: SettlementStatus,
        profit: Decimal,
        payout: Decimal,
        applied_odds: Vec<ResolvedOdds>,
    ) {
        assert_eq!(result.status, status);
        assert_eq!(result.profit, profit);
        assert_eq!(result.payout, payout);
        assert_eq!(result.applied_odds, applied_odds);
    }

    #[test]
    fn player_win_settles_with_decimal_payout_and_profit() {
        let cards = player_win_cards();
        let spec = single_odds_spec(
            "player-1",
            BetType::Player,
            Decimal::new(1000, 2),
            Decimal::ONE,
        );

        let result = settle_bet(&cards, &spec).expect("Player settlement should succeed");

        assert_eq!(result.stake, Decimal::new(1000, 2));
        assert_single_odds_result(
            &result,
            SettlementStatus::Win,
            Decimal::new(1000, 2),
            Decimal::new(2000, 2),
            vec![ResolvedOdds {
                outcome: None,
                odds: Decimal::ONE,
            }],
        );
    }

    #[test]
    fn player_and_banker_push_on_tie_with_stake_returned() {
        let cards = tie_cards();

        for bet_type in [BetType::Player, BetType::Banker] {
            let spec = single_odds_spec("tie-push", bet_type, Decimal::new(2500, 2), Decimal::ONE);

            let result = settle_bet(&cards, &spec).expect("tie push settlement should succeed");

            assert_single_odds_result(
                &result,
                SettlementStatus::Push,
                Decimal::ZERO,
                Decimal::new(2500, 2),
                Vec::new(),
            );
        }
    }

    #[test]
    fn banker_and_tie_settle_win_and_lose_statuses_with_money() {
        let player_spec = single_odds_spec(
            "player-lose",
            BetType::Player,
            Decimal::new(2000, 2),
            Decimal::ONE,
        );
        let player_lose_result =
            settle_bet(&banker_win_cards(), &player_spec).expect("Player loss should settle");

        assert_single_odds_result(
            &player_lose_result,
            SettlementStatus::Lose,
            Decimal::new(-2000, 2),
            Decimal::ZERO,
            Vec::new(),
        );

        let banker_spec = single_odds_spec(
            "banker-1",
            BetType::Banker,
            Decimal::new(2000, 2),
            Decimal::new(95, 2),
        );
        let banker_result =
            settle_bet(&banker_win_cards(), &banker_spec).expect("Banker win should settle");

        assert_single_odds_result(
            &banker_result,
            SettlementStatus::Win,
            Decimal::new(1900, 2),
            Decimal::new(3900, 2),
            vec![ResolvedOdds {
                outcome: None,
                odds: Decimal::new(95, 2),
            }],
        );

        let banker_lose_result =
            settle_bet(&player_win_cards(), &banker_spec).expect("Banker loss should settle");

        assert_single_odds_result(
            &banker_lose_result,
            SettlementStatus::Lose,
            Decimal::new(-2000, 2),
            Decimal::ZERO,
            Vec::new(),
        );

        let tie_win_spec = single_odds_spec(
            "tie-1",
            BetType::Tie,
            Decimal::new(1000, 2),
            Decimal::new(800, 2),
        );
        let tie_win_result =
            settle_bet(&tie_cards(), &tie_win_spec).expect("Tie win should settle");

        assert_single_odds_result(
            &tie_win_result,
            SettlementStatus::Win,
            Decimal::new(8000, 2),
            Decimal::new(9000, 2),
            vec![ResolvedOdds {
                outcome: None,
                odds: Decimal::new(800, 2),
            }],
        );

        let tie_lose_result =
            settle_bet(&player_win_cards(), &tie_win_spec).expect("Tie loss should settle");

        assert_single_odds_result(
            &tie_lose_result,
            SettlementStatus::Lose,
            Decimal::new(-1000, 2),
            Decimal::ZERO,
            Vec::new(),
        );
    }

    #[test]
    fn perfect_pair_both_sides_uses_only_both_sides_outcome_odds() {
        let cards = DealtHands {
            player: vec![
                card(CardRank::Ace, CardSuit::Clubs),
                card(CardRank::Ace, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::King, CardSuit::Spades),
                card(CardRank::King, CardSuit::Spades),
            ],
        };
        let spec = SettlementSpec {
            id: String::from("perfect-pair-1"),
            bet_type: BetType::PerfectPair,
            amount: Decimal::new(500, 2),
            mode: Some(BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth)),
            odds: SettlementOddsSpec::ByOutcome(vec![
                SettlementOutcomeOdds {
                    outcome: BetOutcome::PerfectPairSingleSide,
                    odds: Decimal::new(2500, 2),
                },
                SettlementOutcomeOdds {
                    outcome: BetOutcome::PerfectPairBothSides,
                    odds: Decimal::new(20000, 2),
                },
            ]),
        };

        let result = settle_bet(&cards, &spec).expect("PerfectPair settlement should succeed");

        assert_eq!(result.status, SettlementStatus::Win);
        assert_eq!(
            result.matched_outcomes,
            vec![BetOutcome::PerfectPairBothSides]
        );
        assert_eq!(
            result.applied_odds,
            vec![ResolvedOdds {
                outcome: Some(BetOutcome::PerfectPairBothSides),
                odds: Decimal::new(20000, 2),
            }]
        );
        assert_eq!(result.profit, Decimal::new(100000, 2));
        assert_eq!(result.payout, Decimal::new(100500, 2));
    }

    #[test]
    fn monkey_standard_settles_monkey_and_no_monkey_with_separate_odds() {
        let monkey_cards = DealtHands {
            player: vec![
                card(CardRank::Jack, CardSuit::Clubs),
                card(CardRank::Queen, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::King, CardSuit::Diamonds),
                card(CardRank::Jack, CardSuit::Diamonds),
            ],
        };
        let no_monkey_cards = DealtHands {
            player: vec![
                card(CardRank::Ace, CardSuit::Clubs),
                card(CardRank::Two, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::Three, CardSuit::Diamonds),
                card(CardRank::Four, CardSuit::Diamonds),
            ],
        };
        let spec = SettlementSpec {
            id: String::from("monkey-standard"),
            bet_type: BetType::Monkey,
            amount: Decimal::new(300, 2),
            mode: Some(BetMode::Monkey(MonkeyMode::Standard)),
            odds: SettlementOddsSpec::ByOutcome(vec![
                SettlementOutcomeOdds {
                    outcome: BetOutcome::Monkey,
                    odds: Decimal::new(5000, 2),
                },
                SettlementOutcomeOdds {
                    outcome: BetOutcome::NoMonkey,
                    odds: Decimal::ONE,
                },
            ]),
        };

        let monkey_result = settle_bet(&monkey_cards, &spec).expect("Monkey should win");
        let no_monkey_result = settle_bet(&no_monkey_cards, &spec).expect("NoMonkey should win");

        assert_eq!(monkey_result.matched_outcomes, vec![BetOutcome::Monkey]);
        assert_single_odds_result(
            &monkey_result,
            SettlementStatus::Win,
            Decimal::new(15000, 2),
            Decimal::new(15300, 2),
            vec![ResolvedOdds {
                outcome: Some(BetOutcome::Monkey),
                odds: Decimal::new(5000, 2),
            }],
        );
        assert_eq!(
            no_monkey_result.matched_outcomes,
            vec![BetOutcome::NoMonkey]
        );
        assert_single_odds_result(
            &no_monkey_result,
            SettlementStatus::Win,
            Decimal::new(300, 2),
            Decimal::new(600, 2),
            vec![ResolvedOdds {
                outcome: Some(BetOutcome::NoMonkey),
                odds: Decimal::ONE,
            }],
        );
    }

    #[test]
    fn monkey_no_monkey_only_wins_only_no_monkey() {
        let monkey_cards = DealtHands {
            player: vec![
                card(CardRank::Jack, CardSuit::Clubs),
                card(CardRank::Queen, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::King, CardSuit::Diamonds),
                card(CardRank::Jack, CardSuit::Diamonds),
            ],
        };
        let no_monkey_cards = DealtHands {
            player: vec![
                card(CardRank::Ace, CardSuit::Clubs),
                card(CardRank::Two, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::Three, CardSuit::Diamonds),
                card(CardRank::Four, CardSuit::Diamonds),
            ],
        };
        let spec = SettlementSpec {
            id: String::from("no-monkey-only"),
            bet_type: BetType::Monkey,
            amount: Decimal::new(700, 2),
            mode: Some(BetMode::Monkey(MonkeyMode::NoMonkeyOnly)),
            odds: SettlementOddsSpec::ByOutcome(vec![SettlementOutcomeOdds {
                outcome: BetOutcome::NoMonkey,
                odds: Decimal::ONE,
            }]),
        };

        let monkey_result = settle_bet(&monkey_cards, &spec).expect("Monkey should lose");
        let no_monkey_result = settle_bet(&no_monkey_cards, &spec).expect("NoMonkey should win");

        assert_single_odds_result(
            &monkey_result,
            SettlementStatus::Lose,
            Decimal::new(-700, 2),
            Decimal::ZERO,
            Vec::new(),
        );
        assert_eq!(
            no_monkey_result.matched_outcomes,
            vec![BetOutcome::NoMonkey]
        );
        assert_single_odds_result(
            &no_monkey_result,
            SettlementStatus::Win,
            Decimal::new(700, 2),
            Decimal::new(1400, 2),
            vec![ResolvedOdds {
                outcome: Some(BetOutcome::NoMonkey),
                odds: Decimal::ONE,
            }],
        );
    }

    #[test]
    fn tiger_settles_two_and_three_card_outcome_odds() {
        let tiger_two_cards = DealtHands {
            player: vec![
                card(CardRank::Five, CardSuit::Clubs),
                card(CardRank::King, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::Six, CardSuit::Diamonds),
                card(CardRank::King, CardSuit::Diamonds),
            ],
        };
        let tiger_three_cards = DealtHands {
            player: vec![
                card(CardRank::Five, CardSuit::Clubs),
                card(CardRank::King, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::Two, CardSuit::Diamonds),
                card(CardRank::Two, CardSuit::Spades),
                card(CardRank::Two, CardSuit::Hearts),
            ],
        };
        let spec = SettlementSpec {
            id: String::from("tiger"),
            bet_type: BetType::Tiger,
            amount: Decimal::new(400, 2),
            mode: None,
            odds: SettlementOddsSpec::ByOutcome(vec![
                SettlementOutcomeOdds {
                    outcome: BetOutcome::TigerTwoCards,
                    odds: Decimal::new(1200, 2),
                },
                SettlementOutcomeOdds {
                    outcome: BetOutcome::TigerThreeCards,
                    odds: Decimal::new(2000, 2),
                },
            ]),
        };

        let tiger_two_result = settle_bet(&tiger_two_cards, &spec).expect("Tiger two should win");
        let tiger_three_result =
            settle_bet(&tiger_three_cards, &spec).expect("Tiger three should win");
        let lose_result = settle_bet(&player_win_cards(), &spec).expect("Tiger loss should settle");

        assert_eq!(
            tiger_two_result.matched_outcomes,
            vec![BetOutcome::TigerTwoCards]
        );
        assert_single_odds_result(
            &tiger_two_result,
            SettlementStatus::Win,
            Decimal::new(4800, 2),
            Decimal::new(5200, 2),
            vec![ResolvedOdds {
                outcome: Some(BetOutcome::TigerTwoCards),
                odds: Decimal::new(1200, 2),
            }],
        );
        assert_eq!(
            tiger_three_result.matched_outcomes,
            vec![BetOutcome::TigerThreeCards]
        );
        assert_single_odds_result(
            &tiger_three_result,
            SettlementStatus::Win,
            Decimal::new(8000, 2),
            Decimal::new(8400, 2),
            vec![ResolvedOdds {
                outcome: Some(BetOutcome::TigerThreeCards),
                odds: Decimal::new(2000, 2),
            }],
        );
        assert_single_odds_result(
            &lose_result,
            SettlementStatus::Lose,
            Decimal::new(-400, 2),
            Decimal::ZERO,
            Vec::new(),
        );
    }

    #[test]
    fn super_tie_zero_through_nine_settle_exact_tie_totals() {
        let cases = [
            (BetType::SuperTie0, CardRank::Ten, Decimal::new(15000, 2)),
            (BetType::SuperTie1, CardRank::Ace, Decimal::new(21500, 2)),
            (BetType::SuperTie2, CardRank::Two, Decimal::new(22000, 2)),
            (BetType::SuperTie3, CardRank::Three, Decimal::new(20000, 2)),
            (BetType::SuperTie4, CardRank::Four, Decimal::new(12000, 2)),
            (BetType::SuperTie5, CardRank::Five, Decimal::new(11000, 2)),
            (BetType::SuperTie6, CardRank::Six, Decimal::new(4500, 2)),
            (BetType::SuperTie7, CardRank::Seven, Decimal::new(4500, 2)),
            (BetType::SuperTie8, CardRank::Eight, Decimal::new(8000, 2)),
            (BetType::SuperTie9, CardRank::Nine, Decimal::new(8000, 2)),
        ];

        for (bet_type, total_rank, odds) in cases {
            let cards = DealtHands {
                player: vec![
                    card(total_rank, CardSuit::Clubs),
                    card(CardRank::King, CardSuit::Clubs),
                ],
                banker: vec![
                    card(total_rank, CardSuit::Diamonds),
                    card(CardRank::King, CardSuit::Diamonds),
                ],
            };
            let spec = single_odds_spec("super-tie", bet_type, Decimal::new(200, 2), odds);

            let result = settle_bet(&cards, &spec).expect("SuperTie should settle");

            assert_single_odds_result(
                &result,
                SettlementStatus::Win,
                Decimal::new(200, 2) * odds,
                Decimal::new(200, 2) + Decimal::new(200, 2) * odds,
                vec![ResolvedOdds {
                    outcome: None,
                    odds,
                }],
            );
        }

        let lose_spec = single_odds_spec(
            "super-tie-lose",
            BetType::SuperTie9,
            Decimal::new(200, 2),
            Decimal::new(8000, 2),
        );
        let lose_result =
            settle_bet(&tie_cards(), &lose_spec).expect("SuperTie loss should settle");

        assert_single_odds_result(
            &lose_result,
            SettlementStatus::Lose,
            Decimal::new(-200, 2),
            Decimal::ZERO,
            Vec::new(),
        );
    }

    #[test]
    fn settlement_rejects_zero_amount() {
        let spec = SettlementSpec {
            id: String::from("zero-amount"),
            bet_type: BetType::Player,
            amount: Decimal::ZERO,
            mode: None,
            odds: SettlementOddsSpec::Simple(Decimal::ONE),
        };

        let err = settle_bet(&player_win_cards(), &spec).expect_err("zero amount should fail");

        assert!(err.contains("greater than zero"));
    }

    #[test]
    fn settlement_rejects_amount_with_more_than_two_decimal_places() {
        let spec = SettlementSpec {
            id: String::from("amount-scale"),
            bet_type: BetType::Player,
            amount: Decimal::new(1001, 3),
            mode: None,
            odds: SettlementOddsSpec::Simple(Decimal::ONE),
        };

        let err = settle_bet(&player_win_cards(), &spec).expect_err("amount scale should fail");

        assert!(err.contains("amount must have at most two decimal places"));
    }

    #[test]
    fn settlement_rejects_odds_with_more_than_two_decimal_places() {
        let spec = SettlementSpec {
            id: String::from("odds-scale"),
            bet_type: BetType::Player,
            amount: Decimal::new(1000, 2),
            mode: None,
            odds: SettlementOddsSpec::Simple(Decimal::new(1234, 3)),
        };

        let err = settle_bet(&player_win_cards(), &spec).expect_err("odds scale should fail");

        assert!(err.contains("odds must have at most two decimal places"));
    }

    #[test]
    fn settlement_rejects_missing_required_outcome_odds() {
        let cards = DealtHands {
            player: vec![
                card(CardRank::Ace, CardSuit::Clubs),
                card(CardRank::Ace, CardSuit::Clubs),
            ],
            banker: vec![
                card(CardRank::King, CardSuit::Spades),
                card(CardRank::King, CardSuit::Spades),
            ],
        };
        let spec = SettlementSpec {
            id: String::from("missing-outcome"),
            bet_type: BetType::PerfectPair,
            amount: Decimal::new(500, 2),
            mode: Some(BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth)),
            odds: SettlementOddsSpec::ByOutcome(vec![SettlementOutcomeOdds {
                outcome: BetOutcome::PerfectPairSingleSide,
                odds: Decimal::new(2500, 2),
            }]),
        };

        let err = settle_bet(&cards, &spec).expect_err("missing outcome odds should fail");

        assert!(err.contains("missing outcome odds"));
    }

    #[test]
    fn settlement_rejects_irrelevant_outcome_odds() {
        let spec = SettlementSpec {
            id: String::from("irrelevant-outcome"),
            bet_type: BetType::Monkey,
            amount: Decimal::new(500, 2),
            mode: Some(BetMode::Monkey(MonkeyMode::NoMonkeyOnly)),
            odds: SettlementOddsSpec::ByOutcome(vec![
                SettlementOutcomeOdds {
                    outcome: BetOutcome::NoMonkey,
                    odds: Decimal::ONE,
                },
                SettlementOutcomeOdds {
                    outcome: BetOutcome::Monkey,
                    odds: Decimal::new(5000, 2),
                },
            ]),
        };

        let err = settle_bet(&player_win_cards(), &spec).expect_err("irrelevant odds should fail");

        assert!(err.contains("irrelevant outcome odds"));
    }

    #[test]
    fn settlement_rejects_incompatible_mode() {
        let spec = SettlementSpec {
            id: String::from("bad-mode"),
            bet_type: BetType::PerfectPair,
            amount: Decimal::new(500, 2),
            mode: Some(BetMode::Monkey(MonkeyMode::Standard)),
            odds: SettlementOddsSpec::Simple(Decimal::new(2500, 2)),
        };

        let err = settle_bet(&player_win_cards(), &spec).expect_err("bad mode should fail");

        assert!(err.contains("incompatible with PerfectPair"));
    }

    #[test]
    fn settlement_rejects_duplicate_outcome_odds() {
        let spec = SettlementSpec {
            id: String::from("duplicate-outcome"),
            bet_type: BetType::Monkey,
            amount: Decimal::new(500, 2),
            mode: Some(BetMode::Monkey(MonkeyMode::Standard)),
            odds: SettlementOddsSpec::ByOutcome(vec![
                SettlementOutcomeOdds {
                    outcome: BetOutcome::NoMonkey,
                    odds: Decimal::ONE,
                },
                SettlementOutcomeOdds {
                    outcome: BetOutcome::NoMonkey,
                    odds: Decimal::new(200, 2),
                },
            ]),
        };

        let err = settle_bet(&player_win_cards(), &spec).expect_err("duplicate odds should fail");

        assert!(err.contains("duplicate outcome odds"));
    }

    #[test]
    fn settlement_rounds_profit_and_payout_to_two_decimal_places() {
        let spec = SettlementSpec {
            id: String::from("rounding"),
            bet_type: BetType::Player,
            amount: Decimal::new(1001, 2),
            mode: None,
            odds: SettlementOddsSpec::Simple(Decimal::new(123, 2)),
        };

        let result =
            settle_bet(&player_win_cards(), &spec).expect("rounding settlement should pass");

        assert_eq!(result.profit, Decimal::new(1231, 2));
        assert_eq!(result.payout, Decimal::new(2232, 2));
    }
}
