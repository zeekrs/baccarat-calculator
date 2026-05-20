use calculator::bet_registry::{public_bet_id_for_type, variant_bet_id};
use calculator::{
    bet_definitions, default_odds_specs, default_odds_table, public_probability_definitions,
    BetClass, BetId, BetOutcome, BetType, BetVariant, DragonVariant, OddsSettlement, OddsSpec,
};
use serde_json::json;
use std::collections::HashSet;

fn spec_by_bet_id(bet_id: BetId) -> OddsSpec {
    default_odds_table()
        .get(bet_id)
        .unwrap_or_else(|| panic!("missing default odds for {bet_id:?}"))
}

#[test]
fn default_odds_include_expected_values() {
    assert_eq!(spec_by_bet_id(BetId::Player).odds(), Some(1.0));
    assert_eq!(spec_by_bet_id(BetId::Banker).odds(), Some(0.95));
    assert_eq!(spec_by_bet_id(BetId::Tie).odds(), Some(8.0));
    assert_eq!(spec_by_bet_id(BetId::Panda8).odds(), Some(25.0));
    assert_eq!(spec_by_bet_id(BetId::Dragon7).odds(), Some(40.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie0).odds(), Some(150.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie1).odds(), Some(215.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie2).odds(), Some(220.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie3).odds(), Some(200.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie4).odds(), Some(120.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie5).odds(), Some(110.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie6).odds(), Some(45.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie7).odds(), Some(45.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie8).odds(), Some(80.0));
    assert_eq!(spec_by_bet_id(BetId::SuperTie9).odds(), Some(80.0));
    assert_eq!(spec_by_bet_id(BetId::PlayerDragonPoint9).odds(), Some(30.0));
    assert_eq!(spec_by_bet_id(BetId::Heaven9Both).odds(), Some(60.0));

    let player_dragon_push = spec_by_bet_id(BetId::PlayerDragonPush);
    assert_eq!(player_dragon_push.odds(), Some(0.0));
    assert_eq!(
        player_dragon_push.settlement(),
        Some(OddsSettlement::Refund)
    );

    assert!(matches!(
        spec_by_bet_id(BetId::Lucky7Aggregate),
        OddsSpec::Aggregate(_)
    ));
    assert!(matches!(
        spec_by_bet_id(BetId::SuperLucky7Aggregate),
        OddsSpec::Aggregate(_)
    ));
    assert_eq!(spec_by_bet_id(BetId::TreasureAll).odds(), Some(5.0));
}

#[test]
fn default_odds_cover_every_public_ev_bet_type() {
    for definition in public_probability_definitions() {
        let spec = default_odds_table()
            .get(definition.id)
            .unwrap_or_else(|| panic!("missing default odds for {:?}", definition.id));

        match definition.class {
            BetClass::AggregateBet if definition.id == BetId::TreasureAll => {
                assert_eq!(spec.odds(), Some(5.0));
                assert!(matches!(spec, OddsSpec::Simple(_)));
            }
            BetClass::AggregateBet => {
                let children = spec.children().unwrap_or_else(|| {
                    panic!("missing aggregate children for {:?}", definition.id)
                });
                let expected_children = bet_definitions()
                    .iter()
                    .filter(|candidate| {
                        candidate.bet_type() == definition.id.bet_type()
                            && candidate.variant.is_some()
                    })
                    .map(|candidate| candidate.id)
                    .collect::<Vec<_>>();
                let actual_children = children
                    .iter()
                    .map(|child| child.bet_id)
                    .collect::<Vec<_>>();

                assert_eq!(actual_children, expected_children);
                assert!(matches!(spec, OddsSpec::Aggregate(_)));
            }
            BetClass::TerminalPredicate | BetClass::OpeningTwoCombinator => {
                assert!(matches!(
                    spec,
                    OddsSpec::Simple(_) | OddsSpec::ByOutcome(_) | OddsSpec::Variant(_)
                ));
                assert!(spec.matches_definition(*definition));
            }
        }
    }
}

#[test]
fn default_odds_do_not_register_unsupported_probability_contracts() {
    let public_ids = public_probability_definitions()
        .map(|definition| definition.id)
        .collect::<HashSet<_>>();
    let registered_ids = bet_definitions()
        .iter()
        .map(|definition| definition.id)
        .collect::<HashSet<_>>();

    for spec in default_odds_specs() {
        let root_id = spec.bet_id();
        assert!(
            public_ids.contains(&root_id),
            "default odds registered non-public root bet {root_id:?}"
        );

        if let Some(children) = spec.children() {
            for child in children {
                assert!(
                    registered_ids.contains(&child.bet_id),
                    "default odds child {:?} is not registered",
                    child.bet_id
                );
            }
        }
    }
}

#[test]
fn registry_public_bet_id_mapping_is_authoritative_for_default_odds_roots() {
    for definition in public_probability_definitions() {
        let public_id = public_bet_id_for_type(definition.bet_type());

        assert_eq!(public_id, definition.id);
        assert_eq!(public_id.bet_type(), definition.bet_type());
        assert!(default_odds_table().contains(public_id));
    }
}

#[test]
fn registry_variant_mapping_matches_registered_variant_definitions() {
    for definition in bet_definitions()
        .iter()
        .filter(|definition| definition.variant.is_some())
    {
        let variant = definition.variant.unwrap();

        assert_eq!(
            variant_bet_id(definition.bet_type(), variant),
            definition.id
        );
    }
}

#[test]
fn default_odds_use_typed_variant_specs_for_refunds() {
    let push_spec = spec_by_bet_id(BetId::BankerDragonPush);

    assert!(matches!(
        push_spec,
        OddsSpec::Variant(variant)
            if variant.bet_id == BetId::BankerDragonPush
                && variant.variant == BetVariant::Dragon(DragonVariant::Push)
                && variant.settlement == OddsSettlement::Refund
    ));
}

#[test]
fn default_odds_include_outcome_odds_for_outcome_mode_bets() {
    let perfect_pair = spec_by_bet_id(BetId::PerfectPair);
    assert_eq!(perfect_pair.odds(), Some(25.0));
    assert!(matches!(perfect_pair, OddsSpec::ByOutcome(_)));
    assert_eq!(perfect_pair.outcome_odds().unwrap().len(), 1);
    assert_eq!(
        perfect_pair.odds_for_outcome(BetOutcome::PerfectPairSingleSide),
        Some(25.0)
    );
    assert_eq!(
        perfect_pair.odds_for_outcome(BetOutcome::PerfectPairBothSides),
        None
    );

    let monkey = spec_by_bet_id(BetId::Monkey);
    assert_eq!(monkey.odds(), Some(0.0));
    assert!(matches!(monkey, OddsSpec::ByOutcome(_)));
    assert_eq!(monkey.outcome_odds().unwrap().len(), 2);
    assert_eq!(monkey.odds_for_outcome(BetOutcome::Monkey), Some(50.0));
    assert_eq!(monkey.odds_for_outcome(BetOutcome::NoMonkey), Some(1.0));
}

#[test]
fn odds_specs_deserialize_owned_outcome_and_aggregate_children() {
    let outcome_spec: OddsSpec = serde_json::from_value(json!({
        "ByOutcome": {
            "bet_type": "Monkey",
            "odds": 0.0,
            "outcomes": [
                { "outcome": "Monkey", "odds": 50.0 },
                { "outcome": "NoMonkey", "odds": 1.0 }
            ]
        }
    }))
    .expect("ByOutcome odds spec should deserialize owned outcome odds");

    assert_eq!(outcome_spec.bet_type(), BetType::Monkey);
    assert_eq!(outcome_spec.outcome_odds().unwrap().len(), 2);
    assert_eq!(
        outcome_spec.odds_for_outcome(BetOutcome::Monkey),
        Some(50.0)
    );

    let aggregate_spec: OddsSpec = serde_json::from_value(json!({
        "Aggregate": {
            "bet_id": "Lucky6Aggregate",
            "children": [
                {
                    "bet_id": "Lucky6Two",
                    "variant": { "Lucky6": "Two" },
                    "odds": 12.0,
                    "settlement": "Net"
                },
                {
                    "bet_id": "Lucky6Three",
                    "variant": { "Lucky6": "Three" },
                    "odds": 20.0,
                    "settlement": "Net"
                }
            ]
        }
    }))
    .expect("Aggregate odds spec should deserialize owned children");

    let children = aggregate_spec.children().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].bet_id, BetId::Lucky6Two);
    assert_eq!(children[1].bet_id, BetId::Lucky6Three);
}

#[test]
fn default_odds_expose_super_tie_0_to_9_as_simple_public_odds() {
    let expected = [
        (BetId::SuperTie0, BetType::SuperTie0, 150.0),
        (BetId::SuperTie1, BetType::SuperTie1, 215.0),
        (BetId::SuperTie2, BetType::SuperTie2, 220.0),
        (BetId::SuperTie3, BetType::SuperTie3, 200.0),
        (BetId::SuperTie4, BetType::SuperTie4, 120.0),
        (BetId::SuperTie5, BetType::SuperTie5, 110.0),
        (BetId::SuperTie6, BetType::SuperTie6, 45.0),
        (BetId::SuperTie7, BetType::SuperTie7, 45.0),
        (BetId::SuperTie8, BetType::SuperTie8, 80.0),
        (BetId::SuperTie9, BetType::SuperTie9, 80.0),
    ];
    let public_super_ties = public_probability_definitions()
        .filter(|definition| format!("{:?}", definition.bet_type()).starts_with("SuperTie"))
        .collect::<Vec<_>>();

    assert_eq!(public_super_ties.len(), expected.len());

    for (bet_id, bet_type, odds) in expected {
        let definition = public_super_ties
            .iter()
            .find(|definition| definition.id == bet_id)
            .unwrap_or_else(|| panic!("missing public SuperTie definition for {bet_id:?}"));
        assert_eq!(definition.bet_type(), bet_type);

        let spec = spec_by_bet_id(bet_id);
        assert!(matches!(spec, OddsSpec::Simple(_)));
        assert_eq!(spec.odds(), Some(odds));
        assert!(spec.outcome_odds().is_none());
        assert!(spec.children().is_none());
    }
}
