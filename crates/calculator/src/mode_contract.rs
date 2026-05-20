use crate::{BetMode, BetOutcome, BetType, MonkeyMode, PerfectPairMode};

pub(crate) fn winning_outcomes_for_mode(
    bet_type: BetType,
    mode: BetMode,
) -> Result<&'static [BetOutcome], String> {
    match (bet_type, mode) {
        (BetType::PerfectPair, BetMode::PerfectPair(PerfectPairMode::Standard)) => Ok(&[
            BetOutcome::PerfectPairSingleSide,
            BetOutcome::PerfectPairBothSides,
        ]),
        (BetType::PerfectPair, BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth)) => Ok(&[
            BetOutcome::PerfectPairSingleSide,
            BetOutcome::PerfectPairBothSides,
        ]),
        (BetType::Monkey, BetMode::Monkey(MonkeyMode::Standard)) => {
            Ok(&[BetOutcome::Monkey, BetOutcome::NoMonkey])
        }
        (BetType::Monkey, BetMode::Monkey(MonkeyMode::NoMonkeyOnly)) => Ok(&[BetOutcome::NoMonkey]),
        _ => Err(format!(
            "mode {mode:?} is incompatible with bet type {bet_type:?}"
        )),
    }
}

pub(crate) fn required_odds_for_mode(
    bet_type: BetType,
    mode: BetMode,
) -> Result<&'static [BetOutcome], String> {
    match (bet_type, mode) {
        (BetType::PerfectPair, BetMode::PerfectPair(PerfectPairMode::Standard)) => {
            Ok(&[BetOutcome::PerfectPairSingleSide])
        }
        (BetType::PerfectPair, BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth)) => Ok(&[
            BetOutcome::PerfectPairSingleSide,
            BetOutcome::PerfectPairBothSides,
        ]),
        (BetType::Monkey, BetMode::Monkey(MonkeyMode::Standard)) => {
            Ok(&[BetOutcome::Monkey, BetOutcome::NoMonkey])
        }
        (BetType::Monkey, BetMode::Monkey(MonkeyMode::NoMonkeyOnly)) => Ok(&[BetOutcome::NoMonkey]),
        _ => Err(format!(
            "mode {mode:?} is incompatible with bet type {bet_type:?}"
        )),
    }
}

pub(crate) fn allowed_outcome_odds_for_mode(
    bet_type: BetType,
    mode: BetMode,
) -> Result<&'static [BetOutcome], String> {
    required_odds_for_mode(bet_type, mode)
}

pub(crate) fn allows_compatibility_odds_for_required_outcome(
    bet_type: BetType,
    mode: BetMode,
    outcome: BetOutcome,
) -> bool {
    matches!(
        (bet_type, mode, outcome),
        (
            BetType::PerfectPair,
            BetMode::PerfectPair(PerfectPairMode::Standard),
            BetOutcome::PerfectPairSingleSide
        )
    )
}

pub(crate) fn settlement_allowed_outcomes(
    bet_type: BetType,
    mode: Option<BetMode>,
) -> &'static [BetOutcome] {
    match (bet_type, mode) {
        (
            BetType::PerfectPair,
            None
            | Some(BetMode::PerfectPair(PerfectPairMode::Standard | PerfectPairMode::SinglePlusBoth)),
        ) => &[
            BetOutcome::PerfectPairSingleSide,
            BetOutcome::PerfectPairBothSides,
        ],
        (BetType::Monkey, None | Some(BetMode::Monkey(MonkeyMode::Standard))) => {
            &[BetOutcome::Monkey, BetOutcome::NoMonkey]
        }
        (BetType::Monkey, Some(BetMode::Monkey(MonkeyMode::NoMonkeyOnly))) => {
            &[BetOutcome::NoMonkey]
        }
        (BetType::Tiger, None) => &[BetOutcome::TigerTwoCards, BetOutcome::TigerThreeCards],
        _ => &[],
    }
}

pub(crate) fn validate_mode_compatibility(
    bet_type: BetType,
    mode: Option<BetMode>,
) -> Result<(), String> {
    match (bet_type, mode) {
        (BetType::PerfectPair, None | Some(BetMode::PerfectPair(_))) => Ok(()),
        (BetType::Monkey, None | Some(BetMode::Monkey(_))) => Ok(()),
        (BetType::PerfectPair, Some(mode)) => {
            Err(format!("mode {mode:?} is incompatible with PerfectPair"))
        }
        (BetType::Monkey, Some(mode)) => Err(format!("mode {mode:?} is incompatible with Monkey")),
        (_, Some(mode)) => Err(format!("mode {mode:?} is incompatible with {:?}", bet_type)),
        (_, None) => Ok(()),
    }
}
