# calculator

`crates/calculator` is the Rust-owned baccarat math crate. It exposes only pure calculation inputs and outputs: remaining card counts in, all canonical probability results out.

## Public API

```rust
use calculator::{calculate_probabilities, standard_eight_deck_cards};

let cards = standard_eight_deck_cards();
let result = calculate_probabilities(&cards)?;
```

- `CardCount` contains one exact card (`suit + rank`) and its remaining count.
- `calculate_probabilities` does not accept caller-provided bet selections. It returns every registered canonical bet the supplied card counts can calculate.
- `calculate_ev` accepts the same card counts plus per-bet EV specs. It returns one result per spec with `base_ev`, `rebate_ev`, `total_ev`, supplied odds, and the win, lose, push, and effective probabilities.
- Each probability result contains the canonical `bet_type`, its aggregate `probability`, any public `variants`, and any outcome buckets.

Core modeling terms:

- `BetType` is the external canonical bet row returned by `calculate_probabilities` and requested by `calculate_ev`, such as `Player`, `PerfectPair`, or `Monkey`.
- `BetOutcome` is a public outcome bucket inside one `BetType` when that bet needs branch probabilities or branch odds. Examples include `Monkey` and `NoMonkey` for `BetType::Monkey`, plus `PerfectPairSingleSide` and `PerfectPairBothSides` for `BetType::PerfectPair`.
- `BetMode` selects the EV or settlement interpretation for bets with multiple outcome semantics. It does not change the objective probability output from `calculate_probabilities`.
- `OddsSpec` describes odds input: `Simple` for one net odds value, `ByOutcome` for outcome odds on one bet, `Variant` for one child variant, and `Aggregate` for a family made from variant odds.

The crate does not expose `strategy_id`, `request_id`, authority, table context, recommendation envelopes, or server persistence DTOs. Those belong to `apps/server`.

## Canonical Naming Policy

- Results use canonical names directly, such as `Player`, `Banker`, `Dragon7`, `Panda8`, `Lucky7`, and `SuperLucky7`.
- Aggregate family selections use canonical family names such as `Lucky7`, `SuperLucky7`, `PlayerDragon`, `BankerDragon`, `Lucky6`, and `TreasureAll`.

## Card Input

- Callers pass card counts only. Rank and point counts are derived inside the calculator.
- Suits are explicit (`Clubs`, `Diamonds`, `Hearts`, `Spades`), so suit-dependent bets such as `PerfectPair`, red/black, `TigerPair`, and `Fortune4Pair` are calculated from the same input as rank-safe bets.

## Math Contract

- Standard shoe: finite 8-deck baccarat shoe, 416 cards total, 32 cards per rank.
- Rank values: A=1, 2-9 face value, 10/J/Q/K=0; hand totals are modulo 10.
- Drawing rules: natural 8/9 stands; Player draws 0-5 and stands 6-7; Banker follows standard third-card drawing rules.
- Precision policy: integer outcome counts are authoritative where available; tests use named absolute tolerances (`1e-12`) instead of raw float equality.

Other applications may consume this crate, but they must not copy probability math.

## EV Contract

EV is unit-stake only. The API has no variable stake input and returns no amount output. Odds are net odds paid on a winning unit, so `1.0` means win one unit of profit and `0.95` means win `0.95` units of profit.

Simple bets use this formula:

```text
base_ev = win_probability * net_odds - lose_probability
```

Push or refund outcomes contribute `0.0` to `base_ev`. `Player` and `Banker` treat `Tie` as push/refund. Dragon and Natural push variants have odds `0.0`, report their event probability as `push_probability`, and do not add to `win_probability`.

Rebate EV is per spec and uses the selected effective amount mode:

```text
rebate_ev = rebate_rate * effective_probability
total_ev = base_ev + rebate_ev
```

Effective amount modes are:

```text
Standard = 1.0 - push_probability, except Banker win uses resolved banker net odds as its effective amount
TotalStake = 1.0
NonRefund = 1.0 - push_probability
LosingOnly = lose_probability
```

## EV Spec Validation

`PerBetEvCalculationSpec` must have a nonblank `id`. Duplicate trimmed ids are rejected. `odds` values must be finite and nonnegative. `rebate_rate` must be finite and between `0.0` and `1.0`, inclusive. `bet_type` must be one supported public calculator bet type. An empty spec list returns an empty result after card counts are validated.

Outcome odds are validated against the selected `BetMode`. Missing required outcome odds are rejected, and irrelevant outcome odds for that mode are rejected.

The default `PerfectPair` odds model is single-side only: `PerfectPairSingleSide = 25.0`. Platforms that pay a separate both-sides outcome, such as `PerfectPairBothSides = 200.0`, must request `BetMode::PerfectPair(PerfectPairMode::SinglePlusBoth)` and provide explicit outcome odds.

Each spec maps to one concrete bet calculation request. Output order matches input order. Callers choose the odds for each requested bet; the Rust default odds table remains available for callers that want canonical defaults.

## EV Spec Example

Per-bet request:

```rust
use calculator::{BetType, EffectiveAmountMode, OddsSpec, PerBetEvCalculationSpec};

let spec = PerBetEvCalculationSpec {
    id: "player-main-floor".to_owned(),
    bet_type: BetType::Player,
    mode: None,
    odds: OddsSpec::simple(BetType::Player, 1.0),
    rebate_rate: 0.01,
    effective_mode: EffectiveAmountMode::TotalStake,
};

let result = calculate_ev(&cards, &[spec])?;
```

## Verification Commands

Run these from the workspace root after docs or contract changes:

```sh
cargo test -p calculator ev
cargo test -p calculator
cargo check -p calculator
```
