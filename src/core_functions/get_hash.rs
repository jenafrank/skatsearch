use crate::types::player::Player;
use crate::consts::general::{HSH_INIT, HSH_MUL};

/// Calculates hash 'FNV_DIRECT_FAST' style
pub fn get_hash(player: Player, left_cards: u32, right_cards: u32, declarer_cards: u32, trick_cards: u32) -> u64 {
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

#[cfg(test)]
mod tests {

    use super::*;

    use crate::types::player::Player;
    use crate::consts::bitboard::{ACEOFSPADES, ACEOFCLUBS, ACEOFHEARTS};

    #[test]
    fn calc_one_hash() {
        let x = get_hash(Player::Declarer,ACEOFSPADES, ACEOFCLUBS, ACEOFHEARTS, 0);
        println!("{}",x);
        assert!(x > 0);
    }
}