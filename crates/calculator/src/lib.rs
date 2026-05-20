//! Pure baccarat probability, EV, odds, and settlement contracts.
//!
//! `calculator` exposes card-count inputs and public bet outputs only. It does
//! not model server requests, strategies, tables, sessions, or persistence.
//!
//! `BetType` is the caller-facing public bet identifier returned by probability
//! calculation and used by EV and settlement inputs. `BetId` remains the
//! registry identifier used when a public bet needs variant-level precision.

pub mod bet_registry;
mod ev;
mod mode_contract;
pub mod odds;
mod probability;
pub mod settlement;
mod shoe;
mod terminal;

use serde::{Deserialize, Serialize};

/// Baccarat card input types owned by the shared `types` crate and re-exported
/// for calculator callers.
pub use types::baccarat::{Card, CardCount, CardRank, CardSuit};

/// Public bet registry types and variant identifiers.
///
/// Use `BetType` for caller-facing calculator rows. Use `BetId` when working
/// with registry entries or variant odds that need more precision than one
/// public row.
pub use bet_registry::{
    bet_definitions, public_probability_definitions, BetClass, BetDefinition, BetId, BetType,
    BetVariant, CharSiuVariant, DragonVariant, Flame7sVariant, Fortune4PairVariant, Heaven9Variant,
    Lucky6Variant, Lucky7Variant, NaturalVariant, SuperLucky7Variant, TigerPairVariant,
    TigerVariant,
};

/// Per-bet EV calculation entrypoint, request specs, and result rows.
///
/// `calculate_ev` preserves the order of `PerBetEvCalculationSpec` inputs.
pub use ev::{
    calculate_ev, EffectiveAmountMode, PerBetEvCalculationResult, PerBetEvCalculationSpec,
};

/// Odds specs and default odds table used by EV callers.
///
/// Default `PerfectPair` odds are single-side only at 25. Platforms that pay a
/// separate both-sides outcome, such as 200, must pass explicit outcome odds
/// with `PerfectPairMode::SinglePlusBoth`.
pub use odds::{
    default_odds_specs, default_odds_table, OddsSettlement, OddsSpec, OddsTable, OutcomeOdds,
};

/// Probability calculation entrypoint and public probability result rows.
///
/// `calculate_probabilities` returns every registered canonical public bet for
/// the supplied card counts. It does not accept caller-selected bet lists.
pub use probability::{
    calculate_probabilities, BetProbabilityResult, BetVariantProbability, OutcomeProbability,
    ProbabilityCalculationResult,
};

/// Settlement entrypoints, input specs, and Decimal money results.
pub use settlement::{
    settle_bet, settle_bets, AppliedOutcomeOdds, BetSettlementResult, BetSettlementSpec,
    SettledCards, SettlementOddsSpec, SettlementOutcomeOdds, SettlementStatus,
};

/// Creates a full eight-deck baccarat shoe as `CardCount` entries.
pub use shoe::standard_eight_deck_cards;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Public outcome bucket used when one public `BetType` has multiple EV or
/// settlement branches.
///
/// Outcome buckets do not replace `BetType`. They refine odds and result
/// handling for bets such as `Monkey`, `PerfectPair`, `Tiger`, and `Lucky6`.
pub enum BetOutcome {
    /// Banker wins with six using two banker cards for `BetType::Tiger`.
    TigerTwoCards,
    /// Banker wins with six using three banker cards for `BetType::Tiger`.
    TigerThreeCards,
    /// Banker wins with six using two banker cards for `BetType::Lucky6`.
    Lucky6TwoCards,
    /// Banker wins with six using three banker cards for `BetType::Lucky6`.
    Lucky6ThreeCards,
    /// Opening four cards are all face cards for `BetType::Monkey`.
    Monkey,
    /// Opening four cards are all non-face cards for `BetType::Monkey`.
    NoMonkey,
    /// Exactly one side opens with two identical cards for `PerfectPair`.
    PerfectPairSingleSide,
    /// Player and banker both open with two identical cards for `PerfectPair`.
    PerfectPairBothSides,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// EV or settlement interpretation for `BetType::PerfectPair`.
pub enum PerfectPairMode {
    /// Single-side contract only.
    ///
    /// Default `PerfectPair` odds use this mode with 25 odds. A both-sides hit
    /// is treated through the single-side contract unless callers explicitly
    /// request `SinglePlusBoth`.
    Standard,
    /// Separate single-side and both-sides contracts.
    ///
    /// Use this platform exception for both-sides odds such as 200 by supplying
    /// explicit outcome odds for `PerfectPairSingleSide` and
    /// `PerfectPairBothSides`.
    SinglePlusBoth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// EV or settlement interpretation for `BetType::Monkey`.
pub enum MonkeyMode {
    /// Pay either all-face-card `Monkey` or all-non-face-card `NoMonkey` when
    /// the supplied outcome odds include each branch.
    Standard,
    /// Pay only the all-non-face-card `NoMonkey` branch.
    NoMonkeyOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Optional EV or settlement interpretation for bets with outcome branches.
///
/// `BetMode` changes how EV and settlement choose outcome odds. It does not
/// change objective probability output from `calculate_probabilities`.
pub enum BetMode {
    /// PerfectPair branch handling.
    PerfectPair(PerfectPairMode),
    /// Monkey branch handling.
    Monkey(MonkeyMode),
}

/// Standard baccarat contract frozen before the calculator core is implemented.
pub mod standard_baccarat {
    pub use types::baccarat::{
        CARDS_PER_DECK, RANKS_PER_DECK, STANDARD_DECK_COUNT, STANDARD_SHOE_CARD_COUNT,
        SUITS_PER_DECK,
    };

    pub const fn is_natural_total(total: u8) -> bool {
        matches!(total % 10, 8 | 9)
    }

    pub const fn player_draws_third_card(player_initial_total: u8, natural_present: bool) -> bool {
        !natural_present && player_initial_total % 10 <= 5
    }

    pub const fn banker_draws_third_card(
        banker_initial_total: u8,
        player_third_card_value: Option<u8>,
        natural_present: bool,
    ) -> bool {
        if natural_present {
            return false;
        }

        let banker_total = banker_initial_total % 10;
        match player_third_card_value {
            Option::None => banker_total <= 5,
            Some(player_third) => match banker_total {
                0..=2 => true,
                3 => player_third % 10 != 8,
                4 => matches!(player_third % 10, 2..=7),
                5 => matches!(player_third % 10, 4..=7),
                6 => matches!(player_third % 10, 6 | 7),
                _ => false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probability::{
        both_perfect_pair_orders, calculate_probability_snapshot, falling4_from_count, ordered4,
        outcome_probability_breakdown, probability_for_definition, sum_falling2,
        OutcomeProbabilityBreakdown, ProbabilitySnapshot,
    };
    use crate::shoe::ShoeCounts;
    use crate::terminal::{
        terminal_outcome_from_ordered_points, terminal_predicate_matches, TerminalOutcome,
        TerminalWinner,
    };

    const OUTCOME_TOLERANCE: f64 = 1e-12;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= OUTCOME_TOLERANCE,
            "expected {actual:.15} to be within {OUTCOME_TOLERANCE} of {expected:.15}"
        );
    }

    fn standard_probabilities() -> ProbabilitySnapshot {
        let counts = ShoeCounts::from_cards(&standard_eight_deck_cards())
            .expect("standard cards should parse");

        calculate_probability_snapshot(counts).expect("standard cards should calculate")
    }

    fn breakdown_for(
        bet_id: BetId,
        probabilities: &ProbabilitySnapshot,
    ) -> OutcomeProbabilityBreakdown {
        let definition = bet_definitions()
            .iter()
            .find(|definition| definition.id == bet_id)
            .unwrap_or_else(|| panic!("missing bet definition for {bet_id:?}"));

        outcome_probability_breakdown(definition, probabilities)
            .expect("test fixture bet definition should have probability coverage")
    }

    #[test]
    fn standard_player_and_banker_treat_tie_as_push() {
        let probabilities = standard_probabilities();

        let player = breakdown_for(BetId::Player, &probabilities);
        assert_close(player.win_probability, probabilities.standard.player);
        assert_close(player.push_probability, probabilities.standard.tie);
        assert_close(player.lose_probability, probabilities.standard.banker);

        let banker = breakdown_for(BetId::Banker, &probabilities);
        assert_close(banker.win_probability, probabilities.standard.banker);
        assert_close(banker.push_probability, probabilities.standard.tie);
        assert_close(banker.lose_probability, probabilities.standard.player);
    }

    #[test]
    fn aggregate_push_variants_contribute_to_push_probability() {
        let probabilities = standard_probabilities();

        let player_dragon = breakdown_for(BetId::PlayerDragonAggregate, &probabilities);
        let player_dragon_push = breakdown_for(BetId::PlayerDragonPush, &probabilities);
        assert!(player_dragon_push.push_probability > 0.0);
        assert_close(player_dragon_push.win_probability, 0.0);
        assert_close(
            player_dragon.push_probability,
            player_dragon_push.push_probability,
        );
        assert_close(
            player_dragon.win_probability
                + player_dragon.lose_probability
                + player_dragon.push_probability,
            1.0,
        );

        let banker_natural = breakdown_for(BetId::BankerNaturalAggregate, &probabilities);
        let banker_natural_push = breakdown_for(BetId::BankerNaturalPush, &probabilities);
        assert!(banker_natural_push.push_probability > 0.0);
        assert_close(banker_natural_push.win_probability, 0.0);
        assert_close(
            banker_natural.push_probability,
            banker_natural_push.push_probability,
        );
        assert_close(
            banker_natural.win_probability
                + banker_natural.lose_probability
                + banker_natural.push_probability,
            1.0,
        );
    }

    #[test]
    fn ev_outcome_decomposition_sums_to_one_for_every_public_result() {
        let probabilities = standard_probabilities();

        for definition in public_probability_definitions() {
            let breakdown = outcome_probability_breakdown(definition, &probabilities)
                .expect("public definition should have probability coverage");

            assert!(breakdown.win_probability.is_finite());
            assert!(breakdown.lose_probability.is_finite());
            assert!(breakdown.push_probability.is_finite());
            assert_close(
                breakdown.win_probability + breakdown.lose_probability + breakdown.push_probability,
                1.0,
            );
        }
    }

    #[test]
    fn registered_public_probability_and_ev_branches_do_not_return_unsupported_contracts() {
        let probabilities = standard_probabilities();

        for definition in public_probability_definitions() {
            let probability = probability_for_definition(definition, &probabilities)
                .unwrap_or_else(|error| panic!("{:?} probability failed: {error}", definition.id));
            let breakdown = outcome_probability_breakdown(definition, &probabilities)
                .unwrap_or_else(|error| panic!("{:?} EV failed: {error}", definition.id));

            assert!(probability.probability.is_finite());
            assert!(breakdown.win_probability.is_finite());
            assert!(breakdown.lose_probability.is_finite());
            assert!(breakdown.push_probability.is_finite());
        }
    }

    #[test]
    fn super_tie_public_rows_exist_and_match_exact_tie_totals() {
        let expected = [
            (BetId::SuperTie0, BetType::SuperTie0, 0),
            (BetId::SuperTie1, BetType::SuperTie1, 1),
            (BetId::SuperTie2, BetType::SuperTie2, 2),
            (BetId::SuperTie3, BetType::SuperTie3, 3),
            (BetId::SuperTie4, BetType::SuperTie4, 4),
            (BetId::SuperTie5, BetType::SuperTie5, 5),
            (BetId::SuperTie6, BetType::SuperTie6, 6),
            (BetId::SuperTie7, BetType::SuperTie7, 7),
            (BetId::SuperTie8, BetType::SuperTie8, 8),
            (BetId::SuperTie9, BetType::SuperTie9, 9),
        ];
        let result = calculate_probabilities(&standard_eight_deck_cards())
            .expect("standard SuperTie probabilities should calculate");

        for (bet_id, bet_type, total) in expected {
            let row = result
                .bets
                .iter()
                .find(|row| row.bet_type == bet_type)
                .unwrap_or_else(|| panic!("missing public SuperTie row for {bet_type:?}"));

            assert!(row.probability.is_finite());
            assert!(row.probability > 0.0);
            assert!(row.variants.is_empty());
            assert!(row.outcomes.is_empty());

            let definition = bet_definitions()
                .iter()
                .find(|definition| definition.id == bet_id)
                .unwrap_or_else(|| panic!("missing SuperTie definition for {bet_id:?}"));
            let matching_tie = TerminalOutcome::from_totals(total, total, 2, 2, false);
            let wrong_total_tie =
                TerminalOutcome::from_totals((total + 1) % 10, (total + 1) % 10, 2, 2, false);
            let non_tie = TerminalOutcome::from_totals(total, (total + 1) % 10, 2, 2, false);

            assert!(terminal_predicate_matches(definition, matching_tie));
            assert!(!terminal_predicate_matches(definition, wrong_total_tie));
            assert!(!terminal_predicate_matches(definition, non_tie));
        }
    }

    #[test]
    fn calculate_probabilities_returns_monkey_outcome_buckets() {
        let result = calculate_probabilities(&standard_eight_deck_cards())
            .expect("standard cards should calculate");
        let monkey = result
            .bets
            .iter()
            .find(|bet| bet.bet_type == BetType::Monkey)
            .expect("Monkey should be a public probability result");

        assert!(monkey.variants.is_empty());
        assert_eq!(monkey.outcomes.len(), 2);

        let monkey_probability = monkey
            .outcomes
            .iter()
            .find(|outcome| outcome.outcome == BetOutcome::Monkey)
            .expect("Monkey outcome should be present")
            .probability;
        let no_monkey_probability = monkey
            .outcomes
            .iter()
            .find(|outcome| outcome.outcome == BetOutcome::NoMonkey)
            .expect("NoMonkey outcome should be present")
            .probability;

        assert_close(
            monkey.probability,
            monkey_probability + no_monkey_probability,
        );
        assert_close(
            monkey_probability,
            falling4_from_count(96) as f64 / ordered4(416) as f64,
        );
        assert_close(
            no_monkey_probability,
            falling4_from_count(320) as f64 / ordered4(416) as f64,
        );
    }

    #[test]
    fn calculate_probabilities_returns_perfect_pair_mutually_exclusive_outcome_buckets() {
        let result = calculate_probabilities(&standard_eight_deck_cards())
            .expect("standard cards should calculate");
        let perfect_pair = result
            .bets
            .iter()
            .find(|bet| bet.bet_type == BetType::PerfectPair)
            .expect("PerfectPair should be a public probability result");

        assert!(perfect_pair.variants.is_empty());
        assert_eq!(perfect_pair.outcomes.len(), 2);

        let single_side_probability = perfect_pair
            .outcomes
            .iter()
            .find(|outcome| outcome.outcome == BetOutcome::PerfectPairSingleSide)
            .expect("PerfectPairSingleSide outcome should be present")
            .probability;
        let both_sides_probability = perfect_pair
            .outcomes
            .iter()
            .find(|outcome| outcome.outcome == BetOutcome::PerfectPairBothSides)
            .expect("PerfectPairBothSides outcome should be present")
            .probability;
        let card_counts = ShoeCounts::from_cards(&standard_eight_deck_cards())
            .expect("standard card counts should parse");
        let pair_orders = sum_falling2(&card_counts.card);
        let both_orders = both_perfect_pair_orders(&card_counts.card);
        let denominator = ordered4(card_counts.total);

        assert_close(
            perfect_pair.probability,
            single_side_probability + both_sides_probability,
        );
        assert_close(
            single_side_probability,
            (2 * pair_orders * 414 * 413 - 2 * both_orders) as f64 / denominator as f64,
        );
        assert_close(
            both_sides_probability,
            both_orders as f64 / denominator as f64,
        );
    }

    #[test]
    fn unsupported_opening_probability_branch_returns_contract_error() {
        let probabilities = standard_probabilities();
        let unsupported = BetDefinition {
            id: BetId::Dragon7,
            class: BetClass::OpeningTwoCombinator,
            suit_dependent: false,
            variant: None,
        };

        let error = probability_for_definition(&unsupported, &probabilities)
            .expect_err("unsupported opening branch should fail explicitly");

        assert!(error.contains("unsupported calculator contract"));
        assert!(error.contains("without probability coverage"));
    }

    #[test]
    fn unsupported_aggregate_ev_branch_returns_contract_error() {
        let probabilities = standard_probabilities();
        let unsupported = BetDefinition {
            id: BetId::AnyPair,
            class: BetClass::AggregateBet,
            suit_dependent: false,
            variant: None,
        };

        let error = outcome_probability_breakdown(&unsupported, &probabilities)
            .expect_err("unsupported aggregate branch should fail explicitly");

        assert!(error.contains("unsupported calculator contract"));
        assert!(error.contains("without probability/EV coverage"));
    }

    #[test]
    fn shoe_counts_derive_rank_point_and_card_sources() {
        let card_counts = ShoeCounts::from_cards(&standard_eight_deck_cards())
            .expect("standard card counts should parse");

        assert_eq!(
            card_counts.total,
            standard_baccarat::STANDARD_SHOE_CARD_COUNT
        );
        assert_eq!(
            card_counts.rank[0],
            u16::from(standard_baccarat::STANDARD_DECK_COUNT)
                * u16::from(standard_baccarat::SUITS_PER_DECK)
        );
        assert_eq!(card_counts.point[0], 128);
        assert_eq!(
            card_counts.card[0],
            u16::from(standard_baccarat::STANDARD_DECK_COUNT)
        );
    }

    #[test]
    fn terminal_engine_handles_natural_and_third_card_boundaries() {
        let natural_stop = terminal_outcome_from_ordered_points(&[8, 7, 1, 2])
            .expect("naturals should stop at four cards");
        assert!(natural_stop.natural);
        assert_eq!(natural_stop.total_card_count, 4);

        let banker_three_draws_on_player_third_seven =
            terminal_outcome_from_ordered_points(&[2, 3, 3, 3, 7, 2])
                .expect("banker 3 should draw on player third-card 7");
        assert_eq!(banker_three_draws_on_player_third_seven.banker_len, 3);
    }

    #[test]
    fn registry_definitions_map_to_public_bet_types() {
        for definition in bet_definitions() {
            assert_eq!(definition.bet_type(), definition.id.bet_type());
        }
    }

    #[test]
    fn terminal_predicates_match_representative_cases() {
        let cases = [
            (
                BetId::Dragon7,
                TerminalOutcome {
                    player_total: 0,
                    banker_total: 7,
                    player_len: 2,
                    banker_len: 3,
                    natural: false,
                    winner: TerminalWinner::Banker,
                    margin: 7,
                    total_card_count: 5,
                },
            ),
            (
                BetId::Panda8,
                TerminalOutcome {
                    player_total: 8,
                    banker_total: 0,
                    player_len: 3,
                    banker_len: 2,
                    natural: false,
                    winner: TerminalWinner::Player,
                    margin: 8,
                    total_card_count: 5,
                },
            ),
        ];

        for (id, outcome) in cases {
            let definition = bet_definitions()
                .iter()
                .find(|definition| definition.id == id)
                .expect("definition should exist");
            assert!(terminal_predicate_matches(definition, outcome));
        }
    }

    fn ev_spec(id: &str, bet_type: BetType, odds: f64) -> PerBetEvCalculationSpec {
        PerBetEvCalculationSpec {
            id: id.to_owned(),
            bet_type,
            mode: None,
            odds: OddsSpec::simple(bet_type, odds),
            rebate_rate: 0.01,
            effective_mode: EffectiveAmountMode::NonRefund,
        }
    }

    fn outcome_ev_spec(
        id: &str,
        bet_type: BetType,
        mode: BetMode,
        odds: OddsSpec,
    ) -> PerBetEvCalculationSpec {
        PerBetEvCalculationSpec {
            id: id.to_owned(),
            bet_type,
            mode: Some(mode),
            odds,
            rebate_rate: 0.0,
            effective_mode: EffectiveAmountMode::TotalStake,
        }
    }

    fn assert_probability_dependent_fields_match(
        actual: &PerBetEvCalculationResult,
        expected: &PerBetEvCalculationResult,
    ) {
        assert_close(actual.effective_probability, expected.effective_probability);
        assert_close(actual.win_probability, expected.win_probability);
        assert_close(actual.lose_probability, expected.lose_probability);
        assert_close(actual.push_probability, expected.push_probability);
    }

    #[test]
    fn ev_per_bet_specs_return_results_in_input_order_and_reuse_probability_fields() {
        let result = calculate_ev(
            &standard_eight_deck_cards(),
            &[
                ev_spec("player-default", BetType::Player, 1.0),
                ev_spec("banker", BetType::Banker, 0.95),
                ev_spec("player-override", BetType::Player, 2.0),
            ],
        )
        .expect("per-bet EV should calculate");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, "player-default");
        assert_eq!(result[1].id, "banker");
        assert_eq!(result[2].id, "player-override");
        assert_eq!(result[0].bet_type, BetType::Player);
        assert_eq!(result[1].bet_type, BetType::Banker);
        assert_eq!(result[2].bet_type, BetType::Player);
        assert_eq!(result[2].odds, 2.0);
        assert_probability_dependent_fields_match(&result[2], &result[0]);
        assert_close(
            result[2].base_ev - result[0].base_ev,
            result[0].win_probability,
        );
        assert_close(
            result[2].total_ev - result[0].total_ev,
            result[0].win_probability,
        );
    }

    #[test]
    fn ev_per_bet_specs_calculate_aggregate_family_with_supplied_odds() {
        let result = calculate_ev(
            &standard_eight_deck_cards(),
            &[
                ev_spec("dragon-default", BetType::PlayerDragon, 1.0),
                ev_spec("dragon-promo", BetType::PlayerDragon, 9.0),
            ],
        )
        .expect("aggregate per-bet EV should calculate");

        assert_probability_dependent_fields_match(&result[1], &result[0]);
        assert_close(
            result[1].base_ev - result[0].base_ev,
            result[0].win_probability * 8.0,
        );
        assert_close(
            result[1].total_ev - result[0].total_ev,
            result[0].win_probability * 8.0,
        );
    }

    #[test]
    fn ev_outcome_odds_calculate_monkey_standard_mode() {
        const MONKEY_OUTCOME_ODDS: &[OutcomeOdds] = &[
            OutcomeOdds {
                outcome: BetOutcome::Monkey,
                odds: 50.0,
            },
            OutcomeOdds {
                outcome: BetOutcome::NoMonkey,
                odds: 1.0,
            },
        ];
        let probabilities = standard_probabilities();
        let monkey_probability = probabilities.opening_two.monkey.monkey.as_f64();
        let no_monkey_probability = probabilities.opening_two.monkey.no_monkey.as_f64();

        let result = calculate_ev(
            &standard_eight_deck_cards(),
            &[outcome_ev_spec(
                "monkey-standard",
                BetType::Monkey,
                BetMode::Monkey(MonkeyMode::Standard),
                OddsSpec::by_outcome(BetType::Monkey, 0.0, MONKEY_OUTCOME_ODDS),
            )],
        )
        .expect("Monkey outcome EV should calculate");

        assert_close(
            result[0].base_ev,
            monkey_probability * 50.0 + no_monkey_probability
                - (1.0 - monkey_probability - no_monkey_probability),
        );
    }

    #[test]
    fn ev_outcome_odds_reject_missing_required_outcome_odds() {
        const PERFECT_PAIR_SINGLE_ONLY: &[OutcomeOdds] = &[OutcomeOdds {
            outcome: BetOutcome::PerfectPairSingleSide,
            odds: 25.0,
        }];

        let error = calculate_ev(
            &standard_eight_deck_cards(),
            &[outcome_ev_spec(
                "perfect-pair-missing-both",
                BetType::PerfectPair,
                BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth),
                OddsSpec::by_outcome(BetType::PerfectPair, 25.0, PERFECT_PAIR_SINGLE_ONLY),
            )],
        )
        .expect_err("missing both-side odds should fail");

        assert!(error.contains("missing odds for outcome"));
        assert!(error.contains("PerfectPairBothSides"));
    }

    #[test]
    fn ev_outcome_odds_reject_irrelevant_outcome_odds_for_mode() {
        const NO_MONKEY_WITH_IRRELEVANT_MONKEY: &[OutcomeOdds] = &[
            OutcomeOdds {
                outcome: BetOutcome::NoMonkey,
                odds: 1.0,
            },
            OutcomeOdds {
                outcome: BetOutcome::Monkey,
                odds: 50.0,
            },
        ];

        let error = calculate_ev(
            &standard_eight_deck_cards(),
            &[outcome_ev_spec(
                "no-monkey-extra-monkey",
                BetType::Monkey,
                BetMode::Monkey(MonkeyMode::NoMonkeyOnly),
                OddsSpec::by_outcome(BetType::Monkey, 0.0, NO_MONKEY_WITH_IRRELEVANT_MONKEY),
            )],
        )
        .expect_err("irrelevant Monkey outcome odds should fail");

        assert!(error.contains("irrelevant odds for outcome"));
        assert!(error.contains("Monkey"));
    }

    #[test]
    fn ev_outcome_odds_reject_incompatible_mode() {
        let error = calculate_ev(
            &standard_eight_deck_cards(),
            &[outcome_ev_spec(
                "player-with-monkey-mode",
                BetType::Player,
                BetMode::Monkey(MonkeyMode::Standard),
                OddsSpec::simple(BetType::Player, 1.0),
            )],
        )
        .expect_err("incompatible mode should fail");

        assert!(error.contains("incompatible with bet type"));
    }

    #[test]
    fn ev_request_validation_accepts_empty_spec_list_after_validating_cards() {
        let result = calculate_ev(&standard_eight_deck_cards(), &[])
            .expect("empty spec list should calculate successfully");

        assert!(result.is_empty());

        let invalid_cards = [CardCount {
            card: Card {
                suit: CardSuit::Clubs,
                rank: CardRank::Ace,
            },
            count: u32::from(standard_baccarat::STANDARD_DECK_COUNT) + 1,
        }];
        let error = calculate_ev(&invalid_cards, &[]).expect_err("cards should validate first");
        assert!(error.contains("exceeds standard maximum"));
    }

    #[test]
    fn ev_request_validation_preserves_spec_id() {
        let result = calculate_ev(
            &standard_eight_deck_cards(),
            &[ev_spec("spec-a", BetType::PlayerDragon, 31.0)],
        )
        .expect("valid spec should calculate successfully");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "spec-a");
    }

    #[test]
    fn ev_request_validation_rejects_blank_spec_id() {
        let error = calculate_ev(
            &standard_eight_deck_cards(),
            &[PerBetEvCalculationSpec {
                id: String::from("   "),
                bet_type: BetType::Player,
                mode: None,
                odds: OddsSpec::simple(BetType::Player, 1.0),
                rebate_rate: 0.1,
                effective_mode: EffectiveAmountMode::TotalStake,
            }],
        )
        .expect_err("blank spec id should fail");

        assert!(error.contains("spec id"));
    }

    #[test]
    fn ev_request_validation_rejects_duplicate_spec_id() {
        let spec = ev_spec("spec-a", BetType::Player, 1.0);
        let error = calculate_ev(&standard_eight_deck_cards(), &[spec.clone(), spec])
            .expect_err("duplicate spec id should fail");

        assert!(error.contains("duplicate EV spec id"));
    }

    #[test]
    fn ev_request_validation_rejects_invalid_odds_and_rebate() {
        let invalid_odds_error = calculate_ev(
            &standard_eight_deck_cards(),
            &[PerBetEvCalculationSpec {
                id: String::from("spec-b"),
                bet_type: BetType::Player,
                mode: None,
                odds: OddsSpec::simple(BetType::Player, f64::NAN),
                rebate_rate: 0.1,
                effective_mode: EffectiveAmountMode::TotalStake,
            }],
        )
        .expect_err("NaN odds should fail");
        assert!(invalid_odds_error.contains("invalid odds"));

        let invalid_rebate_error = calculate_ev(
            &standard_eight_deck_cards(),
            &[PerBetEvCalculationSpec {
                id: String::from("spec-c"),
                bet_type: BetType::Player,
                mode: None,
                odds: OddsSpec::simple(BetType::Player, 1.0),
                rebate_rate: 1.1,
                effective_mode: EffectiveAmountMode::TotalStake,
            }],
        )
        .expect_err("rebate above 1.0 should fail");
        assert!(invalid_rebate_error.contains("rebate rate"));
    }
}
