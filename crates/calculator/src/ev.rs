use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{
    bet_registry::{public_probability_definitions, BetDefinition, BetId, BetType},
    mode_contract::{
        allowed_outcome_odds_for_mode, allows_compatibility_odds_for_required_outcome,
        required_odds_for_mode, winning_outcomes_for_mode,
    },
    odds::{default_odds_table, OddsSettlement, OddsSpec},
    probability::{
        calculate_probability_snapshot, outcome_probabilities_for_definition,
        outcome_probability_breakdown, variant_probabilities_for_definition, BetVariantProbability,
        OutcomeProbability, OutcomeProbabilityBreakdown, ProbabilitySnapshot,
    },
    shoe::ShoeCounts,
    BetMode, BetOutcome, CardCount, PerfectPairMode,
};

/// Controls which probability mass is eligible for rebate EV.
///
/// Real platforms typically refund the unit stake on a push/tie, so push
/// probability should not contribute to rebate. `Standard` encodes this
/// industry-default behavior and is the recommended choice for live tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectiveAmountMode {
    /// Industry-default rebate basis: pushes refund the stake (excluded from
    /// rebate) and Banker net odds absorb the house commission.
    ///
    /// - Banker: `P(lose) + P(win) × odds`. With `odds = 0.95` this yields
    ///   `P(player) + 0.95 × P(banker)`, matching commission-aware platforms.
    /// - Every other bet: `1 - P(push)`. Bets without a push (most side bets)
    ///   collapse to `1.0`, while Player, Dragon, Natural, etc. exclude their
    ///   refund probability.
    Standard,
    /// Treat the full unit stake as effective regardless of outcome.
    ///
    /// Only use this for platforms that pay rebate on the gross wager including
    /// pushes/refunds; otherwise prefer `Standard`.
    TotalStake,
    /// Exclude push or refund probability from the effective amount.
    ///
    /// Equivalent to `Standard` for non-Banker bets; differs from `Standard`
    /// for Banker because it ignores the commission discount on wins.
    NonRefund,
    /// Use only losing probability as the effective amount.
    ///
    /// Suitable when rebate is paid only on the portion of stake that the
    /// platform actually retains.
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
            // Default to the commission- and push-aware rebate basis; pushes
            // refund the stake and should not earn rebate on real platforms.
            effective_mode: EffectiveAmountMode::Standard,
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
    ///
    /// For `Simple`, `ByOutcome`, and `Variant` specs this is the caller-supplied
    /// net odds value. For `Aggregate` specs this is the probability-weighted
    /// average payout per winning unit, computed as
    /// `(base_ev + lose_probability) / win_probability` so that the summary row
    /// remains a single scalar. Consumers should not assume
    /// `win_probability × odds - lose_probability == base_ev` for aggregate
    /// rows; use the per-variant odds in the input spec for reverse derivation.
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
    variants: Vec<BetVariantProbability>,
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
            variants: variant_probabilities_for_definition(definition, probabilities)?,
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

    let (base_ev, odds) = if spec.mode.is_some() {
        // Outcome-bucket EV path: PerfectPair / Monkey style branched payouts.
        let odds = spec.odds.odds().ok_or_else(|| missing_odds_error(spec))?;
        (outcome_base_ev(spec, definition, decomposition)?, odds)
    } else if spec.odds.children().is_some() {
        // Aggregate EV path: each variant pays its own odds; weighted summary.
        let ev = aggregate_base_ev(spec, decomposition)?;
        let win_prob = decomposition.breakdown.win_probability;
        // Weighted-average payout per winning unit, used only for the summary
        // `odds` field; downstream rebate logic for Banker never enters this
        // branch because Banker is not an aggregate bet.
        let weighted_odds = if win_prob > 0.0 {
            (ev + decomposition.breakdown.lose_probability) / win_prob
        } else {
            0.0
        };
        (ev, weighted_odds)
    } else {
        // Simple / ByOutcome EV path: one net odds for every winning outcome.
        let odds = spec.odds.odds().ok_or_else(|| missing_odds_error(spec))?;
        (
            decomposition.breakdown.win_probability * odds
                - decomposition.breakdown.lose_probability,
            odds,
        )
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

fn aggregate_base_ev(
    spec: &PerBetEvCalculationSpec,
    decomposition: &BetEvDecomposition,
) -> Result<f64, String> {
    let children = spec
        .odds
        .children()
        .ok_or_else(|| format!("EV spec {} expected aggregate odds", spec.id))?;

    // Completeness invariant: aggregate EV is well-defined only when the odds
    // spec lists exactly the same variants the calculator knows about.
    //
    // Otherwise a missing winning variant gets folded into `lose_probability`
    // (since lose is derived from the full breakdown) and the EV silently
    // underestimates the bet. Catch the mismatch instead of letting it through.
    for child in children {
        let known = decomposition
            .variants
            .iter()
            .any(|variant| variant.variant == child.variant);
        if !known {
            return Err(format!(
                "EV spec {} aggregate child {:?} does not match any calculator variant for {:?}",
                spec.id, child.variant, spec.bet_type
            ));
        }
    }
    for variant in &decomposition.variants {
        let listed = children
            .iter()
            .any(|child| child.variant == variant.variant);
        if !listed {
            return Err(format!(
                "EV spec {} aggregate odds for {:?} is missing variant {:?}",
                spec.id, spec.bet_type, variant.variant
            ));
        }
    }

    let mut win_ev = 0.0;
    for child in children.iter().filter(|c| c.settlement == OddsSettlement::Net) {
        let probability = decomposition
            .variants
            .iter()
            .find(|variant| variant.variant == child.variant)
            .map(|variant| variant.probability)
            .expect("variant matched in completeness check above");
        win_ev += probability * child.odds;
    }

    Ok(win_ev - decomposition.breakdown.lose_probability)
}

fn missing_odds_error(spec: &PerBetEvCalculationSpec) -> String {
    format!(
        "EV spec {} for {:?} requires simple odds, outcome odds with a selected mode, \
         or aggregate odds with a child for every calculator variant",
        spec.id, spec.bet_type
    )
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
                // Banker rebate basis discounts the commission absorbed by the
                // net odds; callers must always provide a resolved odds value
                // when calculating Banker in Standard mode.
                let banker_odds = odds.expect(
                    "Standard mode for Banker requires resolved net odds; \
                     this is enforced by `calculate_per_bet_ev_result`",
                );
                breakdown.lose_probability + breakdown.win_probability * banker_odds
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
        if let Some(children) = spec.odds.children() {
            for child in children {
                if !child.odds.is_finite() || child.odds < 0.0 {
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
        if spec.odds.odds().is_none() && spec.mode.is_none() && spec.odds.children().is_none() {
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
