use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{
    bet_registry::{public_probability_definitions, BetDefinition, BetId, BetType},
    mode_contract::{
        allowed_outcome_odds_for_mode, allows_compatibility_odds_for_required_outcome,
        required_odds_for_mode, winning_outcomes_for_mode,
    },
    odds::{default_odds_table, OddsSpec},
    probability::{
        calculate_probability_snapshot, outcome_probabilities_for_definition,
        outcome_probability_breakdown, OutcomeProbability, OutcomeProbabilityBreakdown,
        ProbabilitySnapshot,
    },
    shoe::ShoeCounts,
    BetMode, BetOutcome, CardCount, PerfectPairMode,
};

/// Controls which probability mass is eligible for rebate EV.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectiveAmountMode {
    /// Standard rebate basis for the bet, with Banker using resolved net odds.
    Standard,
    /// Treat the full unit stake as effective.
    TotalStake,
    /// Exclude push or refund probability from the effective amount.
    NonRefund,
    /// Use only losing probability as the effective amount.
    LosingOnly,
}

/// Caller-supplied EV request for one bet calculation.
///
/// Specs are validated independently and output rows preserve input order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerBetEvCalculationSpec {
    /// Caller-defined nonblank identifier copied into the result row.
    pub id: String,
    /// Caller-facing public bet to calculate.
    pub bet_type: BetType,
    /// Optional branch interpretation for outcome odds.
    pub mode: Option<BetMode>,
    /// Odds used for this spec instead of forcing default odds.
    pub odds: OddsSpec,
    /// Rebate rate from 0.0 through 1.0.
    pub rebate_rate: f64,
    /// Basis used to calculate `rebate_ev`.
    pub effective_mode: EffectiveAmountMode,
}

impl Default for PerBetEvCalculationSpec {
    fn default() -> Self {
        Self {
            id: String::new(),
            bet_type: BetType::Player,
            mode: None,
            odds: OddsSpec::simple(BetType::Player, 1.0),
            rebate_rate: 0.0,
            effective_mode: EffectiveAmountMode::TotalStake,
        }
    }
}

/// EV result for one `PerBetEvCalculationSpec`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerBetEvCalculationResult {
    /// Caller-defined spec identifier.
    pub id: String,
    /// Public bet calculated for this row.
    pub bet_type: BetType,
    /// Resolved net odds used for the base EV summary.
    pub odds: f64,
    /// Unit-stake EV before rebate.
    pub base_ev: f64,
    /// Unit-stake rebate contribution.
    pub rebate_ev: f64,
    /// Sum of `base_ev` and `rebate_ev`.
    pub total_ev: f64,
    /// Probability mass selected by `effective_mode`.
    pub effective_probability: f64,
    /// Probability that contributes winning odds.
    pub win_probability: f64,
    /// Probability that loses the unit stake.
    pub lose_probability: f64,
    /// Probability that returns stake without EV contribution.
    pub push_probability: f64,
}

pub(crate) struct BetEvDecomposition {
    definition: &'static BetDefinition,
    breakdown: OutcomeProbabilityBreakdown,
    outcomes: Vec<OutcomeProbability>,
}

/// Calculates caller-selected per-bet EV rows for the supplied card counts.
///
/// The same card-count validation and objective probabilities used by
/// `calculate_probabilities` apply here, but the result set is driven by
/// `specs`. Output order always matches input order.
pub fn calculate_ev(
    cards: &[CardCount],
    specs: &[PerBetEvCalculationSpec],
) -> Result<Vec<PerBetEvCalculationResult>, String> {
    validate_ev_specs(specs)?;

    let counts = ShoeCounts::from_cards(cards)?;
    if specs.is_empty() {
        return Ok(Vec::new());
    }

    let probabilities = calculate_probability_snapshot(counts)?;
    let decompositions = ev_outcome_decompositions(&probabilities)?;
    let mut results = Vec::with_capacity(specs.len());

    for spec in specs {
        results.push(calculate_per_bet_ev_result(spec, &decompositions)?);
    }

    Ok(results)
}

fn ev_outcome_decompositions(
    probabilities: &ProbabilitySnapshot,
) -> Result<Vec<BetEvDecomposition>, String> {
    let mut decompositions = Vec::new();

    for definition in public_probability_definitions() {
        let breakdown = outcome_probability_breakdown(definition, probabilities)?;
        if default_odds_table().get(definition.id).is_none() {
            return Err(format!("missing default odds for {:?}", definition.id));
        }

        decompositions.push(BetEvDecomposition {
            definition,
            breakdown,
            outcomes: outcome_probabilities_for_definition(definition, probabilities),
        });
    }

    Ok(decompositions)
}

fn calculate_per_bet_ev_result(
    spec: &PerBetEvCalculationSpec,
    decompositions: &[BetEvDecomposition],
) -> Result<PerBetEvCalculationResult, String> {
    let Some(decomposition) = decompositions
        .iter()
        .find(|decomposition| decomposition.definition.bet_type() == spec.bet_type)
    else {
        return Err(format!("unsupported bet type {:?}", spec.bet_type));
    };
    let definition = decomposition.definition;
    let odds = spec.odds.odds().ok_or_else(|| {
        format!(
            "EV spec {} for {:?} requires simple odds or a selected mode",
            spec.id, spec.bet_type
        )
    })?;
    let base_ev = if spec.mode.is_some() {
        outcome_base_ev(spec, definition, decomposition)?
    } else {
        decomposition.breakdown.win_probability * odds - decomposition.breakdown.lose_probability
    };
    let breakdown = decomposition.breakdown;
    let effective_probability =
        effective_probability_for_mode(spec.effective_mode, definition, &breakdown, Some(odds));
    let rebate_ev = spec.rebate_rate * effective_probability;

    Ok(PerBetEvCalculationResult {
        id: spec.id.clone(),
        bet_type: definition.bet_type(),
        odds,
        base_ev,
        rebate_ev,
        total_ev: base_ev + rebate_ev,
        effective_probability,
        win_probability: breakdown.win_probability,
        lose_probability: breakdown.lose_probability,
        push_probability: breakdown.push_probability,
    })
}

fn outcome_base_ev(
    spec: &PerBetEvCalculationSpec,
    definition: &BetDefinition,
    decomposition: &BetEvDecomposition,
) -> Result<f64, String> {
    let mode = spec
        .mode
        .ok_or_else(|| format!("EV spec {} is missing mode", spec.id))?;
    let winning_outcomes = winning_outcomes_for_mode(spec.bet_type, mode)?;
    validate_outcome_odds_for_mode(spec, definition.id)?;

    let mut win_ev = 0.0;
    for required in winning_outcomes {
        let probability = decomposition
            .outcomes
            .iter()
            .find(|candidate| candidate.outcome == *required)
            .map(|candidate| candidate.probability)
            .unwrap_or(0.0);
        let odds = odds_for_required_outcome(spec, mode, *required)?;
        win_ev += probability * odds;
    }

    Ok(win_ev - (1.0 - required_outcome_probability(decomposition, winning_outcomes)))
}

fn validate_outcome_odds_for_mode(
    spec: &PerBetEvCalculationSpec,
    bet_id: BetId,
) -> Result<(), String> {
    if spec.odds.bet_id() != bet_id {
        return Err(format!(
            "EV spec {} odds bet {:?} does not match {:?}",
            spec.id,
            spec.odds.bet_id(),
            bet_id
        ));
    }

    let Some(mode) = spec.mode else {
        return Ok(());
    };
    let required_odds = required_odds_for_mode(spec.bet_type, mode)?;
    let allowed_odds = allowed_outcome_odds_for_mode(spec.bet_type, mode)?;
    let outcomes = spec.odds.outcome_odds().unwrap_or(&[]);

    for required in required_odds {
        if spec.odds.odds_for_outcome(*required).is_none()
            && !allows_compatibility_odds_for_required_outcome(spec.bet_type, mode, *required)
        {
            return Err(format!(
                "EV spec {} is missing odds for outcome {:?}",
                spec.id, required
            ));
        }
    }

    for (index, outcome) in outcomes.iter().enumerate() {
        if !allowed_odds.contains(&outcome.outcome) {
            return Err(format!(
                "EV spec {} has irrelevant odds for outcome {:?}",
                spec.id, outcome.outcome
            ));
        }
        if outcomes
            .iter()
            .skip(index + 1)
            .any(|candidate| candidate.outcome == outcome.outcome)
        {
            return Err(format!(
                "EV spec {} has duplicate odds for outcome {:?}",
                spec.id, outcome.outcome
            ));
        }
    }

    Ok(())
}

fn odds_for_required_outcome(
    spec: &PerBetEvCalculationSpec,
    mode: BetMode,
    outcome: BetOutcome,
) -> Result<f64, String> {
    let odds = match mode {
        BetMode::PerfectPair(PerfectPairMode::Standard) => spec
            .odds
            .odds_for_outcome(BetOutcome::PerfectPairSingleSide)
            .or_else(|| spec.odds.odds()),
        _ => spec.odds.odds_for_outcome(outcome),
    }
    .ok_or_else(|| format!("EV spec {} is missing odds", spec.id))?;
    if !odds.is_finite() || odds < 0.0 {
        return Err(format!("EV spec {} has invalid odds", spec.id));
    }
    Ok(odds)
}

fn required_outcome_probability(
    decomposition: &BetEvDecomposition,
    required_outcomes: &[BetOutcome],
) -> f64 {
    decomposition
        .outcomes
        .iter()
        .filter(|candidate| required_outcomes.contains(&candidate.outcome))
        .map(|candidate| candidate.probability)
        .sum()
}

fn effective_probability_for_mode(
    mode: EffectiveAmountMode,
    definition: &BetDefinition,
    breakdown: &OutcomeProbabilityBreakdown,
    odds: Option<f64>,
) -> f64 {
    match mode {
        EffectiveAmountMode::Standard => {
            if definition.id == BetId::Banker {
                breakdown.lose_probability + breakdown.win_probability * odds.unwrap_or(1.0)
            } else {
                1.0 - breakdown.push_probability
            }
        }
        EffectiveAmountMode::TotalStake => 1.0,
        EffectiveAmountMode::NonRefund => 1.0 - breakdown.push_probability,
        EffectiveAmountMode::LosingOnly => breakdown.lose_probability,
    }
}

fn validate_ev_specs(specs: &[PerBetEvCalculationSpec]) -> Result<(), String> {
    let mut ids = HashSet::new();

    for spec in specs {
        let id = spec.id.trim();
        if id.is_empty() {
            return Err(String::from("EV spec id cannot be blank"));
        }
        if !ids.insert(id.to_owned()) {
            return Err(format!("duplicate EV spec id: {id}"));
        }
        if spec.odds.bet_type() != spec.bet_type {
            return Err(format!("EV spec {id} odds do not match bet type"));
        }
        if let Some(odds) = spec.odds.odds() {
            if !odds.is_finite() || odds < 0.0 {
                return Err(format!("EV spec {id} has invalid odds"));
            }
        }
        if let Some(outcomes) = spec.odds.outcome_odds() {
            for outcome in outcomes {
                if !outcome.odds.is_finite() || outcome.odds < 0.0 {
                    return Err(format!("EV spec {id} has invalid odds"));
                }
            }
        }
        if let Some(mode) = spec.mode {
            winning_outcomes_for_mode(spec.bet_type, mode)?;
            let definition = public_probability_definitions()
                .find(|definition| definition.bet_type() == spec.bet_type)
                .ok_or_else(|| format!("unsupported bet type {:?}", spec.bet_type))?;
            validate_outcome_odds_for_mode(spec, definition.id)?;
        }
        if spec.odds.odds().is_none() && spec.mode.is_none() {
            return Err(format!("EV spec {id} has invalid odds"));
        }
        if !spec.rebate_rate.is_finite() || spec.rebate_rate < 0.0 || spec.rebate_rate > 1.0 {
            return Err(format!("EV spec {id} has invalid rebate rate"));
        }
        if !public_probability_definitions()
            .any(|definition| definition.bet_type() == spec.bet_type)
        {
            return Err(format!("unsupported bet type {:?}", spec.bet_type));
        }
    }

    Ok(())
}
