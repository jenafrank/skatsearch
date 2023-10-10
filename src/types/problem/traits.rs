use super::Problem;

impl Default for Problem {
    fn default() -> Self {
        Problem { 
            declarer_cards_all: 0, 
            left_cards_all: 0, 
            right_cards_all: 0, 
            trick_cards: 0,
            trick_suit: 0,
            game_type: Default::default(), 
            start_player: Default::default(), 
            augen_total: 0, 
            nr_of_cards: 0, 
            points_to_win: 0,
            transposition_table: Default::default(), 
            counters: Default::default() 
        }
     }
}