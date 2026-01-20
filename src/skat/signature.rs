use crate::consts::bitboard::*;
use crate::traits::StringConverter;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HandSignature {
    // Jacks (Jacks bitmask: CJ=8, SJ=4, HJ=2, DJ=1)
    pub jacks: u8,
    // Count of Aces
    pub aces: u8,
    // Count of Tens
    pub tens: u8,
    // Count of Tens that are accompanied by the Ace of the same suit
    pub attached_tens: u8,
    // Count of suit combinations: 10 + King + (7, 8, or 9)
    pub ten_king_small: u8,
    // Total cards (should be 10 usually)
    // Total cards (should be 10 usually)
    pub card_count: u8,
    // Number of "Full" cards (Aces/Tens) in the Skat (if known/applicable)
    pub skat_fulls: u8,
    // Trump Count (for Suit games, includes Jacks + Suit cards)
    pub trump_count: u8,
    // General Analysis Metrics
    pub standing_tens: u8,
    pub blank_tens: u8,
    pub max_suit_len: u8,
}

impl HandSignature {
    pub fn from_hand(hand: u32) -> Self {
        Self::from_hand_and_skat(hand, 0)
    }

    pub fn from_hand_and_skat(hand: u32, skat: u32) -> Self {
        Self::from_hand_and_skat_suit(hand, skat, None)
    }

    pub fn from_hand_and_skat_suit(hand: u32, skat: u32, suit: Option<u8>) -> Self {
        let mut jacks: u8 = 0;
        if (hand & JACKOFCLUBS) != 0 {
            jacks |= 8;
        }
        if (hand & JACKOFSPADES) != 0 {
            jacks |= 4;
        }
        if (hand & JACKOFHEARTS) != 0 {
            jacks |= 2;
        }
        if (hand & JACKOFDIAMONDS) != 0 {
            jacks |= 1;
        }

        let mut aces = 0;
        let mut tens = 0;
        let mut attached_tens = 0;
        let mut ten_king_small = 0;
        let mut blank_tens = 0;
        let mut suit_lengths = [0u8; 4];
        // Start with Jacks
        let mut trump_count = (jacks as u32).count_ones() as u8;

        let suits = [
            (CLUBS, ACEOFCLUBS, TENOFCLUBS, KINGOFCLUBS),
            (SPADES, ACEOFSPADES, TENOFSPADES, KINGOFSPADES),
            (HEARTS, ACEOFHEARTS, TENOFHEARTS, KINGOFHEARTS),
            (DIAMONDS, ACEOFDIAMONDS, TENOFDIAMONDS, KINGOFDIAMONDS),
        ];

        for (i, (suit_mask, ace, ten, king)) in suits.iter().enumerate() {
            let is_trump_suit = if let Some(s) = suit {
                s as usize == i
            } else {
                false
            };

            // If explicit suit game:
            if is_trump_suit {
                let trumps_in_suit = ((hand & *suit_mask & !JACKS) as u32).count_ones() as u8;
                trump_count += trumps_in_suit;
            }

            // Count cards in suit (excluding Jacks) for max_suit_len
            let cards_in_suit = ((hand & *suit_mask & !JACKS) as u32).count_ones() as u8;
            suit_lengths[i] = cards_in_suit;

            let has_ace = (hand & ace) != 0;
            // If it is trump suit, Ace is a TRUMP (usually high trump), but for analysis we might want to separate "Trumps" vs "Side Aces".
            // User requested "Trump Length" and "Side Aces".
            // So if is_trump_suit, DO NOT increment `aces`.

            if !is_trump_suit {
                if has_ace {
                    aces += 1;
                }
                let has_ten = (hand & ten) != 0;
                if has_ten {
                    tens += 1;
                    if has_ace {
                        attached_tens += 1;
                    }
                    // Blank Ten = Ten is the ONLY card in that suit (excluding Jacks)
                    if cards_in_suit == 1 {
                        blank_tens += 1;
                    }
                }

                // Check for 10-K-Small logic
                // Need 10 AND King AND (one of 7, 8, 9 in that suit)
                if has_ten && (hand & king) != 0 {
                    let small_cards_mask = match *suit_mask {
                        CLUBS => NINEOFCLUBS | EIGHTOFCLUBS | SEVENOFCLUBS,
                        SPADES => NINEOFSPADES | EIGHTOFSPADES | SEVENOFSPADES,
                        HEARTS => NINEOFHEARTS | EIGHTOFHEARTS | SEVENOFHEARTS,
                        DIAMONDS => NINEOFDIAMONDS | EIGHTOFDIAMONDS | SEVENOFDIAMONDS,
                        _ => 0,
                    };

                    if (hand & small_cards_mask) != 0 {
                        ten_king_small += 1;
                    }
                }
            }
        }

        let mut max_suit_len = 0;
        for &len in &suit_lengths {
            if len > max_suit_len {
                max_suit_len = len;
            }
        }

        let mut skat_fulls = 0;
        let fulls_mask = ACEOFCLUBS
            | TENOFCLUBS
            | ACEOFSPADES
            | TENOFSPADES
            | ACEOFHEARTS
            | TENOFHEARTS
            | ACEOFDIAMONDS
            | TENOFDIAMONDS;
        let skat_full_cards = skat & fulls_mask;
        skat_fulls = skat_full_cards.count_ones() as u8;

        HandSignature {
            jacks,
            aces,
            tens,
            attached_tens,
            ten_king_small,
            card_count: hand.count_ones() as u8,
            skat_fulls,
            trump_count,
            standing_tens: attached_tens,
            blank_tens,
            max_suit_len,
        }
    }

    pub fn jacks_string(&self) -> String {
        let mut jacks_str = String::new();
        if (self.jacks & 8) != 0 {
            jacks_str.push('C');
        }
        if (self.jacks & 4) != 0 {
            jacks_str.push('S');
        }
        if (self.jacks & 2) != 0 {
            jacks_str.push('H');
        }
        if (self.jacks & 1) != 0 {
            jacks_str.push('D');
        }
        if jacks_str.is_empty() {
            jacks_str.push('-');
        }
        jacks_str
    }

    pub fn to_csv_header() -> String {
        "Cards,JacksMask,TrumpCount,Aces,Tens,AttachedTens,BlankTens,MaxSuitLen,TenKingSmall,SkatFulls,SkatCards,SkatPoints,WinProb".to_string()
    }

    pub fn to_csv_row(&self, hand: u32, skat: u32, win_prob: f32) -> String {
        let jacks_str = self.jacks_string();

        let cards_str = hand.__str();
        let skat_str = skat.__str();

        // Calculate Skat Points
        let mut skat_points = 0;
        for (mask, val) in GRAND_CONN {
            if (skat & mask) != 0 {
                skat_points += val;
            }
        }

        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{:.4}",
            cards_str,
            jacks_str,
            self.trump_count,
            self.aces,
            self.tens,
            self.attached_tens,
            self.blank_tens,
            self.max_suit_len,
            self.ten_king_small,
            self.skat_fulls,
            skat_str,
            skat_points,
            win_prob
        )
    }
}

impl std::fmt::Display for HandSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Jacks: {:04b}, Trumps: {}, Aces: {}, Tens: {}, Att.Tens: {}, 10-K-x: {}, SkatFulls: {}",
            self.jacks,
            self.trump_count,
            self.aces,
            self.tens,
            self.attached_tens,
            self.ten_king_small,
            self.skat_fulls
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_extraction() {
        // Hand: CJ, SJ, CA, CT, C7, HK, H8, D7, D8, D9 (10 cards)
        let hand = JACKOFCLUBS
            | JACKOFSPADES
            | ACEOFCLUBS
            | TENOFCLUBS
            | SEVENOFCLUBS
            | KINGOFHEARTS
            | EIGHTOFHEARTS
            | SEVENOFDIAMONDS
            | EIGHTOFDIAMONDS
            | NINEOFDIAMONDS;

        let sig = HandSignature::from_hand(hand);

        assert_eq!(sig.jacks, 12); // CJ(8) + SJ(4) = 12
        assert_eq!(sig.aces, 1); // CA only
        assert_eq!(sig.tens, 1); // CT only
        assert_eq!(sig.attached_tens, 1); // CT is with CA
        assert_eq!(sig.ten_king_small, 0); // CT has CA, but not King. HK has 8 but no 10.

        // Test Ten-King-Small
        // Hand: H10, HK, H7
        let hand2 = TENOFHEARTS | KINGOFHEARTS | SEVENOFHEARTS;
        let sig2 = HandSignature::from_hand(hand2);
        assert_eq!(sig2.tens, 1);
        assert_eq!(sig2.ten_king_small, 1);
    }
}
