use crate::types::tt_entry::TtEntry;

pub mod constructors;
pub mod implementation;
pub mod traits;

pub struct TtTable {
    pub data: Vec<TtEntry>
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
        let x = TtTable::create();

        assert_eq!(x.data[0].occupied, false);
    }

    #[test]
    fn all_zero() {
        let x = TtTable::create();

        for i in 1..TT_SIZE {
            assert_eq!(x.data[i].occupied, false);
            assert_eq!(x.data[i].player, Player::default());
            assert_eq!(x.data[i].cards, 0);
        }
    }

    #[test]
    fn add_one_entry() {
        let mut x = TtTable::create();

        let p = Problem::default();
        let s = State::create_initial_state_from_problem(&p);

        x.write(&s,1,0,120,(1, 60));

        assert_eq!(x.get_occupied_slots(), 1);
    }

    #[test]
    fn three_with_different_players() {
        let mut x = TtTable::create();

        let p = Problem::default();
        let s1 = State::create_initial_state_from_problem(&p);
        let mut s2 = s1;
        let mut s3 = s1;

        s2.player = Player::Left;
        s3.player = Player::Right;

        print_hash("S1", &s1);
        print_hash("S2", &s2);
        print_hash("S3", &s3);

        x.write_without_hash(&s1, 0, 120, 60);
        x.write_without_hash(&s2, 0, 120, 60);
        x.write_without_hash(&s3, 0, 120, 60);

        assert_eq!(x.get_occupied_slots(), 3);
    }

    fn print_hash(prefix: &str, state: &State) {
        println!("{}-Hash: {}",prefix,get_hash(state.player, state.get_all_unplayed_cards(), state.trick_cards));
        println!("{}-Mapped Hash: {}",prefix,get_mapped_hash(state.player, state.get_all_unplayed_cards(), state.trick_cards));
    }
}