use crate::consts::bitboard::*;
use crate::traits::Bitboard;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;

pub struct UncertainProblem {
    
    pub game_type: Game,

    // Primary values    
    pub my_cards: u32,
    pub my_player: Player,
    pub all_cards: u32,
    pub points_to_win: u8,
    pub start_player: Player,    
 
    // Facts
    pub facts: [Facts; 2]
}

#[derive(Clone, Copy)]
pub struct Facts {     
    pub player: Player,   
    pub no_trump: bool,
    pub no_clubs: bool,
    pub no_spades: bool,
    pub no_hearts: bool,
    pub no_diamonds: bool
}

impl Facts {
    pub fn new() -> Self {
        Facts {
            player: Player::Declarer,
            no_trump: false,
            no_clubs: false,
            no_spades: false,
            no_hearts: false,
            no_diamonds: false,
    }
}
}


impl UncertainProblem {
    pub fn new() -> Self {
        UncertainProblem {
            game_type: Game::Farbe,

            // Primary values
            my_cards: 0u32,
            my_player: Player::Declarer,
            all_cards: 0u32,
            points_to_win: 0u8,
            start_player: Player::Declarer,    
        
            // Facts
            facts: [Facts::new(); 2]    
        }
    }

    pub fn generate_concrete_problem(&self) -> Problem {

        self.validate();

        let mut problem = Problem::new();

        problem.game_type = self.game_type;
        problem.start_player = self.start_player;

        set_cards_for_problem(&mut problem, self.my_cards, self.my_player);
        set_cards_for_other_players(&mut problem, self.all_cards, self.my_cards, self.facts);        
                
        if verify_card_distribution(&problem) {
            return problem
        } else {
            panic!("Something went wrong in randomly select cards with given facts.");
        }
    }

    fn validate(&self) {
        self.validate_facts();
        self.validate_all_cards();
    }

    fn validate_facts(&self) {
        assert!(self.my_player != self.facts[0].player);
        assert!(self.my_player != self.facts[1].player);
        assert!(self.facts[0].player != self.facts[1].player);
    }

    fn validate_all_cards(&self) {
        assert!(self.all_cards & self.my_cards == self.my_cards);
        
        // currently uncertain problems can only be solved before a trick starts:
        assert!(self.all_cards.count_ones() % 3 == 0); 
    }

}

fn verify_card_distribution(problem: &Problem) -> bool {

    assert!(problem.declarer_cards_all & problem.left_cards_all == 0);
    assert!(problem.declarer_cards_all & problem.right_cards_all == 0);
    assert!(problem.left_cards_all & problem.right_cards_all == 0);

    return true
}

fn set_cards_for_other_players(problem: &mut Problem, all_cards: u32, my_cards: u32, facts: [Facts; 2]) {

    let mut cards_player_1 = all_cards ^ my_cards;
    let mut cards_player_2 = cards_player_1;

    cards_player_1 = cancel_cards(cards_player_1, facts[0], problem.game_type);
    cards_player_2 = cancel_cards(cards_player_2, facts[1], problem.game_type);

    randomly_cancel_out_shared_cards(&mut cards_player_1, &mut cards_player_2);    

    set_cards_to_player(problem, cards_player_1, facts[0].player);
    set_cards_to_player(problem, cards_player_2, facts[1].player);
}

fn randomly_cancel_out_shared_cards(cards_player_1: &mut u32, cards_player_2: &mut u32) {
    
    let min_cards = (*cards_player_1 | *cards_player_2).count_ones() / 2;
    
    let common_cards = *cards_player_1 & *cards_player_2;
    let decomposed_common_cards = common_cards.__decompose();

    for i in 0..decomposed_common_cards.1 {
        let current_card = decomposed_common_cards.0[i];
        let random_number = rand::random::<u8>() % 2;
        let mut player_1_gets_card = random_number == 0;

        if (*cards_player_1).count_ones() == min_cards {
            player_1_gets_card = false;
        }

        if (*cards_player_2).count_ones() == min_cards {
            player_1_gets_card = true;
        }

        if player_1_gets_card {
            *cards_player_1 = *cards_player_1 & !current_card;
        } else {
            *cards_player_2 = *cards_player_2 & !current_card;
        }
    }
}

fn set_cards_to_player(problem: &mut Problem, cards: u32, player: Player) {
    match player {
        Player::Declarer => problem.declarer_cards_all = cards,
        Player::Left => problem.left_cards_all = cards,
        Player::Right => problem.right_cards_all = cards
    }
}

fn cancel_cards(cards: u32, facts: Facts, game: Game) -> u32 {
    
    let mut ret_cards = cards;

    if facts.no_trump {
        ret_cards = 
            match game {
                Game::Farbe => ret_cards & !TRUMP_FARBE,
                Game::Grand => ret_cards & !TRUMP_GRAND,
                Game::Null => ret_cards & !TRUMP_NULL,
            }
    }

    if facts.no_clubs {
        ret_cards = 
            match game {
                Game::Farbe => ret_cards & !TRUMP_FARBE,
                Game::Grand => ret_cards & !CLUBS,
                Game::Null => ret_cards & !NULL_CLUBS,
            }
    }

    
    if facts.no_spades {
        ret_cards = 
            match game {
                Game::Farbe => ret_cards & !SPADES,
                Game::Grand => ret_cards & !SPADES,
                Game::Null => ret_cards & !NULL_SPADES,
            }
    }

        
    if facts.no_hearts {
        ret_cards = 
            match game {
                Game::Farbe => ret_cards & !HEARTS,
                Game::Grand => ret_cards & !HEARTS,
                Game::Null => ret_cards & !NULL_HEARTS,
            }
    }

        
    if facts.no_diamonds {
        ret_cards = 
            match game {
                Game::Farbe => ret_cards & !DIAMONDS,
                Game::Grand => ret_cards & !DIAMONDS,
                Game::Null => ret_cards & !NULL_DIAMONDS,
            }
    }

    ret_cards
}

fn set_cards_for_problem(problem: &mut Problem, my_cards: u32, my_player: Player) {    
    match my_player {
        Player::Declarer => problem.declarer_cards_all = my_cards,
        Player::Left => problem.left_cards_all = my_cards,
        Player::Right => problem.right_cards_all = my_cards
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use crate::{traits::{BitConverter, StringConverter}, types::{game::Game, player::Player}, uncertain::uncertain_problem::Facts};
    use super::UncertainProblem;

    #[test]
    fn test_problem_generation() {
        let mut uproblem = UncertainProblem::new();

        uproblem.all_cards = "CA CT SA ST HA HT DA DT D9".__bit();
        uproblem.my_cards = "CA CT SA".__bit();
        uproblem.game_type = Game::Farbe;
        uproblem.my_player = Player::Declarer;
        uproblem.start_player = Player::Declarer;

        let mut fact1 = Facts::new();
        let mut fact2 = Facts::new();
        fact1.player = Player::Left;
        fact1.no_diamonds = true;
        fact2.player = Player::Right;        
        uproblem.facts = [fact1, fact2];

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards_all.__str());
        println!("Left cards    : {}", problem.left_cards_all.__str());
        println!("Right cards   : {}", problem.right_cards_all.__str());

    }
    
}

