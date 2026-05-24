use calculator::{
	calculate_ev, calculate_probabilities, default_odds_specs, standard_eight_deck_cards,
	RebateBasis, EvSpec,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench_standard_probabilities(c: &mut Criterion) {
	let cards = standard_eight_deck_cards();

	c.bench_function("baccarat_calculator_standard_probabilities", |b| {
		b.iter(|| black_box(calculate_probabilities(black_box(&cards)).unwrap()))
	});
}

fn bench_standard_ev(c: &mut Criterion) {
	let cards = standard_eight_deck_cards();
	let specs: Vec<_> = default_odds_specs()
		.iter()
		.enumerate()
		.filter_map(|(index, odds)| {
			odds.odds().map(|_| EvSpec {
				id: format!("bench-{index}"),
				bet_type: odds.bet_type(),
				mode: None,
				odds: odds.clone(),
				rebate_rate: 0.01,
				effective_mode: RebateBasis::TotalStake,
			})
		})
		.collect();

	c.bench_function("baccarat_calculator_standard_ev_default_specs", |b| {
		b.iter(|| black_box(calculate_ev(black_box(&cards), black_box(&specs)).unwrap()))
	});
}

criterion_group!(benches, bench_standard_probabilities, bench_standard_ev);
criterion_main!(benches);
