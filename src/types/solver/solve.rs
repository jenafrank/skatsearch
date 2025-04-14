use super::Solver;
use super::retargs::SolveAllCardsRet;
use super::retargs::SolveRet;
use super::retargs::SolveWithSkatRet;
use super::retargs::SolveWithSkatRetLine;
use super::retargs::SolveWinRet;
use crate::consts::bitboard::JACKOFCLUBS;
use crate::traits::Augen;
use crate::traits::Bitboard;
use crate::types::counter::Counters;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::state::State;
use crate::types::tt_table::TtTable;
use rayon::prelude::*; 
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};


/// Analyses a twelve-card hand during the task of putting away two cards (Skat) before
/// game starts. It analyses all 66 cases and calculating the best play for each of them
/// using the same transposition table for speed-up reasons.
/// # Variants of arguments
/// * _false_ _false_: Returns exact values for all 66 games.
/// * _true_ _false_: Returns best skat by means of alpha-window narrowing. Thus, the
/// 66-array does also contain wrong values.
/// * _false_ _true_: Returns some skat for which the game will be won.
/// The routine always takes into account the value of the skat which is neglected by default
/// in the basic search routines.

#[derive(Clone, Copy)]
pub enum AccelerationMode {
    NotAccelerating,
    AlphaBetaAccelerating,
    WinningOnly,
}

impl Solver {

    pub fn solve_with_skat_alpha_cut_parallel(
        left_cards: u32,
        right_cards: u32,
        declarer_cards: u32,
        game: Game,
        first_player: Player,
    ) -> SolveWithSkatRet {
        
        let skat = !(left_cards | right_cards | declarer_cards);
        let cards12 = skat | declarer_cards;
        let cards12_array = cards12.__decompose_twelve();
        let skat_combinations = Solver::generate_skat_combinations(&cards12_array);
        
        let best_skat: Arc<Mutex<Option<SolveWithSkatRetLine>>> = Arc::new(Mutex::new(None));
        let all_skats: Arc<Mutex<Vec<SolveWithSkatRetLine>>> = Arc::new(Mutex::new(Vec::new()));
        let all_counters: Arc<Mutex<Vec<Counters>>> = Arc::new(Mutex::new(Vec::new()));

        let game_alpha: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
        let tt: Arc<Mutex<TtTable>> = Arc::new(Mutex::new(TtTable::new()));
        
        skat_combinations.into_par_iter().for_each(|(skat_card_1, skat_card_2)| {           
            
            let current_drueckung = skat_card_1 | skat_card_2;
            let skat_value = current_drueckung.__get_value();
            let current_declarer_cards = cards12 ^ current_drueckung;
    
            let current_problem = Problem::create(
                current_declarer_cards,
                left_cards,
                right_cards,
                game,
                first_player,
            );
            
            let mut solver = Solver::new(current_problem, Some(tt.lock().unwrap().clone()));
            // let mut solver = Solver::new(current_problem, None);
            let current_game_alpha = game_alpha.lock().unwrap().clone();
            let mut current_alpha = 0;
            if current_game_alpha > skat_value { current_alpha = current_game_alpha - skat_value; }
            let solution = solver.solve_double_dummy(current_alpha, 120, 5);
            
            let game_value = solution.best_value + skat_value;                
            let skat_ret_line = SolveWithSkatRetLine {
              skat_card_1,
              skat_card_2,
              value: game_value,
            };            
    
            all_skats.lock().unwrap().push(skat_ret_line.clone());
            all_counters.lock().unwrap().push(solution.counters.clone());
    
            if game_value > game_alpha.lock().unwrap().clone() {                
                *game_alpha.lock().unwrap() = game_value; 
                *best_skat.lock().unwrap() = Some(skat_ret_line);               
            }

            tt.lock().unwrap().add_entries(solver.tt);
        });
    
        let best_skat_result = best_skat.lock().unwrap().clone();
        let all_skats_result = all_skats.lock().unwrap().clone();
        let all_counters_result = all_counters.lock().unwrap().clone();

        let acc_counter = Counters::accumulate(all_counters_result);
    
        SolveWithSkatRet {            
            best_skat: best_skat_result,
            all_skats: all_skats_result,
            counters: acc_counter,
        }
    }

    pub fn solve_with_skat(
        left_cards: u32,
        right_cards: u32,
        declarer_cards: u32,
        game: Game,
        start_player: Player,
        accelerating_mode: AccelerationMode,
    ) -> SolveWithSkatRet {

        let mut ret: SolveWithSkatRet = SolveWithSkatRet {
            best_skat: None,
            all_skats: Vec::new(),
            counters: Counters::new(),
        };

        let p = Problem::create(
            declarer_cards, 
            left_cards, 
            right_cards, 
            game, 
            start_player);

        let mut solver = Solver::new(p, None);

        let initial_state = State::create_initial_state_from_problem(&solver.problem);
        let skatcards_bitmask = !initial_state.get_all_unplayed_cards();
        let twelve_cards_bitmask = skatcards_bitmask | initial_state.declarer_cards;
        let twelve_cards = twelve_cards_bitmask.__decompose_twelve();

        let mut alpha = 0;

        let skat_combinations = Solver::generate_skat_combinations(&twelve_cards);
        
        for (skat_card_1, skat_card_2) in skat_combinations {
            
            let skat_bitmask = skat_card_1 | skat_card_2;
            let skat_value = skat_bitmask.__get_value();
            
            let player_hand_bitmask = twelve_cards_bitmask ^ skat_bitmask;

            solver.problem.set_declarer_cards(player_hand_bitmask);

            let game_value = solver.evaluate_skat_combination(                
                skat_value,
                accelerating_mode,
                alpha,
                &mut ret.counters
            );

            ret.all_skats.push(SolveWithSkatRetLine {
                skat_card_1,
                skat_card_2,
                value: game_value,
            });

            Solver::update_best_skat(&mut ret, skat_card_1, skat_card_2, game_value, &mut alpha);
        }       

        ret
    }

    fn generate_skat_combinations(cards: &[u32]) -> Vec<(u32, u32)> {
        let mut combinations = Vec::new();
        for i in 0..11 {
            for j in i + 1..12 {
                combinations.push((cards[i], cards[j]));
            }
        }
        combinations
    }

    fn evaluate_skat_combination(
        &mut self,
        skat_value: u8,
        mode: AccelerationMode,
        alpha: u8,
        cnt: &mut Counters,
    ) -> u8 {
        let mut current_game_state = State::create_initial_state_from_problem(&self.problem);
    
        match mode {
            AccelerationMode::AlphaBetaAccelerating => {
                if alpha > skat_value {
                    current_game_state.alpha = alpha - skat_value;
                }
            },
            AccelerationMode::WinningOnly => {
                if alpha >= 61 {
                    return 0; // Early return, Wert spielt hier keine Rolle
                }
                current_game_state.alpha = 60 - skat_value;
                current_game_state.beta = current_game_state.alpha + 1;
            },
            AccelerationMode::NotAccelerating => {
                // Hier erfolgt keine Änderung an current_game_state
            },
        }
    
        let result = self.solve_double_dummy(current_game_state.alpha, current_game_state.beta, 1);
    
        cnt.add(result.counters);
    
        result.best_value + skat_value
    }

    fn update_best_skat(        
        ret: &mut SolveWithSkatRet,
        skat_card_1: u32,
        skat_card_2: u32,
        game_value: u8,
        alpha: &mut u8,
    ) {
        if game_value > *alpha {
            ret.best_skat = Some(SolveWithSkatRetLine {
                skat_card_1,
                skat_card_2,
                value: game_value,
            });
            *alpha = game_value;
        }
    }  
}

impl Solver {

    /// Investigates all legal moves for a given state and returns an option array
    /// with 0. card under investigation 1. follow-up card from tree search (tree root) and
    /// 2. value of search
    pub fn solve_all_cards(&mut self, alpha: u8, beta: u8) -> SolveAllCardsRet {
        let initial_state = State::create_initial_state_from_problem(&self.problem);        
        self.get_all_cards(initial_state, alpha, beta)
    }

    pub fn solve_win(&mut self) -> SolveWinRet {
        
        let mut cnt = Counters::new();
        
        let mut alpha = self.problem.points_to_win() - 1;
        let mut beta = self.problem.points_to_win();

        if self.problem.game_type() == Game::Null {
            alpha = 0;
            beta = 1;
        } 

        let state = self.problem.new_state(alpha, beta);
        let (best_card, value) = self.problem.search(&state, &mut self.tt, &mut cnt);
        
        let mut declarer_wins = value > alpha;
        
        if self.problem.game_type() == Game::Null {
            declarer_wins = !declarer_wins;
        }        

        SolveWinRet {
            best_card,
            declarer_wins,
            counters: cnt
        }
    }

    // works currently only with 10 cards, since all cards not part of the full deck
    // are considered as skat and thus as points fot the declarer.
    pub fn solve_win_10tricks(&mut self) -> SolveWinRet {

        let mut cnt = Counters::new();

        let mut state = State::create_initial_state_from_problem(&self.problem);

        let skat_value = self.problem.get_skat().__get_value();

        let threshold_farbe_and_grand = 60 - skat_value;

        match self.problem.game_type() {
            Game::Farbe => {
                state.alpha = threshold_farbe_and_grand;
                state.beta = threshold_farbe_and_grand + 1;
            }
            Game::Grand => {
                state.alpha = threshold_farbe_and_grand;
                state.beta = threshold_farbe_and_grand + 1;
            }
            Game::Null => {
                state.alpha = 0;
                state.beta = 1;
            }
        }

        let result = self.problem.search(&state, &mut self.tt, &mut cnt);
        let val = result.1;

        let declarer_wins = if self.problem.game_type() == Game::Null {
            val == 0
        } else {
            val > threshold_farbe_and_grand
        };

        SolveWinRet {
            best_card: result.0,
            declarer_wins,
            counters: cnt
        }
    }

    // unclear, if the right best card is determined. complicated. in search routine we should
    // identify, if any best card has been detected so far
    pub fn solve_double_dummy(&mut self, alpha: u8, beta: u8, width: u8) -> SolveRet {
        
        let mut cnt = Counters::new();
        let mut result = (0u32, 0u8);        
        
        let mut current_alpha = alpha;
        while current_alpha < beta {
            let current_beta = std::cmp::min(current_alpha + width, beta);
            
            let mut state = State::create_initial_state_from_problem(&self.problem);
            state.alpha = current_alpha;
            state.beta = current_beta;
            result = self.problem.search(&state, &mut self.tt, &mut cnt);

            if result.1 < current_beta {
                break;
            }
            
            current_alpha = current_beta;
        }

        SolveRet { best_card: result.0, best_value: result.1, counters: cnt }
    }   

    pub fn solve(&mut self) -> SolveRet {
        let state = State::create_initial_state_from_problem(&self.problem);
        let result = self.get(state);

        println!(" Iters: {}, Slots: {}, Writes: {}, Reads: {}, ExactReads: {}, Collisions: {}, Breaks: {}",
        result.counters.iters,
        self.tt.get_occupied_slots(),
        result.counters.writes,
        result.counters.reads,
        result.counters.exactreads,
        result.counters.collisions,
        result.counters.breaks);

        result
    }

}

#[cfg(test)]
mod tests {
    use crate::{consts::bitboard::SPADES, types::{player::Player, problem_builder::ProblemBuilder, solver::Solver}};

    #[test]
    fn test_solve_win() {

        let problem = ProblemBuilder::new_farbspiel()
        .cards_all("SA SK", "ST SQ", "S9 S8")
        .turn(Player::Declarer)
        .threshold(14)
        .build();

        let mut solver = Solver::new(problem, None);
        let result = solver.solve_win();

        assert_eq!(result.declarer_wins, true);
    }

    #[test]
    fn test_solve_win_intertrick() {
        let problem = ProblemBuilder::new_farbspiel()
        .cards_all("SA SK", "ST SQ", "S9 S8")
        .turn(Player::Left)
        .threshold(3)
        .trick(SPADES, "SA")
        .build();

        let mut solver = Solver::new(problem, None);
        let result = solver.solve_win();

        assert_eq!(result.declarer_wins, true);
    }
}