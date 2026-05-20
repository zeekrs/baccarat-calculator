# AGENTS.md

## Repository Shape

- This is a Rust workspace extracted from `baccarat-next`, not the full app stack.
- Workspace members are only `crates/calculator` and `crates/types`; `calculator` depends on local `types` via `path = "../types"`.
- Root `Cargo.toml` sets `publish = false` and `license = "UNLICENSED"`; do not assume crates.io publishing is enabled.
- `target/` is ignored by `.gitignore`; never include build artifacts in changes.

## High-Value Commands

- Run focused verification from the workspace root:
  - `cargo test -p calculator`
  - `cargo check -p calculator`
- For EV-only changes, `cargo test -p calculator ev` is documented in `crates/calculator/README.md`, but probability/registry/default-odds changes should still run the full calculator test suite.

## Crate Boundaries

- `crates/types` owns baccarat card and public bet types in `src/baccarat.rs`.
- `crates/calculator/src/lib.rs` is the public API surface; it re-exports `Card`, `CardCount`, `CardRank`, `CardSuit`, registry types, odds types, and settlement types.
- Main calculator entrypoints are `standard_eight_deck_cards`, `calculate_probabilities`, and `calculate_ev`.
- Settlement entrypoints are `settle_bet` and `settle_bets` in `crates/calculator/src/settlement.rs`; money and settlement odds use `rust_decimal::Decimal`.

## Calculator Contracts

- `calculate_probabilities` takes only card counts and returns every registered canonical public bet; it is not a caller-selected bet API.
- `calculate_ev` takes the same card counts plus per-bet specs and preserves spec order in its output.
- `BetType` is the external canonical row; `BetOutcome` is a branch bucket; `BetMode` changes EV/settlement interpretation and must not change objective probability output.
- Do not leak server, strategy, request id, table context, or persistence DTO concepts into `calculator`; the crate is pure math plus settlement.
- Settlement validates amount scale, odds scale, mode compatibility, duplicate/irrelevant outcome odds, and missing required outcome odds; keep these fail-closed behaviors covered by tests.

## Tests And Fixtures

- `crates/calculator/tests/standard_baccarat_contract.rs` is the broad contract suite; it imports fixtures with `#[path = "fixtures/..."]`, not normal module layout.
- Source baseline fixtures cover the old source project's 41 public bets and 50 variants; tests intentionally treat newer public bets such as `Monkey` and `SuperTie0..9` separately.
- `crates/calculator/tests/default_odds.rs` verifies default odds coverage for every public EV bet; update it with registry/default odds changes.
- Probability and EV assertions use explicit tolerances such as `1e-10`, `1e-12`, and `1e-9`; avoid replacing them with exact float equality.

## Consumption Notes

- Cargo git dependencies can reference the repo URL and package name; Cargo traverses the git repo to find member `Cargo.toml` files.
- Because `calculator` has a local path dependency on `types`, keep both crates in this repository when pushing for other projects to consume by Git.
