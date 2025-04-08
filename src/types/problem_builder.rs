use rand::seq::index::sample;
use crate::{traits::{BitConverter, Augen, Bitboard}, core_functions::get_all_unplayed_cards::get_all_unplayed_cards, uncertain::facts::Facts, consts::bitboard::*};
use super::{game::Game, player::Player, problem::Problem};

pub struct ProblemBuilder {
    declarer_cards: Option<u32>,
    left_cards: Option<u32>,
    right_cards: Option<u32>,
    game_type: Option<Game>,
    start_player: Option<Player>,
    threshold_upper: Option<u8>,
    trick_cards: Option<u32>,
    trick_suit: Option<u32>,
}

impl ProblemBuilder {

    pub fn new(game: Game) -> ProblemBuilder {
        let mut builder = ProblemBuilder::default();
        builder.game_type = Some(game);
        builder
    }

    pub fn new_farbspiel() -> ProblemBuilder {
        ProblemBuilder::new(Game::Farbe)
    }

    pub fn new_grand() -> ProblemBuilder {
        ProblemBuilder::new(Game::Grand)
    }

    pub fn new_null() -> ProblemBuilder {
        let builder = ProblemBuilder::new(Game::Null);
        builder.threshold(1)        
    }

    /// Assign cards to a specific player
    /// 
    pub fn cards(mut self, player: Player, cards: &str) -> ProblemBuilder {
        let cards_bit = cards.__bit();
        match player {
            Player::Declarer => self.declarer_cards = Some(cards_bit),
            Player::Left => self.left_cards = Some(cards_bit),
            Player::Right => self.right_cards = Some(cards_bit)
        }
        self
    }

    pub fn cards_all(mut self, declarer_cards: &str, left_cards: &str, right_cards: &str) -> ProblemBuilder {
        self.declarer_cards = Some(declarer_cards.__bit());
        self.left_cards = Some(left_cards.__bit());
        self.right_cards = Some(right_cards.__bit());
        self
    }

    pub fn turn(mut self, player: Player) -> ProblemBuilder {
        self.start_player = Some(player);
        self
    }

    pub fn threshold(mut self, threshold_upper: u8) -> ProblemBuilder {
        self.threshold_upper = Some(threshold_upper);
        self
    }

    pub fn threshold_half(mut self) -> ProblemBuilder {
        let declarer_cards = self.declarer_cards.expect("No declarer cards have been set.");
        let left_cards = self.left_cards.expect("No declarer cards have been set.");
        let right_cards = self.right_cards.expect("No declarer cards have been set.");


        let all_cards = get_all_unplayed_cards(declarer_cards, left_cards, right_cards);
        self.threshold_upper = Some((all_cards.__get_value() / 2) + 1);
        self
    }

    pub fn trick_from_uproblem(mut self, trick_card_previous_player: u32, trick_card_next_player: u32) -> ProblemBuilder {
        let trick_cards = trick_card_previous_player | trick_card_next_player;
        let leading_card = if trick_card_next_player > 0 { trick_card_next_player } else { trick_card_previous_player };
        self.trick_cards = Some( trick_cards );
        self.trick_suit = Some( leading_card );
        self
    }

    pub fn trick(mut self, trick_suit: u32, trick_cards: &str) -> ProblemBuilder{
        self.trick_cards = Some(trick_cards.__bit());
        self.trick_suit = Some(trick_suit);
        self
    }

    pub fn build(self) -> Problem {

        self.validate();

        let mut problem = Problem::new();

        if let Some(game_type) = self.game_type {
            problem.set_game_type(game_type);
        }

        if let Some(cards) = self.declarer_cards {
            problem.set_declarer_cards(cards);
        }

        if let Some(cards) = self.left_cards {
            problem.set_left_cards(cards);
        }

        if let Some(cards) = self.right_cards {
            problem.set_right_cards(cards);
        }

        if let Some(start_player) = self.start_player {
            problem.set_start_player(start_player);
        }

        if let Some(threshold_upper) = self.threshold_upper {
            problem.set_threshold_upper(threshold_upper);
        }

        if let Some(trick_cards) = self.trick_cards {
            problem.set_trick_cards(trick_cards);
        }

        if let Some(trick_suit) = self.trick_suit {
            problem.set_trick_suit(trick_suit);
        }

        problem
    }

    fn validate(&self) {

        let declarer_cards = self.declarer_cards.expect("Declarer cards missing.");
        let left_cards = self.left_cards.expect("Left cards missing.");
        let right_cards = self.right_cards.expect("Right cards missing.");
        let start_player = self.start_player.expect("No start player available.");
        let trick_cards = self.trick_cards.expect("No trick cards available.");
        
        assert_eq!(declarer_cards & left_cards, 0);
        assert_eq!(left_cards & right_cards, 0);
        assert_eq!(declarer_cards & right_cards, 0);

        // check card numbers with respect to cards_in_trick
        let nr_cards_declarer = declarer_cards.count_ones();
        let nr_cards_left = left_cards.count_ones();
        let nr_cards_right = right_cards.count_ones();

        assert_eq!(nr_cards_declarer, nr_cards_left);
        assert_eq!(nr_cards_left, nr_cards_right);

        let nr_cards_declarer_in_trick = (declarer_cards & trick_cards).count_ones();
        let nr_cards_left_in_trick = (left_cards & trick_cards).count_ones();
        let nr_cards_right_in_trick = (right_cards & trick_cards).count_ones();

        let nr_trick_cards = trick_cards.count_ones();

        assert!(nr_trick_cards <=2);

        if nr_trick_cards >= 1 {
            match start_player {
                Player::Declarer => assert_eq!(nr_cards_right_in_trick, 1),
                Player::Left => assert_eq!(nr_cards_declarer_in_trick, 1),
                Player::Right => assert_eq!(nr_cards_left_in_trick, 1),
            }
        }

        if nr_trick_cards == 2 {
            match start_player {
                Player::Declarer => assert_eq!(nr_cards_left_in_trick, 1),
                Player::Left => assert_eq!(nr_cards_right_in_trick, 1),
                Player::Right => assert_eq!(nr_cards_declarer_in_trick, 1),
            }
        }

        // ToDo: check threshold lower than total augen value
        //         
    }

    pub fn set_cards_for_problem(mut self, my_cards: u32, my_player: Player) -> ProblemBuilder {
        self.set_cards_for_problem_core(my_cards, my_player);
        self
    }

    pub fn set_cards_for_other_players(mut self, 
        all_cards: u32, 
        card_on_table_previous_player: u32, 
        card_on_table_next_player: u32,
        my_cards: u32, 
        my_player: Player, 
        next_player_facts: Facts, 
        previous_player_facts: Facts) -> ProblemBuilder {

            let cards_on_hands_of_both_other_players = all_cards & !my_cards;

            let mut cards_next_player = cards_on_hands_of_both_other_players;
            let mut cards_previous_player = cards_on_hands_of_both_other_players;

            let game_type = self.game_type.expect("No game type available.");
        
            cards_next_player = cancel_cards_with_facts(cards_next_player, next_player_facts, game_type);
            cards_previous_player = cancel_cards_with_facts(cards_previous_player, previous_player_facts, game_type);

            cards_next_player = cards_next_player & !card_on_table_previous_player;
            cards_previous_player = cards_previous_player & !card_on_table_next_player;
        
            let proposed_draw = self.draw_cards(cards_next_player, cards_previous_player, my_cards);
        
            cards_next_player = proposed_draw.0;
            cards_previous_player = proposed_draw.1;
        
            self.set_cards_for_problem_core(cards_next_player, my_player.inc());
            self.set_cards_for_problem_core(cards_previous_player, my_player.dec());    

            self    
    }

    fn set_cards_for_problem_core(&mut self, my_cards: u32, my_player: Player) {
        match my_player {
            Player::Declarer => self.declarer_cards = Some(my_cards),
            Player::Left => self.left_cards = Some(my_cards),
            Player::Right => self.right_cards = Some(my_cards),
        }        
    }

    fn draw_cards(&mut self, cards_player_1: u32, cards_player_2: u32, my_cards: u32) -> (u32, u32) {
        let nr_cards = my_cards.count_ones();
      
        let definite_cards_player_1 = cards_player_1 & !cards_player_2;    
        let definite_cards_player_2 = cards_player_2 & !cards_player_1;
    
        let ambiguous_cards = cards_player_1 & cards_player_2;
        let nr_ambiguous_cards = ambiguous_cards.count_ones();
        let nr_definite_cards_player_1 = definite_cards_player_1.count_ones();
        let nr_definite_cards_player_2 = definite_cards_player_2.count_ones();
        let nr_ambiguous_cards_player_1 = nr_cards - nr_definite_cards_player_1;
        let nr_ambiguous_cards_player_2 = nr_cards - nr_definite_cards_player_2;
    
        assert_eq!(nr_ambiguous_cards_player_1 + nr_ambiguous_cards_player_2, nr_ambiguous_cards);
    
        let draw_player_1 = random_cards(ambiguous_cards, nr_ambiguous_cards_player_1);
    
        let proposed_player_1 = definite_cards_player_1 | draw_player_1;
        let proposed_player_2 = definite_cards_player_2 | (ambiguous_cards & !draw_player_1);
    
        (proposed_player_1, proposed_player_2)    
    }
}

impl Default for ProblemBuilder {
    fn default() -> ProblemBuilder {
        ProblemBuilder {
            declarer_cards: None,
            left_cards: None,
            right_cards: None,
            game_type: None,
            start_player: None,
            threshold_upper: Some(1),
            trick_cards: Some(0),
            trick_suit: Some(0),
        }
    }
}

fn random_cards(cards: u32, nr: u32) -> u32 {
    let mut random_number_generator = rand::thread_rng();

    let cards_dec = cards.__decompose();
    assert!(cards_dec.1 >= nr as usize);

    let indices = sample(&mut random_number_generator, cards_dec.1, nr as usize);
    
    let mut ret = 0;
    for i in indices.iter() {
        ret |= cards_dec.0[i];
    }

    ret
}

fn cancel_cards_with_facts(cards: u32, facts: Facts, game: Game) -> u32 {
    let mut ret_cards = cards;

    if facts.no_trump {
        ret_cards = match game {
            Game::Farbe => ret_cards & !TRUMP_FARBE,
            Game::Grand => ret_cards & !TRUMP_GRAND,
            Game::Null => ret_cards & !TRUMP_NULL,
        }
    }

    if facts.no_clubs {
        ret_cards = match game {
            Game::Farbe => ret_cards & !TRUMP_FARBE,
            Game::Grand => ret_cards & !CLUBS,
            Game::Null => ret_cards & !NULL_CLUBS,
        }
    }

    if facts.no_spades {
        ret_cards = match game {
            Game::Farbe => ret_cards & !SPADES,
            Game::Grand => ret_cards & !SPADES,
            Game::Null => ret_cards & !NULL_SPADES,
        }
    }

    if facts.no_hearts {
        ret_cards = match game {
            Game::Farbe => ret_cards & !HEARTS,
            Game::Grand => ret_cards & !HEARTS,
            Game::Null => ret_cards & !NULL_HEARTS,
        }
    }

    if facts.no_diamonds {
        ret_cards = match game {
            Game::Farbe => ret_cards & !DIAMONDS,
            Game::Grand => ret_cards & !DIAMONDS,
            Game::Null => ret_cards & !NULL_DIAMONDS,
        }
    }

    ret_cards
}
