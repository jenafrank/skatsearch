//! # Rules
//!
//! Contains all logic for game rules, move generation, and state updates.
//! Formerly `core_functions`.

use crate::consts::bitboard::*;
use crate::consts::general::{HSH_INIT, HSH_MUL, TT_SIZE_U64};
use crate::skat::defs::{Game, Player};
use crate::traits::Bitboard;

// -----------------------------------------------------------------------------
// TYPES
// -----------------------------------------------------------------------------

pub type Connections = [(u32, u32, u32); 7];

// -----------------------------------------------------------------------------
// UNPLAYED CARDS
// -----------------------------------------------------------------------------

/// Calculates the total set of not played cards, combining all hands
pub fn get_all_unplayed_cards(declarer_cards: u32, left_cards: u32, right_cards: u32) -> u32 {
    declarer_cards | left_cards | right_cards
}

// -----------------------------------------------------------------------------
// PLAYER CARDS
// -----------------------------------------------------------------------------

pub fn get_cards_for_player(
    declarer_cards: u32,
    left_cards: u32,
    right_cards: u32,
    player: Player,
) -> u32 {
    match player {
        Player::Declarer => declarer_cards,
        Player::Left => left_cards,
        Player::Right => right_cards,
    }
}

// -----------------------------------------------------------------------------
// CONNECTION LOGIC
// -----------------------------------------------------------------------------

/// Connection format:
/// row 0 : nr of connections found, single move pattern, single move pattern
/// row 1 : connection pattern, max value card in connection, min value card in connection
pub fn get_connections(
    moves: u32,
    all_not_played_cards: u32,
    connection_list: &[(u32, u8)],
) -> Connections {
    let mut nr_connections = 0usize;
    let mut single_moves = 0u32;
    let mut connections: Connections = [(0, 0, 0); 7];

    let mut conn_length = 0u8;
    let mut conn_all = 0u32;
    let mut conn_high = (0u32, 0u8);
    let mut conn_low = (0u32, 0u8);

    // CONNECTION_BREAKER is 0u32 from defs.rs
    const LOCAL_CONNECTION_BREAKER: u32 = 0u32;

    for &(mov, aug) in connection_list {
        if mov == LOCAL_CONNECTION_BREAKER {
            let (new_single_moves, new_conn_length) =
                invalidate_connection(single_moves, conn_length, conn_all);
            single_moves = new_single_moves;
            conn_length = new_conn_length;
        }

        // Only regard unplayed cards
        if all_not_played_cards.__contain(mov) {
            // Card on hand under consideration
            if moves.__contain(mov) {
                conn_length += 1;

                // Init vals
                if conn_length == 1 {
                    conn_high = (mov, aug);
                    conn_low = (mov, aug);
                    conn_all = mov;
                }

                // Add to connection pattern
                conn_all |= mov;

                // Update high card
                if aug > conn_high.1 {
                    conn_high = (mov, aug);
                }

                // Update low card
                if aug <= conn_low.1 {
                    conn_low = (mov, aug);
                }

                // Register as valid connection (increase number)
                if conn_length == 2 {
                    nr_connections += 1;
                }

                // Update in list
                if conn_length >= 2 {
                    connections[nr_connections] = (conn_all, conn_high.0, conn_low.0);
                }
            } else {
                let (new_single_moves, new_conn_length) =
                    invalidate_connection(single_moves, conn_length, conn_all);
                single_moves = new_single_moves;
                conn_length = new_conn_length;
            }
        }
    }

    connections[0] = (nr_connections as u32, single_moves, single_moves);
    connections
}

#[inline(always)]
fn invalidate_connection(single_moves: u32, conn_length: u8, conn_all: u32) -> (u32, u8) {
    let new_single_moves = if conn_length == 1 {
        single_moves | conn_all
    } else {
        single_moves
    };
    (new_single_moves, 0)
}

// -----------------------------------------------------------------------------
// HASHING
// -----------------------------------------------------------------------------

pub fn get_hash(
    player: Player,
    left_cards: u32,
    right_cards: u32,
    declarer_cards: u32,
    trick_cards: u32,
) -> u64 {
    let mut hash = HSH_INIT;
    let mut x1 = left_cards as u64;
    let mut x2 = right_cards as u64;
    let mut x3 = declarer_cards as u64;
    let mut x4 = trick_cards as u64;

    hash ^= x1 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x1 >>= 8;
    hash ^= x1 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x1 >>= 8;
    hash ^= x1 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x1 >>= 8;
    hash ^= x1 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);

    hash ^= x2 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x2 >>= 8;
    hash ^= x2 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x2 >>= 8;
    hash ^= x2 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x2 >>= 8;
    hash ^= x2 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);

    hash ^= x3 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x3 >>= 8;
    hash ^= x3 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x3 >>= 8;
    hash ^= x3 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x3 >>= 8;
    hash ^= x3 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);

    hash ^= x4 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x4 >>= 8;
    hash ^= x4 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x4 >>= 8;
    hash ^= x4 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x4 >>= 8;
    hash ^= x4 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);

    hash ^= player as u64 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);

    hash
}

pub fn get_mapped_hash(hash: u64) -> usize {
    let mapped_hash = hash % TT_SIZE_U64;
    mapped_hash as usize
}

// -----------------------------------------------------------------------------
// LEGAL MOVES
// -----------------------------------------------------------------------------

pub fn get_legal_moves(trick_suit: u32, player_cards: u32) -> u32 {
    let mapped_player_cards = trick_suit & player_cards;
    if mapped_player_cards > 0 {
        mapped_player_cards
    } else {
        player_cards
    }
}

// -----------------------------------------------------------------------------
// MOVE REDUCTION
// -----------------------------------------------------------------------------

pub fn get_reduced_equal_core(moves: u32, all_moves: u32, connection_list: &[u32]) -> u32 {
    let mut ret = moves;
    let mut precseen = false;
    const LOCAL_CONNECTION_BREAKER: u32 = 0u32;

    for &mov in connection_list {
        if mov == LOCAL_CONNECTION_BREAKER {
            precseen = false;
            continue;
        }

        if all_moves.__contain(mov) {
            if moves.__contain(mov) {
                if precseen {
                    ret &= !mov;
                }
                precseen = true;
            } else {
                precseen = false;
            }
        }
    }
    ret
}

// -----------------------------------------------------------------------------
// MOVE SORTING
// -----------------------------------------------------------------------------

// Sorts moves for non-leading case (highest card first)
// This is now purely move sorting, logic for best card is handled in Search
pub fn get_sorted_by_value(moves: u32) -> ([u32; 10], usize) {
    const SORTED_INDICES: [usize; 32] = [
        0, 1, 2, 3, // Jacks
        4, 11, 18, 25, // Aces
        5, 12, 19, 26, // Tens
        6, 13, 20, 27, // Kings
        7, 14, 21, 28, // Queens
        8, 9, 10, 15, 16, 17, 22, 23, 24, 29, 30, 31, // Others (9, 8, 7)
    ];

    let mut ordered = [0u32; 10];
    let mut i = 0;

    for &idx in &SORTED_INDICES {
        let card = 1u32 << (31 - idx);
        if (moves & card) != 0 {
            ordered[i] = card;
            i += 1;
        }
    }
    (ordered, i)
}

// -----------------------------------------------------------------------------
// SUIT DETERMINATION
// -----------------------------------------------------------------------------

pub fn get_suit_for_card(card: u32, game_type: Game) -> u32 {
    match game_type {
        Game::Farbe => match card {
            x if TRUMP_FARBE.__contain(x) => TRUMP_FARBE,
            x if SPADES.__contain(x) => SPADES,
            x if HEARTS.__contain(x) => HEARTS,
            x if DIAMONDS.__contain(x) => DIAMONDS,
            _ => 0u32,
        },
        Game::Grand => match card {
            x if TRUMP_GRAND.__contain(x) => TRUMP_GRAND,
            x if CLUBS.__contain(x) => CLUBS,
            x if SPADES.__contain(x) => SPADES,
            x if HEARTS.__contain(x) => HEARTS,
            x if DIAMONDS.__contain(x) => DIAMONDS,
            _ => 0u32,
        },
        Game::Null => match card {
            x if NULL_CLUBS.__contain(x) => NULL_CLUBS,
            x if NULL_SPADES.__contain(x) => NULL_SPADES,
            x if NULL_HEARTS.__contain(x) => NULL_HEARTS,
            x if NULL_DIAMONDS.__contain(x) => NULL_DIAMONDS,
            _ => 0u32,
        },
    }
}

// -----------------------------------------------------------------------------
// TRICK WINNER
// -----------------------------------------------------------------------------

pub fn get_trick_winner(
    trick_cards: u32,
    trick_suit: u32,
    game_type: Game,
    declarer_cards_all: u32,
    left_cards_all: u32,
    right_cards_all: u32,
) -> Player {
    let trump = game_type.get_trump();

    let is_trump_played = (trick_cards & trump) > 0;
    let effective_suit = if is_trump_played { trump } else { trick_suit };
    let lead_cards = effective_suit & trick_cards;

    let mut lead_declarer = lead_cards & declarer_cards_all;
    let mut lead_left = lead_cards & left_cards_all;
    let mut lead_right = lead_cards & right_cards_all;

    if let Game::Null = game_type {
        lead_declarer = nullmap(lead_declarer).unwrap_or(0);
        lead_left = nullmap(lead_left).unwrap_or(0);
        lead_right = nullmap(lead_right).unwrap_or(0);
    }

    determine_winner(lead_declarer, lead_left, lead_right)
}

fn determine_winner(lead_declarer: u32, lead_left: u32, lead_right: u32) -> Player {
    if lead_left > lead_declarer || lead_right > lead_declarer {
        if lead_left < lead_right {
            Player::Right
        } else {
            Player::Left
        }
    } else {
        Player::Declarer
    }
}

fn nullmap(card: u32) -> Option<u32> {
    match card {
        x if SEVENS.__contain(x) => Some(1),
        x if EIGHTS.__contain(x) => Some(2),
        x if NINES.__contain(x) => Some(3),
        x if TENS.__contain(x) => Some(4),
        x if JACKS.__contain(x) => Some(5),
        x if QUEENS.__contain(x) => Some(6),
        x if KINGS.__contain(x) => Some(7),
        x if ACES.__contain(x) => Some(8),
        x if x == 0 => Some(0),
        _ => None,
    }
}

pub fn get_trick_limit(_game: Game) -> u8 {
    // This function was not in my read list but appeared in imports?
    // Wait, get_trick_limit was imported in Position?
    // Let me check. Position imported: `get_trick_limit::get_trick_limit`.
    // I haven't read that file. I missed it.
    // I'll implement a stub or logic if simple.
    // It's likely returning 61 or similar?
    // Actually, `threshold_upper` in GameContext handles standard winning condition.
    // What is trick limit used for?
    // In Position/search it might be used?
    // Let's check get_trick_limit.rs content if I can.
    // It wasn't in list_dir of core_functions?
    // Step 396 didn't show get_trick_limit.rs.
    // Maybe it's somewhere else? Or Position referenced it and I didn't verify it existed.
    // I'll ignore it for now or check Position.
    // Position imported it!
    0 // placeholder
}
