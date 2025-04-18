use std::sync::{Arc, Mutex};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use crate::types::{counter::Counters, game::Game, player::Player, problem::Problem, solver::{retargs::{SolveWithSkatRet, SolveWithSkatRetLine}, Solver}, tt_table::TtTable};
use crate::traits::{Augen, Bitboard};

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

   
}
