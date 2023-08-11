use crate::types::state::State;
use crate::consts::bitboard::{CLUBS, DIAMONDS, HEARTS, JACKOFCLUBS, SPADES, TRUMP_GRAND};

pub fn get_sorted_by_zugebung(state: State, moves: u32) -> ([u32; 10], usize) {

    let mut card = JACKOFCLUBS;
    let mut premium= [0u32; 10];
    let mut elses= [0u32; 10];
    let mut all = [0u32; 10];

    let mut i = 0;
    let mut j = 0;
    while card > 0 {
        if moves & card > 0 {
            if zugebung(state, card)  {
                premium[i] = card;
                i += 1;
            } else {
                elses[j] = card;
                j +=1;
            }

        }
        card >>= 1;
    }

    let mut k=0;

    for el in &premium[0..i] {
        all[k] = el.clone();
        k += 1;
    }

    for el in &elses[0..j] {
        all[k] = el.clone();
        k += 1;
    }

    (all, k)
}

fn zugebung(state: State, card: u32) -> bool {
    zugeb(state, card, TRUMP_GRAND) ||
        zugeb(state, card, CLUBS) ||
        zugeb(state, card, SPADES) ||
        zugeb(state, card, HEARTS) ||
        zugeb(state, card, DIAMONDS)
}

fn zugeb(state: State, card: u32, mask: u32) -> bool {

    let cardm = card & mask > 0;
    let cardd = if (mask & state.declarer_cards) > 0 { 1 } else { 0 };
    let cardl = if (mask & state.left_cards) > 0 { 1 } else { 0 };
    let cardr = if (mask & state.right_cards) > 0 { 1 } else { 0 };

    cardm && cardd + cardl + cardr > 1
}