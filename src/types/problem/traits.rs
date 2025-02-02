use super::Problem;

impl Default for Problem {
    fn default() -> Self {
        Problem { 
            declarer_cards: 0, 
            left_cards: 0, 
            right_cards: 0, 
            trick_cards: 0,
            trick_suit: 0,
            game_type: Default::default(), 
            start_player: Default::default(), 
            threshold_upper: 0
        }
     }
}