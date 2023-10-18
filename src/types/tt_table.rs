use crate::{types::tt_entry::TtEntry, consts::general::TT_SIZE};

use super::tt_flag::TtFlag;

pub mod implementation;

pub struct TtTable {
    pub data: Vec<TtEntry>
}

static mut TABLE_INSTANCE: Option<TtTable> = None;

impl TtTable {
    pub fn get_new() -> &'static TtTable {
        TtTable::reset();
        TtTable::get()
    }

    pub fn get() -> &'static TtTable {
        unsafe {
            TABLE_INSTANCE.get_or_insert_with(|| create_transposition_table())
        }
    }

    pub fn get_mutable() -> &'static mut TtTable {
        unsafe {
            TABLE_INSTANCE.get_or_insert_with(|| create_transposition_table())
        }
    }

    pub fn reset() {
        unsafe { TABLE_INSTANCE = None };
    }
}

fn create_transposition_table() -> TtTable {

    let mut x = TtTable {
        data: Vec::with_capacity(TT_SIZE)
    };

    for _ in 0..TT_SIZE {
        x.data.push(TtEntry {
            occupied: false,
            player: Default::default(),
            left_cards: 0,
            right_cards: 0,
            declarer_cards: 0,
            value: 0,
            flag: TtFlag::EXACT,
            bestcard: 0
        });
    }

    x
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::consts::general::TT_SIZE;
    use crate::types::player::Player;
    use crate::types::problem::Problem;
    use crate::types::state::State;
    use crate::core_functions::get_hash::get_hash;
    use crate::core_functions::get_mapped_hash::get_mapped_hash;

    #[test]
    fn element_zero() {
        TtTable::reset();

        assert_eq!(TtTable::get().data[0].occupied, false);
    }

    #[test]
    fn all_zero() {
        TtTable::reset();

        for i in 1..TT_SIZE {
            assert_eq!(TtTable::get().data[i].occupied, false);
            assert_eq!(TtTable::get().data[i].player, Player::default());
            assert_eq!(TtTable::get().data[i].left_cards, 0);
            assert_eq!(TtTable::get().data[i].right_cards, 0);
            assert_eq!(TtTable::get().data[i].declarer_cards, 0);
        }
    }

    #[test]
    fn add_one_entry() {
        TtTable::reset();

        let p = Problem::default();
        let s = State::create_initial_state_from_problem(&p);

        TtTable::get_mutable().write(&s,1,0,120,(1, 60));

        assert_eq!(TtTable::get().get_occupied_slots(), 1);
    }

    #[test]
    fn three_with_different_players() {
        TtTable::reset();

        let p = Problem::default();
        let s1 = State::create_initial_state_from_problem(&p);
        let mut s2 = s1;
        let mut s3 = s1;

        s2.player = Player::Left;
        s3.player = Player::Right;

        print_hash("S1", &s1);
        print_hash("S2", &s2);
        print_hash("S3", &s3);

        write_without_hash(&s1, 0, 120, 60);
        write_without_hash(&s2, 0, 120, 60);
        write_without_hash(&s3, 0, 120, 60);

        assert_eq!(TtTable::get().get_occupied_slots(), 3);
    }

    fn print_hash(prefix: &str, state: &State) {
        println!("{}-Hash: {}",prefix,get_hash(state.player, state.left_cards, state.right_cards, state.declarer_cards));
        println!("{}-Mapped Hash: {}",prefix,get_mapped_hash(state.player, state.left_cards, state.right_cards, state.declarer_cards));
    }

    fn write_without_hash(state: &State, alpha: u8, beta: u8, value: u8) {
        let idx = get_mapped_hash(state.player,state.left_cards, state.right_cards, state.declarer_cards);
        TtTable::get_mutable().write(state, idx, alpha, beta, (0, value));
    }
}