use rand::seq::index::sample;

use crate::consts::bitboard::*;
use crate::traits::{Bitboard, Augen};
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;

pub struct UncertainProblem {
    pub game_type: Game,
    pub my_player: Player,
    pub my_cards: u32,
    pub next_player: Player,

    // Primary values
    pub cards_on_table: u32,
    pub all_cards: u32,
    pub active_suit: u32,    
    pub upper_bound_of_null_window: u8,

    // Facts
    pub facts: [Facts; 2],
}

#[derive(Clone, Copy)]
pub struct Facts {
    pub player: Player,
    pub no_trump: bool,
    pub no_clubs: bool,
    pub no_spades: bool,
    pub no_hearts: bool,
    pub no_diamonds: bool,
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

    pub fn zero_fact(player: Player) -> Facts {
        Facts{ player, no_trump: false, no_clubs: false, no_spades: false, no_hearts: false, no_diamonds: false }
    }

    pub fn one_fact(player: Player, no_trump: bool, no_clubs: bool, no_spades: bool, no_hearts: bool, no_diamonds: bool) -> Facts {
        Facts{ player, no_trump: no_trump, no_clubs: no_clubs, no_spades: no_spades, no_hearts: no_hearts, no_diamonds: no_diamonds }
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
            upper_bound_of_null_window: 0u8,
            facts: [Facts::new(); 2],
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
            points_to_win: self.upper_bound_of_null_window,
        };

        set_cards_for_problem(&mut problem, self.my_cards, self.my_player);
        set_cards_for_other_players(&mut problem, self.all_cards, self.cards_on_table, self.my_cards, self.facts);

        problem.augen_total = (problem.declarer_cards_all | problem.left_cards_all | problem.right_cards_all).__get_value();
        problem.nr_of_cards = (problem.declarer_cards_all | problem.left_cards_all | problem.right_cards_all).count_ones() as u8;

        if verify_card_distribution(&problem) {
            return problem;
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
    facts: [Facts; 2],
) {
    let cards_on_hands_of_both_other_players = all_cards & !cards_on_table & !my_cards;

    let mut cards_player_1 = cards_on_hands_of_both_other_players;
    let mut cards_player_2 = cards_on_hands_of_both_other_players;

    cards_player_1 = cancel_cards_with_facts(cards_player_1, facts[0], problem.game_type);
    cards_player_2 = cancel_cards_with_facts(cards_player_2, facts[1], problem.game_type);

    let proposed_draw = draw_cards(problem, cards_player_1, cards_player_2, my_cards);

    cards_player_1 = proposed_draw.0;
    cards_player_2 = proposed_draw.1;

    add_trick_cards_to_all_cards(&mut cards_player_1, &mut cards_player_2, cards_on_table);

    set_cards_to_player(problem, cards_player_1, facts[0].player);
    set_cards_to_player(problem, cards_player_2, facts[1].player);
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
            upper_bound_of_null_window: 1,
            facts: [
                Facts::one_fact(Player::Left, true, false, false, false, true),
                Facts::zero_fact(Player::Right)
            ]
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
            upper_bound_of_null_window: 1,
            cards_on_table: "ST".__bit(),
            active_suit: TRUMP_FARBE,
            facts: [
                Facts::one_fact(Player::Right,false,false,false,false,false),
                Facts::zero_fact(Player::Left)
            ]
        };

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards_all.__str());
        println!("Left cards    : {}", problem.left_cards_all.__str());
        println!("Right cards   : {}", problem.right_cards_all.__str());
    }
}
