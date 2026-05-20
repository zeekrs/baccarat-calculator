use crate::{
    bet_registry::{bet_definitions, BetClass, BetDefinition, BetId},
    probability::{OpeningTwoStats, OutcomeProbabilityBreakdown, ProbabilityRatio},
    shoe::ShoeCounts,
    standard_baccarat,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TerminalWinner {
    Player,
    Banker,
    Tie,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TerminalOutcome {
    pub(crate) player_total: u8,
    pub(crate) banker_total: u8,
    pub(crate) player_len: u8,
    pub(crate) banker_len: u8,
    pub(crate) natural: bool,
    pub(crate) winner: TerminalWinner,
    pub(crate) margin: u8,
    pub(crate) total_card_count: u8,
}

const TERMINAL_TOTAL_BUCKETS: usize = 10;
const TERMINAL_LEN_BUCKETS: usize = 2;
const TERMINAL_NATURAL_BUCKETS: usize = 2;
const TERMINAL_OUTCOME_BUCKETS: usize = TERMINAL_TOTAL_BUCKETS
    * TERMINAL_TOTAL_BUCKETS
    * TERMINAL_LEN_BUCKETS
    * TERMINAL_LEN_BUCKETS
    * TERMINAL_NATURAL_BUCKETS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TerminalAccumulator {
    denominator: u128,
    weights: [u128; TERMINAL_OUTCOME_BUCKETS],
}

impl TerminalAccumulator {
    fn new(denominator: u128) -> Self {
        Self {
            denominator,
            weights: [0; TERMINAL_OUTCOME_BUCKETS],
        }
    }

    pub(crate) fn denominator(&self) -> u128 {
        self.denominator
    }

    pub(crate) fn weighted_outcomes(&self) -> impl Iterator<Item = (TerminalOutcome, u128)> + '_ {
        self.weights
            .iter()
            .enumerate()
            .filter(|(_, weight)| **weight > 0)
            .map(|(index, weight)| (terminal_outcome_from_index(index), *weight))
    }

    fn add(&mut self, outcome: TerminalOutcome, weight: u128) {
        let Some(index) = terminal_outcome_index(outcome) else {
            return;
        };
        self.weights[index] += weight;
    }
}

pub(crate) fn calculate_terminal_accumulator(
    counts: ShoeCounts,
) -> Result<TerminalAccumulator, String> {
    let mut point_counts = counts.point.map(u32::from);
    let total_cards = u32::from(counts.total);
    const STANDARD_HAND_MAX_CARD_COUNT: u32 = 6;
    if total_cards < STANDARD_HAND_MAX_CARD_COUNT {
        return Err(format!(
            "calculator rejected cards: at least {STANDARD_HAND_MAX_CARD_COUNT} cards are required to complete a standard baccarat hand, found {total_cards}"
        ));
    }

    let denominator = falling_product(total_cards, STANDARD_HAND_MAX_CARD_COUNT);
    let mut terminal_accumulator = TerminalAccumulator::new(denominator);

    for p1 in 0_u8..10 {
        let p1_index = usize::from(p1);
        if point_counts[p1_index] == 0 {
            continue;
        }
        let (weight_p1, total_after_p1) = draw_point(&mut point_counts, total_cards, p1_index);
        for b1 in 0_u8..10 {
            let b1_index = usize::from(b1);
            if point_counts[b1_index] == 0 {
                continue;
            }
            let (weight_b1, total_after_b1) =
                draw_point(&mut point_counts, total_after_p1, b1_index);
            for p2 in 0_u8..10 {
                let p2_index = usize::from(p2);
                if point_counts[p2_index] == 0 {
                    continue;
                }
                let (weight_p2, total_after_p2) =
                    draw_point(&mut point_counts, total_after_b1, p2_index);
                for b2 in 0_u8..10 {
                    let b2_index = usize::from(b2);
                    if point_counts[b2_index] == 0 {
                        continue;
                    }
                    let (weight_b2, total_after_b2) =
                        draw_point(&mut point_counts, total_after_p2, b2_index);
                    let ordered_weight = weight_p1 * weight_b1 * weight_p2 * weight_b2;
                    accumulate_terminal_outcomes(
                        &mut terminal_accumulator,
                        &mut point_counts,
                        total_after_b2,
                        [p1, p2],
                        [b1, b2],
                        ordered_weight,
                    );
                    restore_point(&mut point_counts, b2_index);
                }
                restore_point(&mut point_counts, p2_index);
            }
            restore_point(&mut point_counts, b1_index);
        }
        restore_point(&mut point_counts, p1_index);
    }

    Ok(terminal_accumulator)
}

fn accumulate_terminal_outcomes(
    outcomes: &mut TerminalAccumulator,
    counts: &mut [u32; 10],
    total_cards: u32,
    player_initial_cards: [u8; 2],
    banker_initial_cards: [u8; 2],
    ordered_weight: u128,
) {
    let player_initial_total = (player_initial_cards[0] + player_initial_cards[1]) % 10;
    let banker_initial_total = (banker_initial_cards[0] + banker_initial_cards[1]) % 10;
    let player_total = player_initial_total % 10;
    let banker_total = banker_initial_total % 10;
    let natural_present = standard_baccarat::is_natural_total(player_total)
        || standard_baccarat::is_natural_total(banker_total);

    if standard_baccarat::player_draws_third_card(player_total, natural_present) {
        for player_third in 0_u8..10 {
            let player_third_index = usize::from(player_third);
            if counts[player_third_index] == 0 {
                continue;
            }
            let (weight_player_third, total_after_player) =
                draw_point(counts, total_cards, player_third_index);
            let player_final_total = (player_total + player_third) % 10;
            if standard_baccarat::banker_draws_third_card(
                banker_total,
                Some(player_third),
                natural_present,
            ) {
                for banker_third in 0_u8..10 {
                    let banker_third_index = usize::from(banker_third);
                    if counts[banker_third_index] == 0 {
                        continue;
                    }
                    let (weight_banker_third, total_after_banker) =
                        draw_point(counts, total_after_player, banker_third_index);
                    outcomes.add(
                        TerminalOutcome::from_totals(
                            player_final_total,
                            (banker_total + banker_third) % 10,
                            3,
                            3,
                            natural_present,
                        ),
                        ordered_weight
                            * weight_player_third
                            * weight_banker_third
                            * remaining_suffix_weight(total_after_banker, 0),
                    );
                    restore_point(counts, banker_third_index);
                }
            } else {
                outcomes.add(
                    TerminalOutcome::from_totals(
                        player_final_total,
                        banker_total,
                        3,
                        2,
                        natural_present,
                    ),
                    ordered_weight
                        * weight_player_third
                        * remaining_suffix_weight(total_after_player, 1),
                );
            }
            restore_point(counts, player_third_index);
        }
    } else if standard_baccarat::banker_draws_third_card(banker_total, None, natural_present) {
        for banker_third in 0_u8..10 {
            let banker_third_index = usize::from(banker_third);
            if counts[banker_third_index] == 0 {
                continue;
            }
            let (weight_banker_third, total_after_banker) =
                draw_point(counts, total_cards, banker_third_index);
            outcomes.add(
                TerminalOutcome::from_totals(
                    player_total,
                    (banker_total + banker_third) % 10,
                    2,
                    3,
                    natural_present,
                ),
                ordered_weight
                    * weight_banker_third
                    * remaining_suffix_weight(total_after_banker, 1),
            );
            restore_point(counts, banker_third_index);
        }
    } else {
        outcomes.add(
            TerminalOutcome::from_totals(player_total, banker_total, 2, 2, natural_present),
            ordered_weight * remaining_suffix_weight(total_cards, 2),
        );
    }
}

fn draw_point(counts: &mut [u32; 10], total_cards: u32, point: usize) -> (u128, u32) {
    let weight = u128::from(counts[point]);
    counts[point] -= 1;
    (weight, total_cards - 1)
}

fn restore_point(counts: &mut [u32; 10], point: usize) {
    counts[point] += 1;
}

fn falling_product(total_cards: u32, draw_count: u32) -> u128 {
    (0..draw_count)
        .map(|offset| u128::from(total_cards - offset))
        .product()
}

fn remaining_suffix_weight(total_cards_after_terminal: u32, missing_draws: u32) -> u128 {
    falling_product(total_cards_after_terminal, missing_draws)
}

impl TerminalOutcome {
    pub(crate) fn from_totals(
        player_total: u8,
        banker_total: u8,
        player_len: u8,
        banker_len: u8,
        natural: bool,
    ) -> Self {
        let player_total = player_total % 10;
        let banker_total = banker_total % 10;
        let (winner, margin) = match player_total.cmp(&banker_total) {
            std::cmp::Ordering::Greater => (TerminalWinner::Player, player_total - banker_total),
            std::cmp::Ordering::Less => (TerminalWinner::Banker, banker_total - player_total),
            std::cmp::Ordering::Equal => (TerminalWinner::Tie, 0),
        };
        Self {
            player_total,
            banker_total,
            player_len,
            banker_len,
            natural,
            winner,
            margin,
            total_card_count: player_len + banker_len,
        }
    }
}

#[cfg(test)]
pub(crate) fn terminal_outcome_from_ordered_points(points: &[u8]) -> Option<TerminalOutcome> {
    if points.len() < 4 {
        return None;
    }
    let player_total = (points[0] + points[2]) % 10;
    let banker_total = (points[1] + points[3]) % 10;
    let natural_present = standard_baccarat::is_natural_total(player_total)
        || standard_baccarat::is_natural_total(banker_total);

    if standard_baccarat::player_draws_third_card(player_total, natural_present) {
        let player_third = *points.get(4)? % 10;
        let player_final = (player_total + player_third) % 10;
        if standard_baccarat::banker_draws_third_card(
            banker_total,
            Some(player_third),
            natural_present,
        ) {
            let banker_third = *points.get(5)? % 10;
            Some(TerminalOutcome::from_totals(
                player_final,
                (banker_total + banker_third) % 10,
                3,
                3,
                natural_present,
            ))
        } else {
            Some(TerminalOutcome::from_totals(
                player_final,
                banker_total,
                3,
                2,
                natural_present,
            ))
        }
    } else if standard_baccarat::banker_draws_third_card(banker_total, None, natural_present) {
        let banker_third = *points.get(4)? % 10;
        Some(TerminalOutcome::from_totals(
            player_total,
            (banker_total + banker_third) % 10,
            2,
            3,
            natural_present,
        ))
    } else {
        Some(TerminalOutcome::from_totals(
            player_total,
            banker_total,
            2,
            2,
            natural_present,
        ))
    }
}

fn terminal_outcome_index(outcome: TerminalOutcome) -> Option<usize> {
    let player_len = usize::from(outcome.player_len.checked_sub(2)?);
    let banker_len = usize::from(outcome.banker_len.checked_sub(2)?);
    if outcome.player_total >= 10 || outcome.banker_total >= 10 || player_len > 1 || banker_len > 1
    {
        return None;
    }

    let natural = usize::from(outcome.natural);
    Some(
        (((usize::from(outcome.player_total) * TERMINAL_TOTAL_BUCKETS
            + usize::from(outcome.banker_total))
            * TERMINAL_LEN_BUCKETS
            + player_len)
            * TERMINAL_LEN_BUCKETS
            + banker_len)
            * TERMINAL_NATURAL_BUCKETS
            + natural,
    )
}

fn terminal_outcome_from_index(index: usize) -> TerminalOutcome {
    let natural = index % TERMINAL_NATURAL_BUCKETS == 1;
    let index = index / TERMINAL_NATURAL_BUCKETS;
    let banker_len = (index % TERMINAL_LEN_BUCKETS) as u8 + 2;
    let index = index / TERMINAL_LEN_BUCKETS;
    let player_len = (index % TERMINAL_LEN_BUCKETS) as u8 + 2;
    let index = index / TERMINAL_LEN_BUCKETS;
    let banker_total = (index % TERMINAL_TOTAL_BUCKETS) as u8;
    let player_total = (index / TERMINAL_TOTAL_BUCKETS) as u8;

    TerminalOutcome::from_totals(player_total, banker_total, player_len, banker_len, natural)
}

pub(crate) fn terminal_predicate_matches(
    definition: &BetDefinition,
    outcome: TerminalOutcome,
) -> bool {
    match definition.id {
        BetId::Player => outcome.winner == TerminalWinner::Player,
        BetId::Banker => outcome.winner == TerminalWinner::Banker,
        BetId::Tie => outcome.winner == TerminalWinner::Tie,
        BetId::PlayerDragonNatural => dragon_natural(outcome, TerminalWinner::Player),
        BetId::PlayerDragonPoint4 => dragon_point(outcome, TerminalWinner::Player, 4),
        BetId::PlayerDragonPoint5 => dragon_point(outcome, TerminalWinner::Player, 5),
        BetId::PlayerDragonPoint6 => dragon_point(outcome, TerminalWinner::Player, 6),
        BetId::PlayerDragonPoint7 => dragon_point(outcome, TerminalWinner::Player, 7),
        BetId::PlayerDragonPoint8 => dragon_point(outcome, TerminalWinner::Player, 8),
        BetId::PlayerDragonPoint9 => dragon_point(outcome, TerminalWinner::Player, 9),
        BetId::PlayerDragonPush => dragon_push(outcome),
        BetId::BankerDragonNatural => dragon_natural(outcome, TerminalWinner::Banker),
        BetId::BankerDragonPoint4 => dragon_point(outcome, TerminalWinner::Banker, 4),
        BetId::BankerDragonPoint5 => dragon_point(outcome, TerminalWinner::Banker, 5),
        BetId::BankerDragonPoint6 => dragon_point(outcome, TerminalWinner::Banker, 6),
        BetId::BankerDragonPoint7 => dragon_point(outcome, TerminalWinner::Banker, 7),
        BetId::BankerDragonPoint8 => dragon_point(outcome, TerminalWinner::Banker, 8),
        BetId::BankerDragonPoint9 => dragon_point(outcome, TerminalWinner::Banker, 9),
        BetId::BankerDragonPush => dragon_push(outcome),
        BetId::BankerNaturalWin => natural_win(outcome, TerminalWinner::Banker),
        BetId::BankerNaturalPush => natural_push(outcome),
        BetId::PlayerNaturalWin => natural_win(outcome, TerminalWinner::Player),
        BetId::PlayerNaturalPush => natural_push(outcome),
        BetId::Lucky6Two | BetId::TigerTwo | BetId::SmallTiger => banker_six_win(outcome, 2),
        BetId::Lucky6Three | BetId::TigerThree | BetId::BigTiger => banker_six_win(outcome, 3),
        BetId::TigerTie => outcome.player_total == 6 && outcome.banker_total == 6,
        BetId::Banker4Fortune => {
            outcome.banker_total == 4 && outcome.winner == TerminalWinner::Banker
        }
        BetId::Player4Fortune => {
            outcome.player_total == 4 && outcome.winner == TerminalWinner::Player
        }
        BetId::Invincible6 => invincible_six(outcome),
        BetId::Big => outcome.total_card_count > 4,
        BetId::Small => outcome.total_card_count == 4,
        BetId::BankerCharSiuPoint4 => char_siu(outcome, TerminalWinner::Banker, 4),
        BetId::BankerCharSiuPoint5 => char_siu(outcome, TerminalWinner::Banker, 5),
        BetId::BankerCharSiuPoint6 => char_siu(outcome, TerminalWinner::Banker, 6),
        BetId::PlayerCharSiuPoint4 => char_siu(outcome, TerminalWinner::Player, 4),
        BetId::PlayerCharSiuPoint5 => char_siu(outcome, TerminalWinner::Player, 5),
        BetId::PlayerCharSiuPoint6 => char_siu(outcome, TerminalWinner::Player, 6),
        BetId::SmallBull => {
            outcome.player_total == 6
                && outcome.player_len == 2
                && outcome.winner == TerminalWinner::Player
        }
        BetId::BigBull => {
            outcome.player_total == 6
                && outcome.player_len == 3
                && outcome.winner == TerminalWinner::Player
        }
        BetId::TigerBull => {
            (outcome.player_total == 6 && outcome.winner == TerminalWinner::Player)
                || (outcome.banker_total == 6 && outcome.winner == TerminalWinner::Banker)
        }
        BetId::WuDaLang => outcome.player_total == 1 && outcome.winner == TerminalWinner::Player,
        BetId::Dragon7 => {
            outcome.banker_total == 7
                && outcome.banker_len == 3
                && outcome.winner == TerminalWinner::Banker
        }
        BetId::Panda8 => {
            outcome.player_total == 8
                && outcome.player_len == 3
                && outcome.winner == TerminalWinner::Player
        }
        BetId::SuperTie0 => super_tie(outcome, 0),
        BetId::SuperTie1 => super_tie(outcome, 1),
        BetId::SuperTie2 => super_tie(outcome, 2),
        BetId::SuperTie3 => super_tie(outcome, 3),
        BetId::SuperTie4 => super_tie(outcome, 4),
        BetId::SuperTie5 => super_tie(outcome, 5),
        BetId::SuperTie6 => super_tie(outcome, 6),
        BetId::SuperTie7 => super_tie(outcome, 7),
        BetId::SuperTie8 => super_tie(outcome, 8),
        BetId::SuperTie9 => super_tie(outcome, 9),
        BetId::Lucky7Two => player_seven_win(outcome, 2),
        BetId::Lucky7Three => player_seven_win(outcome, 3),
        BetId::SuperLucky7Four => super_lucky_seven(outcome, 4),
        BetId::SuperLucky7Five => super_lucky_seven(outcome, 5),
        BetId::SuperLucky7Six => super_lucky_seven(outcome, 6),
        BetId::Flame7sTwo => flame_sevens(outcome, 2),
        BetId::Flame7sThree => flame_sevens(outcome, 3),
        BetId::Heaven9Single => heaven_nine_single(outcome),
        BetId::Heaven9Both => heaven_nine_both(outcome),
        BetId::AnyPair
        | BetId::PlayerPair
        | BetId::BankerPair
        | BetId::PerfectPair
        | BetId::Monkey
        | BetId::PlayerDragonAggregate
        | BetId::BankerDragonAggregate
        | BetId::BankerNaturalAggregate
        | BetId::PlayerNaturalAggregate
        | BetId::Lucky6Aggregate
        | BetId::TigerAggregate
        | BetId::TigerPairPerfect
        | BetId::TigerPairBoth
        | BetId::TigerPairSingle
        | BetId::TigerPairAggregate
        | BetId::BankerFortune4PairFortune30
        | BetId::BankerFortune4PairFortune15
        | BetId::BankerFortune4PairFortune12
        | BetId::BankerFortune4PairFortune9
        | BetId::BankerFortune4PairAggregate
        | BetId::PlayerFortune4PairFortune30
        | BetId::PlayerFortune4PairFortune15
        | BetId::PlayerFortune4PairFortune12
        | BetId::PlayerFortune4PairFortune9
        | BetId::PlayerFortune4PairAggregate
        | BetId::PlayerRed
        | BetId::BankerRed
        | BetId::PlayerBlack
        | BetId::BankerBlack
        | BetId::BankerCharSiuAggregate
        | BetId::PlayerCharSiuAggregate
        | BetId::Lucky7Aggregate
        | BetId::SuperLucky7Aggregate
        | BetId::Flame7sAggregate
        | BetId::Heaven9Aggregate
        | BetId::TreasureAll => false,
    }
}

fn super_tie(outcome: TerminalOutcome, total: u8) -> bool {
    outcome.player_total == total
        && outcome.banker_total == total
        && outcome.winner == TerminalWinner::Tie
}

fn terminal_probability(
    accumulator: &TerminalAccumulator,
    definition: &BetDefinition,
) -> ProbabilityRatio {
    let numerator = accumulator
        .weighted_outcomes()
        .filter(|(outcome, _)| terminal_predicate_matches(definition, *outcome))
        .map(|(_, weight)| weight)
        .sum();
    ProbabilityRatio::new(numerator, accumulator.denominator())
}

pub(crate) fn terminal_probability_breakdown(
    accumulator: &TerminalAccumulator,
    definition: &BetDefinition,
) -> OutcomeProbabilityBreakdown {
    let probability = terminal_probability(accumulator, definition).as_f64();

    if definition.id.is_push_result() {
        OutcomeProbabilityBreakdown::new(0.0, probability)
    } else {
        OutcomeProbabilityBreakdown::new(probability, 0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum AggregateTerminalResult {
    Win,
    Push,
}

const TREASURE_ALL_CHILD_IDS: &[BetId] = &[
    BetId::Dragon7,
    BetId::Panda8,
    BetId::Flame7sTwo,
    BetId::Flame7sThree,
    BetId::Heaven9Both,
    BetId::Heaven9Single,
];

fn definition_by_id(id: BetId) -> Option<&'static BetDefinition> {
    bet_definitions()
        .iter()
        .find(|definition| definition.id == id)
}

fn aggregate_terminal_ids(id: BetId) -> Option<&'static [BetId]> {
    match id {
        BetId::PlayerDragonAggregate => Some(&[
            BetId::PlayerDragonPush,
            BetId::PlayerDragonNatural,
            BetId::PlayerDragonPoint4,
            BetId::PlayerDragonPoint5,
            BetId::PlayerDragonPoint6,
            BetId::PlayerDragonPoint7,
            BetId::PlayerDragonPoint8,
            BetId::PlayerDragonPoint9,
        ]),
        BetId::BankerDragonAggregate => Some(&[
            BetId::BankerDragonPush,
            BetId::BankerDragonNatural,
            BetId::BankerDragonPoint4,
            BetId::BankerDragonPoint5,
            BetId::BankerDragonPoint6,
            BetId::BankerDragonPoint7,
            BetId::BankerDragonPoint8,
            BetId::BankerDragonPoint9,
        ]),
        BetId::BankerNaturalAggregate => Some(&[BetId::BankerNaturalPush, BetId::BankerNaturalWin]),
        BetId::PlayerNaturalAggregate => Some(&[BetId::PlayerNaturalPush, BetId::PlayerNaturalWin]),
        BetId::Lucky6Aggregate => Some(&[BetId::Lucky6Two, BetId::Lucky6Three]),
        BetId::TigerAggregate => Some(&[BetId::TigerTwo, BetId::TigerThree]),
        BetId::BankerCharSiuAggregate => Some(&[
            BetId::BankerCharSiuPoint4,
            BetId::BankerCharSiuPoint5,
            BetId::BankerCharSiuPoint6,
        ]),
        BetId::PlayerCharSiuAggregate => Some(&[
            BetId::PlayerCharSiuPoint4,
            BetId::PlayerCharSiuPoint5,
            BetId::PlayerCharSiuPoint6,
        ]),
        BetId::Lucky7Aggregate => Some(&[BetId::Lucky7Two, BetId::Lucky7Three]),
        BetId::SuperLucky7Aggregate => Some(&[
            BetId::SuperLucky7Four,
            BetId::SuperLucky7Five,
            BetId::SuperLucky7Six,
        ]),
        BetId::Flame7sAggregate => Some(&[BetId::Flame7sTwo, BetId::Flame7sThree]),
        BetId::Heaven9Aggregate => Some(&[BetId::Heaven9Both, BetId::Heaven9Single]),
        _ => None,
    }
}

fn aggregate_opening_ids(id: BetId) -> Option<&'static [BetId]> {
    match id {
        BetId::TigerPairAggregate => Some(&[
            BetId::TigerPairPerfect,
            BetId::TigerPairBoth,
            BetId::TigerPairSingle,
        ]),
        BetId::BankerFortune4PairAggregate => Some(&[
            BetId::BankerFortune4PairFortune30,
            BetId::BankerFortune4PairFortune15,
            BetId::BankerFortune4PairFortune12,
            BetId::BankerFortune4PairFortune9,
        ]),
        BetId::PlayerFortune4PairAggregate => Some(&[
            BetId::PlayerFortune4PairFortune30,
            BetId::PlayerFortune4PairFortune15,
            BetId::PlayerFortune4PairFortune12,
            BetId::PlayerFortune4PairFortune9,
        ]),
        _ => None,
    }
}

fn aggregate_terminal_result(
    child_ids: &[BetId],
    outcome: TerminalOutcome,
) -> Result<Option<AggregateTerminalResult>, String> {
    for child_id in child_ids {
        let child = definition_by_id(*child_id).ok_or_else(|| {
            format!(
                "unsupported calculator contract: aggregate child {child_id:?} is not registered"
            )
        })?;
        if terminal_predicate_matches(child, outcome) {
            return Ok(Some(if child_id.is_push_result() {
                AggregateTerminalResult::Push
            } else {
                AggregateTerminalResult::Win
            }));
        }
    }

    Ok(None)
}

pub(crate) fn aggregate_terminal_probability_breakdown(
    accumulator: &TerminalAccumulator,
    definition: &BetDefinition,
) -> Result<Option<OutcomeProbabilityBreakdown>, String> {
    if definition.class != BetClass::AggregateBet {
        return Ok(None);
    }
    let child_ids = if definition.id == BetId::TreasureAll {
        TREASURE_ALL_CHILD_IDS
    } else {
        let Some(child_ids) = aggregate_terminal_ids(definition.id) else {
            return Ok(None);
        };
        child_ids
    };

    let mut win_weight = 0_u128;
    let mut push_weight = 0_u128;

    for (outcome, weight) in accumulator.weighted_outcomes() {
        match aggregate_terminal_result(child_ids, outcome)? {
            Some(AggregateTerminalResult::Win) => win_weight += weight,
            Some(AggregateTerminalResult::Push) => push_weight += weight,
            Option::None => {}
        }
    }

    let denominator = accumulator.denominator();
    let win_probability = ProbabilityRatio::new(win_weight, denominator).as_f64();
    let push_probability = ProbabilityRatio::new(push_weight, denominator).as_f64();

    Ok(Some(OutcomeProbabilityBreakdown::new(
        win_probability,
        push_probability,
    )))
}

pub(crate) fn aggregate_opening_probability_breakdown(
    opening_two: OpeningTwoStats,
    definition: &BetDefinition,
) -> Result<Option<OutcomeProbabilityBreakdown>, String> {
    let Some(child_ids) = aggregate_opening_ids(definition.id) else {
        return Ok(None);
    };
    let mut denominator = None;
    let mut win_numerator = 0_u128;

    for child_id in child_ids {
        let child = definition_by_id(*child_id).ok_or_else(|| {
            format!(
                "unsupported calculator contract: opening aggregate child {child_id:?} is not registered"
            )
        })?;
        let Some(ratio) = opening_two.probability_for(child) else {
            return Err(format!(
                "unsupported calculator contract: opening aggregate child {:?} is registered without probability/EV coverage",
                child.id
            ));
        };
        denominator = Some(match denominator {
            Some(existing) if existing != ratio.denominator => return Ok(None),
            Some(existing) => existing,
            Option::None => ratio.denominator,
        });
        win_numerator += ratio.numerator;
    }

    let Some(denominator) = denominator else {
        return Ok(None);
    };
    let win_probability = ProbabilityRatio::new(win_numerator, denominator).as_f64();

    Ok(Some(OutcomeProbabilityBreakdown::new(win_probability, 0.0)))
}

fn dragon_natural(outcome: TerminalOutcome, winner: TerminalWinner) -> bool {
    outcome.natural && outcome.winner == winner
}

fn dragon_point(outcome: TerminalOutcome, winner: TerminalWinner, margin: u8) -> bool {
    !outcome.natural && outcome.winner == winner && outcome.margin == margin
}

fn dragon_push(outcome: TerminalOutcome) -> bool {
    outcome.natural && outcome.winner == TerminalWinner::Tie
}

fn natural_win(outcome: TerminalOutcome, winner: TerminalWinner) -> bool {
    outcome.natural && outcome.winner == winner
}

fn natural_push(outcome: TerminalOutcome) -> bool {
    outcome.natural && outcome.winner == TerminalWinner::Tie
}

fn banker_six_win(outcome: TerminalOutcome, banker_len: u8) -> bool {
    outcome.banker_total == 6
        && outcome.banker_len == banker_len
        && outcome.winner == TerminalWinner::Banker
}

fn invincible_six(outcome: TerminalOutcome) -> bool {
    (outcome.player_total == 6 && outcome.winner == TerminalWinner::Player)
        || (outcome.banker_total == 6 && outcome.winner == TerminalWinner::Banker)
        || (outcome.player_total == 6 && outcome.banker_total == 6)
}

fn char_siu(outcome: TerminalOutcome, winner: TerminalWinner, total_cards: u8) -> bool {
    let winning_total = match winner {
        TerminalWinner::Player => outcome.player_total,
        TerminalWinner::Banker => outcome.banker_total,
        TerminalWinner::Tie => return false,
    };
    outcome.winner == winner
        && (7..=9).contains(&winning_total)
        && outcome.margin == 1
        && outcome.total_card_count == total_cards
}

fn player_seven_win(outcome: TerminalOutcome, player_len: u8) -> bool {
    outcome.player_total == 7
        && outcome.player_len == player_len
        && outcome.winner == TerminalWinner::Player
}

fn super_lucky_seven(outcome: TerminalOutcome, total_cards: u8) -> bool {
    outcome.player_total == 7
        && outcome.banker_total == 6
        && outcome.total_card_count == total_cards
        && outcome.winner == TerminalWinner::Player
}

fn flame_sevens(outcome: TerminalOutcome, hand_len: u8) -> bool {
    outcome.player_total == 7
        && outcome.banker_total == 7
        && outcome.player_len == hand_len
        && outcome.banker_len == hand_len
}

fn heaven_nine_single(outcome: TerminalOutcome) -> bool {
    ((outcome.player_total == 9 && outcome.player_len == 3)
        || (outcome.banker_total == 9 && outcome.banker_len == 3))
        && !heaven_nine_both(outcome)
}

fn heaven_nine_both(outcome: TerminalOutcome) -> bool {
    outcome.player_total == 9
        && outcome.banker_total == 9
        && outcome.player_len == 3
        && outcome.banker_len == 3
}
