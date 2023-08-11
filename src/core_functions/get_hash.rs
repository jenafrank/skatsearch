use crate::types::player::Player;
use crate::consts::general::{HSH_INIT, HSH_MUL};

/// Calculates hash 'FNV_DIRECT_FAST' style
pub fn get_hash(player: Player, cards: u32, trick_cards: u32) -> u64 {
    let mut hash = HSH_INIT;
    let mut x = cards as u64;
    let mut y = trick_cards as u64;

    hash ^= x & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x >>= 8;

    hash ^= x & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x >>= 8;

    hash ^= x & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    x >>= 8;

    hash ^= x & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);

    hash ^= y & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    y >>= 8;

    hash ^= y & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    y >>= 8;

    hash ^= y & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);
    y >>= 8;

    hash ^= y & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);

    hash ^= player as u64 & 0xFF;
    hash = hash.wrapping_mul(HSH_MUL);

    hash
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::types::player::Player;
    use crate::consts::bitboard::ACEOFSPADES;

    #[test]
    fn calc_one_hash() {
        let x = get_hash(Player::Declarer,ACEOFSPADES, 0u32);
        println!("{}",x);
        assert!(x > 0);
    }
}