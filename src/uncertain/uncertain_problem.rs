use rand::seq::index::sample;

use crate::consts::bitboard::*;
use crate::traits::{Bitboard, Augen};
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;

use super::facts::Facts;

pub struct UncertainProblem {
    game_type: Game,
    my_player: Player,
    my_cards: u32,
    next_player: Player,

    // Primary values
    cards_on_table: u32,
    all_cards: u32,
    active_suit: u32,    
    threshold_upper: u8,

    // Facts
    facts_declarer: Facts,
    facts_left: Facts,
    facts_right: Facts
}

// Gettter
impl UncertainProblem {
    pub fn game_type(&self) -> Game {
        self.game_type
    }

    pub fn my_player(&self) -> Player {
        self.my_player
    }

    pub fn my_cards(&self) -> u32 {
        self.my_cards
    }

    pub fn next_player(&self) -> Player {
        self.next_player
    }

    pub fn cards_on_table(&self) -> u32 {
        self.cards_on_table
    }

    pub fn all_cards(&self) -> u32 {
        self.all_cards
    }

    pub fn active_suit(&self) -> u32 {
        self.active_suit
    }

    pub fn threshold_upper(&self) -> u8 {
        self.threshold_upper
    }

    pub fn facts_declarer(&self) -> Facts {
        self.facts_declarer
    }

    pub fn facts_left(&self) -> Facts {
        self.facts_left
    }

    pub fn facts_right(&self) -> Facts {
        self.facts_right
    }
}

// Setter
impl UncertainProblem {
    pub fn set_game_type(&mut self, game_type: Game) {
        self.game_type = game_type;
    }

    pub fn set_my_player(&mut self, my_player: Player) {
        self.my_player = my_player;
    }

    pub fn set_my_cards(&mut self, my_cards: u32) {
        self.my_cards = my_cards;
    }

    pub fn set_next_player(&mut self, next_player: Player) {
        self.next_player = next_player;
    }

    pub fn set_cards_on_table(&mut self, cards_on_table: u32) {
        self.cards_on_table = cards_on_table;
    }

    pub fn set_all_cards(&mut self, all_cards: u32) {
        self.all_cards = all_cards;
    }

    pub fn set_active_suit(&mut self, active_suit: u32) {
        self.active_suit = active_suit;
    }

    pub fn set_threshold_upper(&mut self, threshold_upper: u8) {
        self.threshold_upper = threshold_upper;
    }

    pub fn set_facts_declarer(&mut self, facts_declarer: Facts) {
        self.facts_declarer = facts_declarer;
    }

    pub fn set_facts_left(&mut self, facts_left: Facts) {
        self.facts_left = facts_left;
    }

    pub fn set_facts_right(&mut self, facts_right: Facts) {
        self.facts_right = facts_right;
    }
}

impl UncertainProblem {
    pub fn new() -> Self {
        UncertainProblem {
            game_type: Game::Farbe,
            my_cards: 0u32,
            my_player: Player::Declarer,
            next_player: Player::Declarer,
            all_cards: 0u32,
            cards_on_table: 0u32,
            active_suit: 0u32,
            threshold_upper: 1u8,
            facts_declarer: Facts::zero_fact(),
            facts_left: Facts::zero_fact(),
            facts_right: Facts::zero_fact()
        }
    }

    pub fn generate_concrete_problem(&self) -> Problem {
        self.validate();

        let mut problem = Problem {
            declarer_cards_all: 0u32,
            left_cards_all: 0u32,
            right_cards_all: 0u32,
            game_type: self.game_type,
            start_player: self.next_player,
            trick_cards: self.cards_on_table,
            trick_suit: self.active_suit,
            augen_total: 0,
            nr_of_cards: 0,
            points_to_win: self.threshold_upper,
        };

        set_cards_for_problem(&mut problem, self.my_cards, self.my_player);
        set_cards_for_other_players(&mut problem, self.all_cards, self.cards_on_table, self.my_cards, self.my_player, 
            self.next_player_facts(), self.previous_player_facts());

        problem.augen_total = (problem.declarer_cards_all | problem.left_cards_all | problem.right_cards_all).__get_value();
        problem.nr_of_cards = (problem.declarer_cards_all | problem.left_cards_all | problem.right_cards_all).count_ones() as u8;

        if verify_card_distribution(&problem) {
            return problem;
        } else {
            panic!("Something went wrong in randomly select cards with given facts.");
        }
    }

    fn next_player_facts(&self) -> Facts {
        match self.my_player {
            Player::Declarer => self.facts_left,
            Player::Left => self.facts_right,
            Player::Right => self.facts_declarer,
        }
    }

    fn previous_player_facts(&self) -> Facts {
        match self.my_player {
            Player::Declarer => self.facts_right,
            Player::Left => self.facts_declarer,
            Player::Right => self.facts_left,
        }
    }

    fn validate(&self) {
        self.validate_all_cards();
    }

    fn validate_all_cards(&self) {
        assert!(self.all_cards & self.my_cards == self.my_cards);
        assert!(self.all_cards & self.cards_on_table == self.cards_on_table);        

        // currently uncertain problems can only be solved before a trick starts:
        assert!(self.all_cards.count_ones() % 3 == 0);
    }

}

fn verify_card_distribution(problem: &Problem) -> bool {
    assert!(problem.declarer_cards_all & problem.left_cards_all == 0);
    assert!(problem.declarer_cards_all & problem.right_cards_all == 0);
    assert!(problem.left_cards_all & problem.right_cards_all == 0);

    return true;
}

fn set_cards_for_other_players(
    problem: &mut Problem,
    all_cards: u32,
    cards_on_table: u32,
    my_cards: u32,
    my_player: Player,
    next_player_facts: Facts,
    previous_player_facts: Facts
) {
    let cards_on_hands_of_both_other_players = all_cards & !cards_on_table & !my_cards;

    let mut cards_next_player = cards_on_hands_of_both_other_players;
    let mut cards_previous_player = cards_on_hands_of_both_other_players;

    cards_next_player = cancel_cards_with_facts(cards_next_player, next_player_facts, problem.game_type);
    cards_previous_player = cancel_cards_with_facts(cards_previous_player, previous_player_facts, problem.game_type);

    let proposed_draw = draw_cards(problem, cards_next_player, cards_previous_player, my_cards);

    cards_next_player = proposed_draw.0;
    cards_previous_player = proposed_draw.1;

    add_trick_cards_to_all_cards(&mut cards_next_player, &mut cards_previous_player, cards_on_table);

    set_cards_to_player(problem, cards_next_player, my_player.inc());
    set_cards_to_player(problem, cards_previous_player, my_player.dec());
}

fn add_trick_cards_to_all_cards(cards_player_1: &mut u32, cards_player_2: &mut u32, cards_on_table: u32) {
    let nr_of_trick_cards = cards_on_table.count_ones();

    if nr_of_trick_cards == 0 {
        return;
    }

    if nr_of_trick_cards == 1 {
        let trick_card = cards_on_table.__decompose().0[0];
        let nr_cards_player_1 = cards_player_1.count_ones();
        let nr_cards_player_2 = cards_player_2.count_ones();
        
        if  nr_cards_player_1 < nr_cards_player_2 {
            *cards_player_1 |= trick_card;
        } else {
            *cards_player_2 |= trick_card;
        }

        return;
    }

    if nr_of_trick_cards == 2 {
        *cards_player_1 |= cards_on_table;
        *cards_player_2 |= cards_on_table;
        return;
    }

    panic!("Illegal number of trick cards.");

}

fn draw_cards(problem: &Problem, cards_player_1: u32, cards_player_2: u32, my_cards: u32) -> (u32, u32) {
    
    let nr_trick_cards = problem.trick_cards.count_ones();
    let my_nr_cards = my_cards.count_ones();
    
    let nr_player1_cards = match nr_trick_cards {
        0 => my_nr_cards,
        1 => my_nr_cards - 1,
        2 => my_nr_cards - 1,      
        _ => panic!("Illegal number of trick cards.")
    };

    let nr_player2_cards = match nr_trick_cards {
        0 => my_nr_cards,
        1 => my_nr_cards,
        2 => my_nr_cards - 1,
        _ => panic!("Illegal number of trick cards.")
    };

    let definite_cards_player_1 = cards_player_1 & !cards_player_2;    
    let definite_cards_player_2 = cards_player_2 & !cards_player_1;

    let ambiguous_cards = cards_player_1 & cards_player_2;
    let nr_ambiguous_cards = ambiguous_cards.count_ones();
    let nr_definite_cards_player_1 = definite_cards_player_1.count_ones();
    let nr_definite_cards_player_2 = definite_cards_player_2.count_ones();
    let nr_ambiguous_cards_player_1 = nr_player1_cards - nr_definite_cards_player_1;
    let nr_ambiguous_cards_player_2 = nr_player2_cards - nr_definite_cards_player_2;

    assert!(nr_ambiguous_cards_player_1 + nr_ambiguous_cards_player_2 == nr_ambiguous_cards);

    let draw_player_1 = random_cards(ambiguous_cards, nr_ambiguous_cards_player_1);

    let proposed_player_1 = definite_cards_player_1 | draw_player_1;
    let proposed_player_2 = definite_cards_player_2 | (ambiguous_cards & !draw_player_1);

    (proposed_player_1, proposed_player_2)    
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

fn set_cards_to_player(problem: &mut Problem, cards: u32, player: Player) {
    match player {
        Player::Declarer => problem.declarer_cards_all = cards,
        Player::Left => problem.left_cards_all = cards,
        Player::Right => problem.right_cards_all = cards,
    }
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

fn set_cards_for_problem(problem: &mut Problem, my_cards: u32, my_player: Player) {
    match my_player {
        Player::Declarer => problem.declarer_cards_all = my_cards,
        Player::Left => problem.left_cards_all = my_cards,
        Player::Right => problem.right_cards_all = my_cards,
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::UncertainProblem;
    use crate::{
        traits::{BitConverter, StringConverter},
        types::{game::Game, player::Player},
        uncertain::uncertain_problem::Facts, consts::bitboard::TRUMP_FARBE,
    };

    #[test]
    fn test_problem_generation() {

        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            all_cards: "CA CT SA ST HA HT DA DT D9".__bit(),
            my_cards: "CA CT SA".__bit(),
            cards_on_table: 0u32,
            active_suit: 0u32,
            my_player: Player::Declarer,
            next_player: Player::Declarer,
            threshold_upper: 1,
            facts_declarer: Facts::zero_fact(),
            facts_left: Facts::one_fact(true, false, false, false, false),
            facts_right: Facts::zero_fact()            
        };

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards_all.__str());
        println!("Left cards    : {}", problem.left_cards_all.__str());
        println!("Right cards   : {}", problem.right_cards_all.__str());
    }

    #[test]
    fn test_inter_trick_problem_generation() {

        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            all_cards: "CA CT SA ST HA HT DA DT D9".__bit(),
            my_cards: "CA CT SA".__bit(),
            my_player: Player::Declarer,
            next_player: Player::Declarer,
            threshold_upper: 1,
            cards_on_table: "ST".__bit(),
            active_suit: TRUMP_FARBE,
            facts_declarer: Facts::zero_fact(),
            facts_left: Facts::zero_fact(),
            facts_right: Facts::zero_fact(),
        };

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards_all.__str());
        println!("Left cards    : {}", problem.left_cards_all.__str());
        println!("Right cards   : {}", problem.right_cards_all.__str());
    }
}
