use std::cmp;
use crate::types::game::Game;
use crate::types::state::State;
use crate::consts::bitboard::{CLUBS, DIAMONDS, HEARTS, JACKOFCLUBS, SPADES};
use crate::types::player::Player;
use crate::types::suit_data::SuitData;
use crate::traits::Bitboard;

/// Nur für Ausspiel-Fall (FORCING + BRANCH-REDUCING idea):
pub fn get_sorted_main(state: State, moves: u32, game: Game) -> ([u32; 10], usize) {

    let mut colorsuit: [SuitData; 4] = [Default::default(); 4];
    let mut ret: [u32; 10] = Default::default();
    let mut retsize: usize = 0;
    let mut remaining_moves = moves;

    let p1 = state.player;
    let p2 = state.player.inc();
    let p3 = state.player.inc().inc();

    let c1 = state.get_cards_for_player(p1);
    let c2 = state.get_cards_for_player(p2);
    let c3 = state.get_cards_for_player(p3);

    let trumpsuit = get_suit_analysis(game.get_trump(), c1, c2, c3);

    let mut nr_color_suits = 3usize;
    colorsuit[0] = get_suit_analysis(DIAMONDS, c1, c2, c3);
    colorsuit[1] = get_suit_analysis(HEARTS, c1, c2, c3);
    colorsuit[2] = get_suit_analysis(SPADES, c1, c2, c3);

    if matches!(game, Game::Grand) {
        colorsuit[3] = get_suit_analysis(CLUBS, c1, c2, c3);
        nr_color_suits = 4usize;
    }

    colorsuit.sort_by_key( |x| x.factor);

    // Prio 1: Trumpf ( Nur bei Alleinspieler )
    if matches!(state.player, Player::Declarer) {
        add_card(trumpsuit.suit_pattern, &mut remaining_moves, &mut ret, &mut retsize);
    }

    // Prio 2: Steht und Laeuft - alle muessen bedienen
    for i in 0..nr_color_suits {
        let item = colorsuit[i];
        if item.factor > 0 && item.ups_p1 > 0 {
            add_card(item.suit_pattern, &mut remaining_moves, &mut ret, &mut retsize);
        }
    }

    // Prio 3: Steht nicht, laeuft trotzdem
    for i in 0..nr_color_suits {
        let item = colorsuit[i];
        if item.factor > 0 {
            add_card(item.suit_pattern, &mut remaining_moves, &mut ret, &mut retsize);
        }
    }

    // Prio 4: Rest
    add_remaining_cards(remaining_moves, &mut ret, &mut retsize);

    (ret, retsize)
}

fn get_suit_analysis(suit: u32, cards_player: u32, cards_other_player_1: u32, cards_other_player_2: u32)
                     -> SuitData {

    let mut ret: SuitData = Default::default();

    ret.suit_pattern = suit;

    let mut s = suit;
    let mut c1 = cards_player;
    let mut c2 = cards_other_player_1;
    let mut c3 = cards_other_player_2;

    while s > 0 {

        // is element of suit
        if s.__is_odd()  {

            if c1.__is_odd() {

                ret.cnt_p1 += 1;
                ret.ups_p1 += 1;

                ret.ups_p2 = 0;
                ret.ups_p3 = 0;

            } else if c2.__is_odd() {

                ret.cnt_p2 += 1;
                ret.ups_p2 += 1;

                ret.ups_p1 = 0;
                ret.ups_p3 = 0;

            } else if c3.__is_odd() {

                ret.cnt_p3 += 1;
                ret.ups_p3 += 1;

                ret.ups_p1 = 0;
                ret.ups_p2 = 0;

            }
        }

        next_card(&mut s, &mut c1, &mut c2, &mut c3);
    }

    ret.factor = ret.cnt_p1 * ret.cnt_p2 * ret.cnt_p3 * 10;

    if ret.factor == 0 {
        ret.factor = cmp::max(ret.cnt_p1 * ret.cnt_p2, ret.cnt_p1 * ret.cnt_p3);
        ret.factor = cmp::max(ret.factor, ret.cnt_p2 * ret.cnt_p3);
    }

    ret
}

fn add_card(suit: u32, remaining_moves: &mut u32, arr: &mut [u32; 10], len: &mut usize) {
    let mut x = JACKOFCLUBS;

    loop {
        if x & *remaining_moves > 0 && x & suit > 0 {
            arr[*len] = x;
            *len += 1;
            *remaining_moves &= !x;
            break;
        }

        if x == 0 {
            break;
        }

        x >>= 1;
    }
}

fn add_remaining_cards(moves: u32, arr: &mut [u32; 10], len: &mut usize) {
    let mut x = JACKOFCLUBS;

    while x > 0 {
        if x & moves > 0 {
            arr[*len] = x;
            *len += 1;
        }
        x >>= 1;
    }
}

fn next_card(s: &mut u32, c1: &mut u32, c2: &mut u32, c3: &mut u32) {
    *s >>= 1;
    *c1 >>= 1;
    *c2 >>= 1;
    *c3 >>= 1;
}

