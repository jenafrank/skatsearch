use super::Problem;

impl Default for Problem {
    fn default() -> Self {
        Problem { 
            declarer_cards_all: 0, 
            left_cards_all: 0, 
            right_cards_all: 0, 
            game_type: Default::default(), 
            start_player: Default::default(), 
            augen_total: 0, 
            nr_of_cards: 0, 
            transposition_table: Default::default(), 
            counters: Default::default() 
        }
     }
}