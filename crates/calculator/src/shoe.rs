use crate::{standard_baccarat, Card, CardCount, CardRank, CardSuit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShoeCounts {
    pub(crate) card: [u16; 52],
    pub(crate) rank: [u16; 13],
    pub(crate) point: [u16; 10],
    pub(crate) total: u16,
}

impl ShoeCounts {
    pub(crate) fn from_cards(cards: &[CardCount]) -> Result<Self, String> {
        card_counts_from_entries(cards)
    }
}

/// Returns a full standard baccarat shoe with eight copies of every exact card.
///
/// The result is suitable as the starting input for `calculate_probabilities`
/// and `calculate_ev` before any cards have been removed.
pub fn standard_eight_deck_cards() -> Vec<CardCount> {
    CardSuit::ALL
        .into_iter()
        .flat_map(|suit| {
            CardRank::ALL.into_iter().map(move |rank| CardCount {
                card: Card { suit, rank },
                count: u32::from(standard_baccarat::STANDARD_DECK_COUNT),
            })
        })
        .collect()
}

fn rank_label(rank_index: usize) -> &'static str {
    CardRank::ALL[rank_index].label()
}

fn card_counts_from_entries(cards: &[CardCount]) -> Result<ShoeCounts, String> {
    if cards.is_empty() {
        return Err(String::from(
            "calculator rejected cards: card counts must not be empty",
        ));
    }

    let mut counts = ShoeCounts {
        card: [0; 52],
        rank: [0; 13],
        point: [0; 10],
        total: 0,
    };

    for entry in cards {
        let (card_index, rank_index, point_index) = card_indexes(entry.card);
        if counts.card[card_index] != 0 {
            return Err(format!(
                "calculator rejected cards: duplicate card {:?}",
                entry.card
            ));
        }
        if entry.count > u32::from(standard_baccarat::STANDARD_DECK_COUNT) {
            return Err(format!(
                "calculator rejected cards: card {:?} count {} exceeds standard maximum {}",
                entry.card,
                entry.count,
                standard_baccarat::STANDARD_DECK_COUNT
            ));
        }

        let count = u16::try_from(entry.count).map_err(|_| {
            format!(
                "calculator rejected cards: card {:?} count {} exceeds supported range",
                entry.card, entry.count
            )
        })?;
        counts.card[card_index] = count;
        counts.rank[rank_index] = counts.rank[rank_index].checked_add(count).ok_or_else(|| {
            format!(
                "calculator rejected cards: rank '{}' count exceeds supported range",
                rank_label(rank_index)
            )
        })?;
        if counts.rank[rank_index]
            > u16::from(standard_baccarat::STANDARD_DECK_COUNT)
                * u16::from(standard_baccarat::SUITS_PER_DECK)
        {
            return Err(format!(
                "calculator rejected cards: rank '{}' count {} exceeds standard maximum {}",
                rank_label(rank_index),
                counts.rank[rank_index],
                u16::from(standard_baccarat::STANDARD_DECK_COUNT)
                    * u16::from(standard_baccarat::SUITS_PER_DECK)
            ));
        }
        counts.point[point_index] =
            counts.point[point_index]
                .checked_add(count)
                .ok_or_else(|| {
                    format!(
                        "calculator rejected cards: point '{}' count exceeds supported range",
                        point_index
                    )
                })?;
        counts.total = counts.total.checked_add(count).ok_or_else(|| {
            String::from("calculator rejected cards: remaining card count exceeds supported range")
        })?;
    }

    if counts.total > standard_baccarat::STANDARD_SHOE_CARD_COUNT {
        return Err(format!(
            "calculator rejected cards: remaining card count {} exceeds standard shoe",
            counts.total
        ));
    }

    Ok(counts)
}

fn card_indexes(card: Card) -> (usize, usize, usize) {
    let rank_index = card.rank.index();
    let suit_index = card.suit.index();
    let card_index = suit_index * usize::from(standard_baccarat::RANKS_PER_DECK) + rank_index;
    let point_index = usize::from(card.rank.baccarat_value());
    (card_index, rank_index, point_index)
}
