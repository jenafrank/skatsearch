use crate::consts::bitboard::CARDS;
use crate::traits::StringConverter; // For __str backup

pub fn format_hand_for_game(hand: u32, game_name: &str) -> String {
    match game_name {
        "Grand" => hand.__str(),
        "Null" => format_null(hand),
        "Clubs" => format_suit_game(hand, 3),    // 3=Clubs
        "Spades" => format_suit_game(hand, 2),   // 2=Spades
        "Hearts" => format_suit_game(hand, 1),   // 1=Hearts
        "Diamonds" => format_suit_game(hand, 0), // 0=Diamonds
        _ => hand.__str(),
    }
}

fn get_card_name(bit_index: u32) -> &'static str {
    // CARDS[0] = CJ (Bit 31)
    // CARDS[31] = D7 (Bit 0)
    CARDS[(31 - bit_index) as usize]
}

fn format_null(hand: u32) -> String {
    // Null Order: A, K, Q, J, 10, 9, 8, 7.
    // Group by Suit: Clubs, Spades, Hearts, Diamonds.

    let mut cards = Vec::new();

    // Clubs (Bits 21..27 + J=31)
    output_null_suit(&mut cards, hand, 21, 31);
    // Spades (Bits 14..20 + J=30)
    output_null_suit(&mut cards, hand, 14, 30);
    // Hearts (Bits 7..13 + J=29)
    output_null_suit(&mut cards, hand, 7, 29);
    // Diamonds (Bits 0..6 + J=28)
    output_null_suit(&mut cards, hand, 0, 28);

    format_card_vec(cards)
}

fn output_null_suit(out: &mut Vec<&'static str>, hand: u32, base_offset: u32, jack_bit: u32) {
    // Rank order: A(6), K(4), Q(3), J(jack_bit), 10(5), 9(2), 8(1), 7(0).

    // A (base+6)
    if (hand & (1 << (base_offset + 6))) != 0 {
        out.push(get_card_name(base_offset + 6));
    }
    // K (base+4)
    if (hand & (1 << (base_offset + 4))) != 0 {
        out.push(get_card_name(base_offset + 4));
    }
    // Q (base+3)
    if (hand & (1 << (base_offset + 3))) != 0 {
        out.push(get_card_name(base_offset + 3));
    }
    // J (jack_bit)
    if (hand & (1 << jack_bit)) != 0 {
        out.push(get_card_name(jack_bit));
    }
    // 10 (base+5)
    if (hand & (1 << (base_offset + 5))) != 0 {
        out.push(get_card_name(base_offset + 5));
    }
    // 9 (base+2)
    if (hand & (1 << (base_offset + 2))) != 0 {
        out.push(get_card_name(base_offset + 2));
    }
    // 8 (base+1)
    if (hand & (1 << (base_offset + 1))) != 0 {
        out.push(get_card_name(base_offset + 1));
    }
    // 7 (base+0)
    if (hand & (1 << (base_offset + 0))) != 0 {
        out.push(get_card_name(base_offset + 0));
    }
}

fn format_suit_game(hand: u32, trump_suit_idx: u32) -> String {
    // Suit Game Order: Jacks (C, S, H, D), then Trump Suit (A..7), then Side Suits.
    let mut cards = Vec::new();

    // 1. Jacks
    for j in (28..=31).rev() {
        // 31(CJ), 30(SJ), 29(HJ), 28(DJ)
        if (hand & (1 << j)) != 0 {
            cards.push(get_card_name(j));
        }
    }

    // 2. Trump Suit
    let base_offset = trump_suit_idx * 7;
    // Standard Skat Order bits: A(6), 10(5), K(4), Q(3), 9(2), 8(1), 7(0).
    // Iterating 6 down to 0 correctly produces this order.
    for i in (0..7).rev() {
        let bit = base_offset + i;
        if (hand & (1 << bit)) != 0 {
            cards.push(get_card_name(bit));
        }
    }

    // 3. Side Suits (Clubs(3), Spades(2), Hearts(1), Diamonds(0))
    // Skip trump_suit_idx.
    for suit_idx in (0..4).rev() {
        if suit_idx == trump_suit_idx {
            continue;
        }

        let base = suit_idx * 7;
        for i in (0..7).rev() {
            let bit = base + i;
            if (hand & (1 << bit)) != 0 {
                cards.push(get_card_name(bit));
            }
        }
    }

    format_card_vec(cards)
}

fn format_card_vec(cards: Vec<&str>) -> String {
    if cards.is_empty() {
        return "[]".to_string();
    }
    let mut s = String::from("[");
    s.push_str(&cards.join(" "));
    s.push(']');
    s
}
