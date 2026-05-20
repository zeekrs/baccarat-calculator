use calculator::standard_baccarat::*;
use calculator::{
    calculate_ev, calculate_probabilities, default_odds_table, public_probability_definitions,
    standard_eight_deck_cards, BetMode, BetOutcome, BetType, BetVariant, Card, CardCount, CardRank,
    CardSuit, DragonVariant, EffectiveAmountMode, Fortune4PairVariant, Lucky6Variant, MonkeyMode,
    OddsSpec, OutcomeOdds, PerBetEvCalculationSpec, PerfectPairMode, TigerVariant,
};

#[path = "fixtures/source_standard_8_deck.rs"]
mod source_standard_8_deck;
#[path = "fixtures/source_standard_8_deck_variants.rs"]
mod source_standard_8_deck_variants;
#[path = "fixtures/standard_baccarat_golden.rs"]
mod standard_baccarat_golden;

use source_standard_8_deck::SOURCE_STANDARD_8_DECK_BETS;
use source_standard_8_deck_variants::SOURCE_STANDARD_8_DECK_VARIANTS;
use standard_baccarat_golden::{
    DEPLETED_SHOE_FIXTURE, PROBABILITY_ABS_TOLERANCE, PROBABILITY_SUM_ABS_TOLERANCE,
    STANDARD_8_DECK_EV_GOLDEN, STANDARD_8_DECK_FIXTURE, STANDARD_8_DECK_GOLDEN,
};
use std::collections::HashSet;

const SOURCE_BASELINE_TOLERANCE: f64 = 1e-10;
const EV_TOLERANCE: f64 = 1e-12;
const SOURCE_EV_BASELINE_TOLERANCE: f64 = 1e-9;

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {actual:.15} to be within {tolerance} of {expected:.15}"
    );
}

fn assert_probability_close(actual: f64, expected: f64) {
    assert_close(actual, expected, PROBABILITY_ABS_TOLERANCE);
}

fn assert_ev_close(actual: f64, expected: f64) {
    assert_close(actual, expected, EV_TOLERANCE);
}

fn assert_source_ev_close(actual: f64, expected: f64) {
    assert_close(actual, expected, SOURCE_EV_BASELINE_TOLERANCE);
}

fn fixture_cards(ranks: &[(&str, u32); 13]) -> Vec<CardCount> {
    ranks
        .iter()
        .flat_map(|(rank, count)| {
            let rank = CardRank::from_label(rank)
                .unwrap_or_else(|| panic!("fixture rank {rank} should be supported"));
            let base = count / u32::from(SUITS_PER_DECK);
            let remainder = count % u32::from(SUITS_PER_DECK);
            CardSuit::ALL
                .into_iter()
                .enumerate()
                .map(move |(index, suit)| CardCount {
                    card: Card { suit, rank },
                    count: base + u32::from(index < remainder as usize),
                })
        })
        .collect()
}

fn standard_cards() -> Vec<CardCount> {
    standard_eight_deck_cards()
}

fn card(suit: CardSuit, rank: CardRank) -> Card {
    Card { suit, rank }
}

#[derive(Clone, Copy, Debug, Default)]
struct BruteForceOutcomeCounts {
    player: u128,
    banker: u128,
    tie: u128,
}

#[derive(Clone, Copy, Debug, Default)]
struct BruteForceOpeningOutcomeCounts {
    total: u128,
    perfect_pair_single_side: u128,
    perfect_pair_both_sides: u128,
    monkey: u128,
    no_monkey: u128,
}

impl BruteForceOutcomeCounts {
    fn add(&mut self, winner: BruteForceWinner, weight: u128) {
        match winner {
            BruteForceWinner::Player => self.player += weight,
            BruteForceWinner::Banker => self.banker += weight,
            BruteForceWinner::Tie => self.tie += weight,
        }
    }

    fn total(self) -> u128 {
        self.player + self.banker + self.tie
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BruteForceWinner {
    Player,
    Banker,
    Tie,
}

fn brute_force_main_probabilities(cards: &[CardCount]) -> (f64, f64, f64) {
    let mut deck = cards
        .iter()
        .flat_map(|entry| std::iter::repeat_n(entry.card, entry.count as usize))
        .collect::<Vec<_>>();
    let mut counts = BruteForceOutcomeCounts::default();
    brute_force_deal(&mut deck, &mut Vec::new(), &mut counts);

    let total = counts.total() as f64;
    (
        counts.player as f64 / total,
        counts.banker as f64 / total,
        counts.tie as f64 / total,
    )
}

fn brute_force_opening_outcomes(cards: &[CardCount]) -> BruteForceOpeningOutcomeCounts {
    let mut deck = cards
        .iter()
        .flat_map(|entry| std::iter::repeat_n(entry.card, entry.count as usize))
        .collect::<Vec<_>>();
    let mut counts = BruteForceOpeningOutcomeCounts::default();
    brute_force_opening_four_cards(&mut deck, &mut Vec::new(), &mut counts);
    counts
}

fn brute_force_opening_four_cards(
    remaining: &mut Vec<Card>,
    dealt: &mut Vec<Card>,
    counts: &mut BruteForceOpeningOutcomeCounts,
) {
    if dealt.len() == 4 {
        counts.total += 1;

        let player_pair = dealt[0] == dealt[2];
        let banker_pair = dealt[1] == dealt[3];
        match (player_pair, banker_pair) {
            (true, true) => counts.perfect_pair_both_sides += 1,
            (true, false) | (false, true) => counts.perfect_pair_single_side += 1,
            (false, false) => {}
        }

        let all_face_cards = dealt
            .iter()
            .all(|card| matches!(card.rank, CardRank::Jack | CardRank::Queen | CardRank::King));
        if all_face_cards {
            counts.monkey += 1;
        }

        let all_non_face_cards = dealt
            .iter()
            .all(|card| !matches!(card.rank, CardRank::Jack | CardRank::Queen | CardRank::King));
        if all_non_face_cards {
            counts.no_monkey += 1;
        }
        return;
    }

    for index in 0..remaining.len() {
        let next = remaining.swap_remove(index);
        dealt.push(next);
        brute_force_opening_four_cards(remaining, dealt, counts);
        dealt.pop();
        remaining.push(next);
        let last_index = remaining.len() - 1;
        remaining.swap(index, last_index);
    }
}

fn brute_force_deal(
    remaining: &mut Vec<Card>,
    dealt: &mut Vec<Card>,
    counts: &mut BruteForceOutcomeCounts,
) {
    if dealt.len() == 6 {
        counts.add(brute_force_winner(dealt), 1);
        return;
    }

    if dealt.len() >= 4 && brute_force_hand_complete(dealt) {
        counts.add(
            brute_force_winner(dealt),
            brute_force_suffix_weight(remaining.len(), 6 - dealt.len()),
        );
        return;
    }

    for index in 0..remaining.len() {
        let next = remaining.swap_remove(index);
        dealt.push(next);
        brute_force_deal(remaining, dealt, counts);
        dealt.pop();
        remaining.push(next);
        let last_index = remaining.len() - 1;
        remaining.swap(index, last_index);
    }
}

fn brute_force_suffix_weight(remaining_cards: usize, draw_count: usize) -> u128 {
    (0..draw_count)
        .map(|offset| (remaining_cards - offset) as u128)
        .product()
}

fn brute_force_hand_complete(dealt: &[Card]) -> bool {
    let player_initial_total = brute_force_total(&[dealt[0], dealt[2]]);
    let banker_initial_total = brute_force_total(&[dealt[1], dealt[3]]);
    let natural = matches!(player_initial_total, 8 | 9) || matches!(banker_initial_total, 8 | 9);
    let player_draws = !natural && player_initial_total <= 5;
    if player_draws && dealt.len() < 5 {
        return false;
    }

    let player_third_value = if player_draws {
        dealt.get(4).map(|card| card.rank.baccarat_value())
    } else {
        None
    };
    let banker_draws = if natural {
        false
    } else if let Some(player_third) = player_third_value {
        match banker_initial_total {
            0..=2 => true,
            3 => player_third != 8,
            4 => matches!(player_third, 2..=7),
            5 => matches!(player_third, 4..=7),
            6 => matches!(player_third, 6 | 7),
            _ => false,
        }
    } else {
        banker_initial_total <= 5
    };

    !banker_draws || dealt.len() == if player_draws { 6 } else { 5 }
}

fn brute_force_winner(dealt: &[Card]) -> BruteForceWinner {
    let (player, banker) = brute_force_final_hands(dealt);
    let player_total = brute_force_total(&player);
    let banker_total = brute_force_total(&banker);

    match player_total.cmp(&banker_total) {
        std::cmp::Ordering::Greater => BruteForceWinner::Player,
        std::cmp::Ordering::Less => BruteForceWinner::Banker,
        std::cmp::Ordering::Equal => BruteForceWinner::Tie,
    }
}

fn brute_force_final_hands(dealt: &[Card]) -> (Vec<Card>, Vec<Card>) {
    let mut player = vec![dealt[0], dealt[2]];
    let mut banker = vec![dealt[1], dealt[3]];
    let player_initial_total = brute_force_total(&player);
    let banker_initial_total = brute_force_total(&banker);
    let natural = matches!(player_initial_total, 8 | 9) || matches!(banker_initial_total, 8 | 9);
    let player_draws = !natural && player_initial_total <= 5;

    let player_third_value = if player_draws {
        let third = dealt[4];
        player.push(third);
        Some(third.rank.baccarat_value())
    } else {
        None
    };
    let banker_draws = if natural {
        false
    } else if let Some(player_third) = player_third_value {
        match banker_initial_total {
            0..=2 => true,
            3 => player_third != 8,
            4 => matches!(player_third, 2..=7),
            5 => matches!(player_third, 4..=7),
            6 => matches!(player_third, 6 | 7),
            _ => false,
        }
    } else {
        banker_initial_total <= 5
    };

    if banker_draws {
        banker.push(dealt[if player_draws { 5 } else { 4 }]);
    }

    (player, banker)
}

fn brute_force_total(cards: &[Card]) -> u8 {
    cards
        .iter()
        .map(|card| card.rank.baccarat_value())
        .sum::<u8>()
        % 10
}

fn probability(numerator: u128, denominator: u128) -> f64 {
    numerator as f64 / denominator as f64
}

fn ev_config(
    id: &str,
    bet_type: BetType,
    odds: f64,
    rebate_rate: f64,
    effective_mode: EffectiveAmountMode,
) -> PerBetEvCalculationSpec {
    PerBetEvCalculationSpec {
        id: id.to_owned(),
        bet_type,
        mode: None,
        odds: calculator::OddsSpec::simple(bet_type, odds),
        rebate_rate,
        effective_mode,
    }
}

fn default_ev_spec(id: &str, bet_type: BetType) -> PerBetEvCalculationSpec {
    let definition = public_probability_definitions()
        .find(|definition| definition.bet_type() == bet_type)
        .unwrap_or_else(|| panic!("missing definition for {bet_type:?}"));
    let odds = default_odds_table()
        .get(definition.id)
        .and_then(|spec| spec.odds())
        .unwrap_or(1.0);

    ev_config(id, bet_type, odds, 0.0, EffectiveAmountMode::TotalStake)
}

fn default_ev_specs() -> Vec<PerBetEvCalculationSpec> {
    public_probability_definitions()
        .map(|definition| {
            default_ev_spec(
                &format!("{:?}", definition.bet_type()),
                definition.bet_type(),
            )
        })
        .collect()
}

fn result_by_bet_type(
    result: &calculator::ProbabilityCalculationResult,
    bet_type: BetType,
) -> &calculator::BetProbabilityResult {
    result
        .bets
        .iter()
        .find(|bet| bet.bet_type == bet_type)
        .unwrap_or_else(|| panic!("missing result for {bet_type:?}"))
}

fn variant_probability(
    result: &calculator::ProbabilityCalculationResult,
    bet_type: BetType,
    variant: BetVariant,
) -> f64 {
    result_by_bet_type(result, bet_type)
        .variants
        .iter()
        .find(|actual| actual.variant == variant)
        .unwrap_or_else(|| panic!("missing variant {variant:?} for {bet_type:?}"))
        .probability
}

fn outcome_probability(
    result: &calculator::ProbabilityCalculationResult,
    bet_type: BetType,
    outcome: BetOutcome,
) -> f64 {
    result_by_bet_type(result, bet_type)
        .outcomes
        .iter()
        .find(|actual| actual.outcome == outcome)
        .unwrap_or_else(|| panic!("missing outcome {outcome:?} for {bet_type:?}"))
        .probability
}

fn ev_result_by_bet_type(
    result: &[calculator::PerBetEvCalculationResult],
    bet_type: BetType,
) -> &calculator::PerBetEvCalculationResult {
    result
        .iter()
        .find(|bet| bet.bet_type == bet_type)
        .unwrap_or_else(|| panic!("missing EV result for {bet_type:?}"))
}

fn source_variant_probability(bet_type: BetType, variant: BetVariant) -> f64 {
    SOURCE_STANDARD_8_DECK_VARIANTS
        .iter()
        .find(|candidate| candidate.bet_type == bet_type && candidate.variant == variant)
        .unwrap_or_else(|| {
            panic!("missing source variant probability for {bet_type:?} {variant:?}")
        })
        .probability
}

fn source_probability(bet_type: BetType) -> f64 {
    SOURCE_STANDARD_8_DECK_BETS
        .iter()
        .find(|candidate| candidate.bet_type == bet_type)
        .unwrap_or_else(|| panic!("missing source probability for {bet_type:?}"))
        .probability
}

fn new_public_bet_types() -> HashSet<BetType> {
    [
        BetType::Monkey,
        BetType::SuperTie0,
        BetType::SuperTie1,
        BetType::SuperTie2,
        BetType::SuperTie3,
        BetType::SuperTie4,
        BetType::SuperTie5,
        BetType::SuperTie6,
        BetType::SuperTie7,
        BetType::SuperTie8,
        BetType::SuperTie9,
    ]
    .into_iter()
    .collect()
}

#[test]
fn pure_probability_api_requires_only_card_counts() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard cards should calculate probabilities");

    assert_eq!(result.bets.len(), public_probability_definitions().count());

    let player = result_by_bet_type(&result, BetType::Player);
    assert_probability_close(
        player.probability,
        STANDARD_8_DECK_GOLDEN.player_probability,
    );

    let dragon7 = result_by_bet_type(&result, BetType::Dragon7);
    assert!(dragon7.probability > 0.0);
}

#[test]
fn pure_probability_api_reports_standard_outcome_probabilities() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard cards should calculate probabilities");

    assert_probability_close(
        result_by_bet_type(&result, BetType::Player).probability,
        STANDARD_8_DECK_GOLDEN.player_probability,
    );
    assert_probability_close(
        result_by_bet_type(&result, BetType::Banker).probability,
        STANDARD_8_DECK_GOLDEN.banker_probability,
    );
    assert_probability_close(
        result_by_bet_type(&result, BetType::Tie).probability,
        STANDARD_8_DECK_GOLDEN.tie_probability,
    );
}

#[test]
fn standard_eight_deck_matches_every_source_project_probability() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate all public bet types");
    let expected_bet_types = SOURCE_STANDARD_8_DECK_BETS
        .iter()
        .map(|expected| expected.bet_type)
        .collect::<HashSet<_>>();
    let actual_bet_types = result
        .bets
        .iter()
        .map(|actual| actual.bet_type)
        .collect::<HashSet<_>>();
    let public_bet_types = public_probability_definitions()
        .map(|definition| definition.bet_type())
        .collect::<HashSet<_>>();

    assert_eq!(expected_bet_types.len(), SOURCE_STANDARD_8_DECK_BETS.len());
    assert_eq!(actual_bet_types, public_bet_types);

    let new_public_bet_types = new_public_bet_types();
    assert!(new_public_bet_types.is_disjoint(&expected_bet_types));
    assert_eq!(
        actual_bet_types.len(),
        expected_bet_types.len() + new_public_bet_types.len()
    );
    assert_eq!(
        actual_bet_types
            .difference(&expected_bet_types)
            .copied()
            .collect::<HashSet<_>>(),
        new_public_bet_types
    );

    for expected in SOURCE_STANDARD_8_DECK_BETS {
        let actual = result_by_bet_type(&result, expected.bet_type);
        assert_close(
            actual.probability,
            expected.probability,
            SOURCE_BASELINE_TOLERANCE,
        );
    }
}

#[test]
fn standard_eight_deck_matches_every_source_project_variant_probability() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate all public bet variants");
    let actual_variant_count = result
        .bets
        .iter()
        .map(|bet| bet.variants.len())
        .sum::<usize>();

    assert_eq!(actual_variant_count, SOURCE_STANDARD_8_DECK_VARIANTS.len());
    for expected in SOURCE_STANDARD_8_DECK_VARIANTS {
        let duplicate_count = SOURCE_STANDARD_8_DECK_VARIANTS
            .iter()
            .filter(|candidate| {
                candidate.bet_type == expected.bet_type && candidate.variant == expected.variant
            })
            .count();
        assert_eq!(duplicate_count, 1);

        let actual = variant_probability(&result, expected.bet_type, expected.variant);
        assert_close(actual, expected.probability, SOURCE_BASELINE_TOLERANCE);
    }
}

#[test]
fn fixture_rank_probabilities_match_golden_values() {
    for fixture in [STANDARD_8_DECK_FIXTURE, DEPLETED_SHOE_FIXTURE] {
        let cards = fixture_cards(&fixture.ranks);
        let result = calculate_probabilities(&cards)
            .unwrap_or_else(|error| panic!("{} should calculate: {error}", fixture.name));

        for expected in fixture.expected_bets {
            let actual = result_by_bet_type(&result, expected.bet_type);
            assert_probability_close(actual.probability, expected.probability);
        }
    }
}

#[test]
fn card_input_enables_suit_dependent_bets() {
    let result = calculate_probabilities(&standard_cards())
        .expect("card counts should calculate suit-dependent bet types");

    let perfect_pair = result_by_bet_type(&result, BetType::PerfectPair);
    assert!(perfect_pair.probability > 0.0);
}

#[test]
fn card_input_returns_all_registered_bets() {
    let result = calculate_probabilities(&standard_cards())
        .expect("card input should calculate every public bet type");

    assert_eq!(result.bets.len(), public_probability_definitions().count());
    assert!(result
        .bets
        .iter()
        .any(|bet| bet.bet_type == BetType::PerfectPair));
}

#[test]
fn probability_output_reports_each_public_bet_type_once() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate public bet types");

    for definition in public_probability_definitions() {
        assert!(
            result
                .bets
                .iter()
                .any(|bet| bet.bet_type == definition.bet_type()),
            "missing public probability for {:?}",
            definition.bet_type()
        );
    }

    let mut seen = std::collections::HashSet::new();
    for bet in &result.bets {
        assert!(
            seen.insert(bet.bet_type),
            "duplicate bet type {:?}",
            bet.bet_type
        );
    }
}

#[test]
fn registered_public_bets_have_probability_and_ev_coverage() {
    let probability_result = calculate_probabilities(&standard_cards())
        .expect("registered public bets should have probability coverage");
    let ev_specs = default_ev_specs();
    let ev_result = calculate_ev(&standard_cards(), &ev_specs)
        .expect("registered public bets should have EV coverage");

    for definition in public_probability_definitions() {
        let probability = result_by_bet_type(&probability_result, definition.bet_type());
        assert!(
            probability.probability.is_finite(),
            "missing finite probability for {:?}",
            definition.id
        );

        let ev = ev_result_by_bet_type(&ev_result, definition.bet_type());
        assert!(
            ev.base_ev.is_finite(),
            "missing finite EV for {:?}",
            definition.id
        );
        assert!(
            ev.win_probability.is_finite()
                && ev.lose_probability.is_finite()
                && ev.push_probability.is_finite(),
            "missing finite EV probability breakdown for {:?}",
            definition.id
        );
    }
}

#[test]
fn probability_output_preserves_public_registry_order() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate public bet types");

    let expected = public_probability_definitions()
        .map(|definition| definition.bet_type())
        .collect::<Vec<_>>();
    let actual = result
        .bets
        .iter()
        .map(|bet| bet.bet_type)
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn ev_batch_uses_default_odds_and_preserves_registry_order() {
    let specs = default_ev_specs();
    let result =
        calculate_ev(&standard_cards(), &specs).expect("standard cards should calculate EV");

    let expected_order = public_probability_definitions()
        .map(|definition| definition.bet_type())
        .collect::<Vec<_>>();
    let actual_order = result.iter().map(|bet| bet.bet_type).collect::<Vec<_>>();
    assert_eq!(actual_order, expected_order);

    let player = ev_result_by_bet_type(&result, BetType::Player);
    assert_eq!(player.odds, 1.0);
    assert_ev_close(
        player.win_probability,
        STANDARD_8_DECK_GOLDEN.player_probability,
    );
    assert_ev_close(
        player.lose_probability,
        STANDARD_8_DECK_GOLDEN.banker_probability,
    );
    assert_ev_close(
        player.push_probability,
        STANDARD_8_DECK_GOLDEN.tie_probability,
    );
    assert_ev_close(
        player.base_ev,
        STANDARD_8_DECK_GOLDEN.player_probability * 1.0 - STANDARD_8_DECK_GOLDEN.banker_probability,
    );
    assert_ev_close(player.rebate_ev, 0.0);
    assert_ev_close(player.total_ev, player.base_ev);
    assert_ev_close(player.effective_probability, 1.0);

    let banker = ev_result_by_bet_type(&result, BetType::Banker);
    assert_eq!(banker.odds, 0.95);
    assert_ev_close(
        banker.win_probability,
        STANDARD_8_DECK_GOLDEN.banker_probability,
    );
    assert_ev_close(
        banker.lose_probability,
        STANDARD_8_DECK_GOLDEN.player_probability,
    );
    assert_ev_close(
        banker.push_probability,
        STANDARD_8_DECK_GOLDEN.tie_probability,
    );
    assert_ev_close(
        banker.base_ev,
        STANDARD_8_DECK_GOLDEN.banker_probability * 0.95
            - STANDARD_8_DECK_GOLDEN.player_probability,
    );
    assert_ev_close(banker.total_ev, banker.base_ev);

    let tie = ev_result_by_bet_type(&result, BetType::Tie);
    assert_eq!(tie.odds, 8.0);
    assert_ev_close(tie.win_probability, STANDARD_8_DECK_GOLDEN.tie_probability);
    assert_ev_close(
        tie.lose_probability,
        1.0 - STANDARD_8_DECK_GOLDEN.tie_probability,
    );
    assert_ev_close(tie.push_probability, 0.0);
    assert_ev_close(
        tie.base_ev,
        STANDARD_8_DECK_GOLDEN.tie_probability * 8.0
            - (1.0 - STANDARD_8_DECK_GOLDEN.tie_probability),
    );
    assert_ev_close(tie.total_ev, tie.base_ev);

    let player_dragon = ev_result_by_bet_type(&result, BetType::PlayerDragon);
    assert_eq!(player_dragon.odds, 1.0);
    assert!(player_dragon.win_probability > 0.0);
    assert!(player_dragon.push_probability > 0.0);
}

#[test]
fn ev_rebate_modes_use_the_expected_effective_probability() {
    let rebate_rate = 0.125;
    let result = calculate_ev(
        &standard_cards(),
        &[
            ev_config(
                "standard",
                BetType::Player,
                1.0,
                rebate_rate,
                EffectiveAmountMode::Standard,
            ),
            ev_config(
                "total-stake",
                BetType::Player,
                1.0,
                rebate_rate,
                EffectiveAmountMode::TotalStake,
            ),
            ev_config(
                "non-refund",
                BetType::Player,
                1.0,
                rebate_rate,
                EffectiveAmountMode::NonRefund,
            ),
            ev_config(
                "losing-only",
                BetType::Player,
                1.0,
                rebate_rate,
                EffectiveAmountMode::LosingOnly,
            ),
            ev_config(
                "standard-banker",
                BetType::Banker,
                0.95,
                rebate_rate,
                EffectiveAmountMode::Standard,
            ),
        ],
    )
    .expect("standard cards should calculate EV");

    let standard_player = result
        .iter()
        .find(|bet| bet.id == "standard")
        .expect("missing standard player EV");
    assert_ev_close(
        standard_player.effective_probability,
        1.0 - STANDARD_8_DECK_GOLDEN.tie_probability,
    );
    assert_ev_close(
        standard_player.rebate_ev,
        rebate_rate * (1.0 - STANDARD_8_DECK_GOLDEN.tie_probability),
    );

    let standard_banker = result
        .iter()
        .find(|bet| bet.id == "standard-banker")
        .expect("missing standard banker EV");
    assert_ev_close(
        standard_banker.effective_probability,
        STANDARD_8_DECK_GOLDEN.player_probability
            + STANDARD_8_DECK_GOLDEN.banker_probability * 0.95,
    );
    assert_ev_close(
        standard_banker.rebate_ev,
        rebate_rate
            * (STANDARD_8_DECK_GOLDEN.player_probability
                + STANDARD_8_DECK_GOLDEN.banker_probability * 0.95),
    );

    let total_stake_player = result
        .iter()
        .find(|bet| bet.id == "total-stake")
        .expect("missing total-stake player EV");
    assert_ev_close(total_stake_player.effective_probability, 1.0);
    assert_ev_close(total_stake_player.rebate_ev, rebate_rate);
    assert_ev_close(
        total_stake_player.total_ev,
        total_stake_player.base_ev + rebate_rate,
    );

    let non_refund_player = result
        .iter()
        .find(|bet| bet.id == "non-refund")
        .expect("missing non-refund player EV");
    assert_ev_close(
        non_refund_player.effective_probability,
        1.0 - STANDARD_8_DECK_GOLDEN.tie_probability,
    );
    assert_ev_close(
        non_refund_player.rebate_ev,
        rebate_rate * (1.0 - STANDARD_8_DECK_GOLDEN.tie_probability),
    );
    assert_ev_close(
        non_refund_player.total_ev,
        non_refund_player.base_ev + rebate_rate * (1.0 - STANDARD_8_DECK_GOLDEN.tie_probability),
    );

    let losing_only_player = result
        .iter()
        .find(|bet| bet.id == "losing-only")
        .expect("missing losing-only player EV");
    assert_ev_close(
        losing_only_player.effective_probability,
        STANDARD_8_DECK_GOLDEN.banker_probability,
    );
    assert_ev_close(
        losing_only_player.rebate_ev,
        rebate_rate * STANDARD_8_DECK_GOLDEN.banker_probability,
    );
    assert_ev_close(
        losing_only_player.total_ev,
        losing_only_player.base_ev + rebate_rate * STANDARD_8_DECK_GOLDEN.banker_probability,
    );
}

#[test]
fn ev_standard_eight_deck_matches_golden_values() {
    let rebate_rate = 0.125;
    let result = calculate_ev(
        &standard_cards(),
        &[
            ev_config(
                "standard",
                BetType::Banker,
                0.95,
                rebate_rate,
                EffectiveAmountMode::Standard,
            ),
            ev_config(
                "total-stake",
                BetType::Player,
                1.0,
                rebate_rate,
                EffectiveAmountMode::TotalStake,
            ),
            ev_config(
                "total-stake-banker",
                BetType::Banker,
                0.95,
                rebate_rate,
                EffectiveAmountMode::TotalStake,
            ),
            ev_config(
                "total-stake-tie",
                BetType::Tie,
                8.0,
                rebate_rate,
                EffectiveAmountMode::TotalStake,
            ),
            ev_config(
                "non-refund",
                BetType::Player,
                1.0,
                rebate_rate,
                EffectiveAmountMode::NonRefund,
            ),
            ev_config(
                "losing-only",
                BetType::Player,
                1.0,
                rebate_rate,
                EffectiveAmountMode::LosingOnly,
            ),
        ],
    )
    .expect("standard cards should calculate golden EV values");

    let standard_banker = result.iter().find(|bet| bet.id == "standard").unwrap();
    assert_ev_close(
        standard_banker.effective_probability,
        STANDARD_8_DECK_GOLDEN.player_probability
            + STANDARD_8_DECK_GOLDEN.banker_probability * 0.95,
    );
    assert_ev_close(
        standard_banker.rebate_ev,
        rebate_rate
            * (STANDARD_8_DECK_GOLDEN.player_probability
                + STANDARD_8_DECK_GOLDEN.banker_probability * 0.95),
    );

    let player = result.iter().find(|bet| bet.id == "total-stake").unwrap();
    assert_ev_close(
        player.base_ev,
        STANDARD_8_DECK_EV_GOLDEN.player_default_base_ev,
    );
    assert_ev_close(
        player.effective_probability,
        STANDARD_8_DECK_EV_GOLDEN.player_total_stake_effective_probability,
    );
    assert_ev_close(player.rebate_ev, rebate_rate);
    assert_ev_close(player.total_ev, player.base_ev + rebate_rate);

    let banker = result
        .iter()
        .find(|bet| bet.id == "total-stake-banker")
        .unwrap();
    assert_ev_close(
        banker.base_ev,
        STANDARD_8_DECK_EV_GOLDEN.banker_default_base_ev,
    );

    let tie = result
        .iter()
        .find(|bet| bet.id == "total-stake-tie")
        .unwrap();
    assert_ev_close(tie.base_ev, STANDARD_8_DECK_EV_GOLDEN.tie_default_base_ev);

    let non_refund_player = result.iter().find(|bet| bet.id == "non-refund").unwrap();
    assert_ev_close(
        non_refund_player.effective_probability,
        STANDARD_8_DECK_EV_GOLDEN.player_non_refund_effective_probability,
    );
    assert_ev_close(
        non_refund_player.rebate_ev,
        rebate_rate * STANDARD_8_DECK_EV_GOLDEN.player_non_refund_effective_probability,
    );

    let losing_only_player = result.iter().find(|bet| bet.id == "losing-only").unwrap();
    assert_ev_close(
        losing_only_player.effective_probability,
        STANDARD_8_DECK_EV_GOLDEN.player_losing_only_effective_probability,
    );
    assert_ev_close(
        losing_only_player.rebate_ev,
        rebate_rate * STANDARD_8_DECK_EV_GOLDEN.player_losing_only_effective_probability,
    );
}

#[test]
fn ev_standard_eight_deck_all_bets_and_variants_match_golden_values() {
    let specs = default_ev_specs();
    let result = calculate_ev(&standard_cards(), &specs)
        .expect("standard cards should calculate all EV golden values");

    assert_eq!(result.len(), public_probability_definitions().count());

    for bet in result.iter().filter(|bet| {
        SOURCE_STANDARD_8_DECK_BETS
            .iter()
            .any(|source| source.bet_type == bet.bet_type)
    }) {
        let expected_probability = source_probability(bet.bet_type);
        let expected_win = match bet.bet_type {
            BetType::Player => source_probability(BetType::Player),
            BetType::Banker => source_probability(BetType::Banker),
            _ => expected_probability,
        };
        let expected_push = match bet.bet_type {
            BetType::Player | BetType::Banker => source_probability(BetType::Tie),
            BetType::PlayerDragon => source_variant_probability(
                BetType::PlayerDragon,
                BetVariant::Dragon(DragonVariant::Push),
            ),
            BetType::BankerDragon => source_variant_probability(
                BetType::BankerDragon,
                BetVariant::Dragon(DragonVariant::Push),
            ),
            BetType::PlayerNatural => source_variant_probability(
                BetType::PlayerNatural,
                BetVariant::Natural(calculator::NaturalVariant::Push),
            ),
            BetType::BankerNatural => source_variant_probability(
                BetType::BankerNatural,
                BetVariant::Natural(calculator::NaturalVariant::Push),
            ),
            _ => 0.0,
        };
        let expected_lose = 1.0 - expected_win - expected_push;

        assert_source_ev_close(bet.win_probability, expected_win);
        assert_source_ev_close(bet.push_probability, expected_push);
        assert_source_ev_close(bet.lose_probability, expected_lose);
        assert_source_ev_close(bet.effective_probability, 1.0);
        assert_source_ev_close(bet.base_ev, expected_win * bet.odds - expected_lose);
        assert_source_ev_close(bet.rebate_ev, 0.0);
        assert_source_ev_close(bet.total_ev, bet.base_ev);
    }

    let source_bet_types = SOURCE_STANDARD_8_DECK_BETS
        .iter()
        .map(|expected| expected.bet_type)
        .collect::<HashSet<_>>();
    let actual_bet_types = result
        .iter()
        .map(|actual| actual.bet_type)
        .collect::<HashSet<_>>();
    let new_public_bet_types = new_public_bet_types();
    assert!(new_public_bet_types.is_disjoint(&source_bet_types));
    assert_eq!(
        actual_bet_types
            .difference(&source_bet_types)
            .copied()
            .collect::<HashSet<_>>(),
        new_public_bet_types
    );
}

#[test]
fn ev_request_validation_accepts_zero_odds_and_rejects_invalid_odds() {
    let zero_odds = calculate_ev(
        &standard_cards(),
        &[ev_config(
            "zero",
            BetType::Player,
            0.0,
            0.0,
            EffectiveAmountMode::TotalStake,
        )],
    )
    .expect("zero odds should be allowed");
    assert_eq!(ev_result_by_bet_type(&zero_odds, BetType::Player).odds, 0.0);

    let negative_error = calculate_ev(
        &standard_cards(),
        &[ev_config(
            "negative",
            BetType::Player,
            -0.1,
            0.0,
            EffectiveAmountMode::TotalStake,
        )],
    )
    .expect_err("negative odds should fail");
    assert!(negative_error.contains("invalid odds"));

    let nan_error = calculate_ev(
        &standard_cards(),
        &[ev_config(
            "nan",
            BetType::Player,
            f64::NAN,
            0.0,
            EffectiveAmountMode::TotalStake,
        )],
    )
    .expect_err("NaN odds should fail");
    assert!(nan_error.contains("invalid odds"));

    let infinity_error = calculate_ev(
        &standard_cards(),
        &[ev_config(
            "infinity",
            BetType::Player,
            f64::INFINITY,
            0.0,
            EffectiveAmountMode::TotalStake,
        )],
    )
    .expect_err("infinite odds should fail");
    assert!(infinity_error.contains("invalid odds"));
}

#[test]
fn probability_output_does_not_expose_concrete_selection_strings() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate public bet types");
    let json = serde_json::to_string(&result).expect("probability result should serialize");

    assert!(!json.contains("selection"));
    assert!(!json.contains("BankerDragon_Natural"));
    assert!(!json.contains("Lucky6_Two"));
    assert!(!json.contains("Tiger_Two"));
}

#[test]
fn probability_output_includes_concrete_variant_probabilities() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate public bet variants");

    let player = result_by_bet_type(&result, BetType::Player);
    assert_eq!(player.variants, vec![]);

    let banker_dragon = result_by_bet_type(&result, BetType::BankerDragon);
    assert!(banker_dragon.variants.iter().any(|variant| {
        variant.variant == BetVariant::Dragon(DragonVariant::Natural)
            && (0.0..=1.0).contains(&variant.probability)
    }));

    let lucky6 = result_by_bet_type(&result, BetType::Lucky6);
    assert!(lucky6.variants.iter().any(|variant| {
        variant.variant == BetVariant::Lucky6(Lucky6Variant::Two)
            && (0.0..=1.0).contains(&variant.probability)
    }));

    let tiger = result_by_bet_type(&result, BetType::Tiger);
    assert!(tiger.variants.iter().any(|variant| {
        variant.variant == BetVariant::Tiger(TigerVariant::Two)
            && (0.0..=1.0).contains(&variant.probability)
    }));
}

#[test]
fn fortune4_pair_uses_diamonds_not_spades_for_fortune30() {
    let cards = vec![
        CardCount {
            card: Card {
                suit: CardSuit::Diamonds,
                rank: CardRank::Four,
            },
            count: 2,
        },
        CardCount {
            card: Card {
                suit: CardSuit::Spades,
                rank: CardRank::Four,
            },
            count: 0,
        },
        CardCount {
            card: Card {
                suit: CardSuit::Clubs,
                rank: CardRank::Ace,
            },
            count: 4,
        },
    ];
    let result = calculate_probabilities(&cards)
        .expect("non-uniform suit shoe should calculate Fortune4Pair probabilities");

    assert_close(
        variant_probability(
            &result,
            BetType::BankerFortune4Pair,
            BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune30),
        ),
        2.0 / 30.0,
        1e-12,
    );
    assert_close(
        variant_probability(
            &result,
            BetType::BankerFortune4Pair,
            BetVariant::Fortune4Pair(Fortune4PairVariant::Fortune15),
        ),
        0.0,
        1e-12,
    );
}

#[test]
fn probability_output_reports_monkey_outcome_contract() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate public bet types");
    let monkey = result_by_bet_type(&result, BetType::Monkey);

    assert!(monkey.variants.is_empty());
    assert_eq!(monkey.outcomes.len(), 2);

    let monkey_probability = outcome_probability(&result, BetType::Monkey, BetOutcome::Monkey);
    let no_monkey_probability = outcome_probability(&result, BetType::Monkey, BetOutcome::NoMonkey);

    assert!(monkey_probability > 0.0);
    assert!(no_monkey_probability > 0.0);
    assert_probability_close(
        monkey.probability,
        monkey_probability + no_monkey_probability,
    );
}

#[test]
fn probability_output_reports_perfect_pair_exclusive_outcome_contract() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate public bet types");
    let perfect_pair = result_by_bet_type(&result, BetType::PerfectPair);

    assert!(perfect_pair.variants.is_empty());
    assert_eq!(perfect_pair.outcomes.len(), 2);

    let single_side_probability = outcome_probability(
        &result,
        BetType::PerfectPair,
        BetOutcome::PerfectPairSingleSide,
    );
    let both_sides_probability = outcome_probability(
        &result,
        BetType::PerfectPair,
        BetOutcome::PerfectPairBothSides,
    );

    assert!(single_side_probability > 0.0);
    assert!(both_sides_probability > 0.0);
    assert_probability_close(
        perfect_pair.probability,
        single_side_probability + both_sides_probability,
    );
}

#[test]
fn probability_output_reports_super_tie_0_to_9_without_aggregate_row() {
    let result = calculate_probabilities(&standard_cards())
        .expect("standard card counts should calculate public bet types");
    let expected_super_ties = [
        BetType::SuperTie0,
        BetType::SuperTie1,
        BetType::SuperTie2,
        BetType::SuperTie3,
        BetType::SuperTie4,
        BetType::SuperTie5,
        BetType::SuperTie6,
        BetType::SuperTie7,
        BetType::SuperTie8,
        BetType::SuperTie9,
    ];
    let super_tie_rows = result
        .bets
        .iter()
        .filter(|bet| format!("{:?}", bet.bet_type).starts_with("SuperTie"))
        .collect::<Vec<_>>();

    assert_eq!(super_tie_rows.len(), expected_super_ties.len());

    let super_tie_total = expected_super_ties
        .into_iter()
        .map(|bet_type| {
            let row = result_by_bet_type(&result, bet_type);
            assert!(
                row.probability > 0.0,
                "{bet_type:?} should have probability"
            );
            assert!(
                row.variants.is_empty(),
                "{bet_type:?} should not expose variants"
            );
            assert!(
                row.outcomes.is_empty(),
                "{bet_type:?} should not expose outcomes"
            );
            row.probability
        })
        .sum::<f64>();

    assert_probability_close(
        super_tie_total,
        result_by_bet_type(&result, BetType::Tie).probability,
    );
}

#[test]
fn ev_outcome_odds_validation_rejects_missing_and_irrelevant_outcomes() {
    const MONKEY_ONLY_ODDS: &[OutcomeOdds] = &[OutcomeOdds {
        outcome: BetOutcome::Monkey,
        odds: 50.0,
    }];
    const NO_MONKEY_PLUS_IRRELEVANT_ODDS: &[OutcomeOdds] = &[
        OutcomeOdds {
            outcome: BetOutcome::NoMonkey,
            odds: 1.0,
        },
        OutcomeOdds {
            outcome: BetOutcome::Monkey,
            odds: 50.0,
        },
    ];
    const PERFECT_PAIR_SINGLE_ONLY_ODDS: &[OutcomeOdds] = &[OutcomeOdds {
        outcome: BetOutcome::PerfectPairSingleSide,
        odds: 25.0,
    }];

    let missing = PerBetEvCalculationSpec {
        id: String::from("missing-no-monkey"),
        bet_type: BetType::Monkey,
        mode: Some(BetMode::Monkey(MonkeyMode::Standard)),
        odds: OddsSpec::by_outcome(BetType::Monkey, 0.0, MONKEY_ONLY_ODDS),
        rebate_rate: 0.0,
        effective_mode: EffectiveAmountMode::TotalStake,
    };
    let missing_error = calculate_ev(&standard_cards(), &[missing])
        .expect_err("standard Monkey mode should require both outcome odds");
    assert!(missing_error.contains("missing odds for outcome NoMonkey"));

    let irrelevant = PerBetEvCalculationSpec {
        id: String::from("irrelevant-monkey"),
        bet_type: BetType::Monkey,
        mode: Some(BetMode::Monkey(MonkeyMode::NoMonkeyOnly)),
        odds: OddsSpec::by_outcome(BetType::Monkey, 0.0, NO_MONKEY_PLUS_IRRELEVANT_ODDS),
        rebate_rate: 0.0,
        effective_mode: EffectiveAmountMode::TotalStake,
    };
    let irrelevant_error = calculate_ev(&standard_cards(), &[irrelevant])
        .expect_err("NoMonkeyOnly mode should reject Monkey odds");
    assert!(irrelevant_error.contains("irrelevant odds for outcome Monkey"));

    let strict_perfect_pair = PerBetEvCalculationSpec {
        id: String::from("missing-both-sides"),
        bet_type: BetType::PerfectPair,
        mode: Some(BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth)),
        odds: OddsSpec::by_outcome(BetType::PerfectPair, 25.0, PERFECT_PAIR_SINGLE_ONLY_ODDS),
        rebate_rate: 0.0,
        effective_mode: EffectiveAmountMode::TotalStake,
    };
    let strict_error = calculate_ev(&standard_cards(), &[strict_perfect_pair])
        .expect_err("SinglePlusBoth mode should require both PerfectPair outcome odds");
    assert!(strict_error.contains("missing odds for outcome PerfectPairBothSides"));
}

#[test]
fn ev_specs_accept_bet_type_oriented_odds_specs() {
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
    let specs = [
        PerBetEvCalculationSpec {
            id: String::from("player-simple"),
            bet_type: BetType::Player,
            mode: None,
            odds: OddsSpec::simple(BetType::Player, 1.0),
            rebate_rate: 0.0,
            effective_mode: EffectiveAmountMode::TotalStake,
        },
        PerBetEvCalculationSpec {
            id: String::from("monkey-outcomes"),
            bet_type: BetType::Monkey,
            mode: Some(BetMode::Monkey(MonkeyMode::Standard)),
            odds: OddsSpec::by_outcome(BetType::Monkey, 0.0, MONKEY_OUTCOME_ODDS),
            rebate_rate: 0.0,
            effective_mode: EffectiveAmountMode::TotalStake,
        },
    ];

    let result = calculate_ev(&standard_cards(), &specs)
        .expect("BetType-oriented odds specs should calculate EV");

    assert_eq!(result.len(), specs.len());
    assert_eq!(result[0].bet_type, BetType::Player);
    assert_eq!(result[1].bet_type, BetType::Monkey);
}

#[test]
fn ev_specs_can_explicitly_price_perfect_pair_both_sides_mode() {
    const PERFECT_PAIR_SINGLE_PLUS_BOTH_ODDS: &[OutcomeOdds] = &[
        OutcomeOdds {
            outcome: BetOutcome::PerfectPairSingleSide,
            odds: 25.0,
        },
        OutcomeOdds {
            outcome: BetOutcome::PerfectPairBothSides,
            odds: 200.0,
        },
    ];
    let specs = [PerBetEvCalculationSpec {
        id: String::from("perfect-pair-single-plus-both"),
        bet_type: BetType::PerfectPair,
        mode: Some(BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth)),
        odds: OddsSpec::by_outcome(
            BetType::PerfectPair,
            25.0,
            PERFECT_PAIR_SINGLE_PLUS_BOTH_ODDS,
        ),
        rebate_rate: 0.0,
        effective_mode: EffectiveAmountMode::TotalStake,
    }];

    let result = calculate_ev(&standard_cards(), &specs)
        .expect("explicit PerfectPair both-sides odds should calculate EV");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].bet_type, BetType::PerfectPair);
}

#[test]
fn ev_outcome_odds_validation_rejects_duplicate_outcomes() {
    const DUPLICATE_MONKEY_ODDS: &[OutcomeOdds] = &[
        OutcomeOdds {
            outcome: BetOutcome::Monkey,
            odds: 50.0,
        },
        OutcomeOdds {
            outcome: BetOutcome::Monkey,
            odds: 55.0,
        },
        OutcomeOdds {
            outcome: BetOutcome::NoMonkey,
            odds: 1.0,
        },
    ];

    let duplicate = PerBetEvCalculationSpec {
        id: String::from("duplicate-monkey"),
        bet_type: BetType::Monkey,
        mode: Some(BetMode::Monkey(MonkeyMode::Standard)),
        odds: OddsSpec::by_outcome(BetType::Monkey, 0.0, DUPLICATE_MONKEY_ODDS),
        rebate_rate: 0.0,
        effective_mode: EffectiveAmountMode::TotalStake,
    };
    let duplicate_error = calculate_ev(&standard_cards(), &[duplicate])
        .expect_err("duplicate outcome odds should fail");

    assert!(duplicate_error.contains("duplicate odds for outcome Monkey"));
}

#[test]
fn invalid_card_count_returns_fail_closed_error() {
    let invalid_cards = [CardCount {
        card: Card {
            suit: CardSuit::Clubs,
            rank: CardRank::Ace,
        },
        count: u32::from(STANDARD_DECK_COUNT) + 1,
    }];
    let error = calculate_probabilities(&invalid_cards).unwrap_err();
    assert!(error.contains("exceeds standard maximum"));
}

#[test]
fn standard_outcome_probabilities_sum_to_one() {
    let result = calculate_probabilities(&standard_cards())
        .expect("public probability API should calculate");

    let total = [BetType::Player, BetType::Banker, BetType::Tie]
        .into_iter()
        .map(|bet_type| result_by_bet_type(&result, bet_type).probability)
        .sum::<f64>();
    assert_close(total, 1.0, PROBABILITY_SUM_ABS_TOLERANCE);
}

#[test]
fn small_shoe_main_probabilities_match_independent_bruteforce_oracle() {
    let cards = vec![
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Ten),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Ace),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Two),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Three),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Four),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Five),
            count: 1,
        },
    ];
    let (expected_player, expected_banker, expected_tie) = brute_force_main_probabilities(&cards);
    let result = calculate_probabilities(&cards)
        .expect("small shoe should calculate main bet probabilities");

    assert_close(
        result_by_bet_type(&result, BetType::Player).probability,
        expected_player,
        1e-12,
    );
    assert_close(
        result_by_bet_type(&result, BetType::Banker).probability,
        expected_banker,
        1e-12,
    );
    assert_close(
        result_by_bet_type(&result, BetType::Tie).probability,
        expected_tie,
        1e-12,
    );
}

#[test]
fn small_shoe_perfect_pair_outcomes_match_independent_bruteforce_oracle() {
    let cards = vec![
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Ace),
            count: 2,
        },
        CardCount {
            card: card(CardSuit::Diamonds, CardRank::Two),
            count: 2,
        },
        CardCount {
            card: card(CardSuit::Hearts, CardRank::Three),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Spades, CardRank::Four),
            count: 1,
        },
    ];
    let expected = brute_force_opening_outcomes(&cards);
    let result = calculate_probabilities(&cards)
        .expect("small shoe should calculate PerfectPair outcome probabilities");

    assert_close(
        outcome_probability(
            &result,
            BetType::PerfectPair,
            BetOutcome::PerfectPairSingleSide,
        ),
        probability(expected.perfect_pair_single_side, expected.total),
        1e-12,
    );
    assert_close(
        outcome_probability(
            &result,
            BetType::PerfectPair,
            BetOutcome::PerfectPairBothSides,
        ),
        probability(expected.perfect_pair_both_sides, expected.total),
        1e-12,
    );
}

#[test]
fn small_shoe_monkey_outcomes_match_independent_bruteforce_oracle() {
    let cards = vec![
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Jack),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Diamonds, CardRank::Queen),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Hearts, CardRank::King),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Spades, CardRank::Jack),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Clubs, CardRank::Ace),
            count: 1,
        },
        CardCount {
            card: card(CardSuit::Diamonds, CardRank::Two),
            count: 1,
        },
    ];
    let expected = brute_force_opening_outcomes(&cards);
    let result = calculate_probabilities(&cards)
        .expect("small shoe should calculate Monkey outcome probabilities");

    assert_close(
        outcome_probability(&result, BetType::Monkey, BetOutcome::Monkey),
        probability(expected.monkey, expected.total),
        1e-12,
    );
    assert_close(
        outcome_probability(&result, BetType::Monkey, BetOutcome::NoMonkey),
        probability(expected.no_monkey, expected.total),
        1e-12,
    );
}
