use crate::{types::tt_entry::TtEntry, consts::general::TT_SIZE};

use super::tt_flag::TtFlag;

pub mod implementation;

#[derive(Clone)]
pub struct TtTable {
    pub data: Vec<TtEntry>
}

impl TtTable {
    pub fn new() -> TtTable {

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
                trick_cards: 0,
                value: 0,
                flag: TtFlag::EXACT,
                bestcard: 0
            });
        }

        x
    }

    pub fn add_entries(&mut self, other: TtTable) {
        for i in 0..TT_SIZE {
            if !self.data[i].occupied && other.data[i].occupied {
                self.data[i].occupied = true;
                self.data[i].player = other.data[i].player;
                self.data[i].left_cards = other.data[i].left_cards;
                self.data[i].right_cards = other.data[i].right_cards;
                self.data[i].declarer_cards = other.data[i].declarer_cards;
                self.data[i].trick_cards = other.data[i].trick_cards;
                self.data[i].value = other.data[i].value;
                self.data[i].flag = other.data[i].flag;
                self.data[i].bestcard = other.data[i].bestcard;
            }
        }
    }
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
        let tt = TtTable::new();

        assert_eq!(tt.data[0].occupied, false);
    }

    #[test]
    fn all_zero() {
        let tt: TtTable = TtTable::new();

        for i in 1..TT_SIZE {
            assert_eq!(tt.data[i].occupied, false);
            assert_eq!(tt.data[i].player, Player::default());
            assert_eq!(tt.data[i].left_cards, 0);
            assert_eq!(tt.data[i].right_cards, 0);
            assert_eq!(tt.data[i].declarer_cards, 0);
        }
    }

    #[test]
    fn add_one_entry() {
        let mut tt: TtTable = TtTable::new();

        let p = Problem::default();
        let s = State::create_initial_state_from_problem(&p);

        tt.write(&s,1,0,120,(1, 60));

        assert_eq!(tt.get_occupied_slots(), 1);
    }

    #[test]
    fn three_with_different_players() {
        let mut tt: TtTable = TtTable::new();

        let p = Problem::default();
        let s1 = State::create_initial_state_from_problem(&p);
        let mut s2 = s1;
        let mut s3 = s1;

        s2.player = Player::Left;
        s3.player = Player::Right;

        print_hash("S1", &s1);
        print_hash("S2", &s2);
        print_hash("S3", &s3);

        write_without_hash(&mut tt,&s1, 0, 120, 60);
        write_without_hash(&mut tt,&s2, 0, 120, 60);
        write_without_hash(&mut tt,&s3, 0, 120, 60);

        assert_eq!(tt.get_occupied_slots(), 3);
    }

    fn print_hash(prefix: &str, state: &State) {
        println!("{}-Hash: {}",prefix,get_hash(state.player, state.left_cards, state.right_cards, state.declarer_cards, state.trick_cards));
        println!("{}-Mapped Hash: {}",prefix,get_mapped_hash(state.player, state.left_cards, state.right_cards, state.declarer_cards, state.trick_cards));
    }

    fn write_without_hash(tt: &mut TtTable, state: &State, alpha: u8, beta: u8, value: u8) {
        let idx = get_mapped_hash(state.player,state.left_cards, state.right_cards, state.declarer_cards, state.trick_cards);
        tt.write(state, idx, alpha, beta, (0, value));
    }
}
